use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::ManuallyDrop,
    ptr::NonNull,
};

use crate::{
    ptr::{ptrs, to_capacity, BufferData},
    raw_vec::RawSoaVec,
    soa::Soa,
    vec::SoaVec,
};

pub struct IntoIter<T>
where
    T: Soa,
{
    buffer: NonNull<BufferData<T>>,
    capacity_in_bytes: usize,
    start: usize,
    end: usize,
}

impl<T> IntoIter<T>
where
    T: Soa,
{
    pub(super) fn new(vec: SoaVec<T>) -> Self {
        let vec = ManuallyDrop::new(vec);
        let buffer = vec.buffer.non_null().cast();
        Self {
            buffer,
            capacity_in_bytes: vec.capacity_in_bytes(),
            start: 0,
            end: vec.len(),
        }
    }

    pub const fn len(&self) -> usize {
        self.end - self.start
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn as_slices(&self) -> T::Slices<'_> {
        let ptrs = T::ptrs_cast_const(self.ptrs());
        let len = self.len();

        unsafe {
            let ptrs = T::ptrs_add(ptrs, self.start);
            let slices = T::slices_from_raw_parts(ptrs, len);
            T::slices_as_refs(slices)
        }
    }

    pub fn as_mut_slices(&mut self) -> T::SlicesMut<'_> {
        let ptrs = self.ptrs();
        let len = self.len();

        unsafe {
            let ptrs = T::ptrs_add_mut(ptrs, self.start);
            let slices = T::slices_from_raw_parts_mut(ptrs, len);
            T::mut_slices_as_refs(slices)
        }
    }

    fn ptrs(&self) -> T::MutPtrs {
        let ptr = self.buffer.as_ptr().cast();
        let capacity = to_capacity::<T>(self.capacity_in_bytes);

        unsafe { ptrs::<T>(ptr, capacity) }
    }

    unsafe fn post_inc_start(&mut self, offset: usize) -> T::Ptrs {
        let ptrs = T::ptrs_cast_const(self.ptrs());
        let ptrs = unsafe { T::ptrs_add(ptrs, self.start) };

        self.start += offset;
        ptrs
    }

    unsafe fn pre_dec_end(&mut self, offset: usize) -> T::Ptrs {
        self.end -= offset;

        let ptrs = T::ptrs_cast_const(self.ptrs());
        unsafe { T::ptrs_add(ptrs, self.end) }
    }
}

unsafe impl<T> Send for IntoIter<T> where T: Soa + Send {}
unsafe impl<T> Sync for IntoIter<T> where T: Soa + Sync {}

impl<T> Debug for IntoIter<T>
where
    T: Soa,
    for<'any> T::Slices<'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("IntoIter").field(&slices).finish()
    }
}

impl<T> Default for IntoIter<T>
where
    T: Soa,
{
    fn default() -> Self {
        let slice = Default::default();
        Self::new(slice)
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
                unsafe {
                    // `IntoIter::alloc` is not used anymore after this and will be dropped by RawVec
                    // let alloc = ManuallyDrop::take(&mut self.0.alloc);
                    // RawVec handles deallocation
                    let _ = RawSoaVec::<T>::from_nonnull_capacity_in_bytes(
                        self.0.buffer.cast(),
                        self.0.capacity_in_bytes,
                    );
                }
            }
        }

        let guard = DropGuard(self);
        // destroy the remaining elements
        if IntoIter::is_empty(guard.0) {
            return;
        }
        let slices = guard.0.as_mut_slices();
        let slices = T::mut_slice_refs_as_ptrs(slices);
        unsafe { T::slices_drop_in_place(slices) }
        // now `guard` will be dropped and do the rest
    }
}

#[allow(clippy::while_let_on_iterator)]
impl<T> Iterator for IntoIter<T>
where
    T: Soa,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if IntoIter::is_empty(self) {
            return None;
        }

        unsafe {
            let ptrs = self.post_inc_start(1);
            let item = T::ptrs_read(ptrs);
            Some(item)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n >= IntoIter::len(self) {
            self.start = self.end;
            return None;
        }

        unsafe {
            self.post_inc_start(n);
        }
        self.next()
    }

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
        // this implementation consists of the following optimizations compared to the
        // default implementation:
        // - do-while loop, as is llvm's preferred loop shape,
        //   see https://releases.llvm.org/16.0.0/docs/LoopTerminology.html#more-canonical-loops
        // - bumps an index instead of a pointer since the latter case inhibits
        //   some optimizations, see #111603
        // - avoids Option wrapping/matching
        if IntoIter::is_empty(&self) {
            return init;
        }
        let mut acc = init;
        let mut i = 0;
        let len = self.len();
        loop {
            // SAFETY: the loop iterates `i in 0..len`, which always is in bounds of
            // the slice allocation
            let ptrs = T::ptrs_cast_const(self.ptrs());
            let item = unsafe {
                let ptrs = T::ptrs_add(ptrs, i);
                T::ptrs_read(ptrs)
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

    fn for_each<F>(mut self, mut f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        while let Some(x) = self.next() {
            f(x);
        }
    }

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
    T: Soa,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if IntoIter::is_empty(self) {
            return None;
        }

        unsafe {
            let ptrs = self.pre_dec_end(1);
            let item = T::ptrs_read(ptrs);
            Some(item)
        }
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.len() {
            self.end = self.start;
            return None;
        }

        unsafe {
            self.pre_dec_end(n);
        }
        self.next_back()
    }
}

impl<T> ExactSizeIterator for IntoIter<T>
where
    T: Soa,
{
    fn len(&self) -> usize {
        IntoIter::len(self)
    }
}

impl<T> FusedIterator for IntoIter<T> where T: Soa {}
