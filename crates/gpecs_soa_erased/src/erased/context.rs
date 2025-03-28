use alloc::boxed::Box;
use core::{
    fmt::{self, Debug},
    marker::PhantomData,
};

use crate::{
    assert::validate_layout,
    error::InvalidLayoutError,
    soa::traits::{FieldDescriptor, Soa},
};

pub struct ErasedSoaContext<Fields> {
    descriptors: Box<[FieldDescriptor]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaContext<Fields> {
    #[inline]
    pub fn new<I>(descriptors: I) -> Result<Self, InvalidLayoutError>
    where
        I: IntoIterator<Item: AsRef<FieldDescriptor>>,
    {
        let descriptors = descriptors
            .into_iter()
            .map(|desc| {
                validate_layout::<Fields>(desc.as_ref().layout())?;
                Ok(desc)
            })
            .collect::<Result<Box<[_]>, _>>()?;
        let me = unsafe { Self::actual_new(descriptors) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn new_unchecked<I>(descriptors: I) -> Self
    where
        I: IntoIterator<Item: AsRef<FieldDescriptor>>,
    {
        if cfg!(debug_assertions) {
            return Self::new(descriptors).expect("incorrect inputs");
        }
        unsafe { Self::actual_new(descriptors) }
    }

    #[inline]
    unsafe fn actual_new<I>(descriptors: I) -> Self
    where
        I: IntoIterator<Item: AsRef<FieldDescriptor>>,
    {
        let descriptors = descriptors
            .into_iter()
            .map(|desc| desc.as_ref().clone())
            .collect();
        Self {
            descriptors,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn of<T>(context: &T::Context) -> Result<Self, InvalidLayoutError>
    where
        T: Soa<Fields = Fields>,
    {
        let descriptors = T::field_descriptors(context);
        Self::new(descriptors)
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
