use alloc::boxed::Box;
use core::{
    borrow::Borrow,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    ptr::{self, NonNull},
};

use crate::traits::Soa;

use super::{assert::validate_layout, field::ErasedFieldNonNullPtr};

pub struct ErasedSoaNonNullPtrs<Fields> {
    ptrs: Box<[ErasedFieldNonNullPtr]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaNonNullPtrs<Fields> {
    #[inline]
    #[track_caller]
    pub fn new<I>(ptrs: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldNonNullPtr>,
    {
        Self {
            ptrs: ptrs
                .into_iter()
                .inspect(|ptr| validate_layout::<Fields>(ptr.layout()))
                .collect(),
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
            .inspect(|layout| validate_layout::<T::Fields>(layout.borrow()))
            .map(|layout| layout.borrow().clone());

        let ptrs = field_layouts
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts_mut(ptr, len);
                let buffer = unsafe { NonNull::new_unchecked(ptr) };
                ErasedFieldNonNullPtr::new(field_layout, buffer)
            })
            .collect();
        Self {
            ptrs,
            phantom: PhantomData,
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::NonNullPtrs
    where
        T: Soa<Fields = Fields>,
    {
        let Self { ptrs, .. } = self;

        let field_layouts: Box<[_]> = T::field_layouts(context)
            .into_iter()
            .inspect(|layout| validate_layout::<T::Fields>(layout.borrow()))
            .map(|layout| layout.borrow().clone())
            .collect();
        assert_eq!(field_layouts.len(), ptrs.len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs)
            .inspect(|(&field_layout, ptr)| assert_eq!(field_layout, ptr.layout()))
            .map(|(_, ptr)| ptr.as_ptr().as_ptr());
        let ptrs = T::ptrs_restore_mut(context, ptrs);
        unsafe { T::ptrs_to_nonnull(context, ptrs) }
    }

    #[inline]
    pub fn fields(&self) -> &[ErasedFieldNonNullPtr] {
        let Self { ptrs, .. } = self;
        ptrs.as_ref()
    }

    #[inline]
    pub fn fields_mut(&mut self) -> &mut [ErasedFieldNonNullPtr] {
        let Self { ptrs, .. } = self;
        ptrs.as_mut()
    }

    #[inline]
    pub fn into_fields(self) -> Box<[ErasedFieldNonNullPtr]> {
        let Self { ptrs, .. } = self;
        ptrs
    }
}

impl<Fields> Debug for ErasedSoaNonNullPtrs<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { ptrs, .. } = self;
        f.debug_tuple("ErasedSoaNonNullPtrs").field(ptrs).finish()
    }
}

impl<Fields> PartialEq for ErasedSoaNonNullPtrs<Fields> {
    fn eq(&self, other: &Self) -> bool {
        let Self { ptrs, phantom } = self;
        *ptrs == other.ptrs && *phantom == other.phantom
    }
}

impl<Fields> Eq for ErasedSoaNonNullPtrs<Fields> {}

impl<Fields> Hash for ErasedSoaNonNullPtrs<Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { ptrs, phantom } = self;
        ptrs.hash(state);
        phantom.hash(state);
    }
}

impl<Fields> Clone for ErasedSoaNonNullPtrs<Fields> {
    fn clone(&self) -> Self {
        let Self { ptrs, phantom } = self;
        Self {
            ptrs: ptrs.clone(),
            phantom: phantom.clone(),
        }
    }
}
