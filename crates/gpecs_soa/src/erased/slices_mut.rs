use alloc::boxed::Box;
use core::{
    alloc::Layout,
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    ptr::NonNull,
    slice,
};

use crate::traits::Soa;

use super::{
    assert_buffer_align, assert_layout, assert_slice_buffer_len, validate_layout, ErasedFieldRef,
    ErasedFieldRefMut, ErasedFieldSlice, ErasedFieldSliceIter, ErasedSoaRefs, ErasedSoaRefsMut,
    ErasedSoaSlicesIter,
};

#[derive(PartialEq, Eq, Hash)]
pub struct ErasedFieldSliceMut<'a> {
    layout: Layout,
    // data is stored inline in a single buffer
    buffer: &'a mut [u8],
    no_send_sync: PhantomData<*const u8>,
}

impl<'a> ErasedFieldSliceMut<'a> {
    #[inline]
    #[track_caller]
    pub fn new(layout: Layout, buffer: &'a mut [u8]) -> Self {
        assert_slice_buffer_len(buffer.len(), layout.size());
        assert_buffer_align(buffer.as_ptr(), layout.align());

        Self {
            layout,
            buffer,
            no_send_sync: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(ptr: &'a mut [T]) -> Self {
        let layout = Layout::new::<T>();
        let buffer = unsafe {
            let data = ptr.as_mut_ptr().cast();
            let len = layout.size() * ptr.len();
            slice::from_raw_parts_mut(data, len)
        };
        Self::new(layout, buffer)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn into<T>(self) -> &'a mut [T] {
        let Self { layout, buffer, .. } = self;
        assert_layout::<T>(&layout);

        let data = buffer.as_mut_ptr().cast();
        let len = buffer.len().checked_div(layout.size()).unwrap_or(0);
        unsafe { slice::from_raw_parts_mut(data, len) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn cast<T>(&self) -> &[T] {
        let Self { layout, buffer, .. } = self;
        assert_layout::<T>(layout);

        let data = buffer.as_ptr().cast();
        let len = buffer.len().checked_div(layout.size()).unwrap_or(0);
        unsafe { slice::from_raw_parts(data, len) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn cast_mut<T>(&mut self) -> &mut [T] {
        let Self { layout, buffer, .. } = self;
        assert_layout::<T>(layout);

        let data = buffer.as_mut_ptr().cast();
        let len = buffer.len().checked_div(layout.size()).unwrap_or(0);
        unsafe { slice::from_raw_parts_mut(data, len) }
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { layout, buffer, .. } = self;
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
    pub fn buffer(&self) -> &[u8] {
        let Self { buffer, .. } = self;
        buffer
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { buffer, .. } = self;
        buffer.as_ptr()
    }

    #[inline]
    pub fn buffer_mut(&mut self) -> &mut [u8] {
        let Self { buffer, .. } = self;
        buffer
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        let Self { buffer, .. } = self;
        buffer.as_mut_ptr()
    }

    #[inline]
    pub fn into_buffer(self) -> &'a mut [u8] {
        let Self { buffer, .. } = self;
        buffer
    }

    #[inline]
    pub fn into_parts(self) -> (Layout, &'a mut [u8]) {
        let Self { layout, buffer, .. } = self;
        (layout, buffer)
    }

    #[inline]
    pub fn iter(&self) -> ErasedFieldSliceIter<'_> {
        let Self { layout, buffer, .. } = self;
        let slice = ErasedFieldSlice::new(*layout, buffer);
        ErasedFieldSliceIter::new(slice)
    }

    #[inline]
    pub fn iter_mut(&mut self) -> ErasedFieldSliceIterMut<'_> {
        let Self { layout, buffer, .. } = self;
        let slice = ErasedFieldSliceMut::new(*layout, buffer);
        ErasedFieldSliceIterMut::new(slice)
    }
}

impl Debug for ErasedFieldSliceMut<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { layout, buffer, .. } = self;
        f.debug_struct("ErasedFieldSliceMut")
            .field("layout", layout)
            .field("buffer", buffer)
            .finish()
    }
}

impl<'a> IntoIterator for &'a ErasedFieldSliceMut<'_> {
    type Item = ErasedFieldRef<'a>;
    type IntoIter = ErasedFieldSliceIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut ErasedFieldSliceMut<'_> {
    type Item = ErasedFieldRefMut<'a>;
    type IntoIter = ErasedFieldSliceIterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a> IntoIterator for ErasedFieldSliceMut<'a> {
    type Item = ErasedFieldRefMut<'a>;
    type IntoIter = ErasedFieldSliceIterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ErasedFieldSliceIterMut::new(self)
    }
}

pub struct ErasedFieldSliceIterMut<'a> {
    layout: Layout,
    buffer: NonNull<u8>,
    start: usize,
    end: usize,
    marker: PhantomData<&'a mut [u8]>,
}

impl<'a> ErasedFieldSliceIterMut<'a> {
    #[inline]
    pub(super) fn new(slice: ErasedFieldSliceMut<'a>) -> Self {
        let end = slice.len();
        let (layout, buffer) = slice.into_parts();
        let buffer = NonNull::new(buffer.as_mut_ptr()).expect("slice ptr should be nonnull");

        Self {
            layout,
            buffer,
            start: 0,
            end,
            marker: PhantomData,
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
    pub fn as_slice(&self) -> ErasedFieldSlice<'_> {
        let len = self.len() * self.layout.size();
        let buffer = unsafe { slice::from_raw_parts(self.buffer.as_ptr(), len) };
        ErasedFieldSlice::new(self.layout, buffer)
    }

    #[inline]
    pub fn into_slice(self) -> ErasedFieldSliceMut<'a> {
        let len = self.len() * self.layout.size();
        let buffer = unsafe { slice::from_raw_parts_mut(self.buffer.as_ptr(), len) };
        ErasedFieldSliceMut::new(self.layout, buffer)
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

impl Debug for ErasedFieldSliceIterMut<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (layout, buffer) = self.as_slice().into_parts();
        f.debug_struct("ErasedFieldSliceIterMut")
            .field("layout", &layout)
            .field("buffer", &buffer)
            .finish()
    }
}

#[allow(clippy::while_let_on_iterator)]
impl<'a> Iterator for ErasedFieldSliceIterMut<'a> {
    type Item = ErasedFieldRefMut<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if ErasedFieldSliceIterMut::is_empty(self) {
            return None;
        }

        let layout = self.layout;
        let ptr = unsafe { self.post_inc_start(1) };
        let buffer = unsafe { slice::from_raw_parts_mut(ptr, layout.size()) };
        Some(ErasedFieldRefMut::new(layout, buffer))
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
        if ErasedFieldSliceIterMut::is_empty(&self) {
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
                let buffer = slice::from_raw_parts_mut(data, layout.size());
                ErasedFieldRefMut::new(layout, buffer)
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

impl DoubleEndedIterator for ErasedFieldSliceIterMut<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if ErasedFieldSliceIterMut::is_empty(self) {
            return None;
        }

        let layout = self.layout;
        let ptr = unsafe { self.pre_dec_end(1) };
        let buffer = unsafe { slice::from_raw_parts_mut(ptr, layout.size()) };
        Some(ErasedFieldRefMut::new(layout, buffer))
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

impl ExactSizeIterator for ErasedFieldSliceIterMut<'_> {
    fn len(&self) -> usize {
        ErasedFieldSliceIterMut::len(self)
    }
}

impl FusedIterator for ErasedFieldSliceIterMut<'_> {}

pub struct ErasedSoaSlicesMut<'a, Fields>
where
    Fields: 'a,
{
    pub(super) len: usize,
    pub(super) slices: Box<[ErasedFieldSliceMut<'a>]>,
    pub(super) phantom: PhantomData<fn() -> Fields>,
}

impl<'a, Fields> ErasedSoaSlicesMut<'a, Fields> {
    #[inline]
    pub fn new<I>(len: usize, slices: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldSliceMut<'a>>,
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
    pub fn from<T>(context: &T::Context, slices: T::SlicesMut<'a>) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let len = T::slices_len_mut(context, &slices);
        let ptrs = T::mut_slice_refs_as_ptrs(context, slices);
        let ptrs = T::ptrs_erase_mut(context, ptrs);
        let field_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>);

        let slices = field_layouts
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let len = field_layout.size() * len;
                let slice = unsafe { slice::from_raw_parts_mut(ptr, len) };
                ErasedFieldSliceMut::new(field_layout, slice)
            })
            .collect();
        Self {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::SlicesMut<'a>
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
            .map(|(_, slice)| slice.into_buffer().as_mut_ptr());
        let ptrs = T::ptrs_restore_mut(context, ptrs);
        let slices = T::slices_from_raw_parts_mut(context, ptrs, len);
        unsafe { T::slice_ptrs_to_slices_mut(context, slices) }
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
    pub fn fields(&self) -> &[ErasedFieldSliceMut<'a>] {
        let Self { slices, .. } = self;
        slices.as_ref()
    }

    #[inline]
    pub fn fields_mut(&mut self) -> &mut [ErasedFieldSliceMut<'a>] {
        let Self { slices, .. } = self;
        slices.as_mut()
    }

    #[inline]
    pub fn into_fields(self) -> Box<[ErasedFieldSliceMut<'a>]> {
        let Self { slices, .. } = self;
        slices
    }

    #[inline]
    pub fn iter(&self) -> ErasedSoaSlicesIter<'_, Fields> {
        let Self { slices, .. } = self;
        let slices = slices.iter().map(IntoIterator::into_iter);
        ErasedSoaSlicesIter::new(slices)
    }

    #[inline]
    pub fn iter_mut(&mut self) -> ErasedSoaSlicesIterMut<'_, Fields> {
        let Self { slices, .. } = self;
        let slices = slices.iter_mut().map(IntoIterator::into_iter);
        ErasedSoaSlicesIterMut::new(slices)
    }
}

impl<Fields> Debug for ErasedSoaSlicesMut<'_, Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { len, slices, .. } = self;

        f.debug_struct("ErasedSoaSlicesMut")
            .field("len", len)
            .field("slices", slices)
            .finish()
    }
}

impl<'a, Fields> IntoIterator for &'a ErasedSoaSlicesMut<'_, Fields> {
    type Item = ErasedSoaRefs<'a, Fields>;
    type IntoIter = ErasedSoaSlicesIter<'a, Fields>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, Fields> IntoIterator for &'a mut ErasedSoaSlicesMut<'_, Fields> {
    type Item = ErasedSoaRefsMut<'a, Fields>;
    type IntoIter = ErasedSoaSlicesIterMut<'a, Fields>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, Fields> IntoIterator for ErasedSoaSlicesMut<'a, Fields> {
    type Item = ErasedSoaRefsMut<'a, Fields>;
    type IntoIter = ErasedSoaSlicesIterMut<'a, Fields>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { slices, .. } = self;
        let slices = slices.into_vec().into_iter().map(IntoIterator::into_iter);
        ErasedSoaSlicesIterMut::new(slices)
    }
}

unsafe impl<Fields> Send for ErasedSoaSlicesMut<'_, Fields> where Fields: Send {}
unsafe impl<Fields> Sync for ErasedSoaSlicesMut<'_, Fields> where Fields: Sync {}

pub struct ErasedSoaSlicesIterMut<'a, Fields>
where
    Fields: 'a,
{
    slices: Box<[ErasedFieldSliceIterMut<'a>]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<'a, Fields> ErasedSoaSlicesIterMut<'a, Fields> {
    #[inline]
    fn new<I>(slices: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldSliceIterMut<'a>>,
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

impl<'a, Fields> Iterator for ErasedSoaSlicesIterMut<'a, Fields> {
    type Item = ErasedSoaRefsMut<'a, Fields>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if ErasedSoaSlicesIterMut::is_empty(self) {
            return None;
        }

        let refs = self.slices.iter_mut().flat_map(Iterator::next);
        Some(ErasedSoaRefsMut::new(refs))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<Fields> DoubleEndedIterator for ErasedSoaSlicesIterMut<'_, Fields> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if ErasedSoaSlicesIterMut::is_empty(self) {
            return None;
        }

        let refs = self
            .slices
            .iter_mut()
            .flat_map(DoubleEndedIterator::next_back);
        Some(ErasedSoaRefsMut::new(refs))
    }
}

impl<Fields> ExactSizeIterator for ErasedSoaSlicesIterMut<'_, Fields> {
    #[inline]
    fn len(&self) -> usize {
        ErasedSoaSlicesIterMut::len(self)
    }
}

unsafe impl<Fields> Send for ErasedSoaSlicesIterMut<'_, Fields> where Fields: Send {}
unsafe impl<Fields> Sync for ErasedSoaSlicesIterMut<'_, Fields> where Fields: Sync {}
