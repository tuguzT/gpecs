use gpecs_sparse::view::EpochSparseView;

use crate::{
    entity::Entity,
    soa::identity::{Identity, IdentitySlice},
};

type Inner<'a, Meta> = EpochSparseView<'a, 'a, Entity, Identity<Meta>>;

pub struct EntityRegistryView<'a, Meta> {
    inner: Inner<'a, Meta>,
}

impl<'a, Meta> EntityRegistryView<'a, Meta> {
    #[inline]
    pub(super) fn from_inner(inner: Inner<'a, Meta>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn into_slices(self) -> (&'a [Entity], &'a [Meta]) {
        let Self { inner } = self;

        let (entities, metas) = inner.into_dense_slices().into_parts();
        let metas = metas.as_inner();
        (entities, metas)
    }
}
