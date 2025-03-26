use alloc::boxed::Box;
use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    slice,
};

use crate::traits::Soa;

use super::{
    assert::{assert_layouts, validate_layout},
    field::ErasedFieldRefMut,
};

pub struct ErasedSoaRefsMut<'a, Fields>
where
    Fields: 'a,
{
    refs: Box<[ErasedFieldRefMut<'a>]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<'a, Fields> ErasedSoaRefsMut<'a, Fields> {
    #[inline]
    #[track_caller]
    pub fn new<I>(refs: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldRefMut<'a>>,
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
    pub fn from<T>(context: &T::Context, refs: T::RefsMut<'a>) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let ptrs = T::mut_refs_as_ptrs(context, refs);
        let ptrs = T::ptrs_erase_mut(context, ptrs);
        let descriptors = T::field_descriptors(context)
            .into_iter()
            .inspect(|desc| validate_layout::<Fields>(desc.as_ref().layout()))
            .map(|desc| desc.as_ref().clone());

        let refs = descriptors
            .zip(ptrs)
            .map(|(desc, ptr)| {
                let len = desc.layout().size();
                let buffer = unsafe { slice::from_raw_parts_mut(ptr, len) };
                unsafe { ErasedFieldRefMut::new_unchecked(desc, buffer) }
            })
            .collect();
        Self {
            refs,
            phantom: PhantomData,
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::RefsMut<'a>
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
            .map(|(_, r#ref)| r#ref.into_buffer().as_mut_ptr());
        let ptrs = T::ptrs_restore_mut(context, ptrs);
        unsafe { T::ptrs_to_refs_mut(context, ptrs) }
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
