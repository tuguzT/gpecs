use alloc::boxed::Box;
use core::{
    borrow::Borrow,
    fmt::{self, Debug},
    marker::PhantomData,
    ptr,
};

use crate::traits::Soa;

use super::{
    assert::{assert_layouts, validate_layout},
    field::ErasedFieldMutPtr,
};

pub struct ErasedSoaMutPtrs<Fields> {
    ptrs: Box<[ErasedFieldMutPtr]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaMutPtrs<Fields> {
    #[inline]
    #[track_caller]
    pub fn new<I>(ptrs: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldMutPtr>,
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
    pub fn from<T>(context: &T::Context, ptrs: T::MutPtrs) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let ptrs = T::ptrs_erase_mut(context, ptrs);
        let descriptors = T::field_descriptors(context)
            .into_iter()
            .inspect(|desc| validate_layout::<Fields>(desc.borrow().layout()))
            .map(|desc| desc.borrow().clone());

        let ptrs: Box<[_]> = descriptors
            .zip(ptrs)
            .map(|(desc, ptr)| {
                let len = desc.layout().size();
                let ptr = ptr::slice_from_raw_parts_mut(ptr, len);
                ErasedFieldMutPtr::new(desc, ptr)
            })
            .collect();
        Self {
            ptrs,
            phantom: PhantomData,
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::MutPtrs
    where
        T: Soa<Fields = Fields>,
    {
        let Self { ptrs, .. } = self;

        let descriptors: Box<[_]> = T::field_descriptors(context)
            .into_iter()
            .inspect(|desc| validate_layout::<Fields>(desc.borrow().layout()))
            .map(|desc| desc.borrow().clone())
            .collect();
        assert_eq!(descriptors.len(), ptrs.len());

        let ptrs = descriptors
            .iter()
            .zip(ptrs)
            .inspect(|(desc, ptr)| assert_layouts(desc.layout(), ptr.descriptor().layout()))
            .map(|(_, ptr)| ptr.as_ptr());
        T::ptrs_restore_mut(context, ptrs)
    }

    #[inline]
    pub fn fields(&self) -> &[ErasedFieldMutPtr] {
        let Self { ptrs, .. } = self;
        ptrs.as_ref()
    }

    #[inline]
    pub fn fields_mut(&mut self) -> &mut [ErasedFieldMutPtr] {
        let Self { ptrs, .. } = self;
        ptrs.as_mut()
    }

    #[inline]
    pub fn into_fields(self) -> Box<[ErasedFieldMutPtr]> {
        let Self { ptrs, .. } = self;
        ptrs
    }
}

impl<Fields> Debug for ErasedSoaMutPtrs<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { ptrs, .. } = self;
        f.debug_tuple("ErasedSoaMutPtrs").field(ptrs).finish()
    }
}

impl<Fields> Clone for ErasedSoaMutPtrs<Fields> {
    fn clone(&self) -> Self {
        let Self { ptrs, phantom } = self;
        Self {
            ptrs: ptrs.clone(),
            phantom: phantom.clone(),
        }
    }
}
