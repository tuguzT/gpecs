use alloc::{boxed::Box, vec};
use core::{
    alloc::Layout,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    ptr, slice,
};

use crate::traits::Soa;

use super::validate_layout;

// data is stored inline in a single buffer
type ErasedFieldSlicePtr = *const [u8];

pub struct ErasedSoaSlicePtrs<Fields> {
    pub(super) len: usize,
    pub(super) slices: Box<[(Layout, ErasedFieldSlicePtr)]>,
    pub(super) phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaSlicePtrs<Fields> {
    #[inline]
    pub fn new<I>(len: usize, slices: I) -> Self
    where
        I: IntoIterator<Item = (Layout, ErasedFieldSlicePtr)>,
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
                let slice = ptr::slice_from_raw_parts(ptr.cast(), len);
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
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                slice.cast()
            });
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
}

impl<Fields> AsRef<[(Layout, ErasedFieldSlicePtr)]> for ErasedSoaSlicePtrs<Fields> {
    fn as_ref(&self) -> &[(Layout, ErasedFieldSlicePtr)] {
        let Self { slices, .. } = self;
        slices.as_ref()
    }
}

impl<Fields> AsMut<[(Layout, ErasedFieldSlicePtr)]> for ErasedSoaSlicePtrs<Fields> {
    fn as_mut(&mut self) -> &mut [(Layout, ErasedFieldSlicePtr)] {
        let Self { slices, .. } = self;
        slices.as_mut()
    }
}

impl<Fields> Debug for ErasedSoaSlicePtrs<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ErasedSoaSlicePtrs")
            .field("len", &self.len)
            .field("slices", &self.slices)
            .finish()
    }
}

impl<Fields> PartialEq for ErasedSoaSlicePtrs<Fields> {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.slices == other.slices && self.phantom == other.phantom
    }
}

impl<Fields> Eq for ErasedSoaSlicePtrs<Fields> {}

impl<Fields> Hash for ErasedSoaSlicePtrs<Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.len.hash(state);
        self.slices.hash(state);
        self.phantom.hash(state);
    }
}

impl<Fields> Clone for ErasedSoaSlicePtrs<Fields> {
    fn clone(&self) -> Self {
        Self {
            len: self.len.clone(),
            slices: self.slices.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

impl<'a, Fields> IntoIterator for &'a ErasedSoaSlicePtrs<Fields> {
    type Item = &'a (Layout, ErasedFieldSlicePtr);

    type IntoIter = slice::Iter<'a, (Layout, ErasedFieldSlicePtr)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaSlicePtrs { slices, .. } = self;
        slices.iter()
    }
}

impl<'a, Fields> IntoIterator for &'a mut ErasedSoaSlicePtrs<Fields> {
    type Item = &'a mut (Layout, ErasedFieldSlicePtr);

    type IntoIter = slice::IterMut<'a, (Layout, ErasedFieldSlicePtr)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaSlicePtrs { slices, .. } = self;
        slices.iter_mut()
    }
}

impl<Fields> IntoIterator for ErasedSoaSlicePtrs<Fields> {
    type Item = (Layout, ErasedFieldSlicePtr);

    type IntoIter = vec::IntoIter<(Layout, ErasedFieldSlicePtr)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaSlicePtrs { slices, .. } = self;
        slices.into_vec().into_iter()
    }
}
