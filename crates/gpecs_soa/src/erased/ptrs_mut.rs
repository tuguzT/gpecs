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
type ErasedFieldMutPtr = *mut [u8];

pub struct ErasedSoaMutPtrs<Fields> {
    pub(super) ptrs: Box<[(Layout, ErasedFieldMutPtr)]>,
    pub(super) phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaMutPtrs<Fields> {
    #[inline]
    pub fn new<I>(ptrs: I) -> Self
    where
        I: IntoIterator<Item = (Layout, ErasedFieldMutPtr)>,
    {
        let ptrs = ptrs
            .into_iter()
            .map(|(field_layout, ptr)| {
                assert_eq!(field_layout.size(), ptr.len());
                (field_layout.clone(), ptr)
            })
            .collect();
        Self {
            ptrs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(context: &T::Context, ptrs: T::MutPtrs) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let ptrs = T::ptrs_erase_mut(context, ptrs);
        let field_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>);

        let ptrs: Box<[_]> = field_layouts
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let data = ptr.cast();
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts_mut(data, len);
                (field_layout.clone(), ptr)
            })
            .collect();
        Self {
            ptrs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::MutPtrs
    where
        T: Soa<Fields = Fields>,
    {
        let Self { ptrs, .. } = self;

        let field_layouts: Box<[_]> = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>)
            .collect();
        assert_eq!(field_layouts.len(), ptrs.len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, (layout, ptr))| {
                assert_eq!(field_layout, &layout);
                ptr.cast()
            });
        T::ptrs_restore_mut(context, ptrs)
    }
}

impl<Fields> AsRef<[(Layout, ErasedFieldMutPtr)]> for ErasedSoaMutPtrs<Fields> {
    fn as_ref(&self) -> &[(Layout, ErasedFieldMutPtr)] {
        let Self { ptrs, .. } = self;
        ptrs.as_ref()
    }
}

impl<Fields> AsMut<[(Layout, ErasedFieldMutPtr)]> for ErasedSoaMutPtrs<Fields> {
    fn as_mut(&mut self) -> &mut [(Layout, ErasedFieldMutPtr)] {
        let Self { ptrs, .. } = self;
        ptrs.as_mut()
    }
}

impl<Fields> Debug for ErasedSoaMutPtrs<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ErasedSoaMutPtrs").field(&self.ptrs).finish()
    }
}

impl<Fields> PartialEq for ErasedSoaMutPtrs<Fields> {
    fn eq(&self, other: &Self) -> bool {
        self.ptrs == other.ptrs && self.phantom == other.phantom
    }
}

impl<Fields> Eq for ErasedSoaMutPtrs<Fields> {}

impl<Fields> Hash for ErasedSoaMutPtrs<Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.ptrs.hash(state);
        self.phantom.hash(state);
    }
}

impl<Fields> Clone for ErasedSoaMutPtrs<Fields> {
    fn clone(&self) -> Self {
        Self {
            ptrs: self.ptrs.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

impl<'a, Fields> IntoIterator for &'a ErasedSoaMutPtrs<Fields> {
    type Item = &'a (Layout, ErasedFieldMutPtr);

    type IntoIter = slice::Iter<'a, (Layout, ErasedFieldMutPtr)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaMutPtrs { ptrs, .. } = self;
        ptrs.iter()
    }
}

impl<'a, Fields> IntoIterator for &'a mut ErasedSoaMutPtrs<Fields> {
    type Item = &'a mut (Layout, ErasedFieldMutPtr);

    type IntoIter = slice::IterMut<'a, (Layout, ErasedFieldMutPtr)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaMutPtrs { ptrs, .. } = self;
        ptrs.iter_mut()
    }
}

impl<Fields> IntoIterator for ErasedSoaMutPtrs<Fields> {
    type Item = (Layout, ErasedFieldMutPtr);

    type IntoIter = vec::IntoIter<(Layout, ErasedFieldMutPtr)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaMutPtrs { ptrs, .. } = self;
        ptrs.into_vec().into_iter()
    }
}
