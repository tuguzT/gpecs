use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
};

use gpecs_component::{
    erased::{ErasedDrop, WithErasedDrop, error::NotRegisteredError},
    registry::{
        ComponentId, ComponentInfo, ComponentRegistry, ComponentRegistryView,
        traits::{ComponentIdFrom, ComponentIdFromOrInsertWith, FromComponentType, PushBackArray},
    },
};
use gpecs_soa_erased::CovariantFieldLayouts;
use gpecs_sparse::{
    item::SparseItem,
    set::EpochSparseSet,
    soa::{
        field::{FieldLayouts, FieldLayoutsOutput},
        identity::Identity,
        layout::WithLayout,
    },
};

use crate::{
    bundle::Bundle,
    erased::{
        ComponentIdOrderedIter, ComponentIds, ErasedArchetypeView, IntoIter, Iter,
        error::{
            AlreadyHasComponentError, ArchetypeError, DuplicateComponentError,
            IncompatibleArchetypeError, IncompatibleArchetypeExactError,
            IncompatibleArchetypeViewExactError, MissingComponentError,
        },
    },
};

type Inner<Meta> = EpochSparseSet<u32, Identity<Meta>>;

#[derive(Clone)]
pub struct ErasedArchetype<Meta = ()> {
    components: Inner<Meta>,
}

impl<Meta> ErasedArchetype<Meta> {
    #[inline]
    pub fn from_iter<I>(
        components: &ComponentRegistryView<impl Sized, impl ?Sized>,
        iter: I,
    ) -> Result<Self, ArchetypeError>
    where
        I: IntoIterator<Item = (ComponentId, Meta)>,
    {
        let components = try_collect_opt_components(
            iter.into_iter().map(|(id, meta)| {
                let _ = components.get_component_descriptor(id)?;
                Some((id, meta))
            }),
            |map, (id, meta)| Inner::insert(map, id.into_u32(), meta.into()).is_none(),
            |&(id, _)| id,
        )?;

        let me = Self { components };
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_iter_unchecked<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (ComponentId, Meta)>,
    {
        let components = iter
            .into_iter()
            .map(|(id, meta)| (id.into_u32(), meta.into()))
            .collect();
        Self { components }
    }
}

pub trait FromComponentDescriptor<'a, D>: Sized
where
    D: ?Sized,
{
    fn from_component_descriptor(desc: &'a D) -> Self;
}

impl<'a, D> FromComponentDescriptor<'a, D> for &'a D
where
    D: ?Sized,
{
    #[inline]
    fn from_component_descriptor(desc: &'a D) -> Self {
        desc
    }
}

impl<D> FromComponentDescriptor<'_, D> for ()
where
    D: ?Sized,
{
    #[inline]
    fn from_component_descriptor(_: &D) -> Self {}
}

impl<D> FromComponentDescriptor<'_, D> for Option<ErasedDrop>
where
    D: WithErasedDrop + ?Sized,
{
    #[inline]
    fn from_component_descriptor(desc: &D) -> Self {
        desc.erased_drop()
    }
}

impl<Meta> ErasedArchetype<Meta> {
    #[inline]
    pub fn new<'a, I, T>(
        components: &'a ComponentRegistryView<T, impl ?Sized>,
        component_ids: I,
    ) -> Result<Self, ArchetypeError>
    where
        I: IntoIterator<Item = ComponentId>,
        Meta: FromComponentDescriptor<'a, T>,
    {
        let components = try_collect_opt_components(
            component_ids.into_iter().map(|id| {
                let desc = components.get_component_descriptor(id)?;
                let meta = Meta::from_component_descriptor(desc);
                Some((id, meta))
            }),
            |map, (id, meta)| Inner::insert(map, id.into_u32(), meta.into()).is_none(),
            |&(id, _)| id,
        )?;

        let me = Self { components };
        Ok(me)
    }

    #[inline]
    pub fn of<'a, B, M, T>(
        components: &'a ComponentRegistryView<M, T>,
    ) -> Result<Self, ArchetypeError>
    where
        B: Bundle,
        Meta: FromComponentDescriptor<'a, M>,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let components = try_collect_opt_components(
            B::get_components(components).into_iter().map(|id| {
                let id = id?;
                let desc = components.get_component_descriptor(id)?;
                let meta = Meta::from_component_descriptor(desc);
                Some((id, meta))
            }),
            |map, (id, meta)| Inner::insert(map, id.into_u32(), meta.into()).is_none(),
            |&(id, _)| id,
        )?;

        let me = Self { components };
        Ok(me)
    }

    #[inline]
    pub fn register<'a, B, M, T>(
        components: &'a mut ComponentRegistry<M, T>,
    ) -> Result<Self, DuplicateComponentError>
    where
        B: Bundle,
        Meta: FromComponentDescriptor<'a, M::Item>,
        M: PushBackArray<Item: FromComponentType>,
        T: ComponentIdFromOrInsertWith<Key: FromComponentType> + ?Sized,
    {
        let components = try_collect_components(
            B::register_components(components).into_iter().map(|id| {
                let Some(desc) = components.get_component_descriptor(id) else {
                    unreachable!("descriptor of {id} should be present")
                };
                let meta = Meta::from_component_descriptor(desc);
                (id, meta)
            }),
            |map, (id, meta)| Inner::insert(map, id.into_u32(), meta.into()).is_none(),
            |&(id, _)| id,
        )?;

        let me = Self { components };
        Ok(me)
    }
}

impl<T, U> ErasedArchetype<(T, U)> {
    #[inline]
    pub fn new_with<'a, I, W>(
        components: &'a ComponentRegistryView<W, impl ?Sized>,
        with: I,
    ) -> Result<Self, ArchetypeError>
    where
        I: IntoIterator<Item = (ComponentId, U)>,
        T: FromComponentDescriptor<'a, W>,
    {
        let components = try_collect_opt_components(
            with.into_iter().map(|(id, u)| {
                let desc = components.get_component_descriptor(id)?;
                let t = T::from_component_descriptor(desc);
                Some((id, t, u))
            }),
            |map, (id, t, u)| Inner::insert(map, id.into_u32(), (t, u).into()).is_none(),
            |&(id, _, _)| id,
        )?;

        let me = Self { components };
        Ok(me)
    }
}

impl<Meta> ErasedArchetype<Meta> {
    #[inline]
    pub fn len(&self) -> usize {
        let Self { components } = self;
        components.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn as_view(&self) -> ErasedArchetypeView<'_, Meta> {
        let Self { components } = self;

        let inner = components.as_view_ptr();
        ErasedArchetypeView::from_inner(inner)
    }

    #[inline]
    pub fn as_slices(&self) -> (&[ComponentId], &[Meta], &[SparseItem<u32>]) {
        let (component_ids, metas, sparse) = self.as_view().into_parts();
        (component_ids, metas, sparse)
    }

    #[inline]
    pub fn as_component_ids(&self) -> &[ComponentId] {
        let (component_ids, _, _) = self.as_slices();
        component_ids
    }

    #[inline]
    pub fn as_metas(&self) -> &[Meta] {
        let (_, metas, _) = self.as_slices();
        metas
    }

    #[inline]
    pub fn as_sparse(&self) -> &[SparseItem<u32>] {
        let (_, _, sparse) = self.as_slices();
        sparse
    }

    #[inline]
    pub fn as_ptrs(&self) -> (*const ComponentId, *const Meta, *const SparseItem<u32>) {
        self.as_view().as_ptrs()
    }

    #[inline]
    pub fn contains(&self, component_id: ComponentId) -> bool {
        self.as_view().contains(component_id)
    }

    #[inline]
    pub fn has_components(
        &self,
        of: ErasedArchetypeView<impl Sized>,
    ) -> Result<(), MissingComponentError> {
        self.as_view().has_components(of)
    }

    #[inline]
    pub fn has_no_components(
        &self,
        of: ErasedArchetypeView<impl Sized>,
    ) -> Result<(), AlreadyHasComponentError> {
        self.as_view().has_no_components(of)
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<&Meta> {
        self.as_view().into_get(component_id)
    }

    #[inline]
    pub fn get_index_of(&self, component_id: ComponentId) -> Option<usize> {
        self.as_view().get_index_of(component_id)
    }

    #[inline]
    pub fn get_by_index(&self, index: usize) -> Option<(ComponentId, &Meta)> {
        self.as_view().into_get_by_index(index)
    }

    #[inline]
    pub fn check_compatibility(
        &self,
        other: ErasedArchetypeView<impl Sized>,
    ) -> Result<(), MissingComponentError> {
        self.as_view().check_compatibility(other)
    }

    #[inline]
    pub fn check_compatibility_for<I>(
        &self,
        components: &ComponentRegistryView<impl Sized, impl ?Sized>,
        component_ids: I,
    ) -> Result<(), IncompatibleArchetypeError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        self.as_view()
            .check_compatibility_for(components, component_ids)
    }

    #[inline]
    pub fn check_compatibility_of<B, T>(
        &self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<(), IncompatibleArchetypeError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        self.as_view().check_compatibility_of::<B, T>(components)
    }

    #[inline]
    pub fn check_exact_compatibility(
        &self,
        other: ErasedArchetypeView<impl Sized>,
    ) -> Result<(), IncompatibleArchetypeViewExactError> {
        self.as_view().check_exact_compatibility(other)
    }

    #[inline]
    pub fn check_exact_compatibility_for<I>(
        &self,
        components: &ComponentRegistryView<impl Sized, impl ?Sized>,
        component_ids: I,
    ) -> Result<(), IncompatibleArchetypeExactError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        self.as_view()
            .check_exact_compatibility_for(components, component_ids)
    }

    #[inline]
    pub fn check_exact_compatibility_of<B, T>(
        &self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<(), IncompatibleArchetypeExactError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        self.as_view()
            .check_exact_compatibility_of::<B, T>(components)
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, Meta> {
        self.as_view().into_iter()
    }

    #[inline]
    pub fn component_ids(&self) -> ComponentIds<'_> {
        self.as_view().into_component_ids()
    }

    #[inline]
    pub fn component_id_ordered_iter(&self) -> ComponentIdOrderedIter<'_, Meta> {
        self.as_view().into_component_id_ordered_iter()
    }
}

impl<Meta> Debug for ErasedArchetype<Meta>
where
    Meta: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let components = &self.iter();
        f.debug_struct("ErasedArchetype")
            .field("components", components)
            .finish()
    }
}

impl<Meta> PartialEq for ErasedArchetype<Meta>
where
    Meta: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_view() == other.as_view()
    }
}

impl<Meta> Eq for ErasedArchetype<Meta> where Meta: Eq {}

impl<Meta> PartialOrd for ErasedArchetype<Meta>
where
    Meta: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let other = other.as_view();
        self.as_view().partial_cmp(&other)
    }
}

impl<Meta> Ord for ErasedArchetype<Meta>
where
    Meta: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let other = other.as_view();
        self.as_view().cmp(&other)
    }
}

impl<Meta> Hash for ErasedArchetype<Meta>
where
    Meta: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_view().hash(state);
    }
}

impl<Meta> AsRef<Self> for ErasedArchetype<Meta> {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<Meta> AsMut<Self> for ErasedArchetype<Meta> {
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<Meta> From<ErasedArchetypeView<'_, Meta>> for ErasedArchetype<Meta>
where
    Meta: Clone,
{
    #[inline]
    fn from(archetype: ErasedArchetypeView<'_, Meta>) -> Self {
        let (dense, _) = unsafe { archetype.into_inner().as_ref_unchecked() }.into_parts();
        let dense = dense.to_vec();
        let sparse = archetype.as_sparse().to_vec();

        let components = unsafe { Inner::from_parts_unchecked(dense, sparse) };
        Self { components }
    }
}

impl<'a, Meta> IntoIterator for &'a ErasedArchetype<Meta> {
    type Item = ComponentInfo<&'a Meta>;
    type IntoIter = Iter<'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<Meta> IntoIterator for ErasedArchetype<Meta> {
    type Item = ComponentInfo<Meta>;
    type IntoIter = IntoIter<Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { components } = self;

        let inner = components.into_iter();
        IntoIter::from_inner(inner)
    }
}

impl<'a, Meta> FieldLayouts<'a> for ErasedArchetype<Meta>
where
    Meta: WithLayout + 'a,
{
    type Output = ErasedArchetypeView<'a, Meta>;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        self.as_view()
    }
}

impl<Meta> CovariantFieldLayouts for ErasedArchetype<Meta>
where
    Meta: WithLayout + 'static,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        from
    }
}

#[inline]
fn try_collect_components<S, I>(
    components: I,
    mut insert_fn: impl FnMut(&mut S, I::Item) -> bool,
    mut component_id_fn: impl FnMut(&I::Item) -> ComponentId,
) -> Result<S, DuplicateComponentError>
where
    S: Default,
    I: IntoIterator,
{
    let mut set = S::default();
    components.into_iter().try_for_each(|item| {
        let component_id = component_id_fn(&item);
        let is_unique = insert_fn(&mut set, item);
        is_unique
            .then(Default::default)
            .ok_or_else(|| DuplicateComponentError::new(component_id))
    })?;
    Ok(set)
}

#[inline]
fn try_collect_opt_components<S, I, T>(
    components: I,
    mut insert_fn: impl FnMut(&mut S, T) -> bool,
    mut component_id_fn: impl FnMut(&T) -> ComponentId,
) -> Result<S, ArchetypeError>
where
    S: Default,
    I: IntoIterator<Item = Option<T>>,
{
    let mut set = S::default();
    components
        .into_iter()
        .try_for_each::<_, Result<_, ArchetypeError>>(|item| {
            let Some(item) = item else {
                return Err(NotRegisteredError::new().into());
            };
            let component_id = component_id_fn(&item);
            let is_unique = insert_fn(&mut set, item);
            is_unique
                .then(Default::default)
                .ok_or_else(|| DuplicateComponentError::new(component_id).into())
        })?;
    Ok(set)
}
