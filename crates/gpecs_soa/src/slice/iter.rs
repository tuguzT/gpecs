use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use crate::traits::Soa;

use super::{SoaSlices, SoaSlicesMut};

pub struct Iter<'a, T>
where
    T: Soa + 'a,
{
    context: &'a T::Context,
    ptrs: T::NonNullPtrs,
    start: usize,
    end: usize,
}

impl<'a, T> Iter<'a, T>
where
    T: Soa,
{
    #[inline]
    pub(crate) fn new(slices: SoaSlices<'a, T>) -> Self {
        let (context, ptrs, len) = slices.into_parts();
        Self {
            context,
            ptrs: unsafe { T::ptrs_to_nonnull(context, T::ptrs_cast_mut(context, ptrs)) },
            start: 0,
            end: len,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn context(&self) -> &T::Context {
        self.context
    }

    fn ptrs(&self) -> T::Ptrs {
        let context = self.context();
        let ptrs = T::nonnull_to_ptrs(context, self.ptrs);
        T::ptrs_cast_const(context, ptrs)
    }

    #[inline]
    pub fn as_slices(&self) -> T::Slices<'a> {
        let context = self.context();
        let ptrs = self.ptrs();
        let len = self.len();

        unsafe {
            let ptrs = T::ptrs_add(context, ptrs, self.start);
            let slices = T::slices_from_raw_parts(context, ptrs, len);
            T::slice_ptrs_to_slices(context, slices)
        }
    }

    #[inline]
    unsafe fn post_inc_start(&mut self, offset: usize) -> T::MutPtrs {
        let context = self.context();
        let ptrs = T::ptrs_cast_mut(context, self.ptrs());
        let ptrs = unsafe { T::ptrs_add_mut(context, ptrs, self.start) };

        self.start += offset;
        ptrs
    }

    #[inline]
    unsafe fn pre_dec_end(&mut self, offset: usize) -> T::MutPtrs {
        self.end -= offset;

        let context = self.context();
        let ptrs = T::ptrs_cast_mut(context, self.ptrs());
        unsafe { T::ptrs_add_mut(context, ptrs, self.end) }
    }
}

unsafe impl<T> Send for Iter<'_, T> where T: Soa + Send {}
unsafe impl<T> Sync for Iter<'_, T> where T: Soa + Sync {}

impl<T, U> AsRef<[U]> for Iter<'_, T>
where
    for<'a> T: Soa<Slices<'a> = &'a [U]> + 'a,
{
    fn as_ref(&self) -> &[U] {
        self.as_slices()
    }
}

impl<T> Debug for Iter<'_, T>
where
    T: Soa,
    for<'any> T::Slices<'any>: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("Iter").field(&slices).finish()
    }
}

impl<T> Clone for Iter<'_, T>
where
    T: Soa,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            context: self.context,
            ptrs: self.ptrs,
            start: self.start,
            end: self.end,
        }
    }
}

#[allow(clippy::while_let_on_iterator)]
impl<'a, T> Iterator for Iter<'a, T>
where
    T: Soa,
{
    type Item = T::Refs<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if Iter::is_empty(self) {
            return None;
        }

        let ptrs = unsafe { self.post_inc_start(1) };

        let context = self.context();
        let ptrs = T::ptrs_cast_const(context, ptrs);

        let refs = unsafe { T::ptrs_to_refs(context, ptrs) };
        Some(refs)
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
        if n >= Iter::len(self) {
            self.start = self.end;
            return None;
        }

        unsafe {
            self.post_inc_start(n);
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
        let context = self.context();
        loop {
            // SAFETY: the loop iterates `i in 0..len`, which always is in bounds of
            // the slice allocation
            let ptrs = self.ptrs();
            let item = unsafe {
                let ptrs = T::ptrs_add(context, ptrs, i);
                T::ptrs_to_refs(context, ptrs)
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

impl<T> DoubleEndedIterator for Iter<'_, T>
where
    T: Soa,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if Iter::is_empty(self) {
            return None;
        }

        let ptrs = unsafe { self.pre_dec_end(1) };

        let context = self.context();
        let ptrs = T::ptrs_cast_const(context, ptrs);

        let refs = unsafe { T::ptrs_to_refs(context, ptrs) };
        Some(refs)
    }

    #[inline]
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
    #[inline]
    fn len(&self) -> usize {
        Iter::len(self)
    }
}

impl<T> FusedIterator for Iter<'_, T> where T: Soa {}

pub struct IterMut<'a, T>
where
    T: Soa + 'a,
{
    context: &'a T::Context,
    ptrs: T::NonNullPtrs,
    start: usize,
    end: usize,
}

impl<'a, T> IterMut<'a, T>
where
    T: Soa,
{
    #[inline]
    pub(super) fn new(slices: SoaSlicesMut<'a, T>) -> Self {
        let (context, ptrs, len) = slices.into_parts();
        Self {
            context,
            ptrs: unsafe { T::ptrs_to_nonnull(context, ptrs) },
            start: 0,
            end: len,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn context(&self) -> &T::Context {
        self.context
    }

    fn ptrs(&self) -> T::MutPtrs {
        let context = self.context();
        T::nonnull_to_ptrs(context, self.ptrs)
    }

    #[inline]
    pub fn into_slices(self) -> T::SlicesMut<'a> {
        let context = self.context();
        let ptrs = self.ptrs();
        let len = self.len();

        unsafe {
            let ptrs = T::ptrs_add_mut(context, ptrs, self.start);
            let slices = T::slices_from_raw_parts_mut(context, ptrs, len);
            T::slice_ptrs_to_slices_mut(context, slices)
        }
    }

    #[inline]
    pub fn as_slices(&self) -> T::Slices<'_> {
        let context = self.context();
        let ptrs = T::ptrs_cast_const(context, self.ptrs());
        let len = self.len();

        unsafe {
            let ptrs = T::ptrs_add(context, ptrs, self.start);
            let slices = T::slices_from_raw_parts(context, ptrs, len);
            T::slice_ptrs_to_slices(context, slices)
        }
    }

    #[inline]
    unsafe fn post_inc_start(&mut self, offset: usize) -> T::MutPtrs {
        let context = self.context();
        let ptrs = self.ptrs();
        let ptrs = unsafe { T::ptrs_add_mut(context, ptrs, self.start) };

        self.start += offset;
        ptrs
    }

    #[inline]
    unsafe fn pre_dec_end(&mut self, offset: usize) -> T::MutPtrs {
        self.end -= offset;

        let context = self.context();
        let ptrs = self.ptrs();
        unsafe { T::ptrs_add_mut(context, ptrs, self.end) }
    }
}

unsafe impl<T> Send for IterMut<'_, T> where T: Soa + Send {}
unsafe impl<T> Sync for IterMut<'_, T> where T: Soa + Sync {}

impl<T, U> AsRef<[U]> for IterMut<'_, T>
where
    for<'a> T: Soa<Slices<'a> = &'a [U]> + 'a,
{
    fn as_ref(&self) -> &[U] {
        self.as_slices()
    }
}

impl<T> Debug for IterMut<'_, T>
where
    T: Soa,
    for<'any> T::Slices<'any>: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("IterMut").field(&slices).finish()
    }
}

#[allow(clippy::while_let_on_iterator)]
impl<'a, T> Iterator for IterMut<'a, T>
where
    T: Soa,
{
    type Item = T::RefsMut<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if IterMut::is_empty(self) {
            return None;
        }

        unsafe {
            let ptrs = self.post_inc_start(1);
            let context = self.context();

            let refs = T::ptrs_to_refs_mut(context, ptrs);
            Some(refs)
        }
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
        if n >= IterMut::len(self) {
            self.start = self.end;
            return None;
        }

        unsafe {
            self.post_inc_start(n);
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
        let context = self.context();
        loop {
            // SAFETY: the loop iterates `i in 0..len`, which always is in bounds of
            // the slice allocation
            let ptrs = self.ptrs();
            let item = unsafe {
                let ptrs = T::ptrs_add_mut(context, ptrs, i);
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

impl<T> DoubleEndedIterator for IterMut<'_, T>
where
    T: Soa,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if IterMut::is_empty(self) {
            return None;
        }

        unsafe {
            let ptrs = self.pre_dec_end(1);
            let context = self.context();

            let refs = T::ptrs_to_refs_mut(context, ptrs);
            Some(refs)
        }
    }

    #[inline]
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
    #[inline]
    fn len(&self) -> usize {
        IterMut::len(self)
    }
}

impl<T> FusedIterator for IterMut<'_, T> where T: Soa {}
