use alloc::boxed::Box;
use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    ptr::{self, NonNull},
};

use crate::{
    assert::{check_same_layout, check_same_len, validate_layout},
    error::InvalidLayoutError,
    field::ErasedFieldNonNullPtr,
    soa::traits::Soa,
};

use super::error::{FromValueError, IntoValueError};

pub struct ErasedSoaNonNullPtrs<Fields> {
    ptrs: Box<[ErasedFieldNonNullPtr]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaNonNullPtrs<Fields> {
    #[inline]
    pub fn new<I>(ptrs: I) -> Result<Self, InvalidLayoutError>
    where
        I: IntoIterator<Item = ErasedFieldNonNullPtr>,
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
        I: IntoIterator<Item = ErasedFieldNonNullPtr>,
    {
        if cfg!(debug_assertions) {
            return Self::new(ptrs).expect("incorrect inputs");
        }
        unsafe { Self::actual_new(ptrs) }
    }

    #[inline]
    unsafe fn actual_new<I>(ptrs: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldNonNullPtr>,
    {
        Self {
            ptrs: ptrs.into_iter().collect(),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(
        context: &T::Context,
        ptrs: T::NonNullPtrs,
    ) -> Result<Self, FromValueError<T::NonNullPtrs>>
    where
        T: Soa<Fields = Fields>,
    {
        let descriptors = T::field_descriptors(context)
            .into_iter()
            .map(|desc| {
                validate_layout::<T::Fields>(desc.as_ref().layout())?;
                Ok(desc.as_ref().clone())
            })
            .collect::<Result<Box<[_]>, InvalidLayoutError>>();
        let descriptors = match descriptors {
            Ok(descriptors) => descriptors,
            Err(error) => return Err(FromValueError::new(ptrs, error)),
        };

        let ptrs = T::nonnull_to_ptrs(context, ptrs);
        let ptrs = T::ptrs_erase_mut(context, ptrs);
        let ptrs = descriptors
            .into_vec()
            .into_iter()
            .zip(ptrs)
            .map(|(desc, ptr)| {
                let len = desc.layout().size();
                let ptr = ptr::slice_from_raw_parts_mut(ptr, len);
                let buffer = unsafe { NonNull::new_unchecked(ptr) };
                unsafe { ErasedFieldNonNullPtr::new_unchecked(desc, buffer) }
            });
        let me = unsafe { Self::actual_new(ptrs) };
        Ok(me)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn into<T>(
        self,
        context: &T::Context,
    ) -> Result<T::NonNullPtrs, IntoValueError<Self>>
    where
        T: Soa<Fields = Fields>,
    {
        let Self { ptrs, .. } = &self;
        let result = T::field_descriptors(context)
            .into_iter()
            .zip(ptrs)
            .try_fold(0, |len, (desc, slice)| {
                validate_layout::<Fields>(desc.as_ref().layout())?;
                check_same_layout(slice.descriptor().layout(), desc.as_ref().layout())?;
                Ok(len + 1)
            })
            .and_then(|len| {
                check_same_len(len, ptrs.len())?;
                Ok(())
            });
        if let Err(error) = result {
            return Err(IntoValueError::new(self, error));
        }

        let Self { ptrs, .. } = self;
        let ptrs = ptrs
            .into_vec()
            .into_iter()
            .map(|slice| slice.as_ptr().as_ptr());

        let ptrs = T::ptrs_restore_mut(context, ptrs);
        let ptrs = unsafe { T::ptrs_to_nonnull(context, ptrs) };
        Ok(ptrs)
    }

    #[inline]
    pub fn field_ptrs(&self) -> &[ErasedFieldNonNullPtr] {
        let Self { ptrs, .. } = self;
        ptrs.as_ref()
    }

    #[inline]
    pub fn into_field_ptrs(self) -> Box<[ErasedFieldNonNullPtr]> {
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
