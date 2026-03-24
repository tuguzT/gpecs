use core::iter;

use crate::soa::{
    field::{
        CopiedFieldDescriptors, FieldDescriptor, FieldDescriptors, IntoCopiedFieldDescriptors,
    },
    traits::RawSoa,
};

#[derive(Debug, Clone, Copy)]
pub struct DenseFieldDescriptors<T>
where
    T: ?Sized,
{
    key: FieldDescriptor,
    values: T,
}

impl<T> DenseFieldDescriptors<T> {
    #[inline]
    pub fn new<'a, K, V>(context: &'a V::Context) -> Self
    where
        V: RawSoa + ?Sized,
        V::Context: FieldDescriptors<'a, V, Output = T>,
    {
        let key = FieldDescriptor::of::<K>();
        let values = context.field_descriptors();
        Self { key, values }
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, T) {
        let Self { key, values } = self;
        (key, values)
    }
}

impl<T> IntoIterator for DenseFieldDescriptors<T>
where
    T: IntoIterator<Item: AsRef<FieldDescriptor>>,
{
    type Item = FieldDescriptor;
    type IntoIter = iter::Chain<iter::Once<FieldDescriptor>, CopiedFieldDescriptors<T::IntoIter>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { key, values } = self;

        let values = values.copied_field_descriptors();
        iter::once(key).chain(values)
    }
}
