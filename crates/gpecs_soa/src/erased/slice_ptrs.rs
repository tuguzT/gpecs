use alloc::boxed::Box;
use core::{
    alloc::Layout,
    fmt::{self, Debug},
    hash::{self, Hash},
    iter::FusedIterator,
    marker::PhantomData,
    ptr::{self, NonNull},
};

use crate::traits::Soa;

use super::{
    assert_buffer_align, assert_layout, assert_slice_buffer_len, validate_layout, ErasedFieldPtr,
    ErasedSoaPtrs,
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ErasedFieldSlicePtr {
    layout: Layout,
    // all the data is stored inline in a single buffer
    buffer: *const [u8],
}

impl ErasedFieldSlicePtr {
    #[inline]
    #[track_caller]
    pub fn new(layout: Layout, buffer: *const [u8]) -> Self {
        assert_slice_buffer_len(buffer.len(), layout.size());
        assert_buffer_align(buffer.cast(), layout.align());

        Self { layout, buffer }
    }

    #[inline]
    pub fn from<T>(ptr: *const [T]) -> Self {
        let layout = Layout::new::<T>();
        let buffer = ptr::slice_from_raw_parts(ptr.cast(), layout.size() * ptr.len());
        Self::new(layout, buffer)
    }

    #[inline]
    #[track_caller]
    pub fn into<T>(self) -> *const [T] {
        let Self { layout, buffer } = self;
        assert_layout::<T>(&layout);

        ptr::slice_from_raw_parts(
            buffer.cast(),
            buffer.len().checked_div(layout.size()).unwrap_or(0),
        )
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { layout, buffer } = *self;
        buffer.len().checked_div(layout.size()).unwrap_or(0)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { layout, .. } = *self;
        layout
    }

    #[inline]
    pub fn buffer(&self) -> *const [u8] {
        let Self { buffer, .. } = *self;
        buffer
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { buffer, .. } = self;
        buffer.cast()
    }

    #[inline]
    pub fn into_parts(self) -> (Layout, *const [u8]) {
        let Self { layout, buffer } = self;
        (layout, buffer)
    }

    #[inline]
    pub fn iter(&self) -> ErasedFieldSlicePtrIter {
        let Self { layout, buffer, .. } = *self;
        let slice = ErasedFieldSlicePtr::new(layout, buffer);
        ErasedFieldSlicePtrIter::new(slice)
    }
}

impl IntoIterator for &ErasedFieldSlicePtr {
    type Item = ErasedFieldPtr;
    type IntoIter = ErasedFieldSlicePtrIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for ErasedFieldSlicePtr {
    type Item = ErasedFieldPtr;
    type IntoIter = ErasedFieldSlicePtrIter;

    fn into_iter(self) -> Self::IntoIter {
        ErasedFieldSlicePtrIter::new(self)
    }
}

pub struct ErasedFieldSlicePtrIter {
    layout: Layout,
    buffer: NonNull<u8>,
    start: usize,
    end: usize,
}

impl ErasedFieldSlicePtrIter {
    #[inline]
    pub(super) fn new(slice: ErasedFieldSlicePtr) -> Self {
        let end = slice.len();
        let (layout, buffer) = slice.into_parts();
        let buffer = NonNull::new(buffer as *mut _).expect("slice ptr should be nonnull");

        Self {
            layout,
            buffer,
            start: 0,
            end,
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
    pub fn as_slice(&self) -> ErasedFieldSlicePtr {
        let len = self.len() * self.layout.size();
        let buffer = ptr::slice_from_raw_parts(self.buffer.as_ptr(), len);
        ErasedFieldSlicePtr::new(self.layout, buffer)
    }

    #[inline]
    unsafe fn post_inc_start(&mut self, offset: usize) -> *mut u8 {
        let ptr = unsafe { self.buffer.as_ptr().add(self.start * self.layout.size()) };

        self.start += offset;
        ptr
    }

    #[inline]
    unsafe fn pre_dec_end(&mut self, offset: usize) -> *mut u8 {
        self.end -= offset;

        let ptr = unsafe { self.buffer.as_ptr().add(self.end * self.layout.size()) };
        ptr
    }
}

impl Debug for ErasedFieldSlicePtrIter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (layout, buffer) = self.as_slice().into_parts();
        f.debug_struct("ErasedFieldSlicePtrIter")
            .field("layout", &layout)
            .field("buffer", &buffer)
            .finish()
    }
}

impl Clone for ErasedFieldSlicePtrIter {
    fn clone(&self) -> Self {
        Self {
            layout: self.layout.clone(),
            buffer: self.buffer.clone(),
            start: self.start.clone(),
            end: self.end.clone(),
        }
    }
}

#[allow(clippy::while_let_on_iterator)]
impl<'a> Iterator for ErasedFieldSlicePtrIter {
    type Item = ErasedFieldPtr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if ErasedFieldSlicePtrIter::is_empty(self) {
            return None;
        }

        let layout = self.layout;
        let ptr = unsafe { self.post_inc_start(1) };
        let buffer = ptr::slice_from_raw_parts(ptr, layout.size());
        Some(ErasedFieldPtr::new(layout, buffer))
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
        if ErasedFieldSlicePtrIter::is_empty(&self) {
            return init;
        }
        let mut acc = init;
        let mut i = 0;
        let len = self.len();
        let ptr = self.buffer.as_ptr();
        loop {
            // SAFETY: the loop iterates `i in 0..len`, which always is in bounds of
            // the slice allocation
            let item = unsafe {
                let layout = self.layout;
                let data = ptr.add(i * layout.size());
                let buffer = ptr::slice_from_raw_parts(data, layout.size());
                ErasedFieldPtr::new(layout, buffer)
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

impl DoubleEndedIterator for ErasedFieldSlicePtrIter {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if ErasedFieldSlicePtrIter::is_empty(self) {
            return None;
        }

        let layout = self.layout;
        let ptr = unsafe { self.pre_dec_end(1) };
        let buffer = ptr::slice_from_raw_parts(ptr, layout.size());
        Some(ErasedFieldPtr::new(layout, buffer))
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

impl ExactSizeIterator for ErasedFieldSlicePtrIter {
    fn len(&self) -> usize {
        ErasedFieldSlicePtrIter::len(self)
    }
}

impl FusedIterator for ErasedFieldSlicePtrIter {}

pub struct ErasedSoaSlicePtrs<Fields> {
    pub(super) len: usize,
    pub(super) slices: Box<[ErasedFieldSlicePtr]>,
    pub(super) phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaSlicePtrs<Fields> {
    #[inline]
    pub fn new<I>(len: usize, slices: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldSlicePtr>,
    {
        Self {
            len,
            slices: slices
                .into_iter()
                .inspect(|slice| assert_eq!(slice.len(), len))
                .collect(),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(context: &T::Context, slices: T::SlicePtrs) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let len = T::slice_ptrs_len(context, slices.clone());
        let ptrs = T::slice_ptrs_as_ptrs(context, slices);
        let ptrs = T::ptrs_erase(context, ptrs);
        let field_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>);

        let slices = field_layouts
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let len = field_layout.size() * len;
                let slice = ptr::slice_from_raw_parts(ptr, len);
                ErasedFieldSlicePtr::new(field_layout, slice)
            })
            .collect();
        Self {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::SlicePtrs
    where
        T: Soa<Fields = Fields>,
    {
        let Self { slices, len, .. } = self;

        let field_layouts: Box<[_]> = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>)
            .collect();
        assert_eq!(slices.len(), field_layouts.len());

        let ptrs = field_layouts
            .iter()
            .zip(slices)
            .inspect(|(&field_layout, slice)| assert_eq!(field_layout, slice.layout()))
            .map(|(_, slice)| slice.as_ptr());
        let ptrs = T::ptrs_restore(context, ptrs);
        T::slices_from_raw_parts(context, ptrs, len)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn fields(&self) -> &[ErasedFieldSlicePtr] {
        let Self { slices, .. } = self;
        slices.as_ref()
    }

    #[inline]
    pub fn fields_mut(&mut self) -> &mut [ErasedFieldSlicePtr] {
        let Self { slices, .. } = self;
        slices.as_mut()
    }

    #[inline]
    pub fn into_fields(self) -> Box<[ErasedFieldSlicePtr]> {
        let Self { slices, .. } = self;
        slices
    }

    #[inline]
    pub fn iter(&self) -> ErasedSoaSlicePtrsIter<Fields> {
        let Self { slices, .. } = self;
        let slices = slices.iter().map(IntoIterator::into_iter);
        ErasedSoaSlicePtrsIter::new(slices)
    }
}

impl<Fields> Debug for ErasedSoaSlicePtrs<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { len, slices, .. } = self;

        f.debug_struct("ErasedSoaSlicePtrs")
            .field("len", len)
            .field("slices", slices)
            .finish()
    }
}

impl<Fields> PartialEq for ErasedSoaSlicePtrs<Fields> {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            len,
            slices,
            phantom,
        } = self;

        *len == other.len && *slices == other.slices && *phantom == other.phantom
    }
}

impl<Fields> Eq for ErasedSoaSlicePtrs<Fields> {}

impl<Fields> Hash for ErasedSoaSlicePtrs<Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self {
            len,
            slices,
            phantom,
        } = self;

        len.hash(state);
        slices.hash(state);
        phantom.hash(state);
    }
}

impl<Fields> Clone for ErasedSoaSlicePtrs<Fields> {
    fn clone(&self) -> Self {
        let Self {
            len,
            slices,
            phantom,
        } = self;

        Self {
            len: len.clone(),
            slices: slices.clone(),
            phantom: phantom.clone(),
        }
    }
}

impl<Fields> IntoIterator for &ErasedSoaSlicePtrs<Fields> {
    type Item = ErasedSoaPtrs<Fields>;
    type IntoIter = ErasedSoaSlicePtrsIter<Fields>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<Fields> IntoIterator for ErasedSoaSlicePtrs<Fields> {
    type Item = ErasedSoaPtrs<Fields>;
    type IntoIter = ErasedSoaSlicePtrsIter<Fields>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { slices, .. } = self;
        let slices = slices.into_vec().into_iter().map(IntoIterator::into_iter);
        ErasedSoaSlicePtrsIter::new(slices)
    }
}

pub struct ErasedSoaSlicePtrsIter<Fields> {
    slices: Box<[ErasedFieldSlicePtrIter]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaSlicePtrsIter<Fields> {
    #[inline]
    pub(super) fn new<I>(slices: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldSlicePtrIter>,
    {
        Self {
            slices: slices.into_iter().collect(),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { slices, .. } = self;
        let mut lens = slices.iter().map(ExactSizeIterator::len);

        let first = lens.next().expect("SoA should contain at least one field");
        assert!(lens.all(|len| len == first));
        first
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<Fields> Iterator for ErasedSoaSlicePtrsIter<Fields> {
    type Item = ErasedSoaPtrs<Fields>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if ErasedSoaSlicePtrsIter::is_empty(self) {
            return None;
        }

        let ptrs = self.slices.iter_mut().flat_map(Iterator::next);
        Some(ErasedSoaPtrs::new(ptrs))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<Fields> DoubleEndedIterator for ErasedSoaSlicePtrsIter<Fields> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if ErasedSoaSlicePtrsIter::is_empty(self) {
            return None;
        }

        let ptrs = self
            .slices
            .iter_mut()
            .flat_map(DoubleEndedIterator::next_back);
        Some(ErasedSoaPtrs::new(ptrs))
    }
}

impl<Fields> ExactSizeIterator for ErasedSoaSlicePtrsIter<Fields> {
    #[inline]
    fn len(&self) -> usize {
        ErasedSoaSlicePtrsIter::len(self)
    }
}
