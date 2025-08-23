use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
};

use crate::{traits::Soa, wrapper::NonNullPtrs};

use super::{SoaSlices, SoaSlicesMut};

pub struct Iter<'c, 'a, T>
where
    T: Soa + ?Sized + 'a,
{
    context: &'c T::Context,
    ptrs: NonNullPtrs<'c, T>,
    start: usize,
    end: usize,
    phantom: PhantomData<&'a ()>,
}

impl<'c, 'a, T> Iter<'c, 'a, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    pub(crate) fn new(slices: SoaSlices<'c, 'a, T>) -> Self {
        let (context, ptrs, len) = slices.into_parts();
        let ptrs = unsafe { T::ptrs_to_nonnull(context, T::ptrs_cast_mut(context, ptrs)) };
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
    pub fn context(&self) -> &'c T::Context {
        let Self { context, .. } = *self;
        context
    }

    fn ptrs(&self) -> T::Ptrs<'c> {
        let Self { context, ptrs, .. } = self;
        let ptrs = ptrs.clone().into_inner();
        let ptrs = T::nonnull_to_ptrs(context, ptrs);
        T::ptrs_cast_const(context, ptrs)
    }

    #[inline]
    pub fn as_slices(&self) -> T::Slices<'c, 'a> {
        let Self { context, start, .. } = *self;
        let ptrs = self.ptrs();
        let len = self.len();

        unsafe {
            let ptrs = T::ptrs_add(context, ptrs, start);
            let slices = T::slices_from_raw_parts(context, ptrs, len);
            T::slice_ptrs_to_slices(context, slices)
        }
    }

    #[inline]
    unsafe fn post_inc_start<'b>(
        start: &mut usize,
        ptrs: T::MutPtrs<'b>,
        context: &'b T::Context,
        offset: usize,
    ) -> T::MutPtrs<'b> {
        let old_start = *start;
        *start += offset;

        unsafe { T::ptrs_add_mut(context, ptrs, old_start) }
    }

    #[inline]
    unsafe fn pre_dec_end<'b>(
        end: &mut usize,
        ptrs: T::MutPtrs<'b>,
        context: &'b T::Context,
        offset: usize,
    ) -> T::MutPtrs<'b> {
        *end -= offset;

        unsafe { T::ptrs_add_mut(context, ptrs, *end) }
    }
}

unsafe impl<T> Send for Iter<'_, '_, T>
where
    T: Soa + ?Sized,
    T::Fields: Send,
    T::Context: Send,
{
}

unsafe impl<T> Sync for Iter<'_, '_, T>
where
    T: Soa + ?Sized,
    T::Fields: Sync,
    T::Context: Sync,
{
}

impl<T, U> AsRef<[U]> for Iter<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T: Soa<Slices<'c, 'any> = &'any [U]> + 'any,
{
    fn as_ref(&self) -> &[U] {
        self.as_slices()
    }
}

impl<T> Debug for Iter<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Debug,
{
    #[inline]
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
        let Self {
            context,
            ref ptrs,
            start,
            end,
            phantom,
        } = *self;
        Self {
            context,
            ptrs: ptrs.clone(),
            start,
            end,
            phantom,
        }
    }
}

#[expect(clippy::while_let_on_iterator)]
impl<'c, 'a, T> Iterator for Iter<'c, 'a, T>
where
    T: Soa + ?Sized,
{
    type Item = T::Refs<'c, 'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if Iter::is_empty(self) {
            return None;
        }

        let Self {
            context,
            ref ptrs,
            ref mut start,
            ..
        } = *self;
        let ptrs = ptrs.clone().into_inner();
        let ptrs = T::nonnull_to_ptrs(context, ptrs);
        let ptrs = unsafe { Self::post_inc_start(start, ptrs, context, 1) };
        let ptrs = T::ptrs_cast_const(context, ptrs);

        let refs = unsafe { T::ptrs_to_refs(context, ptrs) };
        Some(refs)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = Iter::len(self);
        (len, Some(len))
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        Iter::len(&self)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n >= Iter::len(self) {
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
        let ptrs = T::nonnull_to_ptrs(context, ptrs);
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
        if Iter::is_empty(&self) {
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
        let len = Iter::len(&self);
        let mut acc = init;
        let mut i = 0;
        loop {
            // SAFETY: the loop iterates `i in 0..len`, which always is in bounds of
            // the slice allocation
            let ptrs = {
                let ptrs = ptrs.clone().into_inner();
                let ptrs = T::nonnull_to_ptrs(context, ptrs);
                T::ptrs_cast_const(context, ptrs)
            };
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
        let n = Iter::len(self);
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
        let n = Iter::len(self);
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

impl<T> DoubleEndedIterator for Iter<'_, '_, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if Iter::is_empty(self) {
            return None;
        }

        let Self {
            context,
            ref ptrs,
            ref mut end,
            ..
        } = *self;
        let ptrs = ptrs.clone().into_inner();
        let ptrs = T::nonnull_to_ptrs(context, ptrs);
        let ptrs = unsafe { Self::pre_dec_end(end, ptrs, context, 1) };
        let ptrs = T::ptrs_cast_const(context, ptrs);

        let refs = unsafe { T::ptrs_to_refs(context, ptrs) };
        Some(refs)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        if n >= Iter::len(self) {
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
        let ptrs = T::nonnull_to_ptrs(context, ptrs);
        unsafe {
            Self::pre_dec_end(end, ptrs, context, n);
        }
        self.next_back()
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
    pub(super) fn new(slices: SoaSlicesMut<'c, 'a, T>) -> Self {
        let (context, ptrs, len) = slices.into_parts();
        let ptrs = unsafe { T::ptrs_to_nonnull(context, ptrs) };
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

    fn ptrs(&self) -> T::MutPtrs<'_> {
        let Self { context, ptrs, .. } = self;
        let ptrs = ptrs.clone().into_inner();
        T::nonnull_to_ptrs(context, ptrs)
    }

    #[inline]
    pub fn into_slices(self) -> T::SlicesMut<'c, 'a> {
        let len = self.len();
        let Self { context, ptrs, .. } = self;
        let ptrs = ptrs.into_inner();
        let ptrs = T::nonnull_to_ptrs(context, ptrs);

        unsafe {
            let ptrs = T::ptrs_add_mut(context, ptrs, self.start);
            let slices = T::slices_from_raw_parts_mut(context, ptrs, len);
            T::slice_mut_ptrs_to_slices(context, slices)
        }
    }

    #[inline]
    pub fn as_slices(&self) -> T::Slices<'_, '_> {
        let Self { context, start, .. } = *self;
        let len = self.len();
        let ptrs = self.ptrs();
        let ptrs = T::ptrs_cast_const(context, ptrs);

        unsafe {
            let ptrs = T::ptrs_add(context, ptrs, start);
            let slices = T::slices_from_raw_parts(context, ptrs, len);
            T::slice_ptrs_to_slices(context, slices)
        }
    }

    #[inline]
    unsafe fn post_inc_start<'b>(
        start: &mut usize,
        ptrs: T::MutPtrs<'b>,
        context: &'b T::Context,
        offset: usize,
    ) -> T::MutPtrs<'b> {
        let old_start = *start;
        *start += offset;

        unsafe { T::ptrs_add_mut(context, ptrs, old_start) }
    }

    #[inline]
    unsafe fn pre_dec_end<'b>(
        end: &mut usize,
        ptrs: T::MutPtrs<'b>,
        context: &'b T::Context,
        offset: usize,
    ) -> T::MutPtrs<'b> {
        *end -= offset;

        unsafe { T::ptrs_add_mut(context, ptrs, *end) }
    }
}

unsafe impl<T> Send for IterMut<'_, '_, T>
where
    T: Soa + ?Sized,
    T::Fields: Send,
    T::Context: Send,
{
}

unsafe impl<T> Sync for IterMut<'_, '_, T>
where
    T: Soa + ?Sized,
    T::Fields: Sync,
    T::Context: Sync,
{
}

impl<T, U> AsRef<[U]> for IterMut<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T: Soa<Slices<'c, 'any> = &'any [U]> + 'any,
{
    fn as_ref(&self) -> &[U] {
        self.as_slices()
    }
}

impl<T> Debug for IterMut<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Debug,
{
    #[inline]
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
        let ptrs = T::nonnull_to_ptrs(context, ptrs);
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
        let ptrs = T::nonnull_to_ptrs(context, ptrs);
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
            let ptrs = T::nonnull_to_ptrs(context, ptrs);
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
        let ptrs = T::nonnull_to_ptrs(context, ptrs);
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
        let ptrs = T::nonnull_to_ptrs(context, ptrs);
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
