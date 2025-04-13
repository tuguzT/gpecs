use std::any::TypeId;

use crate::{
    archetype::registry::{ArchetypeId, ArchetypeInfo, ArchetypeRegistry, EntityArchetype},
    bundle::{
        error::{DuplicateComponentError, GetComponentsError},
        Bundle,
    },
    component::{
        registry::{ComponentId, ComponentInfo, ComponentRegistry},
        Component,
    },
    entity::{
        registry::{self as entities, EntityRegistry},
        Entity,
    },
    world::registry::{WorldId, WorldRegistry},
};

pub type Worlds = WorldRegistry;
pub type Entities = EntityRegistry<EntityArchetype>;
pub type Components = ComponentRegistry;
pub type Archetypes = ArchetypeRegistry;

pub type ContextPartsRefs<'a> = (&'a Worlds, &'a Entities, &'a Components, &'a Archetypes);
pub type ContextPartsRefsMut<'a> = (
    &'a mut Worlds,
    &'a mut Entities,
    &'a mut Components,
    &'a mut Archetypes,
);
pub type ContextParts = (Worlds, Entities, Components, Archetypes);

pub type TrySpawnError = entities::TrySpawnError<EntityArchetype>;

#[derive(Debug, Default)]
pub struct Context {
    worlds: Worlds,
    entities: Entities,
    components: Components,
    archetypes: Archetypes,
}

impl Context {
    #[inline]
    pub fn new() -> Self {
        Self {
            worlds: Worlds::new(),
            entities: Entities::new(),
            components: Components::new(),
            archetypes: Archetypes::new(),
        }
    }

    #[inline]
    pub fn as_parts(&self) -> ContextPartsRefs {
        let Self {
            worlds,
            entities,
            components,
            archetypes,
        } = self;
        (worlds, entities, components, archetypes)
    }

    #[inline]
    pub fn worlds(&self) -> &Worlds {
        let Self { worlds, .. } = self;
        worlds
    }

    #[inline]
    pub fn entities(&self) -> &Entities {
        let Self { entities, .. } = self;
        entities
    }

    #[inline]
    pub fn components(&self) -> &Components {
        let Self { components, .. } = self;
        components
    }

    #[inline]
    pub fn archetypes(&self) -> &Archetypes {
        let Self { archetypes, .. } = self;
        archetypes
    }

    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn as_parts_mut(&mut self) -> ContextPartsRefsMut {
        let Self {
            worlds,
            entities,
            components,
            archetypes,
        } = self;
        (worlds, entities, components, archetypes)
    }

    #[inline]
    pub fn worlds_mut(&mut self) -> &mut Worlds {
        let Self { worlds, .. } = self;
        worlds
    }

    #[inline]
    pub fn components_mut(&mut self) -> &mut Components {
        let Self { components, .. } = self;
        components
    }

    #[inline]
    pub fn into_parts(self) -> ContextParts {
        let Self {
            worlds,
            entities,
            components,
            archetypes,
        } = self;
        (worlds, entities, components, archetypes)
    }

    #[inline]
    pub fn spawn(&mut self) -> Entity {
        let world = WorldId::default();
        self.spawn_in(world)
    }

    #[inline]
    pub fn try_spawn(&mut self) -> Result<Entity, TrySpawnError> {
        let world = WorldId::default();
        self.try_spawn_in(world)
    }

    #[inline]
    pub fn spawn_in(&mut self, world: WorldId) -> Entity {
        let Self { entities, .. } = self;
        entities.spawn(world, None)
    }

    #[inline]
    pub fn try_spawn_in(&mut self, world: WorldId) -> Result<Entity, TrySpawnError> {
        let Self { entities, .. } = self;
        entities.try_spawn(world, None)
    }

    #[inline]
    pub fn despawn(&mut self, entity: Entity) -> bool {
        let Self {
            entities,
            archetypes,
            ..
        } = self;

        let Some(archetype_id) = entities.despawn(entity) else {
            return false;
        };
        archetypes.destroy_in_place(entity, archetype_id.into())
    }

    #[inline]
    pub fn contains(&self, entity: Entity) -> bool {
        let Self { entities, .. } = self;
        entities.contains(entity)
    }

    #[inline]
    pub fn register_component<C>(&mut self) -> ComponentId
    where
        C: Component,
    {
        let Self { components, .. } = self;
        components.register_component::<C>()
    }

    #[inline]
    pub fn get_component_info(&self, id: ComponentId) -> Option<&ComponentInfo> {
        let Self { components, .. } = self;
        components.get_component_info(id)
    }

    #[inline]
    pub fn component_id_from(&self, type_id: TypeId) -> Option<ComponentId> {
        let Self { components, .. } = self;
        components.component_id_from(type_id)
    }

    #[inline]
    pub fn component_id<C>(&self) -> Option<ComponentId>
    where
        C: Component,
    {
        let Self { components, .. } = self;
        components.component_id::<C>()
    }

    #[inline]
    pub fn register_archetype<B>(
        &mut self,
        context: &B::Context,
    ) -> Result<ArchetypeId, DuplicateComponentError>
    where
        B: Bundle,
    {
        let Self {
            components,
            archetypes,
            ..
        } = self;

        archetypes.register_archetype::<B>(components, context)
    }

    #[inline]
    pub fn get_archetype_info(&self, archetype_id: ArchetypeId) -> Option<&ArchetypeInfo> {
        let Self { archetypes, .. } = self;
        archetypes.get_archetype_info(archetype_id)
    }

    #[inline]
    pub fn archetype_id<B>(
        &self,
        context: &B::Context,
    ) -> Result<Option<ArchetypeId>, GetComponentsError>
    where
        B: Bundle,
    {
        let Self {
            components,
            archetypes,
            ..
        } = self;

        archetypes.archetype_id::<B>(components, context)
    }
}
