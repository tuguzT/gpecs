use core::{
    alloc::Layout,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    ptr, slice,
};

use alloc::{boxed::Box, vec};

use crate::traits::Soa;

use super::validate_layout;

type ErasedFieldPtr = *const [u8];

pub struct ErasedSoaPtrs<Fields> {
    pub(super) ptrs: Box<[(Layout, ErasedFieldPtr)]>,
    pub(super) phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaPtrs<Fields> {
    #[inline]
    pub fn new<I>(ptrs: I) -> Self
    where
        I: IntoIterator<Item = (Layout, ErasedFieldPtr)>,
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
    pub fn from<T>(context: &T::Context, ptrs: T::Ptrs) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let ptrs = T::ptrs_erase(context, ptrs);
        let field_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>);

        let ptrs = field_layouts
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts(ptr.cast(), len);
                (field_layout, ptr)
            })
            .collect();
        Self {
            ptrs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::Ptrs
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
        T::ptrs_restore(context, ptrs)
    }
}

impl<Fields> AsRef<[(Layout, ErasedFieldPtr)]> for ErasedSoaPtrs<Fields> {
    fn as_ref(&self) -> &[(Layout, ErasedFieldPtr)] {
        let Self { ptrs, .. } = self;
        ptrs.as_ref()
    }
}

impl<Fields> AsMut<[(Layout, ErasedFieldPtr)]> for ErasedSoaPtrs<Fields> {
    fn as_mut(&mut self) -> &mut [(Layout, ErasedFieldPtr)] {
        let Self { ptrs, .. } = self;
        ptrs.as_mut()
    }
}

impl<Fields> Debug for ErasedSoaPtrs<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ErasedSoaPtrs").field(&self.ptrs).finish()
    }
}

impl<Fields> PartialEq for ErasedSoaPtrs<Fields> {
    fn eq(&self, other: &Self) -> bool {
        self.ptrs == other.ptrs && self.phantom == other.phantom
    }
}

impl<Fields> Eq for ErasedSoaPtrs<Fields> {}

impl<Fields> Hash for ErasedSoaPtrs<Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.ptrs.hash(state);
        self.phantom.hash(state);
    }
}

impl<Fields> Clone for ErasedSoaPtrs<Fields> {
    fn clone(&self) -> Self {
        Self {
            ptrs: self.ptrs.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

impl<'a, Fields> IntoIterator for &'a ErasedSoaPtrs<Fields> {
    type Item = &'a (Layout, ErasedFieldPtr);

    type IntoIter = slice::Iter<'a, (Layout, ErasedFieldPtr)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaPtrs { ptrs, .. } = self;
        ptrs.iter()
    }
}

impl<'a, Fields> IntoIterator for &'a mut ErasedSoaPtrs<Fields> {
    type Item = &'a mut (Layout, ErasedFieldPtr);

    type IntoIter = slice::IterMut<'a, (Layout, ErasedFieldPtr)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaPtrs { ptrs, .. } = self;
        ptrs.iter_mut()
    }
}

impl<Fields> IntoIterator for ErasedSoaPtrs<Fields> {
    type Item = (Layout, ErasedFieldPtr);

    type IntoIter = vec::IntoIter<(Layout, ErasedFieldPtr)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaPtrs { ptrs, .. } = self;
        ptrs.into_vec().into_iter()
    }
}
