use alloc::boxed::Box;
use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    mem,
};

use crate::traits::{FieldDescriptor, Soa};

use super::{assert::validate_layout, ErasedFieldMutPtr};

type ErasedDropFnParam<'a> = &'a [ErasedFieldMutPtr];
type ErasedDropFn<'context> = Box<dyn Fn(ErasedDropFnParam<'_>) + 'context>;

pub struct ErasedSoaContext<'context, Fields> {
    descriptors: Box<[FieldDescriptor]>,
    drop_fields: Option<ErasedDropFn<'context>>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<'context, Fields> ErasedSoaContext<'context, Fields> {
    #[inline]
    pub fn new<I, O>(descriptors: I, drop_fields: O) -> Self
    where
        I: IntoIterator<Item: AsRef<FieldDescriptor>>,
        O: Into<Option<ErasedDropFn<'context>>>,
    {
        Self {
            descriptors: descriptors
                .into_iter()
                .inspect(|desc| validate_layout::<Fields>(desc.as_ref().layout()))
                .map(|desc| desc.as_ref().clone())
                .collect(),
            drop_fields: drop_fields.into(),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn of<T>(context: T::Context) -> Self
    where
        T: Soa<Fields = Fields>,
        T::Context: 'context,
    {
        let descriptors = T::field_descriptors(&context)
            .into_iter()
            .inspect(|desc| validate_layout::<T::Fields>(desc.as_ref().layout()))
            .map(|desc| desc.as_ref().clone())
            .collect();
        let drop_fields = mem::needs_drop::<T::Fields>()
            .then(|| Box::new(drop_soa::<T>(context)) as ErasedDropFn<'context>);

        Self {
            descriptors,
            drop_fields,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_ref()
    }

    #[inline]
    pub fn drop_in_place<I>(&self, iter: I)
    where
        I: IntoIterator<Item: AsRef<[ErasedFieldMutPtr]>>,
    {
        let Self {
            descriptors,
            drop_fields,
            ..
        } = self;
        let Some(drop_fields) = drop_fields else {
            return;
        };

        iter.into_iter()
            .inspect(|ptrs| {
                let layouts = ptrs.as_ref().iter().map(|ptr| ptr.descriptor().layout());
                let descriptors = descriptors.iter().copied().map(|desc| desc.layout());
                assert!(descriptors.eq(layouts))
            })
            .for_each(|ptrs| drop_fields(ptrs.as_ref()))
    }
}

impl<Fields> Debug for ErasedSoaContext<'_, Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { descriptors, .. } = self;
        f.debug_struct("ErasedSoaContext")
            .field("descriptors", descriptors)
            .finish_non_exhaustive()
    }
}

#[inline]
pub fn drop_soa<'context, T>(context: T::Context) -> impl Fn(ErasedDropFnParam<'_>) + 'context
where
    T: Soa,
    T::Context: 'context,
{
    move |data: ErasedDropFnParam<'_>| unsafe {
        let ptrs = data.iter().map(ErasedFieldMutPtr::as_ptr);
        let ptrs = T::ptrs_restore_mut(&context, ptrs);
        T::ptrs_drop_in_place(&context, ptrs);
    }
}
