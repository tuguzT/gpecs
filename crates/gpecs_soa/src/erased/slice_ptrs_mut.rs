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

// TODO: replace with struct of layout and this
// data is stored inline in a single buffer
type ErasedFieldSliceMutPtr = *mut [u8];

pub struct ErasedSoaSliceMutPtrs<Fields> {
    pub(super) len: usize,
    pub(super) slices: Box<[(Layout, ErasedFieldSliceMutPtr)]>,
    pub(super) phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaSliceMutPtrs<Fields> {
    #[inline]
    pub fn new<I>(len: usize, slices: I) -> Self
    where
        I: IntoIterator<Item = (Layout, ErasedFieldSliceMutPtr)>,
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
    pub fn from<T>(context: &T::Context, slices: T::SliceMutPtrs) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let len = T::slice_ptrs_len_mut(context, slices.clone());
        let ptrs = T::mut_slice_ptrs_as_ptrs(context, slices);
        let ptrs = T::ptrs_erase_mut(context, ptrs);
        let field_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>);

        let slices = field_layouts
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let len = field_layout.size() * len;
                let slice = ptr::slice_from_raw_parts_mut(ptr.cast(), len);
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
    pub unsafe fn into<T>(self, context: &T::Context) -> T::SliceMutPtrs
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
        let ptrs = T::ptrs_restore_mut(context, ptrs);
        T::slices_from_raw_parts_mut(context, ptrs, len)
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

impl<Fields> AsRef<[(Layout, ErasedFieldSliceMutPtr)]> for ErasedSoaSliceMutPtrs<Fields> {
    fn as_ref(&self) -> &[(Layout, ErasedFieldSliceMutPtr)] {
        let Self { slices, .. } = self;
        slices.as_ref()
    }
}

impl<Fields> AsMut<[(Layout, ErasedFieldSliceMutPtr)]> for ErasedSoaSliceMutPtrs<Fields> {
    fn as_mut(&mut self) -> &mut [(Layout, ErasedFieldSliceMutPtr)] {
        let Self { slices, .. } = self;
        slices.as_mut()
    }
}

impl<Fields> Debug for ErasedSoaSliceMutPtrs<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { len, slices, .. } = self;

        f.debug_struct("ErasedSoaSliceMutPtrs")
            .field("len", len)
            .field("slices", slices)
            .finish()
    }
}

impl<Fields> PartialEq for ErasedSoaSliceMutPtrs<Fields> {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            len,
            slices,
            phantom,
        } = self;

        *len == other.len && *slices == other.slices && *phantom == other.phantom
    }
}

impl<Fields> Eq for ErasedSoaSliceMutPtrs<Fields> {}

impl<Fields> Hash for ErasedSoaSliceMutPtrs<Fields> {
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

impl<Fields> Clone for ErasedSoaSliceMutPtrs<Fields> {
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

impl<'a, Fields> IntoIterator for &'a ErasedSoaSliceMutPtrs<Fields> {
    type Item = &'a (Layout, ErasedFieldSliceMutPtr);
    type IntoIter = slice::Iter<'a, (Layout, ErasedFieldSliceMutPtr)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaSliceMutPtrs { slices, .. } = self;
        slices.iter()
    }
}

impl<'a, Fields> IntoIterator for &'a mut ErasedSoaSliceMutPtrs<Fields> {
    type Item = &'a mut (Layout, ErasedFieldSliceMutPtr);
    type IntoIter = slice::IterMut<'a, (Layout, ErasedFieldSliceMutPtr)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaSliceMutPtrs { slices, .. } = self;
        slices.iter_mut()
    }
}

impl<Fields> IntoIterator for ErasedSoaSliceMutPtrs<Fields> {
    type Item = (Layout, ErasedFieldSliceMutPtr);
    type IntoIter = vec::IntoIter<(Layout, ErasedFieldSliceMutPtr)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaSliceMutPtrs { slices, .. } = self;
        slices.into_vec().into_iter()
    }
}
