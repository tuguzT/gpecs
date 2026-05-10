use gpecs_sparse::soa::layout::WithLayout;

use crate::{
    bundle::erased::traits::{ErasedArchetypeIterator, ErasedArchetypeKind, ErasedArchetypeMeta},
    erased::{ErasedArchetype, IntoIter},
};

impl<Meta> ErasedArchetypeKind for ErasedArchetype<Meta>
where
    Meta: ErasedArchetypeMeta,
{
    type Meta = Meta;
}

unsafe impl<Meta> ErasedArchetypeIterator for IntoIter<Meta> where Meta: WithLayout + 'static {}
