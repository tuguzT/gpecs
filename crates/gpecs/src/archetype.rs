use std::{alloc::Layout, collections::BTreeSet};

use as_any::{AsAny, Downcast};
use gpecs_sparse::{set::EpochSparseSet, soa::erased::sorted_layouts_of};

use crate::{
    component::{ComponentId, ComponentRegistry},
    entity::Entity,
    prelude::Component,
    soa::{
        erased::{ErasedSoa, ErasedSoaContext},
        traits::Soa,
    },
};

pub trait Archetype: Soa + 'static {
    fn component_ids(components: &mut ComponentRegistry) -> impl IntoIterator<Item = ComponentId>;
}

pub struct ArchetypeStorage {
    component_ids: BTreeSet<ComponentId>,
    erased_storage: Box<dyn ErasedStorage>,
}

type SparseSet<V> = EpochSparseSet<Entity, V>;

impl ArchetypeStorage {
    #[inline]
    pub fn of<T>(components: &mut ComponentRegistry, context: T::Context) -> Self
    where
        T: Archetype,
    {
        let component_ids = T::component_ids(components).into_iter().collect();
        let storage = SparseSet::<T>::with_context(context);
        Self {
            component_ids,
            erased_storage: Box::new(storage),
        }
    }

    #[inline]
    pub fn entities(&self) -> &[Entity] {
        let Self { erased_storage, .. } = self;
        erased_storage.entities()
    }

    #[inline]
    pub fn get<T>(
        &self,
        components: &mut ComponentRegistry,
        entity: Entity,
    ) -> Result<Option<T::Refs<'_>>, ()>
    where
        T: Archetype,
    {
        let Self {
            component_ids,
            erased_storage,
        } = self;

        let target_component_ids: BTreeSet<_> = T::component_ids(components).into_iter().collect();
        if target_component_ids != *component_ids {
            return Err(());
        }

        let Some(storage) = erased_storage.as_ref().downcast_ref::<SparseSet<T>>() else {
            return Err(());
        };
        let refs = storage.get(entity);
        Ok(refs)
    }

    #[inline]
    pub fn get_mut<T>(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
    ) -> Result<Option<T::RefsMut<'_>>, ()>
    where
        T: Archetype,
    {
        let Self {
            component_ids,
            erased_storage,
        } = self;

        let target_component_ids: BTreeSet<_> = T::component_ids(components).into_iter().collect();
        if target_component_ids != *component_ids {
            return Err(());
        }

        let Some(storage) = erased_storage.as_mut().downcast_mut::<SparseSet<T>>() else {
            return Err(());
        };
        let refs = storage.get_mut(entity);
        Ok(refs)
    }

    #[inline]
    #[allow(unsafe_code)]
    pub fn insert<T>(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
        context: &T::Context,
        value: T,
    ) -> Result<Option<T>, T>
    where
        T: Archetype,
    {
        let Self {
            component_ids,
            erased_storage,
        } = self;

        let target_component_ids: BTreeSet<_> = T::component_ids(components).into_iter().collect();
        if target_component_ids != *component_ids {
            return Err(value);
        }

        let field_layouts = sorted_layouts_of::<T>(context);
        let erased_context = ErasedSoaContext::<T::Fields>::new(field_layouts, None);
        let fields = ErasedSoa::from(context, value).into_fields(&erased_context);
        let Some(fields) = erased_storage.insert(entity, fields) else {
            return Ok(None);
        };

        // TODO: restore correct order of fields (with component ids and the context of self)
        let fields = fields.iter().map(|(_, field)| field.as_ref());
        let value = unsafe { ErasedSoa::new(&erased_context, fields).into::<T>(&context) };
        Ok(Some(value))
    }

    #[inline]
    pub fn remove<T>(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
    ) -> Result<Option<T>, ()>
    where
        T: Archetype,
    {
        let Self {
            component_ids,
            erased_storage,
        } = self;

        let target_component_ids: BTreeSet<_> = T::component_ids(components).into_iter().collect();
        if target_component_ids != *component_ids {
            return Err(());
        }

        let Some(storage) = erased_storage.as_mut().downcast_mut::<SparseSet<T>>() else {
            return Err(());
        };
        let value = storage.remove(entity);
        Ok(value)
    }
}

type ErasedField = Box<[u8]>;
type ErasedFields = Box<[(Layout, ErasedField)]>;

trait ErasedStorage: AsAny {
    fn entities(&self) -> &[Entity];

    fn insert(&mut self, entity: Entity, fields: ErasedFields) -> Option<ErasedFields>;
}

#[allow(unsafe_code)]
impl<T> ErasedStorage for SparseSet<T>
where
    T: Archetype,
{
    fn entities(&self) -> &[Entity] {
        self.as_keys_slice()
    }

    fn insert(&mut self, entity: Entity, fields: ErasedFields) -> Option<ErasedFields> {
        // TODO: restore correct order of fields (with component ids and the context of self)

        let field_layouts = fields.iter().map(|(layout, _)| layout);
        let context = ErasedSoaContext::<T::Fields>::new(field_layouts, None);

        let fields = fields.iter().map(|(_, field)| field.as_ref());
        let value = ErasedSoa::<T::Fields>::new(&context, fields);

        let value = unsafe { value.into(self.context()) };
        let value = self.insert(entity, value)?;
        let fields = ErasedSoa::from(self.context(), value).into_fields(&context);
        Some(fields)
    }
}

impl Archetype for () {
    fn component_ids(_: &mut ComponentRegistry) -> impl IntoIterator<Item = ComponentId> {
        []
    }
}

impl<A> Archetype for (A,)
where
    A: Component,
{
    fn component_ids(components: &mut ComponentRegistry) -> impl IntoIterator<Item = ComponentId> {
        [components.register_component::<A>()]
    }
}

impl<A, B> Archetype for (A, B)
where
    A: Component,
    B: Component,
{
    fn component_ids(components: &mut ComponentRegistry) -> impl IntoIterator<Item = ComponentId> {
        [
            components.register_component::<A>(),
            components.register_component::<B>(),
        ]
    }
}

impl<A, B, C> Archetype for (A, B, C)
where
    A: Component,
    B: Component,
    C: Component,
{
    fn component_ids(components: &mut ComponentRegistry) -> impl IntoIterator<Item = ComponentId> {
        [
            components.register_component::<A>(),
            components.register_component::<B>(),
            components.register_component::<C>(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::entity::EntityRegistry;

    use super::*;

    #[test]
    fn unit_archetype() {
        let mut components = ComponentRegistry::new();
        let mut storage = ArchetypeStorage::of::<()>(&mut components, ());
        assert_eq!(storage.entities(), []);

        let mut entities = EntityRegistry::new();
        let entity = entities.spawn();
        let value = storage
            .insert::<()>(&mut components, entity, &(), ())
            .expect("archetype storage should store unit");
        assert_eq!(value, None);
        assert_eq!(storage.entities(), [entity]);

        let refs = storage
            .get::<()>(&mut components, entity)
            .expect("components by given entity should exist");
        assert_eq!(refs, Some(&()));
        assert_eq!(storage.entities(), [entity]);

        let value = storage
            .remove::<()>(&mut components, entity)
            .expect("components by given entity should exist");
        assert_eq!(value, Some(()));
        assert_eq!(storage.entities(), []);
    }

    #[derive(Debug, PartialEq, Clone, Copy)]
    struct Position {
        x: f32,
        y: f32,
        z: f32,
    }

    #[derive(Debug, PartialEq, Clone, Copy)]
    struct Mass {
        value: f32,
    }

    impl Component for Position {}
    impl Component for Mass {}

    #[test]
    fn tuple_archetype() {
        let mut components = ComponentRegistry::new();
        let mut storage = ArchetypeStorage::of::<(Position, Mass)>(&mut components, ());
        assert_eq!(storage.entities(), []);

        let mut entities = EntityRegistry::new();
        let entity = entities.spawn();

        let position = Position {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let mass = Mass { value: 4.0 };
        let value = storage
            .insert::<(Position, Mass)>(&mut components, entity, &(), (position, mass))
            .expect("archetype storage should store unit");
        assert_eq!(value, None);
        assert_eq!(storage.entities(), [entity]);

        let refs = storage
            .get::<(Position, Mass)>(&mut components, entity)
            .expect("components by given entity should exist");
        assert_eq!(refs, Some((&position, &mass)));
        assert_eq!(storage.entities(), [entity]);

        let value = storage
            .remove::<(Position, Mass)>(&mut components, entity)
            .expect("components by given entity should exist");
        assert_eq!(value, Some((position, mass)));
        assert_eq!(storage.entities(), []);
    }
}
