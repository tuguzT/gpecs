use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::{ManuallyDrop, transmute},
    ptr::NonNull,
};

use crate::{
    alloc::raw_vec::RawSoaVec,
    layout::BufferData,
    ptr::BufferDataPtr,
    traits::{
        MutPtrs, NonNullPtrs, Ptrs, RawSoa, RawSoaContext, SliceMutPtrs, SlicePtrs, Soa, SoaRead,
    },
    vec::SoaVec,
    wrapper::NonNullPtrs as NonNullPtrsWrapper,
};

pub struct IntoIter<T>
where
    T: RawSoa + ?Sized,
{
    ptrs: NonNullPtrsWrapper<'static, T>,
    buffer: NonNull<BufferData<T>>,
    capacity: usize,
    start: usize,
    end: usize,
}

impl<T> IntoIter<T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    pub(super) fn new(vec: SoaVec<T>) -> Self {
        let mut vec = ManuallyDrop::new(vec);

        let buffer = vec.as_mut_ptr();
        let (context, ptrs) = vec.as_mut_ptrs_with_context();

        let ptrs = unsafe { context.ptrs_to_nonnull(ptrs) };
        let ptrs = unsafe { transmute::<NonNullPtrs<'_, T>, NonNullPtrs<'_, T>>(ptrs) };
        Self {
            ptrs: NonNullPtrsWrapper::new(ptrs),
            buffer: unsafe { NonNull::new_unchecked(buffer) },
            capacity: vec.capacity(),
            start: 0,
            end: vec.len(),
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
        let Self { buffer, .. } = *self;
        unsafe { Self::context_of(buffer) }
    }

    #[inline]
    pub fn as_ptrs(&self) -> Ptrs<'_, T> {
        let (_, ptrs) = self.as_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&T::Context, Ptrs<'_, T>) {
        let Self {
            ref ptrs,
            buffer,
            start,
            ..
        } = *self;
        let context = unsafe { Self::context_of(buffer) };

        let ptrs = ptrs.clone().into_inner();
        let ptrs = context.nonnull_to_ptrs(ptrs);
        let ptrs = context.ptrs_cast_const(ptrs);
        let ptrs = unsafe { context.ptrs_add(ptrs, start) };
        (context, ptrs)
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> MutPtrs<'_, T> {
        let (_, ptrs) = self.as_mut_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_mut_ptrs_with_context(&mut self) -> (&T::Context, MutPtrs<'_, T>) {
        let Self {
            ref ptrs,
            buffer,
            start,
            ..
        } = *self;
        let context = unsafe { Self::context_of(buffer) };

        let ptrs = ptrs.clone().into_inner();
        let ptrs = context.nonnull_to_ptrs(ptrs);
        let ptrs = unsafe { context.ptrs_add_mut(ptrs, start) };
        (context, ptrs)
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> SlicePtrs<'_, T> {
        let (_, slices) = self.as_slice_ptrs_with_context();
        slices
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(&self) -> (&T::Context, SlicePtrs<'_, T>) {
        let Self {
            ref ptrs,
            buffer,
            start,
            ..
        } = *self;
        let context = unsafe { Self::context_of(buffer) };

        let len = self.len();
        let ptrs = ptrs.clone().into_inner();
        let ptrs = context.nonnull_to_ptrs(ptrs);
        let ptrs = context.ptrs_cast_const(ptrs);
        let ptrs = unsafe { context.ptrs_add(ptrs, start) };
        let slices = context.slice_ptrs_from_raw_parts(ptrs, len);
        (context, slices)
    }

    #[inline]
    pub fn as_slice_mut_ptrs(&mut self) -> SliceMutPtrs<'_, T> {
        let (_, slices) = self.as_slice_mut_ptrs_with_context();
        slices
    }

    #[inline]
    pub fn as_slice_mut_ptrs_with_context(&mut self) -> (&T::Context, SliceMutPtrs<'_, T>) {
        let Self {
            ref ptrs,
            buffer,
            start,
            ..
        } = *self;
        let context = unsafe { Self::context_of(buffer) };

        let len = self.len();
        let ptrs = ptrs.clone().into_inner();
        let ptrs = context.nonnull_to_ptrs(ptrs);
        let ptrs = unsafe { context.ptrs_add_mut(ptrs, start) };
        let slices = context.slice_mut_ptrs_from_raw_parts(ptrs, len);
        (context, slices)
    }

    #[inline]
    unsafe fn context_of<'a>(buffer: NonNull<BufferData<T>>) -> &'a T::Context {
        let buffer = buffer.as_ptr();
        unsafe { buffer.context() }
    }

    #[inline]
    unsafe fn post_inc_start<'a>(
        start: &mut usize,
        ptrs: Ptrs<'a, T>,
        context: &'a T::Context,
        offset: usize,
    ) -> Ptrs<'a, T> {
        let old_start = *start;
        *start += offset;
        unsafe { context.ptrs_add(ptrs, old_start) }
    }

    #[inline]
    unsafe fn pre_dec_end<'a>(
        end: &mut usize,
        ptrs: Ptrs<'a, T>,
        context: &'a T::Context,
        offset: usize,
    ) -> Ptrs<'a, T> {
        *end -= offset;
        unsafe { context.ptrs_add(ptrs, *end) }
    }
}

impl<T> IntoIter<T>
where
    T: Soa + ?Sized,
{
    #[inline]
    pub fn as_slices(&self) -> T::Slices<'_, '_> {
        let (_, slices) = self.as_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_slices_with_context(&self) -> (&T::Context, T::Slices<'_, '_>) {
        let (context, slices) = self.as_slice_ptrs_with_context();
        let slices = unsafe { T::slice_ptrs_to_slices(context, slices) };
        (context, slices)
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> T::SlicesMut<'_, '_> {
        let (_, slices) = self.as_mut_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_mut_slices_with_context(&mut self) -> (&T::Context, T::SlicesMut<'_, '_>) {
        let (context, slices) = self.as_slice_mut_ptrs_with_context();
        let slices = unsafe { T::slice_mut_ptrs_to_slices(context, slices) };
        (context, slices)
    }
}

unsafe impl<T> Send for IntoIter<T>
where
    T: RawSoa + ?Sized,
    T::Context: Send,
    T::Fields: Send,
{
}

unsafe impl<T> Sync for IntoIter<T>
where
    T: RawSoa + ?Sized,
    T::Context: Sync,
    T::Fields: Sync,
{
}

impl<T, U> AsRef<[U]> for IntoIter<T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Into<&'any [U]>,
{
    #[inline]
    fn as_ref(&self) -> &[U] {
        self.as_slices().into()
    }
}

impl<T> Debug for IntoIter<T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("IntoIter").field(&slices).finish()
    }
}

impl<T> Default for IntoIter<T>
where
    T: RawSoa + ?Sized,
    T::Context: Default,
{
    #[inline]
    fn default() -> Self {
        let vec = SoaVec::new();
        Self::new(vec)
    }
}

impl<T> Drop for IntoIter<T>
where
    T: RawSoa + ?Sized,
{
    fn drop(&mut self) {
        struct DropGuard<'a, T>(&'a mut IntoIter<T>)
        where
            T: RawSoa + ?Sized;

        impl<T> Drop for DropGuard<'_, T>
        where
            T: RawSoa + ?Sized,
        {
            fn drop(&mut self) {
                let Self(iter) = self;
                let IntoIter {
                    buffer, capacity, ..
                } = **iter;

                unsafe {
                    // `IntoIter::alloc` is not used anymore after this and will be dropped by RawVec
                    // let alloc = ManuallyDrop::take(&mut self.0.alloc);

                    // RawVec handles deallocation
                    let _ = RawSoaVec::<T>::from_nonnull(buffer, capacity);
                }
            }
        }

        let mut guard = DropGuard(self);

        // destroy the remaining elements
        let DropGuard(iter) = &mut guard;
        if iter.is_empty() {
            return;
        }

        let Self { buffer, .. } = **iter;
        let context = unsafe { Self::context_of(buffer) };

        let slices = iter.as_slice_mut_ptrs();
        unsafe { context.slices_drop_in_place(slices) }
        // now `guard` will be dropped and do the rest
    }
}

#[expect(clippy::while_let_on_iterator)]
impl<T> Iterator for IntoIter<T>
where
    T: SoaRead,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if Self::is_empty(self) {
            return None;
        }

        let Self {
            buffer,
            ref ptrs,
            ref mut start,
            ..
        } = *self;
        let context = unsafe { Self::context_of(buffer) };

        let ptrs = ptrs.clone().into_inner();
        let ptrs = context.nonnull_to_ptrs(ptrs);
        let ptrs = context.ptrs_cast_const(ptrs);
        let ptrs = unsafe { Self::post_inc_start(start, ptrs, context, 1) };

        let item = unsafe { T::read(context, ptrs) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.len() {
            self.start = self.end;
            return None;
        }

        let Self {
            buffer,
            ref ptrs,
            ref mut start,
            ..
        } = *self;
        let context = unsafe { Self::context_of(buffer) };

        let ptrs = ptrs.clone().into_inner();
        let ptrs = context.nonnull_to_ptrs(ptrs);
        let ptrs = context.ptrs_cast_const(ptrs);
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
        if Self::is_empty(&self) {
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
            start,
            end,
            ..
        } = self;
        let context = self.context();
        let mut acc = init;
        let mut i = start;
        loop {
            // SAFETY: the loop iterates `i in start..end`, which always is in bounds of
            // the slice allocation
            let ptrs = ptrs.clone().into_inner();
            let ptrs = context.nonnull_to_ptrs(ptrs);
            let ptrs = context.ptrs_cast_const(ptrs);
            let ptrs = unsafe { context.ptrs_add(ptrs, i) };
            let item = unsafe { T::read(context, ptrs) };
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
        let n = self.len();
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
        let n = self.len();
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

impl<T> DoubleEndedIterator for IntoIter<T>
where
    T: SoaRead,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if Self::is_empty(self) {
            return None;
        }

        let Self {
            buffer,
            ref ptrs,
            ref mut end,
            ..
        } = *self;
        let context = unsafe { Self::context_of(buffer) };

        let ptrs = ptrs.clone().into_inner();
        let ptrs = context.nonnull_to_ptrs(ptrs);
        let ptrs = context.ptrs_cast_const(ptrs);
        let ptrs = unsafe { Self::pre_dec_end(end, ptrs, context, 1) };

        let item = unsafe { T::read(context, ptrs) };
        Some(item)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.len() {
            self.end = self.start;
            return None;
        }

        let Self {
            buffer,
            ref ptrs,
            ref mut end,
            ..
        } = *self;
        let context = unsafe { Self::context_of(buffer) };

        let ptrs = ptrs.clone().into_inner();
        let ptrs = context.nonnull_to_ptrs(ptrs);
        let ptrs = context.ptrs_cast_const(ptrs);
        unsafe {
            Self::pre_dec_end(end, ptrs, context, n);
        }
        self.next_back()
    }
}

impl<T> ExactSizeIterator for IntoIter<T>
where
    T: SoaRead,
{
    #[inline]
    fn len(&self) -> usize {
        self.len()
    }
}

impl<T> FusedIterator for IntoIter<T> where T: SoaRead {}
