use alloc::boxed::Box;
use core::slice;

use crate::{
    assert::{check_same_layout, check_same_len},
    field::ErasedFieldRef,
    soa::traits::Soa,
};

use super::error::IntoValueError;

#[derive(Debug, Clone)]
pub struct ErasedSoaRefs<'a> {
    refs: Box<[ErasedFieldRef<'a>]>,
}

impl<'a> ErasedSoaRefs<'a> {
    #[inline]
    pub fn new<I>(refs: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldRef<'a>>,
    {
        let refs = refs.into_iter().collect();
        Self { refs }
    }

    #[inline]
    pub fn from<T>(context: &T::Context, refs: T::Refs<'a>) -> Self
    where
        T: Soa,
    {
        let ptrs = T::refs_as_ptrs(context, refs);
        let ptrs = T::ptrs_erase(context, ptrs);
        let refs = T::field_descriptors(context)
            .into_iter()
            .zip(ptrs)
            .map(|(desc, ptr)| {
                let desc = desc.as_ref().clone();
                let len = desc.layout().size();
                let buffer = unsafe { slice::from_raw_parts(ptr, len) };
                unsafe { ErasedFieldRef::new_unchecked(desc, buffer) }
            });
        Self::new(refs)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn into<T>(self, context: &T::Context) -> Result<T::Refs<'a>, IntoValueError<Self>>
    where
        T: Soa,
    {
        let Self { refs, .. } = &self;
        let result = T::field_descriptors(context)
            .into_iter()
            .zip(refs)
            .try_fold(0, |len, (desc, r#ref)| {
                check_same_layout(r#ref.descriptor().layout(), desc.as_ref().layout())?;
                Ok(len + 1)
            })
            .and_then(|len| {
                check_same_len(len, refs.len())?;
                Ok(())
            });
        if let Err(error) = result {
            return Err(IntoValueError::new(self, error));
        }

        let Self { refs, .. } = self;
        let ptrs = refs
            .into_vec()
            .into_iter()
            .map(|r#ref| r#ref.into_buffer().as_ptr());

        let ptrs = T::ptrs_restore(context, ptrs);
        let refs = unsafe { T::ptrs_to_refs(context, ptrs) };
        Ok(refs)
    }

    #[inline]
    pub fn field_refs(&self) -> &[ErasedFieldRef<'a>] {
        let Self { refs, .. } = self;
        refs.as_ref()
    }

    #[inline]
    pub fn into_field_refs(self) -> Box<[ErasedFieldRef<'a>]> {
        let Self { refs, .. } = self;
        refs
    }
}
