use core::{
    fmt::{self, Debug},
    iter,
    marker::PhantomData,
};

use crate::soa::{
    field::{CopiedFieldDescriptors, FieldDescriptor},
    traits::{FieldDescriptors, RawSoa, RawSoaContext},
    wrapper,
};

pub struct DenseFieldDescriptors<'ctx, K, V>
where
    V: RawSoa + ?Sized,
{
    key: FieldDescriptor,
    values: wrapper::FieldDescriptors<'ctx, V>,
    phantom: PhantomData<fn() -> K>,
}

impl<'ctx, K, V> DenseFieldDescriptors<'ctx, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    pub fn new(context: &'ctx V::Context) -> Self {
        Self {
            key: FieldDescriptor::of::<K>(),
            values: wrapper::FieldDescriptors::new(context.field_descriptors()),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, FieldDescriptors<'ctx, V>) {
        let Self { key, values, .. } = self;
        (key, values.into_inner())
    }
}

impl<K, V> Debug for DenseFieldDescriptors<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'ctx> FieldDescriptors<'ctx, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, values, .. } = self;
        f.debug_struct("DenseFieldDescriptors")
            .field("key", key)
            .field("values", values)
            .finish()
    }
}

impl<K, V> Clone for DenseFieldDescriptors<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'ctx> FieldDescriptors<'ctx, V>: Clone,
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

impl<K, V> Copy for DenseFieldDescriptors<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'ctx> FieldDescriptors<'ctx, V>: Copy,
{
}

impl<'ctx, K, V> IntoIterator for DenseFieldDescriptors<'ctx, K, V>
where
    V: RawSoa + ?Sized,
{
    type Item = FieldDescriptor;

    type IntoIter = iter::Chain<
        iter::Once<FieldDescriptor>,
        CopiedFieldDescriptors<<FieldDescriptors<'ctx, V> as IntoIterator>::IntoIter>,
    >;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { key, values, .. } = self;

        let values = CopiedFieldDescriptors(values.into_iter());
        iter::once(key).chain(values)
    }
}
