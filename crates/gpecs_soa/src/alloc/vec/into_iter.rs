use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::{ManuallyDrop, transmute},
    ptr::NonNull,
};

use crate::{
    alloc::raw_vec::RawSoaVec,
    ptr::{BufferData, BufferDataPtr},
    traits::{NonNullPtrs, Soa},
    vec::SoaVec,
};

pub struct IntoIter<T>
where
    T: Soa,
{
    buffer: NonNull<BufferData<T>>,
    capacity: usize,
    ptrs: NonNullPtrs<'static, T>,
    start: usize,
    end: usize,
}

impl<T> IntoIter<T>
where
    T: Soa,
{
    #[inline]
    pub(super) fn new(vec: SoaVec<T>) -> Self {
        let vec = ManuallyDrop::new(vec);

        let buffer = vec.buffer.ptr();
        let context = vec.context();
        let ptrs = unsafe { transmute(T::ptrs_to_nonnull(context, vec.buffer.ptrs())) };
        Self {
            buffer: unsafe { NonNull::new_unchecked(buffer) },
            capacity: vec.capacity(),
            ptrs: NonNullPtrs::new(ptrs),
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
    unsafe fn context_of<'a>(buffer: NonNull<BufferData<T>>) -> &'a T::Context {
        let buffer = buffer.as_ptr().cast_const();
        unsafe { &*buffer.ptr_to_context() }
    }

    #[inline]
    pub fn as_slices(&self) -> T::Slices<'_, '_> {
        let Self { start, .. } = *self;
        let context = self.context();
        let ptrs = T::ptrs_cast_const(context, self.ptrs());
        let len = self.len();

        unsafe {
            let ptrs = T::ptrs_add(context, ptrs, start);
            let slices = T::slices_from_raw_parts(context, ptrs, len);
            T::slice_ptrs_to_slices(context, slices)
        }
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> T::SlicesMut<'_, '_> {
        let Self { start, .. } = *self;
        let context = self.context();
        let ptrs = self.ptrs();
        let len = self.len();

        unsafe {
            let ptrs = T::ptrs_add_mut(context, ptrs, start);
            let slices = T::slices_from_raw_parts_mut(context, ptrs, len);
            T::slice_mut_ptrs_to_slices(context, slices)
        }
    }

    #[inline]
    fn ptrs(&self) -> T::MutPtrs<'_> {
        let Self { ptrs, .. } = self;
        let context = self.context();
        let ptrs = ptrs.clone().into_inner();
        T::nonnull_to_ptrs(context, ptrs)
    }

    #[inline]
    unsafe fn post_inc_start<'a>(
        start: &mut usize,
        ptrs: T::Ptrs<'a>,
        context: &'a T::Context,
        offset: usize,
    ) -> T::Ptrs<'a> {
        let old_start = *start;
        *start += offset;

        unsafe { T::ptrs_add(context, ptrs, old_start) }
    }

    #[inline]
    unsafe fn pre_dec_end<'a>(
        end: &mut usize,
        ptrs: T::Ptrs<'a>,
        context: &'a T::Context,
        offset: usize,
    ) -> T::Ptrs<'a> {
        *end -= offset;

        unsafe { T::ptrs_add(context, ptrs, *end) }
    }
}

unsafe impl<T> Send for IntoIter<T>
where
    T: Soa,
    T::Fields: Send,
    T::Context: Send,
{
}

unsafe impl<T> Sync for IntoIter<T>
where
    T: Soa,
    T::Fields: Sync,
    T::Context: Sync,
{
}

impl<T, U> AsRef<[U]> for IntoIter<T>
where
    for<'c, 'any> T: Soa<Slices<'c, 'any> = &'any [U]> + 'any,
{
    fn as_ref(&self) -> &[U] {
        self.as_slices()
    }
}

impl<T> Debug for IntoIter<T>
where
    T: Soa,
    for<'c, 'any> T::Slices<'c, 'any>: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("IntoIter").field(&slices).finish()
    }
}

impl<T> Default for IntoIter<T>
where
    T: Soa,
    T::Context: Default,
{
    #[inline]
    fn default() -> Self {
        let vec = Default::default();
        Self::new(vec)
    }
}

impl<T> Drop for IntoIter<T>
where
    T: Soa,
{
    fn drop(&mut self) {
        struct DropGuard<'a, T>(&'a mut IntoIter<T>)
        where
            T: Soa;

        impl<T> Drop for DropGuard<'_, T>
        where
            T: Soa,
        {
            fn drop(&mut self) {
                let &mut Self(&mut IntoIter {
                    buffer, capacity, ..
                }) = self;
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
        if IntoIter::is_empty(iter) {
            return;
        }

        let &mut &mut Self { buffer, .. } = iter;
        let context = unsafe { Self::context_of(buffer) };

        let slices = iter.as_mut_slices();
        let slices = T::slices_mut_as_slice_ptrs(context, slices);
        unsafe { T::slices_drop_in_place(context, slices) }
        // now `guard` will be dropped and do the rest
    }
}

#[allow(clippy::while_let_on_iterator)]
impl<T> Iterator for IntoIter<T>
where
    T: Soa,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if IntoIter::is_empty(self) {
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
        let ptrs = T::ptrs_cast_const(context, T::nonnull_to_ptrs(context, ptrs));
        let ptrs = unsafe { Self::post_inc_start(start, ptrs, context, 1) };

        let item = unsafe { T::ptrs_read(context, ptrs) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = IntoIter::len(self);
        (len, Some(len))
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        IntoIter::len(&self)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n >= IntoIter::len(self) {
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
        let ptrs = T::ptrs_cast_const(context, T::nonnull_to_ptrs(context, ptrs));
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
        if IntoIter::is_empty(&self) {
            return init;
        }

        // this implementation consists of the following optimizations compared to the
        // default implementation:
        // - do-while loop, as is llvm's preferred loop shape,
        //   see https://releases.llvm.org/16.0.0/docs/LoopTerminology.html#more-canonical-loops
        // - bumps an index instead of a pointer since the latter case inhibits
        //   some optimizations, see #111603
        // - avoids Option wrapping/matching
        let len = self.len();
        let ptrs = self.ptrs();
        let context = self.context();
        let mut acc = init;
        let mut i = 0;
        loop {
            // SAFETY: the loop iterates `i in 0..len`, which always is in bounds of
            // the slice allocation
            let ptrs = T::ptrs_cast_const(context, ptrs.clone());
            let item = unsafe {
                let ptrs = T::ptrs_add(context, ptrs, i);
                T::ptrs_read(context, ptrs)
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
        let n = IntoIter::len(self);
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
        let n = IntoIter::len(self);
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
    T: Soa,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if IntoIter::is_empty(self) {
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
        let ptrs = T::ptrs_cast_const(context, T::nonnull_to_ptrs(context, ptrs));
        let ptrs = unsafe { Self::pre_dec_end(end, ptrs, context, 1) };

        let item = unsafe { T::ptrs_read(context, ptrs) };
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
        let ptrs = T::ptrs_cast_const(context, T::nonnull_to_ptrs(context, ptrs));
        unsafe {
            Self::pre_dec_end(end, ptrs, context, n);
        }
        self.next_back()
    }
}

impl<T> ExactSizeIterator for IntoIter<T>
where
    T: Soa,
{
    #[inline]
    fn len(&self) -> usize {
        IntoIter::len(self)
    }
}

impl<T> FusedIterator for IntoIter<T> where T: Soa {}
