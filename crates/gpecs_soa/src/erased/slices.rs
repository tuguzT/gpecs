use alloc::{boxed::Box, vec};
use core::{
    alloc::Layout,
    fmt::{self, Debug},
    marker::PhantomData,
    slice,
};

use crate::traits::Soa;

use super::validate_layout;

// TODO: replace with struct of layout and this
// data is stored inline in a single buffer
type ErasedFieldSlice<'a> = &'a [u8];

pub struct ErasedSoaSlices<'a, Fields>
where
    Fields: 'a,
{
    pub(super) len: usize,
    pub(super) slices: Box<[(Layout, ErasedFieldSlice<'a>)]>,
    pub(super) phantom: PhantomData<fn() -> Fields>,
}

impl<'a, Fields> ErasedSoaSlices<'a, Fields> {
    #[inline]
    pub fn new<I>(len: usize, slices: I) -> Self
    where
        I: IntoIterator<Item = (Layout, ErasedFieldSlice<'a>)>,
    {
        let slices = slices
            .into_iter()
            .map(|(field_layout, slice)| {
                assert_eq!(
                    slice.len().checked_div(field_layout.size()).unwrap_or(len),
                    len,
                );
                (field_layout.clone(), slice)
            })
            .collect();
        Self {
            len,
            slices,
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
                let slice = unsafe { slice::from_raw_parts(ptr.cast(), len) };
                (field_layout.clone(), slice)
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
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                slice.as_ptr()
            });
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

impl<'a, Fields> AsRef<[(Layout, ErasedFieldSlice<'a>)]> for ErasedSoaSlices<'a, Fields> {
    fn as_ref(&self) -> &[(Layout, ErasedFieldSlice<'a>)] {
        let Self { slices, .. } = self;
        slices.as_ref()
    }
}

impl<'a, Fields> AsMut<[(Layout, ErasedFieldSlice<'a>)]> for ErasedSoaSlices<'a, Fields> {
    fn as_mut(&mut self) -> &mut [(Layout, ErasedFieldSlice<'a>)] {
        let Self { slices, .. } = self;
        slices.as_mut()
    }
}

impl<'a, Fields> Debug for ErasedSoaSlices<'a, Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ErasedSoaSlices")
            .field("len", &self.len)
            .field("slices", &self.slices)
            .finish()
    }
}

impl<'a, Fields> Clone for ErasedSoaSlices<'a, Fields> {
    fn clone(&self) -> Self {
        Self {
            len: self.len.clone(),
            slices: self.slices.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

impl<'r, 'a, Fields> IntoIterator for &'r ErasedSoaSlices<'a, Fields> {
    type Item = &'r (Layout, ErasedFieldSlice<'a>);

    type IntoIter = slice::Iter<'r, (Layout, ErasedFieldSlice<'a>)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaSlices { slices, .. } = self;
        slices.iter()
    }
}

impl<'r, 'a, Fields> IntoIterator for &'r mut ErasedSoaSlices<'a, Fields> {
    type Item = &'r mut (Layout, ErasedFieldSlice<'a>);

    type IntoIter = slice::IterMut<'r, (Layout, ErasedFieldSlice<'a>)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaSlices { slices, .. } = self;
        slices.iter_mut()
    }
}

impl<'a, Fields> IntoIterator for ErasedSoaSlices<'a, Fields> {
    type Item = (Layout, ErasedFieldSlice<'a>);

    type IntoIter = vec::IntoIter<(Layout, ErasedFieldSlice<'a>)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaSlices { slices, .. } = self;
        slices.into_vec().into_iter()
    }
}

unsafe impl<'a, Fields> Send for ErasedSoaSlices<'a, Fields> where Fields: Sync {}
unsafe impl<'a, Fields> Sync for ErasedSoaSlices<'a, Fields> where Fields: Sync {}
