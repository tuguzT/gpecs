use alloc::boxed::Box;

use crate::soa::{traits::Soa, FieldDescriptor};

#[derive(Debug, Clone)]
pub struct ErasedSoaContext {
    descriptors: Box<[FieldDescriptor]>,
}

impl ErasedSoaContext {
    #[inline]
    pub fn new<I>(descriptors: I) -> Self
    where
        I: IntoIterator<Item: AsRef<FieldDescriptor>>,
    {
        let descriptors = descriptors
            .into_iter()
            .map(|desc| desc.as_ref().clone())
            .collect();
        Self { descriptors }
    }

    #[inline]
    pub fn of<T>(context: &T::Context) -> Self
    where
        T: Soa,
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
