use gpecs_sparse::{
    error::FromPartsError,
    item::{DenseSlices, SparseItem},
    view::EpochSparseView,
};

use crate::{
    entity::Entity,
    soa::{
        identity::{AsIdentitySlice, Identity, IdentitySlice},
        slice::SoaSlices,
    },
};

type Inner<'a, Meta> = EpochSparseView<'a, 'a, Entity, Identity<Meta>>;

pub struct EntityRegistryView<'a, Meta> {
    inner: Inner<'a, Meta>,
}

impl<'a, Meta> EntityRegistryView<'a, Meta> {
    const CONTEXT: &'static () = &();

    #[inline]
    pub(super) fn from_inner(inner: Inner<'a, Meta>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn new(
        entities: &'a [Entity],
        metas: &'a [Meta],
        sparse: &'a [SparseItem<Entity>],
    ) -> Result<Self, FromPartsError<Entity>> {
        let context = Self::CONTEXT;
        let metas = metas.as_identity_slice();
        let slices = DenseSlices::new(context, entities, metas).into_slice_ptrs(context);

        let wrapped_context = Identity::from_inner_ref(context);
        // TODO: find out why `Meta` should be 'static when used without pointer workaround
        let dense = unsafe { SoaSlices::new(wrapped_context, slices.deref(context)) };
        let inner = Inner::new(dense, sparse)?;

        let me = Self::from_inner(inner);
        Ok(me)
    }

    #[inline]
    pub fn into_slices(self) -> (&'a [Entity], &'a [Meta]) {
        let Self { inner } = self;

        let (entities, metas) = inner.into_dense_slices().into_parts();
        let metas = metas.as_inner();
        (entities, metas)
    }
}

// FIXME: `V` does not live long enough
// fn test<V>(values: &[V]) -> SoaSlices<'_, '_, gpecs_sparse::item::DenseItem<(), Identity<V>>> {
//     let context = &();
//     let values = values.as_identity_slice();
//     let slices = DenseSlices::new(context, &[(); 0], values);
//     let context = Identity::from_inner_ref(context);
//     SoaSlices::new(context, slices)
// }
