use alloc::{boxed::Box, vec};
use core::{
    alloc::Layout,
    fmt::{self, Debug},
    marker::PhantomData,
    slice,
};

use crate::traits::Soa;

use super::{assert_buffer_align, assert_slice_buffer_len, validate_layout};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ErasedFieldSliceMut<'a> {
    layout: Layout,
    // data is stored inline in a single buffer
    buffer: &'a mut [u8],
}

impl<'a> ErasedFieldSliceMut<'a> {
    #[inline]
    #[track_caller]
    pub fn new(layout: Layout, buffer: &'a mut [u8]) -> Self {
        assert_slice_buffer_len(buffer.len(), layout.size());
        assert_buffer_align(buffer.as_ptr(), layout.align());

        Self { layout, buffer }
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
        let Self { layout, buffer } = self;
        (layout, buffer)
    }
}

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
                let slice = unsafe { slice::from_raw_parts_mut(ptr.cast(), len) };
                ErasedFieldSliceMut::new(field_layout.clone(), slice)
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
            .map(|(field_layout, mut slice)| {
                assert_eq!(*field_layout, slice.layout());
                slice.buffer_mut().as_mut_ptr()
            });

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
}

impl<'a, Fields> AsRef<[ErasedFieldSliceMut<'a>]> for ErasedSoaSlicesMut<'a, Fields> {
    fn as_ref(&self) -> &[ErasedFieldSliceMut<'a>] {
        let Self { slices, .. } = self;
        slices.as_ref()
    }
}

impl<'a, Fields> AsMut<[ErasedFieldSliceMut<'a>]> for ErasedSoaSlicesMut<'a, Fields> {
    fn as_mut(&mut self) -> &mut [ErasedFieldSliceMut<'a>] {
        let Self { slices, .. } = self;
        slices.as_mut()
    }
}

impl<'a, Fields> Debug for ErasedSoaSlicesMut<'a, Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { len, slices, .. } = self;

        f.debug_struct("ErasedSoaSlicesMut")
            .field("len", len)
            .field("slices", slices)
            .finish()
    }
}

impl<'r, 'a, Fields> IntoIterator for &'r ErasedSoaSlicesMut<'a, Fields> {
    type Item = &'r ErasedFieldSliceMut<'a>;
    type IntoIter = slice::Iter<'r, ErasedFieldSliceMut<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaSlicesMut { slices, .. } = self;
        slices.iter()
    }
}

impl<'r, 'a, Fields> IntoIterator for &'r mut ErasedSoaSlicesMut<'a, Fields> {
    type Item = &'r mut ErasedFieldSliceMut<'a>;
    type IntoIter = slice::IterMut<'r, ErasedFieldSliceMut<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaSlicesMut { slices, .. } = self;
        slices.iter_mut()
    }
}

impl<'a, Fields> IntoIterator for ErasedSoaSlicesMut<'a, Fields> {
    type Item = ErasedFieldSliceMut<'a>;
    type IntoIter = vec::IntoIter<ErasedFieldSliceMut<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaSlicesMut { slices, .. } = self;
        slices.into_vec().into_iter()
    }
}

unsafe impl<'a, Fields> Send for ErasedSoaSlicesMut<'a, Fields> where Fields: Send {}
unsafe impl<'a, Fields> Sync for ErasedSoaSlicesMut<'a, Fields> where Fields: Sync {}
