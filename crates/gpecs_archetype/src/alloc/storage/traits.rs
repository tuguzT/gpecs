use gpecs_soa_erased::{ptr::slice::SlicePtrsAtIndex, storage::AlignedStorage};

use crate::{
    bundle::erased::{
        ErasedBundle,
        traits::{ErasedArchetypeMeta, ErasedBundleDrop},
    },
    erased::ErasedArchetype,
    storage::ErasedArchetypeSoa,
};

impl<Meta, D, S, P> ErasedArchetypeSoa for ErasedBundle<Meta, D, S, P>
where
    Meta: ErasedArchetypeMeta,
    D: ErasedBundleDrop<Meta>,
    S: AlignedStorage<Item: 'static>,
    P: SlicePtrsAtIndex<Item = S::Item>,
{
    type Meta = Meta;
    type Archetype<'a> = &'a ErasedArchetype<Meta>;
    type DropKind = D;
    type Ptrs = P;
}
