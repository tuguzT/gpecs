use alloc::boxed::Box;
use core::{
    borrow::Borrow,
    fmt::{self, Debug},
    marker::PhantomData,
    ptr::{self, NonNull},
};

use crate::traits::Soa;

use super::{
    assert::{assert_layouts, validate_layout},
    field::ErasedFieldNonNullPtr,
};

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
                .inspect(|ptr| validate_layout::<Fields>(ptr.descriptor().layout()))
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
        let descriptors = T::field_descriptors(context)
            .into_iter()
            .inspect(|desc| validate_layout::<T::Fields>(desc.borrow().layout()))
            .map(|desc| desc.borrow().clone());

        let ptrs = descriptors
            .zip(ptrs)
            .map(|(desc, ptr)| {
                let len = desc.layout().size();
                let ptr = ptr::slice_from_raw_parts_mut(ptr, len);
                let buffer = unsafe { NonNull::new_unchecked(ptr) };
                ErasedFieldNonNullPtr::new(desc, buffer)
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

        let descriptors: Box<[_]> = T::field_descriptors(context)
            .into_iter()
            .inspect(|desc| validate_layout::<T::Fields>(desc.borrow().layout()))
            .map(|desc| desc.borrow().clone())
            .collect();
        assert_eq!(descriptors.len(), ptrs.len());

        let ptrs = descriptors
            .iter()
            .zip(ptrs)
            .inspect(|(desc, ptr)| assert_layouts(desc.layout(), ptr.descriptor().layout()))
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

impl<Fields> Clone for ErasedSoaNonNullPtrs<Fields> {
    fn clone(&self) -> Self {
        let Self { ptrs, phantom } = self;
        Self {
            ptrs: ptrs.clone(),
            phantom: phantom.clone(),
        }
    }
}
