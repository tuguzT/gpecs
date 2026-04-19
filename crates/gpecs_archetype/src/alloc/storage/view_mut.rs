use gpecs_component::registry::{
    ComponentRegistryView,
    traits::{ComponentIdFrom, FromComponentType},
};
use gpecs_entity::Entity;
use gpecs_sparse::item::SparseItem;

use crate::{
    bundle::{Bundle, BundleRefs, BundleRefsMut, BundleSlices, BundleSlicesMut},
    erased::{ErasedArchetypeView, error::IncompatibleArchetypeError},
    storage::{ArchetypeStorageViewMut, ErasedArchetypeSoa, NoEpochEntity},
};

impl<'ctx, 'a, T> ArchetypeStorageViewMut<'ctx, 'a, T>
where
    T: ErasedArchetypeSoa + ?Sized,
{
    #[inline]
    pub fn as_bundles_with_archetype<B, M>(
        &self,
        components: &ComponentRegistryView<impl Sized, M>,
    ) -> Result<BundlesWithArchetype<'_, '_, B, T>, IncompatibleArchetypeError>
    where
        B: Bundle,
        M: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        self.as_view()
            .into_bundles_with_archetype::<B, M>(components)
    }

    #[inline]
    pub fn into_bundles_with_archetype<B, M>(
        self,
        components: &ComponentRegistryView<impl Sized, M>,
    ) -> Result<BundlesWithArchetype<'ctx, 'a, B, T>, IncompatibleArchetypeError>
    where
        B: Bundle,
        M: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        self.into_view()
            .into_bundles_with_archetype::<B, M>(components)
    }

    #[inline]
    pub fn as_bundles<B, M>(
        &self,
        components: &ComponentRegistryView<impl Sized, M>,
    ) -> Result<Bundles<'_, '_, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
        M: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        self.as_view().into_bundles::<B, M>(components)
    }

    #[inline]
    pub fn into_bundles<B, M>(
        self,
        components: &ComponentRegistryView<impl Sized, M>,
    ) -> Result<Bundles<'ctx, 'a, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
        M: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        self.into_view().into_bundles::<B, M>(components)
    }

    #[inline]
    pub fn as_mut_bundles_with_archetype<B, M>(
        &mut self,
        components: &ComponentRegistryView<impl Sized, M>,
    ) -> Result<MutBundlesWithArchetype<'_, '_, B, T>, IncompatibleArchetypeError>
    where
        B: Bundle,
        M: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        self.as_mut_view()
            .into_mut_bundles_with_archetype::<B, M>(components)
    }

    #[inline]
    pub fn into_mut_bundles_with_archetype<B, M>(
        self,
        components: &ComponentRegistryView<impl Sized, M>,
    ) -> Result<MutBundlesWithArchetype<'ctx, 'a, B, T>, IncompatibleArchetypeError>
    where
        B: Bundle,
        M: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let (entities, bundles, sparse, archetype) = unsafe { self.into_parts() };
        archetype.check_compatibility_of::<B, M>(components)?;

        let bundles = bundles
            .downcast::<B, M>(components)
            .map_err(|error| error.source)
            .expect("archetype compatibility should have been already checked");
        Ok((entities, bundles, sparse, archetype))
    }

    #[inline]
    pub fn as_mut_bundles<B, M>(
        &mut self,
        components: &ComponentRegistryView<impl Sized, M>,
    ) -> Result<MutBundles<'_, '_, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
        M: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        self.as_mut_view().into_mut_bundles::<B, M>(components)
    }

    #[inline]
    pub fn into_mut_bundles<B, M>(
        self,
        components: &ComponentRegistryView<impl Sized, M>,
    ) -> Result<MutBundles<'ctx, 'a, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
        M: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let (entities, bundles, sparse, _) =
            self.into_mut_bundles_with_archetype::<B, M>(components)?;
        Ok((entities, bundles, sparse))
    }

    #[inline]
    pub fn get_bundle<B, M>(
        &self,
        components: &ComponentRegistryView<impl Sized, M>,
        entity: Entity,
    ) -> Result<Option<BundleRefs<'_, B>>, IncompatibleArchetypeError>
    where
        B: Bundle,
        M: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        self.as_view().into_get_bundle::<B, M>(components, entity)
    }

    #[inline]
    pub fn into_get_bundle<B, M>(
        self,
        components: &ComponentRegistryView<impl Sized, M>,
        entity: Entity,
    ) -> Result<Option<BundleRefs<'a, B>>, IncompatibleArchetypeError>
    where
        B: Bundle,
        M: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        self.into_view().into_get_bundle::<B, M>(components, entity)
    }

    #[inline]
    pub fn get_bundle_mut<B, M>(
        &mut self,
        components: &ComponentRegistryView<impl Sized, M>,
        entity: Entity,
    ) -> Result<Option<BundleRefsMut<'_, B>>, IncompatibleArchetypeError>
    where
        B: Bundle,
        M: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        self.as_mut_view()
            .into_get_bundle_mut::<B, M>(components, entity)
    }

    #[inline]
    pub fn into_get_bundle_mut<B, M>(
        self,
        components: &ComponentRegistryView<impl Sized, M>,
        entity: Entity,
    ) -> Result<Option<BundleRefsMut<'a, B>>, IncompatibleArchetypeError>
    where
        B: Bundle,
        M: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        self.archetype()
            .check_compatibility_of::<B, M>(components)?;

        let Some(bundle) = self.into_get_mut(entity) else {
            return Ok(None);
        };
        let bundle = bundle
            .downcast::<B, M>(components)
            .map_err(|error| error.source)
            .expect("archetype compatibility should have been already checked");
        Ok(Some(bundle))
    }
}

type BundlesWithArchetype<'ctx, 'a, B, T> = (
    &'a [Entity],
    BundleSlices<'a, B>,
    &'a [SparseItem<NoEpochEntity>],
    ErasedArchetypeView<'ctx, <T as ErasedArchetypeSoa>::Meta>,
);
type Bundles<'ctx, 'a, B> = (
    &'a [Entity],
    BundleSlices<'a, B>,
    &'a [SparseItem<NoEpochEntity>],
);

type MutBundlesWithArchetype<'ctx, 'a, B, T> = (
    &'a [Entity],
    BundleSlicesMut<'a, B>,
    &'a [SparseItem<NoEpochEntity>],
    ErasedArchetypeView<'ctx, <T as ErasedArchetypeSoa>::Meta>,
);
type MutBundles<'ctx, 'a, B> = (
    &'a [Entity],
    BundleSlicesMut<'a, B>,
    &'a [SparseItem<NoEpochEntity>],
);
