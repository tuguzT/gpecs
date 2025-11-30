use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
};

use crate::{
    traits::{MutPtrs, Ptrs, RawSoa, RawSoaContext, SlicePtrs, Soa},
    wrapper::{NonNullPtrs, Ptrs as PtrsWrapper},
};

pub struct RawIter<'c, T>
where
    T: RawSoa + ?Sized,
{
    ptrs: PtrsWrapper<'c, T>,
    context: &'c T::Context,
    start: usize,
    end: usize,
}

impl<'c, T> RawIter<'c, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    pub fn new(context: &'c T::Context, slices: SlicePtrs<'c, T>) -> Self {
        let len = context.slice_ptrs_len(&slices);
        let ptrs = context.slice_ptrs_as_ptrs(slices);
        Self {
            ptrs: PtrsWrapper::new(ptrs),
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
    pub fn context(&self) -> &'c T::Context {
        let Self { context, .. } = *self;
        context
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> SlicePtrs<'c, T> {
        let (_, slices) = self.as_slice_ptrs_with_context();
        slices
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(&self) -> (&'c T::Context, SlicePtrs<'c, T>) {
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
    pub fn into_slice_ptrs(self) -> SlicePtrs<'c, T> {
        let (_, slices) = self.into_parts();
        slices
    }

    #[inline]
    pub fn into_parts(self) -> (&'c T::Context, SlicePtrs<'c, T>) {
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
    pub unsafe fn deref<'a>(self) -> Iter<'c, 'a, T> {
        let (context, slices) = self.as_slice_ptrs_with_context();
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

impl<'c, T> From<&'c T::Context> for RawIter<'c, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn from(context: &'c T::Context) -> Self {
        let ptrs = context.ptrs_dangling();
        Self {
            context,
            ptrs: PtrsWrapper::new(ptrs),
            start: 0,
            end: 0,
        }
    }
}

impl<T> Debug for RawIter<'_, T>
where
    T: RawSoa + ?Sized,
    for<'any> SlicePtrs<'any, T>: Debug,
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
impl<'c, T> Iterator for RawIter<'c, T>
where
    T: RawSoa + ?Sized,
{
    type Item = Ptrs<'c, T>;

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
            context, ref ptrs, ..
        } = self;
        let mut acc = init;
        let mut i = 0;
        let len = RawIter::len(&self);
        loop {
            // SAFETY: the loop iterates `i in 0..len`, which always is in bounds of
            // the slice allocation
            let ptrs = ptrs.clone().into_inner();
            let item = unsafe { context.ptrs_add(ptrs, i) };
            acc = f(acc, item);
            // SAFETY: `i` can't overflow since it'll only reach usize::MAX if the
            // slice had that length, in which case we'll break out of the loop
            // after the increment
            i = unsafe { i.unchecked_add(1) };
            if i == len {
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

pub struct Iter<'c, 'a, T>
where
    T: RawSoa + ?Sized + 'a,
{
    inner: RawIter<'c, T>,
    phantom: PhantomData<&'a ()>,
}

impl<'c, T> Iter<'c, '_, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    pub fn len(&self) -> usize {
        let Self { inner, .. } = self;
        inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn context(&self) -> &'c T::Context {
        let Self { inner, .. } = self;
        inner.context()
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> SlicePtrs<'c, T> {
        let (_, slices) = self.as_slice_ptrs_with_context();
        slices
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(&self) -> (&'c T::Context, SlicePtrs<'c, T>) {
        let Self { inner, .. } = self;
        inner.as_slice_ptrs_with_context()
    }

    #[inline]
    pub fn into_slice_ptrs(self) -> SlicePtrs<'c, T> {
        let (_, slices) = self.into_parts();
        slices
    }

    #[inline]
    pub fn into_raw_iter(self) -> RawIter<'c, T> {
        let Self { inner, .. } = self;
        inner
    }

    #[inline]
    pub unsafe fn from_parts(context: &'c T::Context, slices: SlicePtrs<'c, T>) -> Self {
        Self {
            inner: RawIter::new(context, slices),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn into_parts(self) -> (&'c T::Context, SlicePtrs<'c, T>) {
        let Self { inner, .. } = self;
        inner.into_parts()
    }
}

impl<'c, 'a, T> Iter<'c, 'a, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    pub fn new(context: &'c T::Context, slices: T::Slices<'c, 'a>) -> Self {
        let slices = T::slices_as_slice_ptrs(context, slices);
        unsafe { Self::from_parts(context, slices) }
    }

    #[inline]
    pub fn as_slices(&self) -> T::Slices<'c, 'a> {
        let (_, slices) = self.as_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_slices_with_context(&self) -> (&'c T::Context, T::Slices<'c, 'a>) {
        let Self { inner, .. } = self;

        let (context, slices) = inner.as_slice_ptrs_with_context();
        let slices = unsafe { T::slice_ptrs_to_slices(context, slices) };
        (context, slices)
    }
}

impl<T, U> AsRef<[U]> for Iter<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Into<&'any [U]>,
{
    #[inline]
    fn as_ref(&self) -> &[U] {
        self.as_slices().into()
    }
}

impl<T> Debug for Iter<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("Iter").field(&slices).finish()
    }
}

impl<T> Clone for Iter<'_, '_, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { ref inner, phantom } = *self;

        let inner = inner.clone();
        Self { inner, phantom }
    }
}

impl<'c, 'a, T> Iterator for Iter<'c, 'a, T>
where
    T: Soa + ?Sized,
{
    type Item = T::Refs<'c, 'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner, .. } = self;
        let context = inner.context();

        let f = |ptrs| unsafe { T::ptrs_to_refs(context, ptrs) };
        inner.next().map(f)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = Iter::len(self);
        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for Iter<'_, '_, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner, .. } = self;
        let context = inner.context();

        let f = |ptrs| unsafe { T::ptrs_to_refs(context, ptrs) };
        inner.next_back().map(f)
    }
}

impl<T> ExactSizeIterator for Iter<'_, '_, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        Iter::len(self)
    }
}

impl<T> FusedIterator for Iter<'_, '_, T> where T: Soa + ?Sized {}

pub struct IterMut<'c, 'a, T>
where
    T: Soa + ?Sized + 'a,
{
    context: &'c T::Context,
    ptrs: NonNullPtrs<'c, T>,
    start: usize,
    end: usize,
    phantom: PhantomData<&'a ()>,
}

impl<'c, 'a, T> IterMut<'c, 'a, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    pub(super) fn new(context: &'c T::Context, slices: T::SlicesMut<'c, 'a>) -> Self {
        let len = T::slices_mut_len(context, &slices);
        let ptrs = T::slices_mut_as_ptrs(context, slices);
        let ptrs = unsafe { context.ptrs_to_nonnull(ptrs) };
        Self {
            context,
            ptrs: NonNullPtrs::new(ptrs),
            start: 0,
            end: len,
            phantom: PhantomData,
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
    pub fn context(&self) -> &T::Context {
        let Self { context, .. } = *self;
        context
    }

    fn ptrs(&self) -> MutPtrs<'_, T> {
        let Self { context, ptrs, .. } = self;
        let ptrs = ptrs.clone().into_inner();
        context.nonnull_to_ptrs(ptrs)
    }

    #[inline]
    pub fn into_slices(self) -> T::SlicesMut<'c, 'a> {
        let len = self.len();
        let Self { context, ptrs, .. } = self;
        let ptrs = ptrs.into_inner();
        let ptrs = context.nonnull_to_ptrs(ptrs);

        unsafe {
            let ptrs = context.ptrs_add_mut(ptrs, self.start);
            let slices = context.slice_mut_ptrs_from_raw_parts(ptrs, len);
            T::slice_mut_ptrs_to_slices(context, slices)
        }
    }

    #[inline]
    pub fn as_slices(&self) -> T::Slices<'_, '_> {
        let Self { context, start, .. } = *self;
        let len = self.len();
        let ptrs = self.ptrs();
        let ptrs = context.ptrs_cast_const(ptrs);

        let ptrs = unsafe { context.ptrs_add(ptrs, start) };
        let slices = context.slice_ptrs_from_raw_parts(ptrs, len);
        unsafe { T::slice_ptrs_to_slices(context, slices) }
    }

    #[inline]
    unsafe fn post_inc_start<'b>(
        start: &mut usize,
        ptrs: MutPtrs<'b, T>,
        context: &'b T::Context,
        offset: usize,
    ) -> MutPtrs<'b, T> {
        let old_start = *start;
        *start += offset;

        unsafe { context.ptrs_add_mut(ptrs, old_start) }
    }

    #[inline]
    unsafe fn pre_dec_end<'b>(
        end: &mut usize,
        ptrs: MutPtrs<'b, T>,
        context: &'b T::Context,
        offset: usize,
    ) -> MutPtrs<'b, T> {
        *end -= offset;

        unsafe { context.ptrs_add_mut(ptrs, *end) }
    }
}

unsafe impl<T> Send for IterMut<'_, '_, T>
where
    T: Soa + ?Sized,
    T::Context: Send,
    T::Fields: Send,
{
}

unsafe impl<T> Sync for IterMut<'_, '_, T>
where
    T: Soa + ?Sized,
    T::Context: Sync,
    T::Fields: Sync,
{
}

impl<T, U> AsRef<[U]> for IterMut<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Into<&'any [U]>,
{
    #[inline]
    fn as_ref(&self) -> &[U] {
        self.as_slices().into()
    }
}

impl<T> Debug for IterMut<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("IterMut").field(&slices).finish()
    }
}

#[expect(clippy::while_let_on_iterator)]
impl<'c, 'a, T> Iterator for IterMut<'c, 'a, T>
where
    T: Soa + ?Sized,
{
    type Item = T::RefsMut<'c, 'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if IterMut::is_empty(self) {
            return None;
        }

        let Self {
            context,
            ref ptrs,
            ref mut start,
            ..
        } = *self;
        let ptrs = ptrs.clone().into_inner();
        let ptrs = context.nonnull_to_ptrs(ptrs);
        let ptrs = unsafe { Self::post_inc_start(start, ptrs, context, 1) };

        let refs = unsafe { T::ptrs_to_refs_mut(context, ptrs) };
        Some(refs)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = IterMut::len(self);
        (len, Some(len))
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        IterMut::len(&self)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n >= IterMut::len(self) {
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
        let ptrs = context.nonnull_to_ptrs(ptrs);
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
        if IterMut::is_empty(&self) {
            return init;
        }

        // this implementation consists of the following optimizations compared to the
        // default implementation:
        // - do-while loop, as is llvm's preferred loop shape,
        //   see https://releases.llvm.org/16.0.0/docs/LoopTerminology.html#more-canonical-loops
        // - bumps an index instead of a pointer since the latter case inhibits
        //   some optimizations, see #111603
        // - avoids Option wrapping/matching
        let Self { context, ptrs, .. } = &self;
        let len = IterMut::len(&self);
        let mut acc = init;
        let mut i = 0;
        loop {
            // SAFETY: the loop iterates `i in 0..len`, which always is in bounds of
            // the slice allocation
            let ptrs = ptrs.clone().into_inner();
            let ptrs = context.nonnull_to_ptrs(ptrs);
            let item = unsafe {
                let ptrs = context.ptrs_add_mut(ptrs, i);
                T::ptrs_to_refs_mut(context, ptrs)
            };
            acc = f(acc, item);
            // SAFETY: `i` can't overflow since it'll only reach usize::MAX if the
            // slice had that length, in which case we'll break out of the loop
            // after the increment
            i = unsafe { i.unchecked_add(1) };
            if i == len {
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
        let n = IterMut::len(self);
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
        let n = IterMut::len(self);
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

impl<T> DoubleEndedIterator for IterMut<'_, '_, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if IterMut::is_empty(self) {
            return None;
        }

        let Self {
            context,
            ref ptrs,
            ref mut end,
            ..
        } = *self;
        let ptrs = ptrs.clone().into_inner();
        let ptrs = context.nonnull_to_ptrs(ptrs);
        let ptrs = unsafe { Self::pre_dec_end(end, ptrs, context, 1) };

        let refs = unsafe { T::ptrs_to_refs_mut(context, ptrs) };
        Some(refs)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.len() {
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
        let ptrs = context.nonnull_to_ptrs(ptrs);
        unsafe {
            Self::pre_dec_end(end, ptrs, context, n);
        }
        self.next_back()
    }
}

impl<T> ExactSizeIterator for IterMut<'_, '_, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        IterMut::len(self)
    }
}

impl<T> FusedIterator for IterMut<'_, '_, T> where T: Soa + ?Sized {}
