use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    ptr::NonNull,
    slice,
};

use crate::ptr::{ptrs, BufferAlign};

use super::SoaSlice;

pub struct Iter<'a, T, U, V> {
    ptr: NonNull<BufferAlign<T, U, V>>,
    capacity: usize,
    start: usize,
    end: usize,
    phantom: PhantomData<(&'a T, &'a U, &'a V)>,
}

impl<'a, T, U, V> Iter<'a, T, U, V> {
    #[inline]
    pub(super) const fn new(slice: &'a SoaSlice<T, U, V>) -> Self {
        let ptr = slice.as_ptr().cast_mut();
        let ptr = unsafe { NonNull::new_unchecked(ptr) }.cast();
        Self {
            ptr,
            capacity: slice.capacity(),
            start: 0,
            end: slice.len(),
            phantom: PhantomData,
        }
    }

    pub const fn len(&self) -> usize {
        self.end - self.start
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn ptrs(&self) -> (*const T, *const U, *const V) {
        let ptr = self.ptr.as_ptr().cast();
        let len = self.capacity;

        unsafe {
            let (t_ptr, u_ptr, v_ptr) = ptrs::<T, U, V>(ptr, len);
            (t_ptr, u_ptr, v_ptr)
        }
    }

    pub fn as_slices(&self) -> (&'a [T], &'a [U], &'a [V]) {
        let (t_ptr, u_ptr, v_ptr) = self.ptrs();
        let len = self.len();

        unsafe {
            let t_slice = slice::from_raw_parts(t_ptr.add(self.start), len);
            let u_slice = slice::from_raw_parts(u_ptr.add(self.start), len);
            let v_slice = slice::from_raw_parts(v_ptr.add(self.start), len);
            (t_slice, u_slice, v_slice)
        }
    }

    unsafe fn post_inc_start(&mut self, offset: usize) -> (NonNull<T>, NonNull<U>, NonNull<V>) {
        let (t_ptr, u_ptr, v_ptr) = self.ptrs();
        let t_ptr = unsafe { NonNull::new_unchecked(t_ptr.cast_mut()).add(self.start) };
        let u_ptr = unsafe { NonNull::new_unchecked(u_ptr.cast_mut()).add(self.start) };
        let v_ptr = unsafe { NonNull::new_unchecked(v_ptr.cast_mut()).add(self.start) };

        self.start += offset;
        (t_ptr, u_ptr, v_ptr)
    }

    unsafe fn pre_dec_end(&mut self, offset: usize) -> (NonNull<T>, NonNull<U>, NonNull<V>) {
        self.end -= offset;

        let (t_ptr, u_ptr, v_ptr) = self.ptrs();
        let t_ptr = unsafe { NonNull::new_unchecked(t_ptr.cast_mut()).add(self.end) };
        let u_ptr = unsafe { NonNull::new_unchecked(u_ptr.cast_mut()).add(self.end) };
        let v_ptr = unsafe { NonNull::new_unchecked(v_ptr.cast_mut()).add(self.end) };
        (t_ptr, u_ptr, v_ptr)
    }
}

unsafe impl<T, U, V> Send for Iter<'_, T, U, V>
where
    T: Send,
    U: Send,
    V: Send,
{
}

unsafe impl<T, U, V> Sync for Iter<'_, T, U, V>
where
    T: Sync,
    U: Sync,
    V: Sync,
{
}

impl<T, U, V> Debug for Iter<'_, T, U, V>
where
    T: Debug,
    U: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (t_slice, u_slice, v_slice) = self.as_slices();
        f.debug_struct("Iter")
            .field("t_slice", &t_slice)
            .field("u_slice", &u_slice)
            .field("v_slice", &v_slice)
            .finish()
    }
}

impl<T, U, V> Default for Iter<'_, T, U, V> {
    fn default() -> Self {
        let slice = Default::default();
        Self::new(slice)
    }
}

impl<T, U, V> Clone for Iter<'_, T, U, V> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            capacity: self.capacity,
            start: self.start,
            end: self.end,
            phantom: self.phantom,
        }
    }
}

#[allow(clippy::while_let_on_iterator)]
impl<'a, T, U, V> Iterator for Iter<'a, T, U, V> {
    type Item = (&'a T, &'a U, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if Iter::is_empty(self) {
            return None;
        }

        unsafe {
            let (t_ptr, u_ptr, v_ptr) = self.post_inc_start(1);
            Some((t_ptr.as_ref(), u_ptr.as_ref(), v_ptr.as_ref()))
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
        if n >= Iter::len(self) {
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
        if Iter::is_empty(&self) {
            return init;
        }
        let mut acc = init;
        let mut i = 0;
        let len = self.len();
        loop {
            // SAFETY: the loop iterates `i in 0..len`, which always is in bounds of
            // the slice allocation
            let (t_ptr, u_ptr, v_ptr) = self.ptrs();
            let item = unsafe { (&*t_ptr.add(i), &*u_ptr.add(i), &*v_ptr.add(i)) };
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

impl<'a, T, U, V> DoubleEndedIterator for Iter<'a, T, U, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if Iter::is_empty(self) {
            return None;
        }

        unsafe {
            let (t_ptr, u_ptr, v_ptr) = self.pre_dec_end(1);
            Some((t_ptr.as_ref(), u_ptr.as_ref(), v_ptr.as_ref()))
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

impl<T, U, V> ExactSizeIterator for Iter<'_, T, U, V> {
    fn len(&self) -> usize {
        Iter::len(self)
    }
}

impl<T, U, V> FusedIterator for Iter<'_, T, U, V> {}

pub struct IterMut<'a, T, U, V> {
    ptr: NonNull<BufferAlign<T, U, V>>,
    capacity: usize,
    start: usize,
    end: usize,
    phantom: PhantomData<(&'a mut T, &'a mut U, &'a mut V)>,
}

impl<'a, T, U, V> IterMut<'a, T, U, V> {
    #[inline]
    pub(super) fn new(slice: &'a mut SoaSlice<T, U, V>) -> Self {
        let ptr = slice.as_mut_ptr();
        let ptr = unsafe { NonNull::new_unchecked(ptr) }.cast();
        Self {
            ptr,
            capacity: slice.capacity(),
            start: 0,
            end: slice.len(),
            phantom: PhantomData,
        }
    }

    pub const fn len(&self) -> usize {
        self.end - self.start
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn ptrs(&self) -> (*mut T, *mut U, *mut V) {
        let ptr = self.ptr.as_ptr().cast();
        let len = self.capacity;

        unsafe { ptrs::<T, U, V>(ptr, len) }
    }

    pub fn into_slices(self) -> (&'a mut [T], &'a mut [U], &'a mut [V]) {
        let (t_ptr, u_ptr, v_ptr) = self.ptrs();
        let len = self.len();

        unsafe {
            let t_slice = slice::from_raw_parts_mut(t_ptr.add(self.start), len);
            let u_slice = slice::from_raw_parts_mut(u_ptr.add(self.start), len);
            let v_slice = slice::from_raw_parts_mut(v_ptr.add(self.start), len);
            (t_slice, u_slice, v_slice)
        }
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

unsafe impl<T, U, V> Send for IterMut<'_, T, U, V>
where
    T: Send,
    U: Send,
    V: Send,
{
}

unsafe impl<T, U, V> Sync for IterMut<'_, T, U, V>
where
    T: Sync,
    U: Sync,
    V: Sync,
{
}

impl<T, U, V> Debug for IterMut<'_, T, U, V>
where
    T: Debug,
    U: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (t_slice, u_slice, v_slice) = self.as_slices();
        f.debug_struct("IterMut")
            .field("t_slice", &t_slice)
            .field("u_slice", &u_slice)
            .field("v_slice", &v_slice)
            .finish()
    }
}

impl<T, U, V> Default for IterMut<'_, T, U, V> {
    fn default() -> Self {
        let slice = Default::default();
        Self::new(slice)
    }
}

#[allow(clippy::while_let_on_iterator)]
impl<'a, T, U, V> Iterator for IterMut<'a, T, U, V> {
    type Item = (&'a mut T, &'a mut U, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        if IterMut::is_empty(self) {
            return None;
        }

        unsafe {
            let (mut t_ptr, mut u_ptr, mut v_ptr) = self.post_inc_start(1);
            Some((t_ptr.as_mut(), u_ptr.as_mut(), v_ptr.as_mut()))
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
        if n >= IterMut::len(self) {
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
        if IterMut::is_empty(&self) {
            return init;
        }
        let mut acc = init;
        let mut i = 0;
        let len = self.len();
        loop {
            // SAFETY: the loop iterates `i in 0..len`, which always is in bounds of
            // the slice allocation
            let (t_ptr, u_ptr, v_ptr) = self.ptrs();
            let item = unsafe { (&mut *t_ptr.add(i), &mut *u_ptr.add(i), &mut *v_ptr.add(i)) };
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

impl<'a, T, U, V> DoubleEndedIterator for IterMut<'a, T, U, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if IterMut::is_empty(self) {
            return None;
        }

        unsafe {
            let (mut t_ptr, mut u_ptr, mut v_ptr) = self.pre_dec_end(1);
            Some((t_ptr.as_mut(), u_ptr.as_mut(), v_ptr.as_mut()))
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

impl<T, U, V> ExactSizeIterator for IterMut<'_, T, U, V> {
    fn len(&self) -> usize {
        IterMut::len(self)
    }
}

impl<T, U, V> FusedIterator for IterMut<'_, T, U, V> {}
