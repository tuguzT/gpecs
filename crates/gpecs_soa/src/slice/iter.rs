use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    ptr::NonNull,
};

use crate::{
    ptr::{ptrs, BufferData},
    soa::Soa,
};

use super::SoaSlice;

pub struct Iter<'a, T>
where
    T: Soa,
{
    ptr: NonNull<BufferData<T>>,
    capacity: usize,
    start: usize,
    end: usize,
    phantom: PhantomData<&'a T>,
}

impl<'a, T> Iter<'a, T>
where
    T: Soa,
{
    #[inline]
    pub(super) fn new(slice: &'a SoaSlice<T>) -> Self {
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

    fn ptrs(&self) -> T::Ptrs {
        let ptr = self.ptr.as_ptr().cast();
        let len = self.capacity;

        unsafe {
            let ptrs = ptrs::<T>(ptr, len);
            T::ptrs_cast_const(ptrs)
        }
    }

    pub fn as_slices(&self) -> T::Slices<'_> {
        let ptrs = self.ptrs();
        let len = self.len();

        unsafe {
            let ptrs = T::ptrs_add(ptrs, self.start);
            let slices = T::slices_from_raw_parts(ptrs, len);
            T::slices_as_refs(slices)
        }
    }

    unsafe fn post_inc_start(&mut self, offset: usize) -> T::MutPtrs {
        let ptrs = T::ptrs_cast_mut(self.ptrs());
        let ptrs = unsafe { T::ptrs_add_mut(ptrs, self.start) };

        self.start += offset;
        ptrs
    }

    unsafe fn pre_dec_end(&mut self, offset: usize) -> T::MutPtrs {
        self.end -= offset;

        let ptrs = T::ptrs_cast_mut(self.ptrs());
        unsafe { T::ptrs_add_mut(ptrs, self.end) }
    }
}

unsafe impl<T> Send for Iter<'_, T> where T: Soa + Send {}
unsafe impl<T> Sync for Iter<'_, T> where T: Soa + Sync {}

impl<T> Debug for Iter<'_, T>
where
    T: Soa,
    for<'any> T::Slices<'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("Iter").field(&slices).finish()
    }
}

impl<T> Default for Iter<'_, T>
where
    T: Soa,
{
    fn default() -> Self {
        let slice = Default::default();
        Self::new(slice)
    }
}

impl<T> Clone for Iter<'_, T>
where
    T: Soa,
{
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
impl<'a, T> Iterator for Iter<'a, T>
where
    T: Soa,
{
    type Item = T::Refs<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if Iter::is_empty(self) {
            return None;
        }

        unsafe {
            let ptrs = self.post_inc_start(1);
            let ptrs = T::ptrs_cast_const(ptrs);
            let refs = T::as_refs(ptrs);
            Some(refs)
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
            let ptrs = self.ptrs();
            let item = unsafe {
                let ptrs = T::ptrs_add(ptrs, i);
                T::as_refs(ptrs)
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

impl<T> DoubleEndedIterator for Iter<'_, T>
where
    T: Soa,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if Iter::is_empty(self) {
            return None;
        }

        unsafe {
            let ptrs = self.pre_dec_end(1);
            let ptrs = T::ptrs_cast_const(ptrs);
            let refs = T::as_refs(ptrs);
            Some(refs)
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

impl<T> ExactSizeIterator for Iter<'_, T>
where
    T: Soa,
{
    fn len(&self) -> usize {
        Iter::len(self)
    }
}

impl<T> FusedIterator for Iter<'_, T> where T: Soa {}

pub struct IterMut<'a, T>
where
    T: Soa,
{
    ptr: NonNull<BufferData<T>>,
    capacity: usize,
    start: usize,
    end: usize,
    phantom: PhantomData<&'a mut T>,
}

impl<'a, T> IterMut<'a, T>
where
    T: Soa,
{
    #[inline]
    pub(super) fn new(slice: &'a mut SoaSlice<T>) -> Self {
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

    fn ptrs(&self) -> T::MutPtrs {
        let ptr = self.ptr.as_ptr().cast();
        let len = self.capacity;

        unsafe { ptrs::<T>(ptr, len) }
    }

    pub fn into_slices(self) -> T::SlicesMut<'a> {
        let ptrs = self.ptrs();
        let len = self.len();

        unsafe {
            let ptrs = T::ptrs_add_mut(ptrs, self.start);
            let slices = T::slices_from_raw_parts_mut(ptrs, len);
            T::mut_slices_as_refs(slices)
        }
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

    unsafe fn post_inc_start(&mut self, offset: usize) -> T::MutPtrs {
        let ptrs = self.ptrs();
        let ptrs = unsafe { T::ptrs_add_mut(ptrs, self.start) };

        self.start += offset;
        ptrs
    }

    unsafe fn pre_dec_end(&mut self, offset: usize) -> T::MutPtrs {
        self.end -= offset;

        let ptrs = self.ptrs();
        unsafe { T::ptrs_add_mut(ptrs, self.end) }
    }
}

unsafe impl<T> Send for IterMut<'_, T> where T: Soa + Send {}
unsafe impl<T> Sync for IterMut<'_, T> where T: Soa + Sync {}

impl<T> Debug for IterMut<'_, T>
where
    T: Soa,
    for<'any> T::Slices<'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("IterMut").field(&slices).finish()
    }
}

impl<T> Default for IterMut<'_, T>
where
    T: Soa,
{
    fn default() -> Self {
        let slice = Default::default();
        Self::new(slice)
    }
}

#[allow(clippy::while_let_on_iterator)]
impl<'a, T> Iterator for IterMut<'a, T>
where
    T: Soa,
{
    type Item = T::RefsMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if IterMut::is_empty(self) {
            return None;
        }

        unsafe {
            let ptrs = self.post_inc_start(1);
            let refs = T::as_mut_refs(ptrs);
            Some(refs)
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
            let ptrs = self.ptrs();
            let item = unsafe {
                let ptrs = T::ptrs_add_mut(ptrs, i);
                T::as_mut_refs(ptrs)
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

impl<T> DoubleEndedIterator for IterMut<'_, T>
where
    T: Soa,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if IterMut::is_empty(self) {
            return None;
        }

        unsafe {
            let ptrs = self.pre_dec_end(1);
            let refs = T::as_mut_refs(ptrs);
            Some(refs)
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

impl<T> ExactSizeIterator for IterMut<'_, T>
where
    T: Soa,
{
    fn len(&self) -> usize {
        IterMut::len(self)
    }
}

impl<T> FusedIterator for IterMut<'_, T> where T: Soa {}
