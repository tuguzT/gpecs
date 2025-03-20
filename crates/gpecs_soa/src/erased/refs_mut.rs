use alloc::boxed::Box;
use core::{
    borrow::Borrow,
    fmt::{self, Debug},
    marker::PhantomData,
    slice,
};

use crate::traits::Soa;

use super::{assert::validate_layout, field::ErasedFieldRefMut};

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
                .inspect(|r#ref| validate_layout::<Fields>(r#ref.layout()))
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
        let field_layouts = T::field_layouts(context)
            .into_iter()
            .inspect(|layout| validate_layout::<Fields>(layout.borrow()))
            .map(|layout| layout.borrow().clone());

        let refs = field_layouts
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let len = field_layout.size();
                let buffer = unsafe { slice::from_raw_parts_mut(ptr, len) };
                ErasedFieldRefMut::new(field_layout, buffer)
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

        let field_layouts: Box<[_]> = T::field_layouts(context)
            .into_iter()
            .inspect(|layout| validate_layout::<Fields>(layout.borrow()))
            .map(|layout| layout.borrow().clone())
            .collect();
        assert_eq!(field_layouts.len(), refs.len());

        let ptrs = field_layouts
            .iter()
            .zip(refs)
            .inspect(|(&field_layout, r#ref)| assert_eq!(field_layout, r#ref.layout()))
            .map(|(_, r#ref)| r#ref.into_buffer().as_mut_ptr());
        let ptrs = T::ptrs_restore_mut(context, ptrs);
        unsafe { T::ptrs_to_refs_mut(context, ptrs) }
    }

    #[inline]
    pub fn fields(&self) -> &[ErasedFieldRefMut<'a>] {
        let Self { refs, .. } = self;
        refs.as_ref()
    }

    #[inline]
    pub fn fields_mut(&mut self) -> &mut [ErasedFieldRefMut<'a>] {
        let Self { refs, .. } = self;
        refs.as_mut()
    }

    #[inline]
    pub fn into_fields(self) -> Box<[ErasedFieldRefMut<'a>]> {
        let Self { refs, .. } = self;
        refs
    }
}

impl<'a, Fields> AsRef<[ErasedFieldRefMut<'a>]> for ErasedSoaRefsMut<'a, Fields> {
    fn as_ref(&self) -> &[ErasedFieldRefMut<'a>] {
        let Self { refs, .. } = self;
        refs.as_ref()
    }
}

impl<'a, Fields> AsMut<[ErasedFieldRefMut<'a>]> for ErasedSoaRefsMut<'a, Fields> {
    fn as_mut(&mut self) -> &mut [ErasedFieldRefMut<'a>] {
        let Self { refs, .. } = self;
        refs.as_mut()
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
