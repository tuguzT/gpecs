use alloc::boxed::Box;
use core::{
    borrow::Borrow,
    fmt::{self, Debug},
    marker::PhantomData,
    mem,
};

use crate::traits::{FieldDescriptor, Soa};

use super::{assert::validate_layout, ErasedFieldMutPtr};

type ErasedDropFnParam<'a> = &'a [ErasedFieldMutPtr];
type ErasedDropFn = Box<dyn Fn(ErasedDropFnParam<'_>)>;

pub struct ErasedSoaContext<Fields> {
    descriptors: Box<[FieldDescriptor]>,
    drop_fields: Option<ErasedDropFn>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaContext<Fields> {
    #[inline]
    pub fn new<I, O>(descriptors: I, drop_fields: O) -> Self
    where
        I: IntoIterator<Item: Borrow<FieldDescriptor>>,
        O: Into<Option<ErasedDropFn>>,
    {
        Self {
            descriptors: descriptors
                .into_iter()
                .inspect(|desc| validate_layout::<Fields>(desc.borrow().layout()))
                .map(|desc| desc.borrow().clone())
                .collect(),
            drop_fields: drop_fields.into(),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn of<T>(context: T::Context) -> Self
    where
        T: Soa<Fields = Fields>,
        T::Context: 'static,
    {
        let descriptors = T::field_descriptors(&context)
            .into_iter()
            .inspect(|desc| validate_layout::<T::Fields>(desc.borrow().layout()))
            .map(|desc| desc.borrow().clone())
            .collect();

        let drop_fields = move |data: ErasedDropFnParam<'_>| unsafe {
            let ptrs = data.iter().map(ErasedFieldMutPtr::as_ptr);
            let ptrs = T::ptrs_restore_mut(&context, ptrs);
            T::ptrs_drop_in_place(&context, ptrs);
        };
        let drop_fields: Option<ErasedDropFn> = if mem::needs_drop::<T::Fields>() {
            Some(Box::new(drop_fields))
        } else {
            None
        };

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
        I: IntoIterator<Item: Borrow<[ErasedFieldMutPtr]>>,
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
                let layouts = ptrs.borrow().iter().map(|ptr| ptr.descriptor().layout());
                let descriptors = descriptors.iter().copied().map(|desc| desc.layout());
                assert!(descriptors.eq(layouts))
            })
            .for_each(|ptrs| drop_fields(ptrs.borrow()))
    }
}

impl<Fields> Debug for ErasedSoaContext<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { descriptors, .. } = self;
        f.debug_struct("ErasedSoaContext")
            .field("descriptors", descriptors)
            .finish_non_exhaustive()
    }
}
