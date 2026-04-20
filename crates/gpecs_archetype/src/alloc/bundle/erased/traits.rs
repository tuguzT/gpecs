use gpecs_sparse::soa::layout::WithLayout;

use crate::{
    bundle::erased::traits::{ErasedArchetypeIterator, ErasedArchetypeKind},
    erased::{ErasedArchetype, IntoIter},
};

impl<Meta> ErasedArchetypeKind for ErasedArchetype<Meta>
where
    Meta: WithLayout + 'static,
{
    type Meta = Meta;
}

unsafe impl<Meta> ErasedArchetypeIterator for IntoIter<Meta> where Meta: WithLayout + 'static {}
