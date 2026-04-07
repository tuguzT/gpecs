use crate::{
    archetype::erased::{ErasedArchetype, ErasedArchetypeView},
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
