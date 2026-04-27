#![expect(clippy::module_inception)]

use core::fmt::{self, Debug};

use gpecs_component::registry::{
    ComponentId, ComponentRegistry, ComponentRegistryView,
    traits::{ComponentIdFrom, ComponentIdFromOrInsertWith, FromComponentType, PushBackArray},
};
use gpecs_entity::Entity;
use gpecs_soa_erased::{
    ErasedSoaContext,
    ptr::slice::{PtrsItem, SliceItemPtrs},
    soa::{
        self,
        field::FieldLayouts,
        layout::WithLayout,
        traits::{
            RawSoaContext, ReadSoaContext, Refs as ErasedBundleRefs,
            RefsMut as ErasedBundleRefsMut, Slices as ErasedBundles, SlicesMut as ErasedBundlesMut,
            SoaRead, SoaWrite, WriteSoaContext,
        },
    },
    storage::AlignedStorage,
};
use gpecs_sparse::{TryInsertAccess, error::TryReserveError, set::EpochSparseSet};

use crate::{
    bundle::{
        Bundle, BundleRefs, BundleRefsMut, BundleSlices, BundleSlicesMut,
        erased::{
            ErasedBundle, ErasedBundleKind,
            error::DowncastError,
            traits::{ErasedArchetypeKind, ErasedBundleDrop},
        },
    },
    erased::{
        ErasedArchetype, ErasedArchetypeView, FromComponentDescriptor,
        error::{
            ArchetypeError, DuplicateComponentError, IncompatibleArchetypeError,
            IncompatibleArchetypeExactError,
        },
    },
    storage::{
        ArchetypeStorageView, ArchetypeStorageViewMut, ErasedArchetypeSoa, NoEpochEntity,
        error::{
            EntityFoundError, EntityNotFoundError, IncompatibleBundleValueError, MoveIntoError,
            MoveIntoWithInsertBundleError, MoveIntoWithInsertBundleErrorKind,
            MoveIntoWithInsertError, UpdateWithBundleError, UpdateWithError,
        },
    },
};

pub struct ArchetypeStorage<T>
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
        (**sparse_set.context()).field_layouts()
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
    #[expect(clippy::type_complexity)]
    pub fn insert<'a, W, D, S>(
        &'a mut self,
        entity: Entity,
        bundle: ErasedBundleKind<W, D, S, T::Ptrs>,
    ) -> Result<
        Option<ErasedBundleKind<T::Archetype<'a>, D, S, T::Ptrs>>,
        IncompatibleArchetypeExactError,
    >
    where
        T: SoaRead<'a, ErasedBundleKind<T::Archetype<'a>, D, S, T::Ptrs>>
            + SoaWrite<ErasedBundleKind<W, D, S, T::Ptrs>>,
        W: ErasedArchetypeKind<Meta = T::Meta>,
        D: ErasedBundleDrop<T::Meta>,
        S: AlignedStorage<Item = PtrsItem<T::Ptrs>>,
    {
        self.archetype()
            .check_exact_compatibility(bundle.archetype())?;

        let Self { sparse_set } = self;
        let bundle = sparse_set.insert(entity.into(), bundle);
        Ok(bundle)
    }

    #[inline]
    pub fn remove<'a, D, S>(
        &'a mut self,
        entity: Entity,
    ) -> Option<ErasedBundleKind<T::Archetype<'a>, D, S, T::Ptrs>>
    where
        T: SoaRead<'a, ErasedBundleKind<T::Archetype<'a>, D, S, T::Ptrs>>,
        D: ErasedBundleDrop<T::Meta>,
        S: AlignedStorage<Item = PtrsItem<T::Ptrs>>,
    {
        let Self { sparse_set } = self;
        sparse_set.swap_remove(entity.into())
    }

    #[inline]
    pub fn destroy(&mut self, entity: Entity) -> bool {
        let Self { sparse_set } = self;

        sparse_set.swap_remove_into(entity.into(), |context, ptrs| {
            let Some(ptrs) = ptrs else { return false };
            unsafe { context.ptrs_drop_in_place(ptrs) };
            true
        })
    }

    #[inline]
    pub fn destroy_all(&mut self) {
        let Self { sparse_set } = self;
        sparse_set.clear_sparse();
    }

    #[inline]
    pub fn move_into(
        &mut self,
        other: &mut ArchetypeStorage<T>,
        entity: Entity,
    ) -> Result<(), MoveIntoError> {
        self.archetype()
            .check_exact_compatibility(other.archetype())?;

        if !self.contains(entity) {
            let error = EntityNotFoundError::new(entity);
            return Err(error.into());
        }
        if other.contains(entity) {
            let error = EntityFoundError::new(entity);
            return Err(error.into());
        }

        let Self { sparse_set } = self;
        let Self { sparse_set: other } = other;

        sparse_set.swap_remove_into(entity.into(), |_, src| {
            let Some(src) = src else {
                unreachable!("this storage should contain {entity}")
            };

            other.insert_from(entity.into(), |_, dst| {
                let Some(dst) = dst else {
                    unreachable!("epoch of the {entity} should not be relevant")
                };
                let TryInsertAccess::WriteOnly(dst) = dst else {
                    unreachable!("other storage should not contain {entity}");
                };

                let src = src.cast_const();
                unsafe {
                    dst.into_inner()
                        .copy_from_compatible_exact_nonoverlapping(&src, 1);
                }
            });
        });
        Ok(())
    }

    #[inline]
    #[expect(clippy::type_complexity)]
    pub fn update_with<N, D, S>(
        &mut self,
        entity: Entity,
        value: ErasedBundleKind<N, D, S, T::Ptrs>,
    ) -> Result<(), UpdateWithError<ErasedBundleKind<N, D, S, T::Ptrs>>>
    where
        N: ErasedArchetypeKind<Meta = T::Meta>,
        D: ErasedBundleDrop<T::Meta>,
        S: AlignedStorage<Item = PtrsItem<T::Ptrs>>,
    {
        if let Err(error) = self.archetype().check_compatibility(value.archetype()) {
            let source = error.into();
            return Err(UpdateWithError { source, value });
        }
        let Some(bundle) = self.get_mut(entity) else {
            let source = EntityNotFoundError::new(entity).into();
            return Err(UpdateWithError { source, value });
        };

        unsafe {
            let mut dst = bundle.into_ptrs();
            let src = &value.as_ptrs();
            dst.move_from_compatible_nonoverlapping::<_, T::DropKind>(src, 1);
        }
        let _ = value.into_inner();
        Ok(())
    }

    #[inline]
    #[expect(clippy::type_complexity)]
    pub fn move_into_with_insert<N, D, S>(
        &mut self,
        other: &mut ArchetypeStorage<T>,
        entity: Entity,
        value: ErasedBundleKind<N, D, S, T::Ptrs>,
    ) -> Result<(), MoveIntoWithInsertError<ErasedBundleKind<N, D, S, T::Ptrs>>>
    where
        N: ErasedArchetypeKind<Meta = T::Meta>,
        D: ErasedBundleDrop<T::Meta>,
        S: AlignedStorage<Item = PtrsItem<T::Ptrs>>,
    {
        if let Err(error) = self.archetype().has_no_components(value.archetype()) {
            let source = error.into();
            return Err(MoveIntoWithInsertError { source, value });
        }
        if let Err(error) = other.archetype().has_components(value.archetype()) {
            let source = error.into();
            return Err(MoveIntoWithInsertError { source, value });
        }

        if !self.contains(entity) {
            let source = EntityNotFoundError::new(entity).into();
            return Err(MoveIntoWithInsertError { source, value });
        }
        if other.contains(entity) {
            let source = EntityFoundError::new(entity).into();
            return Err(MoveIntoWithInsertError { source, value });
        }

        let Self { sparse_set } = self;
        let Self { sparse_set: other } = other;

        sparse_set.swap_remove_into(entity.into(), |_, src| {
            let Some(src) = src else {
                unreachable!("this storage should contain {entity}")
            };

            other.insert_from(entity.into(), |_, dst| {
                let Some(dst) = dst else {
                    unreachable!("epoch of the {entity} should not be relevant")
                };
                let TryInsertAccess::WriteOnly(dst) = dst else {
                    unreachable!("other storage should not contain {entity}");
                };
                let mut dst = dst.into_inner();

                let src = &src.cast_const();
                unsafe { dst.copy_from_compatible_nonoverlapping(src, 1) }

                let src = &value.as_ptrs();
                unsafe { dst.copy_from_compatible_nonoverlapping(src, 1) }

                let _ = value.into_inner();
            });
        });
        Ok(())
    }

    #[inline]
    pub fn as_bundles_with_archetype<B>(
        &self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
    ) -> Result<BundlesWithArchetype<'_, B, T>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        let (entities, bundles, _, archetype) = self
            .as_view()
            .into_bundles_with_archetype::<B>(components)?;
        Ok((entities, bundles, archetype))
    }

    #[inline]
    pub fn as_bundles<B>(
        &self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
    ) -> Result<BundleSlices<'_, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        let (_, bundles, _) = self.as_view().into_bundles::<B>(components)?;
        Ok(bundles)
    }

    #[inline]
    pub fn as_mut_bundles_with_archetype<B>(
        &mut self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
    ) -> Result<BundlesMutWithArchetype<'_, B, T>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        let (entities, bundles, _, archetype) = self
            .as_mut_view()
            .into_mut_bundles_with_archetype::<B>(components)?;
        Ok((entities, bundles, archetype))
    }

    #[inline]
    pub fn as_mut_bundles<B>(
        &mut self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
    ) -> Result<BundleSlicesMut<'_, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        let (_, bundles, _) = self.as_mut_view().into_mut_bundles::<B>(components)?;
        Ok(bundles)
    }

    #[inline]
    pub fn get_bundle<B>(
        &self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
        entity: Entity,
    ) -> Result<Option<BundleRefs<'_, B>>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        self.as_view().into_get_bundle::<B>(components, entity)
    }

    #[inline]
    pub fn get_bundle_mut<B>(
        &mut self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
        entity: Entity,
    ) -> Result<Option<BundleRefsMut<'_, B>>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        self.as_mut_view()
            .into_get_bundle_mut::<B>(components, entity)
    }

    #[inline]
    pub fn insert_bundle<B>(
        &mut self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
        entity: Entity,
        value: B,
    ) -> Result<Option<B>, IncompatibleBundleValueError<B>>
    where
        B: Bundle,
    {
        if let Err(source) = self
            .archetype()
            .check_exact_compatibility_of::<B>(components)
        {
            let error = IncompatibleBundleValueError { source, value };
            return Err(error);
        }

        let Self { sparse_set } = self;
        let bundle = sparse_set.insert_from(entity.into(), |_, access| match access? {
            TryInsertAccess::ReadWrite(dst) => {
                let Ok(dst) = dst.into_ptrs().downcast::<B>(components) else {
                    unreachable!("exact archetype compatibility should be already checked")
                };
                let value = unsafe { soa::ptr::replace::<B, B, B>(B::CONTEXT, dst, value) };
                Some(value)
            }
            TryInsertAccess::WriteOnly(dst) => {
                let Ok(dst) = dst.into_inner().downcast::<B>(components) else {
                    unreachable!("exact archetype compatibility should be already checked")
                };
                unsafe { B::CONTEXT.write(dst, value) };
                None
            }
        });
        Ok(bundle)
    }

    #[inline]
    pub fn remove_bundle<B>(
        &mut self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
        entity: Entity,
    ) -> Result<Option<B>, IncompatibleArchetypeExactError>
    where
        B: Bundle,
    {
        self.archetype()
            .check_exact_compatibility_of::<B>(components)?;

        let Self { sparse_set } = self;
        let bundle = sparse_set.swap_remove_into(entity.into(), |_, src| {
            let Ok(src) = src?.cast_const().downcast::<B>(components) else {
                unreachable!("exact archetype compatibility should be already checked")
            };
            let bundle = unsafe { B::CONTEXT.read(src) };
            Some(bundle)
        });
        Ok(bundle)
    }

    #[inline]
    pub fn update_with_bundle<B>(
        &mut self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
        entity: Entity,
        value: B,
    ) -> Result<B, UpdateWithBundleError<B>>
    where
        B: Bundle,
    {
        let dest = match self.get_bundle_mut::<B>(components, entity) {
            Ok(Some(bundle)) => bundle,
            Ok(None) => {
                let source = EntityNotFoundError::new(entity).into();
                return Err(UpdateWithBundleError { source, value });
            }
            Err(error) => {
                let source = error.into();
                return Err(UpdateWithBundleError { source, value });
            }
        };

        let bundle = soa::mem::replace::<B, B, B>(B::CONTEXT, dest, value);
        Ok(bundle)
    }

    #[inline]
    pub fn move_into_with_insert_bundle<B>(
        &mut self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
        other: &mut ArchetypeStorage<T>,
        entity: Entity,
        value: B,
    ) -> Result<(), MoveIntoWithInsertBundleError<B>>
    where
        B: Bundle,
    {
        if self
            .archetype()
            .check_compatibility_of::<B>(components)
            .is_ok()
        {
            let source = MoveIntoWithInsertBundleErrorKind::SourceCompatible;
            return Err(MoveIntoWithInsertBundleError { source, value });
        }
        if let Err(error) = other.archetype().check_compatibility_of::<B>(components) {
            let source = error.into();
            return Err(MoveIntoWithInsertBundleError { source, value });
        }

        if !self.contains(entity) {
            let source = EntityNotFoundError::new(entity).into();
            return Err(MoveIntoWithInsertBundleError { source, value });
        }
        if other.contains(entity) {
            let source = EntityFoundError::new(entity).into();
            return Err(MoveIntoWithInsertBundleError { source, value });
        }

        let Self { sparse_set } = self;
        let Self { sparse_set: other } = other;

        sparse_set.swap_remove_into(entity.into(), |_, src| {
            let Some(src) = src else {
                unreachable!("this storage should contain {entity}")
            };

            other.insert_from(entity.into(), |_, dst| {
                let Some(dst) = dst else {
                    unreachable!("epoch of the {entity} should not be relevant")
                };
                let TryInsertAccess::WriteOnly(dst) = dst else {
                    unreachable!("other storage should not contain {entity}");
                };
                let mut dst = dst.into_inner();

                let src = &src.cast_const();
                unsafe { dst.copy_from_compatible_nonoverlapping(src, 1) }

                let dst = dst
                    .downcast::<B>(components)
                    .map_err(DowncastError::into_source)
                    .expect("archetype compatibility should have been checked earlier");
                let _ = unsafe { soa::ptr::replace::<B, B, B>(B::CONTEXT, dst, value) };
            });
        });
        Ok(())
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

impl<Meta, D, S, P> ArchetypeStorage<ErasedBundle<Meta, D, S, P>>
where
    Meta: WithLayout + 'static,
    D: ErasedBundleDrop<Meta>,
    S: AlignedStorage<Item: 'static>,
    P: SliceItemPtrs<Item = S::Item>,
{
    #[inline]
    pub fn new<'a, I, T>(
        components: &'a ComponentRegistryView<T, impl ?Sized>,
        component_ids: I,
    ) -> Result<Self, ArchetypeError>
    where
        I: IntoIterator<Item = ComponentId>,
        T: WithLayout,
        Meta: FromComponentDescriptor<'a, T>,
    {
        let archetype = ErasedArchetype::new(components, component_ids)?;
        let me = Self::from_archetype(archetype);
        Ok(me)
    }

    #[inline]
    pub fn of<'a, B, M, T>(
        components: &'a ComponentRegistryView<M, T>,
    ) -> Result<Self, ArchetypeError>
    where
        B: Bundle,
        M: WithLayout,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
        Meta: FromComponentDescriptor<'a, M>,
    {
        let archetype = ErasedArchetype::of::<B, M, T>(components)?;
        let me = Self::from_archetype(archetype);
        Ok(me)
    }

    #[inline]
    pub fn register<'a, B, M, T>(
        components: &'a mut ComponentRegistry<M, T>,
    ) -> Result<Self, DuplicateComponentError>
    where
        B: Bundle,
        M: PushBackArray<Item: WithLayout + FromComponentType>,
        T: ComponentIdFromOrInsertWith<Key: FromComponentType> + ?Sized,
        Meta: FromComponentDescriptor<'a, M::Item>,
    {
        let archetype = ErasedArchetype::register::<B, M, T>(components)?;
        let me = Self::from_archetype(archetype);
        Ok(me)
    }

    #[inline]
    pub fn from_archetype(archetype: ErasedArchetype<Meta>) -> Self {
        let context = ErasedSoaContext::new(archetype)
            .expect("alignment of byte should be suffisient for any type");
        Self::from_context(context)
    }

    #[inline]
    pub fn into_archetype(self) -> ErasedArchetype<Meta> {
        self.into_context().into_inner()
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
