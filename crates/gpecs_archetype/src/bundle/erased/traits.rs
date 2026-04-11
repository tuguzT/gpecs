use gpecs_component::registry::traits::WithComponentId;
use gpecs_soa_erased::soa::{
    field::{FieldDescriptor, FieldDescriptors},
    identity::Identity,
};

use crate::erased::{ErasedArchetypeView, Iter};

pub trait ErasedArchetypeKind:
    for<'a> FieldDescriptors<'a, Output = ErasedArchetypeView<'a, Self::Meta>>
{
    type Meta: AsRef<FieldDescriptor> + 'static;
}

impl<T> ErasedArchetypeKind for &T
where
    T: ErasedArchetypeKind + ?Sized,
{
    type Meta = T::Meta;
}

impl<T> ErasedArchetypeKind for Identity<T>
where
    T: ErasedArchetypeKind + ?Sized,
{
    type Meta = T::Meta;
}

impl<Meta> ErasedArchetypeKind for ErasedArchetypeView<'_, Meta>
where
    Meta: AsRef<FieldDescriptor> + 'static,
{
    type Meta = Meta;
}

pub unsafe trait ErasedArchetypeIterator:
    Iterator<Item: AsRef<FieldDescriptor>>
    + for<'a> FieldDescriptors<'a, Output: IntoIterator<Item: WithComponentId>>
{
}

unsafe impl<Meta> ErasedArchetypeIterator for Iter<'_, Meta> where
    Meta: AsRef<FieldDescriptor> + 'static
{
}

pub trait IntoErasedArchetypeIterator: IntoIterator<IntoIter: ErasedArchetypeIterator> {}

impl<T> IntoErasedArchetypeIterator for T where
    T: IntoIterator<IntoIter: ErasedArchetypeIterator> + ?Sized
{
}
