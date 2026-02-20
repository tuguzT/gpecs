use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use gpecs_soa_erased::CovariantFieldDescriptors;
use indexmap::map::{Iter as IndexMapIter, Values as IndexMapValues};

use crate::{
    archetype::{
        collect::{try_collect_components, try_collect_opt_components},
        error::{
            ArchetypeError, DuplicateComponentError, IncompatibleArchetypeError,
            IncompatibleArchetypeExactError, MissingComponentError, TooFewComponentsError,
        },
    },
    bundle::{Bundle, erased::utils::get_component_info_fail},
    component::{
        Component,
        registry::{ComponentId, ComponentInfo, ComponentRegistry, DropFn},
    },
    hash::IndexMap,
    soa::field::{FieldDescriptor, FieldDescriptors},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErasedArchetype<Meta = ()> {
    components: IndexMap<ComponentId, Meta>,
}

impl<Meta> ErasedArchetype<Meta> {
    #[inline]
    pub fn with_meta<I>(components: &ComponentRegistry, iter: I) -> Result<Self, ArchetypeError>
    where
        I: IntoIterator<Item = (ComponentId, Meta)>,
    {
        let components = try_collect_opt_components(
            iter.into_iter().map(|(component_id, meta)| {
                let _ = components.get_component_info(component_id)?;
                Some((component_id, meta))
            }),
            |map, (component_id, meta)| IndexMap::insert(map, component_id, meta).is_none(),
            |&(component_id, _)| component_id,
        )?;

        let me = Self { components };
        Ok(me)
    }

    #[inline]
    pub unsafe fn with_meta_unchecked<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (ComponentId, Meta)>,
    {
        let components = FromIterator::from_iter(iter);
        Self { components }
    }
}

pub trait FromComponentInfo: Sized {
    fn from_component_info(info: &ComponentInfo) -> Self;
}

impl FromComponentInfo for () {
    #[inline]
    fn from_component_info(_: &ComponentInfo) -> Self {}
}

impl FromComponentInfo for ComponentInfo {
    #[inline]
    fn from_component_info(info: &ComponentInfo) -> Self {
        info.clone()
    }
}

impl FromComponentInfo for FieldDescriptor {
    #[inline]
    fn from_component_info(info: &ComponentInfo) -> Self {
        info.descriptor()
    }
}

impl FromComponentInfo for Option<DropFn> {
    #[inline]
    fn from_component_info(info: &ComponentInfo) -> Self {
        info.drop_fn()
    }
}

impl<Meta> ErasedArchetype<Meta>
where
    Meta: FromComponentInfo,
{
    #[inline]
    pub fn new<I>(components: &ComponentRegistry, component_ids: I) -> Result<Self, ArchetypeError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let components = try_collect_opt_components(
            component_ids.into_iter().map(|component_id| {
                let component_info = components.get_component_info(component_id)?;
                let meta = Meta::from_component_info(component_info);
                Some((component_id, meta))
            }),
            |map, (component_id, meta)| IndexMap::insert(map, component_id, meta).is_none(),
            |&(component_id, _)| component_id,
        )?;

        let me = Self { components };
        Ok(me)
    }

    #[inline]
    pub fn of<B>(components: &mut ComponentRegistry) -> Result<Self, DuplicateComponentError>
    where
        B: Bundle,
    {
        let components = try_collect_components(
            B::register_components(components)
                .into_iter()
                .map(|component_id| {
                    let component_info = components
                        .get_component_info(component_id)
                        .unwrap_or_else(|| get_component_info_fail(component_id));
                    let meta = Meta::from_component_info(component_info);
                    (component_id, meta)
                }),
            |map, (component_id, meta)| IndexMap::insert(map, component_id, meta).is_none(),
            |&(component_id, _)| component_id,
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
        components.contains_key(&component_id)
    }

    #[inline]
    pub fn has<C>(&self, components: &ComponentRegistry) -> bool
    where
        C: Component,
    {
        let Some(component_id) = components.component_id::<C>() else {
            return false;
        };
        self.contains(component_id)
    }

    #[inline]
    pub fn check_compatibility<M>(
        &self,
        other: &ErasedArchetype<M>,
    ) -> Result<(), IncompatibleArchetypeError> {
        let ErasedArchetype { components } = other;
        self.check_compatibility_inner(components)
    }

    #[inline]
    pub fn check_compatibility_for<I>(
        &self,
        component_ids: I,
    ) -> Result<(), IncompatibleArchetypeError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let component_ids = try_collect_components(
            component_ids,
            |map, component_id| IndexMap::insert(map, component_id, ()).is_none(),
            Clone::clone,
        )?;
        self.check_compatibility_inner(&component_ids)
    }

    #[inline]
    pub fn check_compatibility_of<B>(
        &self,
        components: &ComponentRegistry,
    ) -> Result<(), IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        let component_ids = try_collect_opt_components(
            B::get_components(components),
            |map, component_id| IndexMap::insert(map, component_id, ()).is_none(),
            Clone::clone,
        )?;
        self.check_compatibility_inner(&component_ids)
    }

    #[inline]
    fn check_compatibility_inner<M>(
        &self,
        components: &IndexMap<ComponentId, M>,
    ) -> Result<(), IncompatibleArchetypeError> {
        let mut component_ids = components.keys().copied();
        let Self { components } = self;

        if let Some(component_id) = component_ids.find(|id| !components.contains_key(id)) {
            let error = MissingComponentError::new(component_id);
            return Err(error.into());
        }
        Ok(())
    }

    #[inline]
    pub fn check_exact_compatibility<M>(
        &self,
        other: &ErasedArchetype<M>,
    ) -> Result<(), IncompatibleArchetypeExactError> {
        let ErasedArchetype { components } = other;
        self.check_exact_compatibility_inner(components)
    }

    #[inline]
    pub fn check_exact_compatibility_for<I>(
        &self,
        component_ids: I,
    ) -> Result<(), IncompatibleArchetypeExactError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let components = try_collect_components(
            component_ids,
            |map, component_id| IndexMap::insert(map, component_id, ()).is_none(),
            Clone::clone,
        )?;
        self.check_exact_compatibility_inner(&components)
    }

    #[inline]
    pub fn check_exact_compatibility_of<B>(
        &self,
        components: &ComponentRegistry,
    ) -> Result<(), IncompatibleArchetypeExactError>
    where
        B: Bundle,
    {
        let components = B::get_components(components);
        let components = try_collect_opt_components(
            components,
            |map, component_id| IndexMap::insert(map, component_id, ()).is_none(),
            Clone::clone,
        )?;
        self.check_exact_compatibility_inner(&components)
    }

    #[inline]
    fn check_exact_compatibility_inner<M>(
        &self,
        components: &IndexMap<ComponentId, M>,
    ) -> Result<(), IncompatibleArchetypeExactError> {
        self.check_compatibility_inner(components)?;

        if components.len() != self.len() {
            return Err(TooFewComponentsError.into());
        }
        Ok(())
    }

    #[inline]
    pub fn iter(&self) -> ErasedArchetypeIter<'_, Meta> {
        let Self { components } = self;

        let inner = components.iter();
        ErasedArchetypeIter { inner }
    }

    #[inline]
    pub fn metas(&self) -> ErasedArchetypeMetas<'_, Meta> {
        let Self { components } = self;

        let inner = components.values();
        ErasedArchetypeMetas { inner }
    }
}

impl<'a, Meta> IntoIterator for &'a ErasedArchetype<Meta> {
    type Item = (ComponentId, &'a Meta);
    type IntoIter = ErasedArchetypeIter<'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, Meta> FieldDescriptors<'a> for ErasedArchetype<Meta>
where
    Meta: AsRef<FieldDescriptor> + 'a,
{
    type Output = ErasedArchetypeMetas<'a, Meta>;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        self.metas()
    }
}

impl<Meta> CovariantFieldDescriptors for ErasedArchetype<Meta>
where
    Meta: AsRef<FieldDescriptor> + 'static,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: <Self as FieldDescriptors<'long>>::Output,
    ) -> <Self as FieldDescriptors<'short>>::Output {
        from
    }
}

pub struct ErasedArchetypeIter<'a, Meta = ()> {
    inner: IndexMapIter<'a, ComponentId, Meta>,
}

impl<Meta> Debug for ErasedArchetypeIter<'_, Meta>
where
    Meta: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;
        Debug::fmt(inner, f)
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
    type Item = (ComponentId, &'a Meta);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(|(&id, meta)| (id, meta))
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
        inner.nth(n).map(|(&id, meta)| (id, meta))
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.last().map(|(&id, meta)| (id, meta))
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.map(|(&id, meta)| (id, meta)).collect()
    }
}

impl<Meta> DoubleEndedIterator for ErasedArchetypeIter<'_, Meta> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(|(&id, meta)| (id, meta))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(|(&id, meta)| (id, meta))
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

pub struct ErasedArchetypeMetas<'a, Meta> {
    inner: IndexMapValues<'a, ComponentId, Meta>,
}

impl<Meta> Debug for ErasedArchetypeMetas<'_, Meta>
where
    Meta: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;
        Debug::fmt(inner, f)
    }
}

impl<Meta> Clone for ErasedArchetypeMetas<'_, Meta> {
    fn clone(&self) -> Self {
        let Self { inner } = self;
        let inner = inner.clone();
        Self { inner }
    }
}

impl<'a, Meta> Iterator for ErasedArchetypeMetas<'a, Meta> {
    type Item = &'a Meta;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next()
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
        inner.nth(n)
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.last()
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.collect()
    }
}

impl<Meta> DoubleEndedIterator for ErasedArchetypeMetas<'_, Meta> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back()
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n)
    }
}

impl<Meta> ExactSizeIterator for ErasedArchetypeMetas<'_, Meta> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<Meta> FusedIterator for ErasedArchetypeMetas<'_, Meta> {}

impl<'a, Meta> FieldDescriptors<'a> for ErasedArchetypeMetas<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Output = Self;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        self.clone()
    }
}
