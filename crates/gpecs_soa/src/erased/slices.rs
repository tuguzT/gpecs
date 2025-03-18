use alloc::{boxed::Box, vec};
use core::{
    alloc::Layout,
    fmt::{self, Debug},
    marker::PhantomData,
    slice,
};

use crate::traits::Soa;

use super::{assert_buffer_align, assert_into_size, assert_slice_buffer_len, validate_layout};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ErasedFieldSlice<'a> {
    layout: Layout,
    // data is stored inline in a single buffer
    buffer: &'a [u8],
}

impl<'a> ErasedFieldSlice<'a> {
    #[inline]
    #[track_caller]
    pub fn new(layout: Layout, buffer: &'a [u8]) -> Self {
        assert_slice_buffer_len(buffer.len(), layout.size());
        assert_buffer_align(buffer.as_ptr(), layout.align());

        Self { layout, buffer }
    }

    #[inline]
    pub fn from<T>(ptr: &'a [T]) -> Self {
        let layout = Layout::new::<T>();
        let buffer = unsafe {
            let data = ptr.as_ptr().cast();
            let len = layout.size() * ptr.len();
            slice::from_raw_parts(data, len)
        };
        Self::new(layout, buffer)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn into<T>(self) -> &'a [T] {
        let Self { layout, buffer } = self;
        assert_into_size::<T>(layout.size());

        let data = buffer.as_ptr().cast();
        let len = buffer.len().checked_div(layout.size()).unwrap_or(0);
        unsafe { slice::from_raw_parts(data, len) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn cast<T>(&self) -> &[T] {
        let Self { layout, buffer } = self;
        assert_into_size::<T>(layout.size());

        let data = buffer.as_ptr().cast();
        let len = buffer.len().checked_div(layout.size()).unwrap_or(0);
        unsafe { slice::from_raw_parts(data, len) }
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { layout, buffer } = self;
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
    pub fn into_buffer(self) -> &'a [u8] {
        let Self { buffer, .. } = self;
        buffer
    }

    #[inline]
    pub fn into_parts(self) -> (Layout, &'a [u8]) {
        let Self { layout, buffer } = self;
        (layout, buffer)
    }
}

pub struct ErasedSoaSlices<'a, Fields>
where
    Fields: 'a,
{
    pub(super) len: usize,
    pub(super) slices: Box<[ErasedFieldSlice<'a>]>,
    pub(super) phantom: PhantomData<fn() -> Fields>,
}

impl<'a, Fields> ErasedSoaSlices<'a, Fields> {
    #[inline]
    pub fn new<I>(len: usize, slices: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldSlice<'a>>,
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
    pub fn from<T>(context: &T::Context, slices: T::Slices<'a>) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let len = T::slices_len(context, &slices);
        let ptrs = T::slice_refs_as_ptrs(context, slices);
        let ptrs = T::ptrs_erase(context, ptrs);
        let field_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>);

        let slices = field_layouts
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let len = field_layout.size() * len;
                let slice = unsafe { slice::from_raw_parts(ptr, len) };
                ErasedFieldSlice::new(field_layout, slice)
            })
            .collect();
        Self {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::Slices<'a>
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
            .map(|(_, slice)| slice.into_buffer().as_ptr());
        let ptrs = T::ptrs_restore(context, ptrs);
        let slices = T::slices_from_raw_parts(context, ptrs, len);
        unsafe { T::slice_ptrs_to_slices(context, slices) }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a, Fields> AsRef<[ErasedFieldSlice<'a>]> for ErasedSoaSlices<'a, Fields> {
    fn as_ref(&self) -> &[ErasedFieldSlice<'a>] {
        let Self { slices, .. } = self;
        slices.as_ref()
    }
}

impl<'a, Fields> AsMut<[ErasedFieldSlice<'a>]> for ErasedSoaSlices<'a, Fields> {
    fn as_mut(&mut self) -> &mut [ErasedFieldSlice<'a>] {
        let Self { slices, .. } = self;
        slices.as_mut()
    }
}

impl<'a, Fields> Debug for ErasedSoaSlices<'a, Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { len, slices, .. } = self;

        f.debug_struct("ErasedSoaSlices")
            .field("len", len)
            .field("slices", slices)
            .finish()
    }
}

impl<'a, Fields> Clone for ErasedSoaSlices<'a, Fields> {
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

impl<'r, 'a, Fields> IntoIterator for &'r ErasedSoaSlices<'a, Fields> {
    type Item = &'r ErasedFieldSlice<'a>;
    type IntoIter = slice::Iter<'r, ErasedFieldSlice<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaSlices { slices, .. } = self;
        slices.iter()
    }
}

impl<'r, 'a, Fields> IntoIterator for &'r mut ErasedSoaSlices<'a, Fields> {
    type Item = &'r mut ErasedFieldSlice<'a>;
    type IntoIter = slice::IterMut<'r, ErasedFieldSlice<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaSlices { slices, .. } = self;
        slices.iter_mut()
    }
}

impl<'a, Fields> IntoIterator for ErasedSoaSlices<'a, Fields> {
    type Item = ErasedFieldSlice<'a>;
    type IntoIter = vec::IntoIter<ErasedFieldSlice<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaSlices { slices, .. } = self;
        slices.into_vec().into_iter()
    }
}

unsafe impl<'a, Fields> Send for ErasedSoaSlices<'a, Fields> where Fields: Sync {}
unsafe impl<'a, Fields> Sync for ErasedSoaSlices<'a, Fields> where Fields: Sync {}
