use alloc::boxed::Box;
use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    slice,
};

use crate::{
    assert::{check_same_layout, check_same_len, validate_layout},
    error::InvalidLayoutError,
    field::ErasedFieldRefMut,
    soa::traits::Soa,
};

use super::error::{FromValueError, IntoValueError};

pub struct ErasedSoaRefsMut<'a, Fields>
where
    Fields: 'a,
{
    refs: Box<[ErasedFieldRefMut<'a>]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<'a, Fields> ErasedSoaRefsMut<'a, Fields> {
    #[inline]
    pub fn new<I>(refs: I) -> Result<Self, InvalidLayoutError>
    where
        I: IntoIterator<Item = ErasedFieldRefMut<'a>>,
    {
        let refs = refs
            .into_iter()
            .map(|r#ref| {
                validate_layout::<Fields>(r#ref.descriptor().layout())?;
                Ok(r#ref)
            })
            .collect::<Result<Box<[_]>, _>>()?;
        let me = unsafe { Self::actual_new(refs) };
        Ok(me)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked<I>(refs: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldRefMut<'a>>,
    {
        if cfg!(debug_assertions) {
            return Self::new(refs).expect("incorrect inputs");
        }
        unsafe { Self::actual_new(refs) }
    }

    #[inline]
    unsafe fn actual_new<I>(refs: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldRefMut<'a>>,
    {
        Self {
            refs: refs.into_iter().collect(),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(
        context: &T::Context,
        refs: T::RefsMut<'a>,
    ) -> Result<Self, FromValueError<T::RefsMut<'a>>>
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
            Err(error) => return Err(FromValueError::new(refs, error)),
        };

        let ptrs = T::mut_refs_as_ptrs(context, refs);
        let ptrs = T::ptrs_erase_mut(context, ptrs);
        let refs = descriptors
            .into_vec()
            .into_iter()
            .zip(ptrs)
            .map(|(desc, ptr)| {
                let len = desc.layout().size();
                let buffer = unsafe { slice::from_raw_parts_mut(ptr, len) };
                unsafe { ErasedFieldRefMut::new_unchecked(desc, buffer) }
            });
        let me = unsafe { Self::actual_new(refs) };
        Ok(me)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn into<T>(
        self,
        context: &T::Context,
    ) -> Result<T::RefsMut<'a>, IntoValueError<Self>>
    where
        T: Soa<Fields = Fields>,
    {
        let Self { refs, .. } = &self;
        let result = T::field_descriptors(context)
            .into_iter()
            .zip(refs)
            .try_fold(0, |len, (desc, r#ref)| {
                validate_layout::<Fields>(desc.as_ref().layout())?;
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
            .map(|r#ref| r#ref.into_buffer().as_mut_ptr());

        let ptrs = T::ptrs_restore_mut(context, ptrs);
        let refs = unsafe { T::ptrs_to_refs_mut(context, ptrs) };
        Ok(refs)
    }

    #[inline]
    pub fn field_refs(&self) -> &[ErasedFieldRefMut<'a>] {
        let Self { refs, .. } = self;
        refs.as_ref()
    }

    #[inline]
    pub fn into_field_refs(self) -> Box<[ErasedFieldRefMut<'a>]> {
        let Self { refs, .. } = self;
        refs
    }
}

impl<'a, Fields> Debug for ErasedSoaRefsMut<'a, Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { refs, .. } = self;
        f.debug_tuple("ErasedSoaRefsMut").field(refs).finish()
    }
}

unsafe impl<'a, Fields> Send for ErasedSoaRefsMut<'a, Fields> where Fields: Send {}
unsafe impl<'a, Fields> Sync for ErasedSoaRefsMut<'a, Fields> where Fields: Sync {}
