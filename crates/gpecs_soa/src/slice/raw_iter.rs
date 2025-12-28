use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use crate::{
    slice::{Iter, RawIterMut},
    traits::{Ptrs, RawSoa, RawSoaContext, SlicePtrs},
    wrapper,
};

pub struct RawIter<'ctx, T>
where
    T: RawSoa + ?Sized,
{
    ptrs: wrapper::Ptrs<'ctx, T>,
    context: &'ctx T::Context,
    start: usize,
    end: usize,
}

impl<'ctx, T> RawIter<'ctx, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    pub fn new(context: &'ctx T::Context, slices: SlicePtrs<'ctx, T>) -> Self {
        let len = context.slice_ptrs_len(&slices);
        let ptrs = context.slice_ptrs_as_ptrs(slices);
        Self {
            ptrs: wrapper::Ptrs::new(ptrs),
            context,
            start: 0,
            end: len,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { start, end, .. } = *self;
        end - start
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn context(&self) -> &'ctx T::Context {
        let Self { context, .. } = *self;
        context
    }

    #[inline]
    pub fn as_ptrs(&self) -> Ptrs<'ctx, T> {
        let (_, ptrs) = self.as_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&'ctx T::Context, Ptrs<'ctx, T>) {
        let Self {
            ref ptrs,
            context,
            start,
            ..
        } = *self;

        let ptrs = ptrs.clone().into_inner();
        let ptrs = unsafe { context.ptrs_add(ptrs, start) };
        (context, ptrs)
    }

    #[inline]
    pub fn into_ptrs(self) -> Ptrs<'ctx, T> {
        let (_, ptrs) = self.into_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn into_ptrs_with_context(self) -> (&'ctx T::Context, Ptrs<'ctx, T>) {
        let Self {
            ptrs,
            context,
            start,
            ..
        } = self;

        let ptrs = ptrs.into_inner();
        let ptrs = unsafe { context.ptrs_add(ptrs, start) };
        (context, ptrs)
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> SlicePtrs<'ctx, T> {
        let (_, slices) = self.as_slice_ptrs_with_context();
        slices
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(&self) -> (&'ctx T::Context, SlicePtrs<'ctx, T>) {
        let Self {
            ref ptrs,
            context,
            start,
            ..
        } = *self;

        let len = self.len();
        let ptrs = ptrs.clone().into_inner();
        let ptrs = unsafe { context.ptrs_add(ptrs, start) };
        let slices = context.slice_ptrs_from_raw_parts(ptrs, len);
        (context, slices)
    }

    #[inline]
    pub fn into_slice_ptrs(self) -> SlicePtrs<'ctx, T> {
        let (_, slices) = self.into_slice_ptrs_with_context();
        slices
    }

    #[inline]
    #[doc(alias = "into_parts")]
    pub fn into_slice_ptrs_with_context(self) -> (&'ctx T::Context, SlicePtrs<'ctx, T>) {
        let len = self.len();
        let Self {
            ptrs,
            context,
            start,
            ..
        } = self;

        let ptrs = ptrs.into_inner();
        let ptrs = unsafe { context.ptrs_add(ptrs, start) };
        let slices = context.slice_ptrs_from_raw_parts(ptrs, len);
        (context, slices)
    }

    #[inline]
    pub fn cast_mut(self) -> RawIterMut<'ctx, T> {
        let (context, slices) = self.into_slice_ptrs_with_context();
        let slices = context.slice_ptrs_cast_mut(slices);
        RawIterMut::new(context, slices)
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> Iter<'ctx, 'a, T> {
        let (context, slices) = self.into_slice_ptrs_with_context();
        unsafe { Iter::from_parts(context, slices) }
    }

    #[inline]
    unsafe fn post_inc_start<'b>(
        start: &mut usize,
        ptrs: Ptrs<'b, T>,
        context: &'b T::Context,
        offset: usize,
    ) -> Ptrs<'b, T> {
        let old_start = *start;
        *start += offset;
        unsafe { context.ptrs_add(ptrs, old_start) }
    }

    #[inline]
    unsafe fn pre_dec_end<'b>(
        end: &mut usize,
        ptrs: Ptrs<'b, T>,
        context: &'b T::Context,
        offset: usize,
    ) -> Ptrs<'b, T> {
        *end -= offset;
        unsafe { context.ptrs_add(ptrs, *end) }
    }
}

impl<'ctx, T> From<&'ctx T::Context> for RawIter<'ctx, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn from(context: &'ctx T::Context) -> Self {
        let ptrs = context.ptrs_dangling();
        let slices = context.slice_ptrs_from_raw_parts(ptrs, 0);
        Self::new(context, slices)
    }
}

impl<T> Debug for RawIter<'_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> SlicePtrs<'ctx, T>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slice_ptrs();
        f.debug_tuple("RawIter").field(&slices).finish()
    }
}

impl<T> Clone for RawIter<'_, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self {
            ref ptrs,
            context,
            start,
            end,
        } = *self;

        Self {
            context,
            ptrs: ptrs.clone(),
            start,
            end,
        }
    }
}

#[expect(clippy::while_let_on_iterator)]
impl<'ctx, T> Iterator for RawIter<'ctx, T>
where
    T: RawSoa + ?Sized,
{
    type Item = Ptrs<'ctx, T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if RawIter::is_empty(self) {
            return None;
        }

        let Self {
            context,
            ref ptrs,
            ref mut start,
            ..
        } = *self;
        let ptrs = ptrs.clone().into_inner();
        let ptrs = unsafe { Self::post_inc_start(start, ptrs, context, 1) };
        Some(ptrs)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = RawIter::len(self);
        (len, Some(len))
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        RawIter::len(&self)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n >= RawIter::len(self) {
            self.start = self.end;
            return None;
        }

        let Self {
            context,
            ref ptrs,
            ref mut start,
            ..
        } = *self;
        let ptrs = ptrs.clone().into_inner();
        unsafe {
            Self::post_inc_start(start, ptrs, context, n);
        }
        self.next()
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.next_back()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        if RawIter::is_empty(&self) {
            return init;
        }

        // this implementation consists of the following optimizations compared to the
        // default implementation:
        // - do-while loop, as is llvm's preferred loop shape,
        //   see https://releases.llvm.org/16.0.0/docs/LoopTerminology.html#more-canonical-loops
        // - bumps an index instead of a pointer since the latter case inhibits
        //   some optimizations, see #111603
        // - avoids Option wrapping/matching
        let Self {
            ref ptrs,
            context,
            start,
            end,
        } = self;
        let mut acc = init;
        let mut i = start;
        loop {
            // SAFETY: the loop iterates `i in start..end`, which always is in bounds of
            // the slice allocation
            let ptrs = ptrs.clone().into_inner();
            let item = unsafe { context.ptrs_add(ptrs, i) };
            acc = f(acc, item);
            // SAFETY: `i` can't overflow since it'll only reach usize::MAX if the
            // slice had that length, in which case we'll break out of the loop
            // after the increment
            i = unsafe { i.unchecked_add(1) };
            if i == end {
                break;
            }
        }
        acc
    }

    #[inline]
    fn for_each<F>(mut self, mut f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        while let Some(x) = self.next() {
            f(x);
        }
    }

    #[inline]
    fn all<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        while let Some(x) = self.next() {
            if !f(x) {
                return false;
            }
        }
        true
    }

    #[inline]
    fn any<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        while let Some(x) = self.next() {
            if f(x) {
                return true;
            }
        }
        false
    }

    #[inline]
    fn find<P>(&mut self, mut predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        while let Some(x) = self.next() {
            if predicate(&x) {
                return Some(x);
            }
        }
        None
    }

    #[inline]
    fn find_map<B, F>(&mut self, mut f: F) -> Option<B>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        while let Some(x) = self.next() {
            if let Some(y) = f(x) {
                return Some(y);
            }
        }
        None
    }

    #[inline]
    fn position<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let n = RawIter::len(self);
        let mut i = 0;
        while let Some(x) = self.next() {
            if predicate(x) {
                assert!(i < n);
                return Some(i);
            }
            i += 1;
        }
        None
    }

    #[inline]
    fn rposition<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        P: FnMut(Self::Item) -> bool,
        Self: Sized + ExactSizeIterator + DoubleEndedIterator,
    {
        let n = RawIter::len(self);
        let mut i = n;
        while let Some(x) = self.next_back() {
            i -= 1;
            if predicate(x) {
                assert!(i < n);
                return Some(i);
            }
        }
        None
    }
}

impl<T> DoubleEndedIterator for RawIter<'_, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if RawIter::is_empty(self) {
            return None;
        }

        let Self {
            context,
            ref ptrs,
            ref mut end,
            ..
        } = *self;
        let ptrs = ptrs.clone().into_inner();
        let ptrs = unsafe { Self::pre_dec_end(end, ptrs, context, 1) };
        Some(ptrs)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        if n >= RawIter::len(self) {
            self.end = self.start;
            return None;
        }

        let Self {
            context,
            ref ptrs,
            ref mut end,
            ..
        } = *self;
        let ptrs = ptrs.clone().into_inner();
        unsafe {
            Self::pre_dec_end(end, ptrs, context, n);
        }
        self.next_back()
    }
}

impl<T> ExactSizeIterator for RawIter<'_, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        RawIter::len(self)
    }
}

impl<T> FusedIterator for RawIter<'_, T> where T: RawSoa + ?Sized {}

unsafe impl<T> Send for RawIter<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: Send,
    T::Fields: Send,
{
}

unsafe impl<T> Sync for RawIter<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: Sync,
    T::Fields: Sync,
{
}
