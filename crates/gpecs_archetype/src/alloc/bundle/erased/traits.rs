use gpecs_sparse::soa::field::FieldDescriptor;

use crate::{
    bundle::erased::traits::{ErasedArchetypeIterator, ErasedArchetypeKind},
    erased::{ErasedArchetype, IntoIter},
};

impl<Meta> ErasedArchetypeKind for ErasedArchetype<Meta>
where
    Meta: AsRef<FieldDescriptor> + 'static,
{
    type Meta = Meta;
}

unsafe impl<Meta> ErasedArchetypeIterator for IntoIter<Meta> where
    Meta: AsRef<FieldDescriptor> + 'static
{
}
