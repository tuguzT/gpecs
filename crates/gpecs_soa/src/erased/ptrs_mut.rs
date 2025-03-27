use alloc::boxed::Box;
use core::{
    fmt::{self, Debug},
    iter,
    marker::PhantomData,
    ptr,
};

use crate::traits::Soa;

use super::{
    assert::{assert_layouts, validate_layout},
    error::{FromValueError, InvalidLayoutError},
    field::ErasedFieldMutPtr,
};

pub struct ErasedSoaMutPtrs<Fields> {
    ptrs: Box<[ErasedFieldMutPtr]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaMutPtrs<Fields> {
    #[inline]
    pub fn new<I>(ptrs: I) -> Result<Self, InvalidLayoutError>
    where
        I: IntoIterator<Item = ErasedFieldMutPtr>,
    {
        let ptrs = ptrs
            .into_iter()
            .map(|ptr| {
                validate_layout::<Fields>(ptr.descriptor().layout())?;
                Ok(ptr)
            })
            .collect::<Result<Box<[_]>, _>>()?;
        let me = unsafe { Self::actual_new(ptrs) };
        Ok(me)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked<I>(ptrs: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldMutPtr>,
    {
        if cfg!(debug_assertions) {
            return Self::new(ptrs).expect("incorrect inputs");
        }
        unsafe { Self::actual_new(ptrs) }
    }

    #[inline]
    unsafe fn actual_new<I>(ptrs: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldMutPtr>,
    {
        Self {
            ptrs: ptrs.into_iter().collect(),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(
        context: &T::Context,
        ptrs: T::MutPtrs,
    ) -> Result<Self, FromValueError<T::MutPtrs>>
    where
        T: Soa<Fields = Fields>,
    {
        let descriptors = T::field_descriptors(context)
            .into_iter()
            .map(|desc| {
                validate_layout::<Fields>(desc.as_ref().layout())?;
                Ok(desc.as_ref().clone())
            })
            .collect::<Result<Box<[_]>, InvalidLayoutError>>();
        let descriptors = match descriptors {
            Ok(descriptors) => descriptors,
            Err(error) => return Err(FromValueError::new(ptrs, error)),
        };

        let ptrs = T::ptrs_erase_mut(context, ptrs);
        let ptrs = iter::zip(descriptors, ptrs).map(|(desc, ptr)| {
            let len = desc.layout().size();
            let buffer = ptr::slice_from_raw_parts_mut(ptr, len);
            unsafe { ErasedFieldMutPtr::new_unchecked(desc, buffer) }
        });
        let me = unsafe { Self::actual_new(ptrs) };
        Ok(me)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn into<T>(self, context: &T::Context) -> Result<T::MutPtrs, FromValueError<Self>>
    where
        T: Soa<Fields = Fields>,
    {
        let descriptors = T::field_descriptors(context)
            .into_iter()
            .map(|desc| {
                validate_layout::<Fields>(desc.as_ref().layout())?;
                Ok(desc.as_ref().clone())
            })
            .collect::<Result<Box<[_]>, InvalidLayoutError>>();
        let descriptors = match descriptors {
            Ok(descriptors) => descriptors,
            Err(error) => return Err(FromValueError::new(self, error)),
        };

        let Self { ptrs, .. } = self;
        assert_eq!(descriptors.len(), ptrs.len());

        let ptrs = descriptors
            .iter()
            .zip(ptrs)
            .inspect(|(desc, ptr)| assert_layouts(desc.layout(), ptr.descriptor().layout()))
            .map(|(_, ptr)| ptr.as_ptr());
        let ptrs = T::ptrs_restore_mut(context, ptrs);
        Ok(ptrs)
    }

    #[inline]
    pub fn field_ptrs(&self) -> &[ErasedFieldMutPtr] {
        let Self { ptrs, .. } = self;
        ptrs.as_ref()
    }

    #[inline]
    pub fn into_field_ptrs(self) -> Box<[ErasedFieldMutPtr]> {
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
