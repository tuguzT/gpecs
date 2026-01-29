use core::{
    fmt::{self, Debug},
    iter,
    marker::PhantomData,
};

use crate::soa::{
    field::{
        CopiedFieldDescriptors, FieldDescriptor, FieldDescriptors, FieldDescriptorsIter,
        FieldDescriptorsOutput, IntoCopiedFieldDescriptors,
    },
    traits::RawSoa,
};

pub struct DenseFieldDescriptors<'ctx, K, V>
where
    V: RawSoa + ?Sized,
    V::Context: FieldDescriptors<'ctx>,
{
    key: FieldDescriptor,
    values: FieldDescriptorsOutput<'ctx, V::Context>,
    phantom: PhantomData<fn() -> K>,
}

impl<'ctx, K, V> DenseFieldDescriptors<'ctx, K, V>
where
    V: RawSoa + ?Sized,
    V::Context: FieldDescriptors<'ctx>,
{
    #[inline]
    pub fn new(context: &'ctx V::Context) -> Self {
        Self {
            key: FieldDescriptor::of::<K>(),
            values: context.field_descriptors(),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, FieldDescriptorsOutput<'ctx, V::Context>) {
        let Self { key, values, .. } = self;
        (key, values)
    }
}

impl<'ctx, K, V> Debug for DenseFieldDescriptors<'ctx, K, V>
where
    V: RawSoa + ?Sized,
    V::Context: FieldDescriptors<'ctx, Output: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, values, .. } = self;
        f.debug_struct("DenseFieldDescriptors")
            .field("key", key)
            .field("values", values)
            .finish()
    }
}

impl<'ctx, K, V> Clone for DenseFieldDescriptors<'ctx, K, V>
where
    V: RawSoa + ?Sized,
    V::Context: FieldDescriptors<'ctx, Output: Clone>,
{
    fn clone(&self) -> Self {
        let Self {
            key,
            ref values,
            phantom,
        } = *self;
        Self {
            key,
            values: values.clone(),
            phantom,
        }
    }
}

impl<'ctx, K, V> Copy for DenseFieldDescriptors<'ctx, K, V>
where
    V: RawSoa + ?Sized,
    V::Context: FieldDescriptors<'ctx, Output: Copy>,
{
}

impl<'ctx, K, V> IntoIterator for DenseFieldDescriptors<'ctx, K, V>
where
    V: RawSoa + ?Sized,
    V::Context: FieldDescriptors<'ctx>,
{
    type Item = FieldDescriptor;

    type IntoIter = iter::Chain<
        iter::Once<FieldDescriptor>,
        CopiedFieldDescriptors<FieldDescriptorsIter<'ctx, V::Context>>,
    >;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { key, values, .. } = self;

        let values = values.copied_field_descriptors();
        iter::once(key).chain(values)
    }
}
