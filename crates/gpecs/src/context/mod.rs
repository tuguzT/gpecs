use crate::{
    archetype::registry::{ArchetypeRegistry, EntityArchetype},
    component::registry::ComponentRegistry,
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
}
