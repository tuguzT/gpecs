use std::{
    borrow::Cow,
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    iter::{Enumerate, FusedIterator},
    slice,
};

use gpecs_soa_erased::CovariantFieldDescriptors;
use gpecs_sparse::{
    arena::EpochSparseArena,
    item::SparseItem,
    iter::{IntoIter as SparseIntoIter, Iter as SparseIter},
};

use crate::{
    archetype::error::{
        AlreadyHasComponentError, ArchetypeError, DuplicateComponentError,
        IncompatibleArchetypeError, IncompatibleArchetypeExactError, MissingComponentError,
        TooFewComponentsError,
    },
    bundle::Bundle,
    component::{
        erased::{ErasedDrop, WithErasedDrop, error::NotRegisteredError},
        registry::{
            ComponentId, ComponentInfo, ComponentRegistry,
            traits::{ComponentIdFrom, ComponentIdFromOrInsertWith, FromComponentType},
        },
    },
    soa::{
        field::{FieldDescriptor, FieldDescriptors, FieldDescriptorsOutput},
        identity::{Identity, IdentitySlice},
    },
};

type Inner<Meta> = EpochSparseArena<u32, Identity<Meta>>;

// TODO: split this into different modules
#[derive(Clone)]
pub struct ErasedArchetype<Meta = ()> {
    components: Inner<Meta>,
}

impl<Meta> ErasedArchetype<Meta> {
    #[inline]
    pub fn from_iter<I>(
        components: &ComponentRegistry<impl Sized, impl ?Sized>,
        iter: I,
    ) -> Result<Self, ArchetypeError>
    where
        I: IntoIterator<Item = (ComponentId, Meta)>,
    {
        let components = try_collect_opt_components(
            iter.into_iter().map(|(id, meta)| {
                let _ = components.get_component_info(id)?;
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

pub trait FromComponentInfo<'a, Meta>: Sized
where
    Meta: ?Sized,
{
    fn from_component_info(info: ComponentInfo<&'a Meta>) -> Self;
}

impl<'a, Meta> FromComponentInfo<'a, Meta> for &'a Meta
where
    Meta: ?Sized,
{
    #[inline]
    fn from_component_info(info: ComponentInfo<&'a Meta>) -> Self {
        info.into_meta()
    }
}

impl<'a, T, Meta> FromComponentInfo<'a, Meta> for ComponentInfo<T>
where
    T: FromComponentInfo<'a, Meta>,
{
    #[inline]
    fn from_component_info(info: ComponentInfo<&'a Meta>) -> Self {
        info.map_meta(|_| T::from_component_info(info))
    }
}

impl<Meta> FromComponentInfo<'_, Meta> for ()
where
    Meta: ?Sized,
{
    #[inline]
    fn from_component_info(_: ComponentInfo<&Meta>) -> Self {}
}

impl<Meta> FromComponentInfo<'_, Meta> for FieldDescriptor
where
    Meta: AsRef<FieldDescriptor> + ?Sized,
{
    #[inline]
    fn from_component_info(info: ComponentInfo<&Meta>) -> Self {
        *info.as_ref()
    }
}

impl<Meta> FromComponentInfo<'_, Meta> for Option<ErasedDrop>
where
    Meta: WithErasedDrop + ?Sized,
{
    #[inline]
    fn from_component_info(info: ComponentInfo<&Meta>) -> Self {
        info.erased_drop()
    }
}

impl<Meta> ErasedArchetype<Meta> {
    #[inline]
    pub fn new<'a, I, T>(
        components: &'a ComponentRegistry<T, impl ?Sized>,
        component_ids: I,
    ) -> Result<Self, ArchetypeError>
    where
        I: IntoIterator<Item = ComponentId>,
        Meta: FromComponentInfo<'a, T>,
    {
        let components = try_collect_opt_components(
            component_ids.into_iter().map(|id| {
                let info = components.get_component_info(id)?;
                let meta = Meta::from_component_info(info);
                Some((id, meta))
            }),
            |map, (id, meta)| Inner::insert(map, id.into_u32(), meta.into()).is_none(),
            |&(id, _)| id,
        )?;

        let me = Self { components };
        Ok(me)
    }

    #[inline]
    pub fn of<'a, B, M, T>(components: &'a ComponentRegistry<M, T>) -> Result<Self, ArchetypeError>
    where
        B: Bundle,
        Meta: FromComponentInfo<'a, M>,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let components = try_collect_opt_components(
            B::get_components(components).into_iter().map(|id| {
                let id = id?;
                let info = components.get_component_info(id)?;
                let meta = Meta::from_component_info(info);
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
        Meta: FromComponentInfo<'a, M>,
        M: FromComponentType,
        T: ComponentIdFromOrInsertWith<Key: FromComponentType> + ?Sized,
    {
        let components = try_collect_components(
            B::register_components(components).into_iter().map(|id| {
                let Some(info) = components.get_component_info(id) else {
                    unreachable!("info of {id} should be present")
                };
                let meta = Meta::from_component_info(info);
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
        components: &'a ComponentRegistry<W, impl ?Sized>,
        with: I,
    ) -> Result<Self, ArchetypeError>
    where
        I: IntoIterator<Item = (ComponentId, U)>,
        T: FromComponentInfo<'a, W>,
    {
        let components = try_collect_opt_components(
            with.into_iter().map(|(id, u)| {
                let info = components.get_component_info(id)?;
                let t = T::from_component_info(info);
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
    pub fn contains(&self, component_id: ComponentId) -> bool {
        let Self { components } = self;
        components.contains_key(component_id.into_u32())
    }

    #[inline]
    pub fn has_components(
        &self,
        of: &ErasedArchetype<impl Sized>,
    ) -> Result<(), MissingComponentError> {
        if let Some(id) = of.component_ids().find(|&id| !self.contains(id)) {
            let error = MissingComponentError::new(id);
            return Err(error);
        }
        Ok(())
    }

    #[inline]
    pub fn has_no_components(
        &self,
        of: &ErasedArchetype<impl Sized>,
    ) -> Result<(), AlreadyHasComponentError> {
        if let Some(id) = of.component_ids().find(|&id| self.contains(id)) {
            let error = AlreadyHasComponentError::new(id);
            return Err(error);
        }
        Ok(())
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<&Meta> {
        let Self { components } = self;

        let meta = components.get(component_id.into_u32())?.as_inner();
        Some(meta)
    }

    #[inline]
    pub fn get_index_of(&self, component_id: ComponentId) -> Option<usize> {
        let Self { components } = self;

        let index: usize = component_id.into_u32().try_into().ok()?;
        let sparse_item = components.as_sparse_slice().get(index)?;
        let index_of = sparse_item.dense_index().copied()?;
        index_of.try_into().ok()
    }

    #[inline]
    pub fn get_by_index(&self, index: usize) -> Option<(ComponentId, &Meta)> {
        let Self { components } = self;

        let index = index.try_into().ok()?;
        let (id, meta) = components.get_with_key(index)?;

        let id = unsafe { ComponentId::from_u32(id) };
        Some((id, meta))
    }

    #[inline]
    pub fn check_compatibility(
        &self,
        other: &ErasedArchetype<impl Sized>,
    ) -> Result<(), MissingComponentError> {
        self.has_components(other)
    }

    #[inline]
    pub fn check_compatibility_for<I>(
        &self,
        components: &ComponentRegistry<impl Sized, impl ?Sized>,
        component_ids: I,
    ) -> Result<(), IncompatibleArchetypeError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let other = ErasedArchetype::<()>::new(components, component_ids)?;
        self.check_compatibility(&other)?;
        Ok(())
    }

    #[inline]
    pub fn check_compatibility_of<B, T>(
        &self,
        components: &ComponentRegistry<impl Sized, T>,
    ) -> Result<(), IncompatibleArchetypeError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let other = ErasedArchetype::<()>::of::<B, _, _>(components)?;
        self.check_compatibility(&other)?;
        Ok(())
    }

    #[inline]
    pub fn check_exact_compatibility(
        &self,
        other: &ErasedArchetype<impl Sized>,
    ) -> Result<(), IncompatibleArchetypeExactError> {
        self.check_compatibility(other)?;

        if other.len() != self.len() {
            return Err(TooFewComponentsError.into());
        }
        Ok(())
    }

    #[inline]
    pub fn check_exact_compatibility_for<I>(
        &self,
        components: &ComponentRegistry<impl Sized, impl ?Sized>,
        component_ids: I,
    ) -> Result<(), IncompatibleArchetypeExactError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let other = ErasedArchetype::<()>::new(components, component_ids)?;
        self.check_exact_compatibility(&other)
    }

    #[inline]
    pub fn check_exact_compatibility_of<B, T>(
        &self,
        components: &ComponentRegistry<impl Sized, T>,
    ) -> Result<(), IncompatibleArchetypeExactError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let other = ErasedArchetype::<()>::of::<B, _, _>(components)?;
        self.check_exact_compatibility(&other)
    }

    #[inline]
    pub fn iter(&self) -> ErasedArchetypeIter<'_, Meta> {
        let Self { components } = self;

        let inner = components.iter();
        ErasedArchetypeIter { inner }
    }

    #[inline]
    pub fn component_ids(&self) -> ComponentIds<'_> {
        let Self { components } = self;

        let ids = components.as_key_slice().iter();
        ComponentIds { inner: ids }
    }

    #[inline]
    pub fn sorted_iter(&self) -> ErasedArchetypeSortedIter<'_, Meta> {
        let Self { components } = self;

        let dense = components.as_value_slices().as_inner();
        let sparse = components.as_sparse_slice().iter().enumerate();
        ErasedArchetypeSortedIter { dense, sparse }
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
        self.len() == other.len() && self.iter().eq(other)
    }
}

impl<Meta> Eq for ErasedArchetype<Meta> where Meta: Eq {}

impl<Meta> PartialOrd for ErasedArchetype<Meta>
where
    Meta: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<Meta> Ord for ErasedArchetype<Meta>
where
    Meta: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.iter().cmp(other)
    }
}

impl<Meta> Hash for ErasedArchetype<Meta>
where
    Meta: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        self.iter().for_each(|component| component.hash(state));
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

impl<'a, Meta> From<&'a ErasedArchetype<Meta>> for Cow<'a, ErasedArchetype<Meta>>
where
    Meta: Clone,
{
    #[inline]
    fn from(archetype: &'a ErasedArchetype<Meta>) -> Self {
        Self::Borrowed(archetype)
    }
}

impl<Meta> From<ErasedArchetype<Meta>> for Cow<'_, ErasedArchetype<Meta>>
where
    Meta: Clone,
{
    #[inline]
    fn from(archetype: ErasedArchetype<Meta>) -> Self {
        Self::Owned(archetype)
    }
}

impl<'a, Meta> IntoIterator for &'a ErasedArchetype<Meta> {
    type Item = ComponentInfo<&'a Meta>;
    type IntoIter = ErasedArchetypeIter<'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<Meta> IntoIterator for ErasedArchetype<Meta> {
    type Item = ComponentInfo<Meta>;
    type IntoIter = ErasedArchetypeIntoIter<Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { components } = self;

        let inner = components.into_iter();
        ErasedArchetypeIntoIter { inner }
    }
}

impl<'a, Meta> FieldDescriptors<'a> for ErasedArchetype<Meta>
where
    Meta: AsRef<FieldDescriptor> + 'a,
{
    type Output = &'a Self;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        self
    }
}

impl<Meta> CovariantFieldDescriptors for ErasedArchetype<Meta>
where
    Meta: AsRef<FieldDescriptor> + 'static,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        from
    }
}

pub struct ErasedArchetypeIter<'a, Meta>
where
    Meta: 'a,
{
    inner: SparseIter<'a, 'a, u32, Identity<Meta>>,
}

impl<Meta> Debug for ErasedArchetypeIter<'_, Meta>
where
    Meta: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.clone().map(From::from);
        f.debug_map().entries(entries).finish()
    }
}

impl<Meta> Clone for ErasedArchetypeIter<'_, Meta> {
    fn clone(&self) -> Self {
        let Self { inner } = self;
        let inner = inner.clone();
        Self { inner }
    }
}

impl<'a, Meta> Iterator for ErasedArchetypeIter<'a, Meta> {
    type Item = ComponentInfo<&'a Meta>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(|(&id, meta)| {
            let component_id = unsafe { ComponentId::from_u32(id) };
            let meta = meta.as_inner();
            ComponentInfo::new(component_id, meta)
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.count()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).map(|(&id, meta)| {
            let component_id = unsafe { ComponentId::from_u32(id) };
            let meta = meta.as_inner();
            ComponentInfo::new(component_id, meta)
        })
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.last().map(|(&id, meta)| {
            let component_id = unsafe { ComponentId::from_u32(id) };
            let meta = meta.as_inner();
            ComponentInfo::new(component_id, meta)
        })
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner
            .map(|(&id, meta)| {
                let component_id = unsafe { ComponentId::from_u32(id) };
                let meta = meta.as_inner();
                ComponentInfo::new(component_id, meta)
            })
            .collect()
    }
}

impl<Meta> DoubleEndedIterator for ErasedArchetypeIter<'_, Meta> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(|(&id, meta)| {
            let component_id = unsafe { ComponentId::from_u32(id) };
            let meta = meta.as_inner();
            ComponentInfo::new(component_id, meta)
        })
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(|(&id, meta)| {
            let component_id = unsafe { ComponentId::from_u32(id) };
            let meta = meta.as_inner();
            ComponentInfo::new(component_id, meta)
        })
    }
}

impl<Meta> ExactSizeIterator for ErasedArchetypeIter<'_, Meta> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<Meta> FusedIterator for ErasedArchetypeIter<'_, Meta> {}

impl<'a, Meta> FieldDescriptors<'a> for ErasedArchetypeIter<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Output = Self;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        self.clone()
    }
}

impl<Meta> CovariantFieldDescriptors for ErasedArchetypeIter<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        from
    }
}

#[derive(Clone)]
pub struct ComponentIds<'a> {
    inner: slice::Iter<'a, u32>,
}

impl Debug for ComponentIds<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.clone();
        f.debug_set().entries(entries).finish()
    }
}

impl Iterator for ComponentIds<'_> {
    type Item = ComponentId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(|&id| unsafe { ComponentId::from_u32(id) })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.count()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).map(|&id| unsafe { ComponentId::from_u32(id) })
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.last().map(|&id| unsafe { ComponentId::from_u32(id) })
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner
            .map(|&id| unsafe { ComponentId::from_u32(id) })
            .collect()
    }
}

impl DoubleEndedIterator for ComponentIds<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .next_back()
            .map(|&id| unsafe { ComponentId::from_u32(id) })
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .nth_back(n)
            .map(|&id| unsafe { ComponentId::from_u32(id) })
    }
}

impl ExactSizeIterator for ComponentIds<'_> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl FusedIterator for ComponentIds<'_> {}

pub struct ErasedArchetypeSortedIter<'a, Meta> {
    dense: &'a [Meta],
    sparse: Enumerate<slice::Iter<'a, SparseItem<u32>>>,
}

impl<'a, Meta> ErasedArchetypeSortedIter<'a, Meta> {
    #[inline]
    fn component_from(
        dense: &'a [Meta],
        sparse_index: usize,
        dense_index: u32,
    ) -> ComponentInfo<&'a Meta> {
        let id = sparse_index.try_into().expect("`ComponentId` overflow");
        let component_id = unsafe { ComponentId::from_u32(id) };

        let dense_index: usize = dense_index.try_into().expect("`ComponentId` overflow");
        let meta = &dense[dense_index];

        ComponentInfo::new(component_id, meta)
    }
}

impl<Meta> Debug for ErasedArchetypeSortedIter<'_, Meta>
where
    Meta: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.clone().map(From::from);
        f.debug_map().entries(entries).finish()
    }
}

impl<Meta> Clone for ErasedArchetypeSortedIter<'_, Meta> {
    fn clone(&self) -> Self {
        let Self { dense, sparse } = self;
        let sparse = sparse.clone();
        Self { dense, sparse }
    }
}

impl<'a, Meta> Iterator for ErasedArchetypeSortedIter<'a, Meta> {
    type Item = ComponentInfo<&'a Meta>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut sparse,
            dense,
        } = *self;

        let (sparse_index, dense_index) = sparse.find_map(|(index, item)| {
            let dense_index = item.into_dense_index()?;
            Some((index, dense_index))
        })?;

        let item = Self::component_from(dense, sparse_index, dense_index);
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { dense, sparse } = self;

        let upper = usize::min(dense.len(), sparse.len());
        (0, Some(upper))
    }
}

impl<Meta> DoubleEndedIterator for ErasedArchetypeSortedIter<'_, Meta> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut sparse,
            dense,
        } = *self;

        let (sparse_index, dense_index) = sparse.rev().find_map(|(index, item)| {
            let dense_index = item.into_dense_index()?;
            Some((index, dense_index))
        })?;

        let item = Self::component_from(dense, sparse_index, dense_index);
        Some(item)
    }
}

impl<Meta> FusedIterator for ErasedArchetypeSortedIter<'_, Meta> {}

impl<'a, Meta> FieldDescriptors<'a> for ErasedArchetypeSortedIter<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Output = Self;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        self.clone()
    }
}

impl<Meta> CovariantFieldDescriptors for ErasedArchetypeSortedIter<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        from
    }
}

pub struct ErasedArchetypeIntoIter<Meta> {
    inner: SparseIntoIter<u32, Identity<Meta>, Identity<Meta>>,
}

impl<Meta> ErasedArchetypeIntoIter<Meta> {
    #[inline]
    pub fn iter(&self) -> ErasedArchetypeIter<'_, Meta> {
        let Self { inner } = self;

        let (context, components, metas) = inner.as_slices_with_context();
        let inner = SparseIter::new(context, components, metas);
        ErasedArchetypeIter { inner }
    }
}

impl<Meta> Debug for ErasedArchetypeIntoIter<Meta>
where
    Meta: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.iter().fmt(f)
    }
}

impl<Meta> Clone for ErasedArchetypeIntoIter<Meta>
where
    Meta: Clone,
{
    fn clone(&self) -> Self {
        let Self { inner } = self;
        let inner = inner.clone();
        Self { inner }
    }
}

impl<'a, Meta> IntoIterator for &'a ErasedArchetypeIntoIter<Meta> {
    type Item = ComponentInfo<&'a Meta>;
    type IntoIter = ErasedArchetypeIter<'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<Meta> Iterator for ErasedArchetypeIntoIter<Meta> {
    type Item = ComponentInfo<Meta>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(|(id, meta)| {
            let component_id = unsafe { ComponentId::from_u32(id) };
            let meta = meta.into_inner();
            ComponentInfo::new(component_id, meta)
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.count()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).map(|(id, meta)| {
            let component_id = unsafe { ComponentId::from_u32(id) };
            let meta = meta.into_inner();
            ComponentInfo::new(component_id, meta)
        })
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.last().map(|(id, meta)| {
            let component_id = unsafe { ComponentId::from_u32(id) };
            let meta = meta.into_inner();
            ComponentInfo::new(component_id, meta)
        })
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner
            .map(|(id, meta)| {
                let component_id = unsafe { ComponentId::from_u32(id) };
                let meta = meta.into_inner();
                ComponentInfo::new(component_id, meta)
            })
            .collect()
    }
}

impl<Meta> DoubleEndedIterator for ErasedArchetypeIntoIter<Meta> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(|(id, meta)| {
            let component_id = unsafe { ComponentId::from_u32(id) };
            let meta = meta.into_inner();
            ComponentInfo::new(component_id, meta)
        })
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(|(id, meta)| {
            let component_id = unsafe { ComponentId::from_u32(id) };
            let meta = meta.into_inner();
            ComponentInfo::new(component_id, meta)
        })
    }
}

impl<Meta> ExactSizeIterator for ErasedArchetypeIntoIter<Meta> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<Meta> FusedIterator for ErasedArchetypeIntoIter<Meta> {}

impl<'a, Meta> FieldDescriptors<'a> for ErasedArchetypeIntoIter<Meta>
where
    Meta: AsRef<FieldDescriptor> + 'a,
{
    type Output = ErasedArchetypeIter<'a, Meta>;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        self.into_iter()
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
