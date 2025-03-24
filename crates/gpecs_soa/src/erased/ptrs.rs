use alloc::boxed::Box;
use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    ptr,
};

use crate::traits::Soa;

use super::{
    assert::{assert_layouts, validate_layout},
    field::ErasedFieldPtr,
};

pub struct ErasedSoaPtrs<Fields> {
    ptrs: Box<[ErasedFieldPtr]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaPtrs<Fields> {
    #[inline]
    #[track_caller]
    pub fn new<I>(ptrs: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldPtr>,
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
    pub fn from<T>(context: &T::Context, ptrs: T::Ptrs) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let ptrs = T::ptrs_erase(context, ptrs);
        let descriptors = T::field_descriptors(context)
            .into_iter()
            .inspect(|desc| validate_layout::<Fields>(desc.as_ref().layout()))
            .map(|desc| desc.as_ref().clone());

        let ptrs = descriptors
            .zip(ptrs)
            .map(|(desc, ptr)| {
                let len = desc.layout().size();
                let buffer = ptr::slice_from_raw_parts(ptr, len);
                ErasedFieldPtr::new(desc, buffer)
            })
            .collect();
        Self {
            ptrs,
            phantom: PhantomData,
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::Ptrs
    where
        T: Soa<Fields = Fields>,
    {
        let Self { ptrs, .. } = self;

        let descriptors: Box<[_]> = T::field_descriptors(context)
            .into_iter()
            .inspect(|desc| validate_layout::<Fields>(desc.as_ref().layout()))
            .map(|desc| desc.as_ref().clone())
            .collect();
        assert_eq!(descriptors.len(), ptrs.len());

        let ptrs = descriptors
            .iter()
            .zip(ptrs)
            .inspect(|(desc, ptr)| assert_layouts(desc.layout(), ptr.descriptor().layout()))
            .map(|(_, ptr)| ptr.as_ptr());
        T::ptrs_restore(context, ptrs)
    }

    #[inline]
    pub fn fields(&self) -> &[ErasedFieldPtr] {
        let Self { ptrs, .. } = self;
        ptrs.as_ref()
    }

    #[inline]
    pub fn fields_mut(&mut self) -> &mut [ErasedFieldPtr] {
        let Self { ptrs, .. } = self;
        ptrs.as_mut()
    }

    #[inline]
    pub fn into_fields(self) -> Box<[ErasedFieldPtr]> {
        let Self { ptrs, .. } = self;
        ptrs
    }
}

impl<Fields> Debug for ErasedSoaPtrs<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { ptrs, .. } = self;
        f.debug_tuple("ErasedSoaPtrs").field(ptrs).finish()
    }
}

impl<Fields> Clone for ErasedSoaPtrs<Fields> {
    fn clone(&self) -> Self {
        let Self { ptrs, phantom } = self;
        Self {
            ptrs: ptrs.clone(),
            phantom: phantom.clone(),
        }
    }
}
