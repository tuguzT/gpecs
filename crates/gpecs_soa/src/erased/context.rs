use alloc::boxed::Box;
use core::{
    fmt::{self, Debug},
    marker::PhantomData,
};

use crate::traits::{FieldDescriptor, Soa};

use super::assert::validate_layout;

pub struct ErasedSoaContext<Fields> {
    descriptors: Box<[FieldDescriptor]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaContext<Fields> {
    #[inline]
    pub fn new<I>(descriptors: I) -> Self
    where
        I: IntoIterator<Item: AsRef<FieldDescriptor>>,
    {
        Self {
            descriptors: descriptors
                .into_iter()
                .inspect(|desc| validate_layout::<Fields>(desc.as_ref().layout()))
                .map(|desc| desc.as_ref().clone())
                .collect(),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn of<T>(context: T::Context) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let descriptors = T::field_descriptors(&context)
            .into_iter()
            .inspect(|desc| validate_layout::<T::Fields>(desc.as_ref().layout()))
            .map(|desc| desc.as_ref().clone())
            .collect();
        Self {
            descriptors,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_ref()
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
