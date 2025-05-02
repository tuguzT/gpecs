use alloc::boxed::Box;
use core::ptr;

use crate::{
    assert::{check_same_layout, check_same_len},
    field::ErasedFieldPtr,
    soa::traits::Soa,
};

use super::error::IntoValueError;

#[derive(Debug, Clone)]
pub struct ErasedSoaPtrs {
    ptrs: Box<[ErasedFieldPtr]>,
}

impl ErasedSoaPtrs {
    #[inline]
    pub fn new<I>(ptrs: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldPtr>,
    {
        let ptrs = ptrs.into_iter().collect();
        Self { ptrs }
    }

    #[inline]
    pub fn from<'context, T>(context: &'context T::Context, ptrs: T::Ptrs<'context>) -> Self
    where
        T: Soa,
    {
        let ptrs = T::ptrs_erase(context, ptrs);
        let ptrs = T::field_descriptors(context)
            .into_iter()
            .zip(ptrs)
            .map(|(desc, ptr)| {
                let desc = desc.as_ref().clone();
                let len = desc.layout().size();
                let buffer = ptr::slice_from_raw_parts(ptr, len);
                unsafe { ErasedFieldPtr::new_unchecked(desc, buffer) }
            });
        Self::new(ptrs)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn into<T>(self, context: &T::Context) -> Result<T::Ptrs<'_>, IntoValueError<Self>>
    where
        T: Soa,
    {
        let Self { ptrs, .. } = &self;
        let result = T::field_descriptors(context)
            .into_iter()
            .zip(ptrs)
            .try_fold(0, |len, (desc, slice)| {
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
        let ptrs = ptrs.into_vec().into_iter().map(|slice| slice.as_ptr());

        let ptrs = T::ptrs_restore(context, ptrs);
        Ok(ptrs)
    }

    #[inline]
    pub fn field_ptrs(&self) -> &[ErasedFieldPtr] {
        let Self { ptrs, .. } = self;
        ptrs.as_ref()
    }

    #[inline]
    pub fn into_field_ptrs(self) -> Box<[ErasedFieldPtr]> {
        let Self { ptrs, .. } = self;
        ptrs
    }
}
