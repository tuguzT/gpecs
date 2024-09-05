use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::ManuallyDrop,
    ptr::{self, NonNull},
    slice,
};

use crate::{
    ptr::{ptrs, to_len, BufferAlign},
    raw_vec::RawSoaVec,
    vec::SoaVec,
};

pub struct IntoIter<T, U, V> {
    buffer: NonNull<BufferAlign<T, U, V>>,
    capacity_in_bytes: usize,
    start: usize,
    end: usize,
}

impl<T, U, V> IntoIter<T, U, V> {
    pub(super) fn new(vec: SoaVec<T, U, V>) -> Self {
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

    pub fn as_slices(&self) -> (&[T], &[U], &[V]) {
        let (t_ptr, u_ptr, v_ptr) = self.ptrs();
        let len = self.len();

        unsafe {
            let t_slice = slice::from_raw_parts(t_ptr.add(self.start), len);
            let u_slice = slice::from_raw_parts(u_ptr.add(self.start), len);
            let v_slice = slice::from_raw_parts(v_ptr.add(self.start), len);
            (t_slice, u_slice, v_slice)
        }
    }

    pub fn as_mut_slices(&mut self) -> (&mut [T], &mut [U], &mut [V]) {
        let (t_ptr, u_ptr, v_ptr) = self.ptrs();
        let len = self.len();

        unsafe {
            let t_slice = slice::from_raw_parts_mut(t_ptr.add(self.start), len);
            let u_slice = slice::from_raw_parts_mut(u_ptr.add(self.start), len);
            let v_slice = slice::from_raw_parts_mut(v_ptr.add(self.start), len);
            (t_slice, u_slice, v_slice)
        }
    }

    fn ptrs(&self) -> (*mut T, *mut U, *mut V) {
        let ptr = self.buffer.as_ptr().cast();
        let len = to_len::<T, U, V>(self.capacity_in_bytes);

        unsafe { ptrs::<T, U, V>(ptr, len) }
    }

    unsafe fn post_inc_start(&mut self, offset: usize) -> (NonNull<T>, NonNull<U>, NonNull<V>) {
        let (t_ptr, u_ptr, v_ptr) = self.ptrs();
        let t_ptr = unsafe { NonNull::new_unchecked(t_ptr).add(self.start) };
        let u_ptr = unsafe { NonNull::new_unchecked(u_ptr).add(self.start) };
        let v_ptr = unsafe { NonNull::new_unchecked(v_ptr).add(self.start) };

        self.start += offset;
        (t_ptr, u_ptr, v_ptr)
    }

    unsafe fn pre_dec_end(&mut self, offset: usize) -> (NonNull<T>, NonNull<U>, NonNull<V>) {
        self.end -= offset;

        let (t_ptr, u_ptr, v_ptr) = self.ptrs();
        let t_ptr = unsafe { NonNull::new_unchecked(t_ptr).add(self.end) };
        let u_ptr = unsafe { NonNull::new_unchecked(u_ptr).add(self.end) };
        let v_ptr = unsafe { NonNull::new_unchecked(v_ptr).add(self.end) };
        (t_ptr, u_ptr, v_ptr)
    }
}

unsafe impl<T, U, V> Send for IntoIter<T, U, V>
where
    T: Send,
    U: Send,
    V: Send,
{
}

unsafe impl<T, U, V> Sync for IntoIter<T, U, V>
where
    T: Sync,
    U: Sync,
    V: Sync,
{
}

impl<T, U, V> Debug for IntoIter<T, U, V>
where
    T: Debug,
    U: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (t_slice, u_slice, v_slice) = self.as_slices();
        f.debug_struct("IntoIter")
            .field("t_slice", &t_slice)
            .field("u_slice", &u_slice)
            .field("v_slice", &v_slice)
            .finish()
    }
}

impl<T, U, V> Default for IntoIter<T, U, V> {
    fn default() -> Self {
        let slice = Default::default();
        Self::new(slice)
    }
}

impl<T, U, V> Drop for IntoIter<T, U, V> {
    fn drop(&mut self) {
        struct DropGuard<'a, T, U, V>(&'a mut IntoIter<T, U, V>);

        impl<T, U, V> Drop for DropGuard<'_, T, U, V> {
            fn drop(&mut self) {
                unsafe {
                    // `IntoIter::alloc` is not used anymore after this and will be dropped by RawVec
                    // let alloc = ManuallyDrop::take(&mut self.0.alloc);
                    // RawVec handles deallocation
                    let _ = RawSoaVec::<T, U, V>::from_nonnull_capacity_in_bytes(
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
        let (t_slice, u_slice, v_slice) = guard.0.as_mut_slices();
        unsafe {
            ptr::drop_in_place(t_slice);
            ptr::drop_in_place(u_slice);
            ptr::drop_in_place(v_slice);
        }
        // now `guard` will be dropped and do the rest
    }
}

#[allow(clippy::while_let_on_iterator)]
impl<T, U, V> Iterator for IntoIter<T, U, V> {
    type Item = (T, U, V);

    fn next(&mut self) -> Option<Self::Item> {
        if IntoIter::is_empty(self) {
            return None;
        }

        unsafe {
            let (t_ptr, u_ptr, v_ptr) = self.post_inc_start(1);
            Some((t_ptr.read(), u_ptr.read(), v_ptr.read()))
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
            let (t_ptr, u_ptr, v_ptr) = self.ptrs();
            let item = unsafe {
                (
                    t_ptr.add(i).read(),
                    u_ptr.add(i).read(),
                    v_ptr.add(i).read(),
                )
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

impl<T, U, V> DoubleEndedIterator for IntoIter<T, U, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if IntoIter::is_empty(self) {
            return None;
        }

        unsafe {
            let (t_ptr, u_ptr, v_ptr) = self.pre_dec_end(1);
            Some((t_ptr.read(), u_ptr.read(), v_ptr.read()))
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

impl<T, U, V> ExactSizeIterator for IntoIter<T, U, V> {
    fn len(&self) -> usize {
        IntoIter::len(self)
    }
}

impl<T, U, V> FusedIterator for IntoIter<T, U, V> {}
