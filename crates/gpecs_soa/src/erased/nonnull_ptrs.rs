use core::{
    alloc::Layout,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    ptr::{self, NonNull},
    slice,
};

use alloc::{boxed::Box, vec};

use crate::traits::Soa;

use super::validate_layout;

type ErasedFieldNonNullPtr = NonNull<[u8]>;

pub struct ErasedSoaNonNullPtrs<Fields> {
    pub(super) ptrs: Box<[(Layout, ErasedFieldNonNullPtr)]>,
    pub(super) phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaNonNullPtrs<Fields> {
    #[inline]
    pub fn new<I>(ptrs: I) -> Self
    where
        I: IntoIterator<Item = (Layout, ErasedFieldNonNullPtr)>,
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
    pub fn from<T>(context: &T::Context, ptrs: T::NonNullPtrs) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let ptrs = T::nonnull_to_ptrs(context, ptrs);
        let ptrs = T::ptrs_erase_mut(context, ptrs);
        let field_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>);

        let ptrs = field_layouts
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts_mut(ptr.cast(), len);
                (field_layout.clone(), unsafe { NonNull::new_unchecked(ptr) })
            })
            .collect();
        Self {
            ptrs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::NonNullPtrs
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
                ptr.as_ptr().cast()
            });
        let ptrs = T::ptrs_restore_mut(context, ptrs);
        unsafe { T::ptrs_to_nonnull(context, ptrs) }
    }
}

impl<Fields> AsRef<[(Layout, ErasedFieldNonNullPtr)]> for ErasedSoaNonNullPtrs<Fields> {
    fn as_ref(&self) -> &[(Layout, ErasedFieldNonNullPtr)] {
        let Self { ptrs, .. } = self;
        ptrs.as_ref()
    }
}

impl<Fields> AsMut<[(Layout, ErasedFieldNonNullPtr)]> for ErasedSoaNonNullPtrs<Fields> {
    fn as_mut(&mut self) -> &mut [(Layout, ErasedFieldNonNullPtr)] {
        let Self { ptrs, .. } = self;
        ptrs.as_mut()
    }
}

impl<Fields> Debug for ErasedSoaNonNullPtrs<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ErasedSoaNonNullPtrs")
            .field(&self.ptrs)
            .finish()
    }
}

impl<Fields> PartialEq for ErasedSoaNonNullPtrs<Fields> {
    fn eq(&self, other: &Self) -> bool {
        self.ptrs == other.ptrs && self.phantom == other.phantom
    }
}

impl<Fields> Eq for ErasedSoaNonNullPtrs<Fields> {}

impl<Fields> Hash for ErasedSoaNonNullPtrs<Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.ptrs.hash(state);
        self.phantom.hash(state);
    }
}

impl<Fields> Clone for ErasedSoaNonNullPtrs<Fields> {
    fn clone(&self) -> Self {
        Self {
            ptrs: self.ptrs.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

impl<'a, Fields> IntoIterator for &'a ErasedSoaNonNullPtrs<Fields> {
    type Item = &'a (Layout, ErasedFieldNonNullPtr);

    type IntoIter = slice::Iter<'a, (Layout, ErasedFieldNonNullPtr)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaNonNullPtrs { ptrs, .. } = self;
        ptrs.iter()
    }
}

impl<'a, Fields> IntoIterator for &'a mut ErasedSoaNonNullPtrs<Fields> {
    type Item = &'a mut (Layout, ErasedFieldNonNullPtr);

    type IntoIter = slice::IterMut<'a, (Layout, ErasedFieldNonNullPtr)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaNonNullPtrs { ptrs, .. } = self;
        ptrs.iter_mut()
    }
}

impl<Fields> IntoIterator for ErasedSoaNonNullPtrs<Fields> {
    type Item = (Layout, ErasedFieldNonNullPtr);

    type IntoIter = vec::IntoIter<(Layout, ErasedFieldNonNullPtr)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaNonNullPtrs { ptrs, .. } = self;
        ptrs.into_vec().into_iter()
    }
}
