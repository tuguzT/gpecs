use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use gpecs_types::soa::field::FieldDescriptor;
use indexmap::map::Iter as IndexMapIter;

use crate::{
    archetype::{
        collect::{try_collect_components, try_collect_opt_components},
        error::{
            ArchetypeError, DuplicateComponentError, IncompatibleArchetypeError,
            IncompatibleArchetypeExactError, MissingComponentError, TooFewComponentsError,
        },
    },
    bundle::{Bundle, erased::get_component_info_fail},
    component::{
        Component,
        registry::{ComponentId, ComponentInfo, ComponentRegistry, DropFn},
    },
    hash::IndexMap,
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
        let component_ids = B::get_components(components);
        let component_ids = try_collect_opt_components(
            component_ids,
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
    pub fn components(&self) -> ErasedArchetypeComponents<'_, Meta> {
        let Self { components } = self;

        let components = components.iter();
        ErasedArchetypeComponents { components }
    }
}

pub struct ErasedArchetypeComponents<'a, Meta = ()> {
    components: IndexMapIter<'a, ComponentId, Meta>,
}

impl<Meta> Debug for ErasedArchetypeComponents<'_, Meta>
where
    Meta: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { components } = self;
        Debug::fmt(components, f)
    }
}

impl<Meta> Clone for ErasedArchetypeComponents<'_, Meta> {
    fn clone(&self) -> Self {
        let Self { components } = self;
        let components = components.clone();
        Self { components }
    }
}

impl<'a, Meta> Iterator for ErasedArchetypeComponents<'a, Meta> {
    type Item = (ComponentId, &'a Meta);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { components } = self;
        components.next().map(|(&id, meta)| (id, meta))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { components } = self;
        components.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { components } = self;
        components.count()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { components } = self;
        components.nth(n).map(|(&id, meta)| (id, meta))
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { components } = self;
        components.last().map(|(&id, meta)| (id, meta))
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B
    where
        Self: Sized,
    {
        let Self { components } = self;
        components.map(|(&id, meta)| (id, meta)).collect()
    }
}

impl<Meta> DoubleEndedIterator for ErasedArchetypeComponents<'_, Meta> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { components } = self;
        components.next_back().map(|(&id, meta)| (id, meta))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { components } = self;
        components.nth_back(n).map(|(&id, meta)| (id, meta))
    }
}

impl<Meta> ExactSizeIterator for ErasedArchetypeComponents<'_, Meta> {
    #[inline]
    fn len(&self) -> usize {
        let Self { components } = self;
        components.len()
    }
}

impl<Meta> FusedIterator for ErasedArchetypeComponents<'_, Meta> {}
