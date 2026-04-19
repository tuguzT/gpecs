use gpecs_soa_erased::{
    ptr::slice::SliceItemPtrs, soa::field::FieldDescriptor, storage::AlignedStorage,
};

use crate::{
    bundle::erased::{ErasedBundle, traits::ErasedBundleDrop},
    erased::ErasedArchetype,
    storage::ErasedArchetypeSoa,
};

impl<Meta, D, S, P> ErasedArchetypeSoa for ErasedBundle<Meta, D, S, P>
where
    Meta: AsRef<FieldDescriptor> + 'static,
    D: ErasedBundleDrop<Meta>,
    S: AlignedStorage<Item: 'static>,
    P: SliceItemPtrs<Item = S::Item>,
{
    type Meta = Meta;
    type Archetype<'a> = &'a ErasedArchetype<Meta>;
    type Ptrs = P;
}
