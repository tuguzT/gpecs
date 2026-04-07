use crate::{
    archetype::erased::{ErasedArchetype, ErasedArchetypeView, IntoIter, Iter},
    component::registry::ComponentId,
    soa::{
        field::{FieldDescriptor, FieldDescriptors},
        identity::Identity,
    },
};

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

impl<Meta> ErasedArchetypeKind for ErasedArchetype<Meta>
where
    Meta: AsRef<FieldDescriptor> + 'static,
{
    type Meta = Meta;
}

impl<Meta> ErasedArchetypeKind for ErasedArchetypeView<'_, Meta>
where
    Meta: AsRef<FieldDescriptor> + 'static,
{
    type Meta = Meta;
}

pub unsafe trait ErasedArchetypeIterator:
    Iterator<Item: AsRef<FieldDescriptor>>
    + for<'a> FieldDescriptors<'a, Output: IntoIterator<Item: Into<ComponentId>>>
{
}

unsafe impl<Meta> ErasedArchetypeIterator for Iter<'_, Meta> where
    Meta: AsRef<FieldDescriptor> + 'static
{
}

unsafe impl<Meta> ErasedArchetypeIterator for IntoIter<Meta> where
    Meta: AsRef<FieldDescriptor> + 'static
{
}

pub trait IntoErasedArchetypeIterator: IntoIterator<IntoIter: ErasedArchetypeIterator> {}

impl<T> IntoErasedArchetypeIterator for T where
    T: IntoIterator<IntoIter: ErasedArchetypeIterator> + ?Sized
{
}
