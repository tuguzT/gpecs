use std::fmt::{self, Debug};

use gpecs_soa_erased::ErasedSoaContext;
use gpecs_sparse::{
    TryInsertAccess, error::TryReserveError, set::EpochSparseSet, soa::field::FieldDescriptors,
};

use crate::{
    archetype::{
        erased::{
            ErasedArchetype, ErasedArchetypeView,
            error::{
                ArchetypeError, DuplicateComponentError, IncompatibleArchetypeError,
                IncompatibleArchetypeExactError,
            },
        },
        error::IncompatibleBundleValueError,
        storage::{
            ArchetypeStorageView, ArchetypeStorageViewMut, ErasedDropMeta, NoEpochEntity,
            traits::ErasedArchetypeSoa,
        },
    },
    bundle::{
        Bundle, BundleRefs, BundleRefsMut, BundleSlices, BundleSlicesMut,
        erased::{
            ErasedBorrowedBundle, ErasedBundle, ErasedBundleKind, ShuffledBundle,
            traits::ErasedArchetypeKind,
        },
    },
    component::{
        erased::WithErasedDrop,
        registry::{
            ComponentId, ComponentRegistry, ComponentRegistryView,
            traits::{
                ComponentIdFrom, ComponentIdFromOrInsertWith, FromComponentType, PushBackArray,
            },
        },
    },
    entity::Entity,
    soa::{
        self,
        field::FieldDescriptor,
        traits::{
            RawSoaContext, ReadSoaContext, Refs as ErasedBundleRefs,
            RefsMut as ErasedBundleRefsMut, Slices as ErasedBundles, SlicesMut as ErasedBundlesMut,
            WriteSoaContext,
        },
    },
};

pub struct ArchetypeStorage<T = ErasedBundle<ErasedDropMeta>>
where
    T: ErasedArchetypeSoa + ?Sized,
{
    sparse_set: EpochSparseSet<NoEpochEntity, T>,
}

impl<T> ArchetypeStorage<T>
where
    T: ErasedArchetypeSoa + ?Sized,
{
    #[inline]
    pub fn from_context(context: T::Context) -> Self {
        let sparse_set = EpochSparseSet::with_context(context);
        Self { sparse_set }
    }

    #[inline]
    pub fn into_context(self) -> T::Context {
        let Self { sparse_set } = self;

        let (dense, _) = sparse_set.into_parts();
        dense.into_context().into_inner()
    }

    #[inline]
    pub fn as_view(&self) -> ArchetypeStorageView<'_, '_, T> {
        let Self { sparse_set } = self;

        let inner = sparse_set.as_view_ptr();
        unsafe { ArchetypeStorageView::from_inner(inner) }
    }

    #[inline]
    pub fn as_mut_view(&mut self) -> ArchetypeStorageViewMut<'_, '_, T> {
        let Self { sparse_set } = self;

        let inner = sparse_set.as_mut_view_ptr();
        unsafe { ArchetypeStorageViewMut::from_inner(inner) }
    }

    #[inline]
    pub fn archetype(&self) -> ErasedArchetypeView<'_, T::Meta> {
        let Self { sparse_set } = self;
        (**sparse_set.context()).field_descriptors()
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { sparse_set } = self;
        sparse_set.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        let Self { sparse_set } = self;
        sparse_set.is_empty()
    }

    #[inline]
    pub fn sparse_len(&self) -> usize {
        let Self { sparse_set } = self;
        sparse_set.sparse_len()
    }

    #[inline]
    pub fn sparse_is_empty(&self) -> bool {
        let Self { sparse_set } = self;
        sparse_set.sparse_is_empty()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { sparse_set } = self;
        sparse_set.capacity()
    }

    #[inline]
    pub fn sparse_capacity(&self) -> usize {
        let Self { sparse_set } = self;
        sparse_set.sparse_capacity()
    }

    #[inline]
    pub fn reserve(&mut self, additional_dense: usize, additional_sparse: usize) {
        let Self { sparse_set } = self;
        sparse_set.reserve(additional_dense, additional_sparse);
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional_dense: usize, additional_sparse: usize) {
        let Self { sparse_set } = self;
        sparse_set.reserve_exact(additional_dense, additional_sparse);
    }

    #[inline]
    pub fn try_reserve(
        &mut self,
        additional_dense: usize,
        additional_sparse: usize,
    ) -> Result<(), TryReserveError> {
        let Self { sparse_set } = self;
        sparse_set.try_reserve(additional_dense, additional_sparse)
    }

    #[inline]
    pub fn try_reserve_exact(
        &mut self,
        additional_dense: usize,
        additional_sparse: usize,
    ) -> Result<(), TryReserveError> {
        let Self { sparse_set } = self;
        sparse_set.try_reserve_exact(additional_dense, additional_sparse)
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        let Self { sparse_set } = self;
        sparse_set.shrink_to_fit();
    }

    #[inline]
    pub fn dense_shrink_to_fit(&mut self) {
        let Self { sparse_set } = self;
        sparse_set.dense_shrink_to_fit();
    }

    #[inline]
    pub fn sparse_shrink_to_fit(&mut self) {
        let Self { sparse_set } = self;
        sparse_set.sparse_shrink_to_fit();
    }

    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        let Self { sparse_set } = self;
        sparse_set.shrink_to(min_capacity);
    }

    #[inline]
    pub fn dense_shrink_to(&mut self, min_capacity: usize) {
        let Self { sparse_set } = self;
        sparse_set.dense_shrink_to(min_capacity);
    }

    #[inline]
    pub fn sparse_shrink_to(&mut self, min_capacity: usize) {
        let Self { sparse_set } = self;
        sparse_set.sparse_shrink_to(min_capacity);
    }

    #[inline]
    pub fn as_slices_with_archetype(&self) -> SlicesWithArchetype<'_, T> {
        let (entities, bundles, _, archetype) = self.as_view().into_parts();
        (entities, bundles, archetype)
    }

    #[inline]
    pub fn as_slices(&self) -> (&[Entity], ErasedBundles<'_, '_, T>) {
        let (entities, bundles, _) = self.as_slices_with_archetype();
        (entities, bundles)
    }

    #[inline]
    pub fn as_entities(&self) -> &[Entity] {
        let (entities, _) = self.as_slices();
        entities
    }

    #[inline]
    pub fn as_erased_bundles(&self) -> ErasedBundles<'_, '_, T> {
        let (_, bundles) = self.as_slices();
        bundles
    }

    #[inline]
    pub fn as_mut_slices_with_archetype(&mut self) -> SlicesMutWithArchetype<'_, T> {
        let (entities, bundles, _, archetype) = unsafe { self.as_mut_view().into_parts() };
        (entities, bundles, archetype)
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> (&[Entity], ErasedBundlesMut<'_, '_, T>) {
        let (entities, bundles, _) = self.as_mut_slices_with_archetype();
        (entities, bundles)
    }

    #[inline]
    pub fn as_mut_erased_bundles(&mut self) -> ErasedBundlesMut<'_, '_, T> {
        let (_, bundles) = self.as_mut_slices();
        bundles
    }

    #[inline]
    pub fn contains(&self, entity: Entity) -> bool {
        self.as_view().contains(entity)
    }

    #[inline]
    pub fn get(&self, entity: Entity) -> Option<ErasedBundleRefs<'_, '_, T>> {
        self.as_view().into_get(entity)
    }

    #[inline]
    pub fn get_mut(&mut self, entity: Entity) -> Option<ErasedBundleRefsMut<'_, '_, T>> {
        self.as_mut_view().into_get_mut(entity)
    }

    #[inline]
    pub fn as_bundles_with_archetype<B, M>(
        &self,
        components: &ComponentRegistryView<impl Sized, M>,
    ) -> Result<BundlesWithArchetype<'_, B, T>, IncompatibleArchetypeError>
    where
        B: Bundle,
        M: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let (entities, bundles, archetype) = self.as_slices_with_archetype();
        archetype.check_compatibility_of::<B, M>(components)?;

        let bundles = bundles
            .downcast::<B, M>(components)
            .map_err(|error| error.source)
            .expect("archetype compatibility should have been already checked");
        Ok((entities, bundles, archetype))
    }

    #[inline]
    pub fn as_bundles<B, M>(
        &self,
        components: &ComponentRegistryView<impl Sized, M>,
    ) -> Result<BundleSlices<'_, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
        M: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let (_, bundles, _) = self.as_bundles_with_archetype::<B, M>(components)?;
        Ok(bundles)
    }

    #[inline]
    pub fn as_mut_bundles_with_archetype<B, M>(
        &mut self,
        components: &ComponentRegistryView<impl Sized, M>,
    ) -> Result<BundlesMutWithArchetype<'_, B, T>, IncompatibleArchetypeError>
    where
        B: Bundle,
        M: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let (entities, bundles, archetype) = self.as_mut_slices_with_archetype();
        archetype.check_compatibility_of::<B, M>(components)?;

        let bundles = bundles
            .downcast::<B, M>(components)
            .map_err(|error| error.source)
            .expect("archetype compatibility should have been already checked");
        Ok((entities, bundles, archetype))
    }

    #[inline]
    pub fn as_mut_bundles<B, M>(
        &mut self,
        components: &ComponentRegistryView<impl Sized, M>,
    ) -> Result<BundleSlicesMut<'_, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
        M: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let (_, bundles, _) = self.as_mut_bundles_with_archetype::<B, M>(components)?;
        Ok(bundles)
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
        self.archetype()
            .check_compatibility_of::<B, M>(components)?;

        let Some(bundle) = self.get(entity) else {
            return Ok(None);
        };
        let bundle = bundle
            .downcast::<B, M>(components)
            .map_err(|error| error.source)
            .expect("archetype compatibility should have been already checked");
        Ok(Some(bundle))
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
        self.archetype()
            .check_compatibility_of::<B, M>(components)?;

        let Some(bundle) = self.get_mut(entity) else {
            return Ok(None);
        };
        let bundle = bundle
            .downcast::<B, M>(components)
            .map_err(|error| error.source)
            .expect("archetype compatibility should have been already checked");
        Ok(Some(bundle))
    }

    #[inline]
    pub fn destroy(&mut self, entity: Entity) -> bool {
        let Self { sparse_set } = self;

        sparse_set.swap_remove_into(entity.into(), |context, ptrs| {
            let Some(ptrs) = ptrs else { return false };
            unsafe { T::Context::ptrs_drop_in_place(context, ptrs) };
            true
        })
    }

    #[inline]
    pub fn destroy_all(&mut self) {
        let Self { sparse_set } = self;
        sparse_set.clear_sparse();
    }
}

type SlicesWithArchetype<'a, T> = (
    &'a [Entity],
    ErasedBundles<'a, 'a, T>,
    ErasedArchetypeView<'a, <T as ErasedArchetypeSoa>::Meta>,
);

type SlicesMutWithArchetype<'a, T> = (
    &'a [Entity],
    ErasedBundlesMut<'a, 'a, T>,
    ErasedArchetypeView<'a, <T as ErasedArchetypeSoa>::Meta>,
);

type BundlesWithArchetype<'a, B, T> = (
    &'a [Entity],
    BundleSlices<'a, B>,
    ErasedArchetypeView<'a, <T as ErasedArchetypeSoa>::Meta>,
);

type BundlesMutWithArchetype<'a, B, T> = (
    &'a [Entity],
    BundleSlicesMut<'a, B>,
    ErasedArchetypeView<'a, <T as ErasedArchetypeSoa>::Meta>,
);

impl ArchetypeStorage {
    #[inline]
    pub fn new<I, T>(
        components: &ComponentRegistryView<T, impl ?Sized>,
        component_ids: I,
    ) -> Result<Self, ArchetypeError>
    where
        I: IntoIterator<Item = ComponentId>,
        T: AsRef<FieldDescriptor> + WithErasedDrop,
    {
        let archetype = ErasedArchetype::new(components, component_ids)?;
        let me = Self::from_archetype(archetype);
        Ok(me)
    }

    #[inline]
    pub fn of<B, M, T>(components: &ComponentRegistryView<M, T>) -> Result<Self, ArchetypeError>
    where
        B: Bundle,
        M: AsRef<FieldDescriptor> + WithErasedDrop,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let archetype = ErasedArchetype::of::<B, M, T>(components)?;
        let me = Self::from_archetype(archetype);
        Ok(me)
    }

    #[inline]
    pub fn register<B, M, T>(
        components: &mut ComponentRegistry<M, T>,
    ) -> Result<Self, DuplicateComponentError>
    where
        B: Bundle,
        M: PushBackArray<Item: AsRef<FieldDescriptor> + WithErasedDrop + FromComponentType>,
        T: ComponentIdFromOrInsertWith<Key: FromComponentType> + ?Sized,
    {
        let archetype = ErasedArchetype::register::<B, M, T>(components)?;
        let me = Self::from_archetype(archetype);
        Ok(me)
    }

    #[inline]
    pub fn from_archetype(archetype: ErasedArchetype<ErasedDropMeta>) -> Self {
        let context = ErasedSoaContext::new(archetype)
            .expect("alignment of byte should be suffisient for any type");
        Self::from_context(context)
    }

    #[inline]
    pub fn into_archetype(self) -> ErasedArchetype<ErasedDropMeta> {
        self.into_context().into_inner()
    }

    #[inline]
    pub fn insert_bundle<B, T>(
        &mut self,
        components: &ComponentRegistryView<impl Sized, T>,
        entity: Entity,
        value: B,
    ) -> Result<Option<B>, IncompatibleBundleValueError<B>>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        if let Err(source) = self
            .archetype()
            .check_exact_compatibility_of::<B, T>(components)
        {
            let error = IncompatibleBundleValueError { value, source };
            return Err(error);
        }

        let Self { sparse_set } = self;
        let bundle = sparse_set.insert_from(entity.into(), |_, access| match access? {
            TryInsertAccess::ReadWrite(dst) => {
                let dst = dst
                    .into_ptrs()
                    .downcast::<B, T>(components)
                    .expect("exact archetype compatibility should be already checked");
                let value = unsafe { soa::ptr::replace::<B, _, _>(B::CONTEXT, dst, value) };
                Some(value)
            }
            TryInsertAccess::WriteOnly(dst) => {
                let dst = dst
                    .into_inner()
                    .downcast::<B, T>(components)
                    .expect("exact archetype compatibility should be already checked");
                unsafe { B::CONTEXT.write(dst, value) };
                None
            }
        });
        Ok(bundle)
    }

    #[inline]
    pub fn remove_bundle<B, T>(
        &mut self,
        components: &ComponentRegistryView<impl Sized, T>,
        entity: Entity,
    ) -> Result<Option<B>, IncompatibleArchetypeExactError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        self.archetype()
            .check_exact_compatibility_of::<B, T>(components)?;

        let Self { sparse_set } = self;
        let bundle = sparse_set.swap_remove_into(entity.into(), |_, src| {
            let src = src?
                .cast_const()
                .downcast::<B, T>(components)
                .expect("exact archetype compatibility should be already checked");
            let bundle = unsafe { B::CONTEXT.read(src) };
            Some(bundle)
        });
        Ok(bundle)
    }

    #[inline]
    pub fn insert<T>(
        &mut self,
        entity: Entity,
        bundle: ErasedBundleKind<T>,
    ) -> Result<Option<ErasedBorrowedBundle<'_, ErasedDropMeta>>, IncompatibleArchetypeExactError>
    where
        T: ErasedArchetypeKind<Meta = ErasedDropMeta>,
    {
        let archetype = self.archetype();
        archetype.check_exact_compatibility(bundle.archetype().as_view())?;

        let entity = entity.into();
        let bundle = bundle
            .shuffle(ErasedArchetype::from(archetype))
            .expect("exact archetype compatibility should have been already checked");

        let Self { sparse_set } = self;
        let bundle = match bundle {
            ShuffledBundle::Original(bundle) => sparse_set.insert(entity, bundle),
            ShuffledBundle::Other(bundle) => sparse_set.insert(entity, bundle),
        };
        Ok(bundle)
    }

    #[inline]
    pub fn remove(&mut self, entity: Entity) -> Option<ErasedBorrowedBundle<'_, ErasedDropMeta>> {
        let Self { sparse_set } = self;
        sparse_set.swap_remove(entity.into())
    }
}

impl<T> Debug for ArchetypeStorage<T>
where
    T: ErasedArchetypeSoa + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let component_ids = &self.archetype().into_component_ids();
        f.debug_struct("ArchetypeStorage")
            .field("component_ids", component_ids)
            .finish_non_exhaustive()
    }
}
