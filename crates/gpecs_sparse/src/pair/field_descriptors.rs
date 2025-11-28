use core::{
    fmt::{self, Debug},
    iter,
    marker::PhantomData,
};

use crate::soa::{
    field::{CopiedFieldDescriptors, FieldDescriptor},
    traits::{RawSoaContext, Soa},
    wrapper::FieldDescriptors,
};

pub struct KeyValueFieldDescriptors<'context, K, V>
where
    V: Soa + ?Sized,
{
    key: FieldDescriptor,
    values: FieldDescriptors<'context, V>,
    phantom: PhantomData<fn() -> K>,
}

impl<'context, K, V> KeyValueFieldDescriptors<'context, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub fn new(context: &'context V::Context) -> Self {
        Self {
            key: FieldDescriptor::of::<K>(),
            values: FieldDescriptors::new(context.field_descriptors()),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, FieldDescriptors<'context, V>) {
        let Self { key, values, .. } = self;
        (key, values)
    }
}

impl<'context, K, V> Debug for KeyValueFieldDescriptors<'context, K, V>
where
    V: Soa + ?Sized,
    FieldDescriptors<'context, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, values, .. } = self;
        f.debug_struct("KeyValueFieldLayouts")
            .field("key", key)
            .field("values", values)
            .finish()
    }
}

impl<'context, K, V> Clone for KeyValueFieldDescriptors<'context, K, V>
where
    V: Soa + ?Sized,
    FieldDescriptors<'context, V>: Clone,
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

impl<'context, K, V> Copy for KeyValueFieldDescriptors<'context, K, V>
where
    V: Soa + ?Sized,
    FieldDescriptors<'context, V>: Copy,
{
}

impl<'context, K, V> IntoIterator for KeyValueFieldDescriptors<'context, K, V>
where
    V: Soa + ?Sized,
{
    type Item = FieldDescriptor;

    type IntoIter = iter::Chain<
        iter::Once<FieldDescriptor>,
        CopiedFieldDescriptors<<FieldDescriptors<'context, V> as IntoIterator>::IntoIter>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        let Self { key, values, .. } = self;

        let values = CopiedFieldDescriptors(values.into_iter());
        iter::once(key).chain(values)
    }
}
