use alloc::boxed::Box;
use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    slice,
};

use crate::traits::Soa;

use super::{
    assert::{assert_layouts, validate_layout},
    field::ErasedFieldRef,
};

pub struct ErasedSoaRefs<'a, Fields>
where
    Fields: 'a,
{
    refs: Box<[ErasedFieldRef<'a>]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<'a, Fields> ErasedSoaRefs<'a, Fields> {
    #[inline]
    #[track_caller]
    pub fn new<I>(refs: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldRef<'a>>,
    {
        Self {
            refs: refs
                .into_iter()
                .inspect(|r#ref| validate_layout::<Fields>(r#ref.descriptor().layout()))
                .collect(),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(context: &T::Context, refs: T::Refs<'a>) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let ptrs = T::refs_as_ptrs(context, refs);
        let ptrs = T::ptrs_erase(context, ptrs);
        let descriptors = T::field_descriptors(context)
            .into_iter()
            .inspect(|desc| validate_layout::<Fields>(desc.as_ref().layout()))
            .map(|desc| desc.as_ref().clone());

        let refs: Box<[_]> = descriptors
            .zip(ptrs)
            .map(|(desc, ptr)| {
                let len = desc.layout().size();
                let buffer = unsafe { slice::from_raw_parts(ptr, len) };
                ErasedFieldRef::new(desc, buffer)
            })
            .collect();
        Self {
            refs,
            phantom: PhantomData,
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::Refs<'a>
    where
        T: Soa<Fields = Fields>,
    {
        let Self { refs, .. } = self;

        let descriptors: Box<[_]> = T::field_descriptors(context)
            .into_iter()
            .inspect(|desc| validate_layout::<Fields>(desc.as_ref().layout()))
            .map(|desc| desc.as_ref().clone())
            .collect();
        assert_eq!(descriptors.len(), refs.len());

        let ptrs = descriptors
            .iter()
            .zip(refs)
            .inspect(|(desc, r#ref)| assert_layouts(desc.layout(), r#ref.descriptor().layout()))
            .map(|(_, r#ref)| r#ref.into_buffer().as_ptr());
        let ptrs = T::ptrs_restore(context, ptrs);
        unsafe { T::ptrs_to_refs(context, ptrs) }
    }

    #[inline]
    pub fn field_refs(&self) -> &[ErasedFieldRef<'a>] {
        let Self { refs, .. } = self;
        refs.as_ref()
    }

    #[inline]
    pub fn field_refs_mut(&mut self) -> &mut [ErasedFieldRef<'a>] {
        let Self { refs, .. } = self;
        refs.as_mut()
    }

    #[inline]
    pub fn into_field_refs(self) -> Box<[ErasedFieldRef<'a>]> {
        let Self { refs, .. } = self;
        refs
    }
}

impl<'a, Fields> Debug for ErasedSoaRefs<'a, Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { refs, .. } = self;
        f.debug_tuple("ErasedSoaRefs").field(refs).finish()
    }
}

impl<'a, Fields> Clone for ErasedSoaRefs<'a, Fields> {
    fn clone(&self) -> Self {
        let Self { refs, phantom } = self;
        Self {
            refs: refs.clone(),
            phantom: phantom.clone(),
        }
    }
}

unsafe impl<'a, Fields> Send for ErasedSoaRefs<'a, Fields> where Fields: Sync {}
unsafe impl<'a, Fields> Sync for ErasedSoaRefs<'a, Fields> where Fields: Sync {}
