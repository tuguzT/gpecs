use std::any::TypeId;

use error::{EntityNotFoundError, RemoveBundleError};

use crate::{
    archetype::{
        error::{ArchetypeError, DuplicateComponentError},
        registry::{
            ArchetypeId, ArchetypeInfo, ArchetypeRegistry, Bundles, BundlesMut, EntityArchetype,
        },
    },
    bundle::{Bundle, BundleRefs, BundleRefsMut},
    component::{
        Component,
        registry::{ComponentId, ComponentInfo, ComponentRegistry},
    },
    entity::{
        Entity,
        registry::{self as entities, EntityRegistry},
    },
    world::registry::{WorldId, WorldRegistry},
};

use self::error::{
    EntityHasNoDataError, IncompatibleBundleError, InsertBundleError, InsertBundleExactError,
    RemoveBundleExactError,
};

pub mod error;

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
    pub fn as_parts(&self) -> ContextPartsRefs<'_> {
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
    pub unsafe fn as_parts_mut(&mut self) -> ContextPartsRefsMut<'_> {
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
    pub fn spawn_world(&mut self) -> WorldId {
        let Self { worlds, .. } = self;
        worlds.spawn()
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
        let Self { entities, .. } = self;
        entities.despawn(entity).is_some()
    }

    #[inline]
    pub fn contains(&self, entity: Entity) -> bool {
        let Self { entities, .. } = self;
        entities.contains(entity)
    }

    #[inline]
    pub fn destroy_all(&mut self) {
        let Self {
            entities,
            archetypes,
            ..
        } = self;

        entities.clear();
        archetypes.destroy_all();
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
    pub fn get_component_info(&self, component_id: ComponentId) -> Option<&ComponentInfo> {
        let Self { components, .. } = self;
        components.get_component_info(component_id)
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
    pub fn register_archetype_of<B>(&mut self) -> Result<ArchetypeId, DuplicateComponentError>
    where
        B: Bundle,
    {
        let Self {
            components,
            archetypes,
            ..
        } = self;
        archetypes.register_archetype_of::<B>(components)
    }

    #[inline]
    pub fn get_archetype_info(&self, archetype_id: ArchetypeId) -> Option<&ArchetypeInfo> {
        let Self { archetypes, .. } = self;
        archetypes.get_archetype_info(archetype_id)
    }

    #[inline]
    pub fn archetype_id_of<B>(&self) -> Result<Option<ArchetypeId>, ArchetypeError>
    where
        B: Bundle,
    {
        let Self {
            components,
            archetypes,
            ..
        } = self;
        archetypes.archetype_id_of::<B>(components)
    }

    #[inline]
    pub fn get_bundle<B>(
        &self,
        entity: Entity,
    ) -> Result<BundleRefs<'_, B>, IncompatibleBundleError>
    where
        B: Bundle,
    {
        let Self {
            entities,
            components,
            archetypes,
            ..
        } = self;

        let Some(archetype_id) = entities.get(entity).copied() else {
            return Err(EntityNotFoundError::new(entity).into());
        };
        let Some(archetype_id) = archetype_id else {
            return Err(EntityHasNoDataError::new(entity).into());
        };

        let location = archetype_id.into();
        let bundle = archetypes
            .get_bundle_with::<B>(components, entity, location)?
            .expect("entity should contain data");
        Ok(bundle)
    }

    #[inline]
    pub fn get_bundle_mut<B>(
        &mut self,
        entity: Entity,
    ) -> Result<BundleRefsMut<'_, B>, IncompatibleBundleError>
    where
        B: Bundle,
    {
        let Self {
            entities,
            components,
            archetypes,
            ..
        } = self;

        let Some(archetype_id) = entities.get(entity).copied() else {
            return Err(EntityNotFoundError::new(entity).into());
        };
        let Some(archetype_id) = archetype_id else {
            return Err(EntityHasNoDataError::new(entity).into());
        };

        let location = archetype_id.into();
        let bundle = archetypes
            .get_bundle_mut_with::<B>(components, entity, location)?
            .expect("entity should contain data");
        Ok(bundle)
    }

    #[inline]
    pub fn bundles<B>(&self) -> Result<Bundles<'_, '_, B>, ArchetypeError>
    where
        B: Bundle,
    {
        let Self {
            components,
            archetypes,
            ..
        } = self;
        archetypes.bundles::<B>(components)
    }

    #[inline]
    pub fn bundles_mut<B>(&mut self) -> Result<BundlesMut<'_, '_, B>, ArchetypeError>
    where
        B: Bundle,
    {
        let Self {
            components,
            archetypes,
            ..
        } = self;
        archetypes.bundles_mut::<B>(components)
    }

    #[inline]
    pub fn insert_bundle_exact<B>(
        &mut self,
        entity: Entity,
        value: B,
    ) -> Result<(), InsertBundleExactError<B>>
    where
        B: Bundle,
    {
        let Self {
            entities,
            components,
            archetypes,
            ..
        } = self;

        let Some(archetype_id) = entities.get_mut(entity) else {
            let kind = EntityNotFoundError::new(entity).into();
            return Err(InsertBundleExactError { value, kind });
        };

        let location = archetype_id.as_ref().copied().into();
        let new_archetype_id =
            archetypes.insert_bundle_exact_with::<B>(components, entity, value, location)?;

        *archetype_id = Some(new_archetype_id);
        Ok(())
    }

    #[inline]
    pub fn insert_bundle<B>(&mut self, entity: Entity, value: B) -> Result<(), InsertBundleError<B>>
    where
        B: Bundle,
    {
        let Self {
            entities,
            components,
            archetypes,
            ..
        } = self;

        let Some(archetype_id) = entities.get_mut(entity) else {
            let kind = EntityNotFoundError::new(entity).into();
            return Err(InsertBundleError { value, kind });
        };

        let location = archetype_id.as_ref().copied().into();
        let new_archetype_id =
            archetypes.insert_bundle_with::<B>(components, entity, value, location)?;

        *archetype_id = Some(new_archetype_id);
        Ok(())
    }

    #[inline]
    pub fn remove_bundle<B>(&mut self, entity: Entity) -> Result<(), RemoveBundleError>
    where
        B: Bundle,
    {
        let Self {
            entities,
            components,
            archetypes,
            ..
        } = self;

        let Some(archetype_id) = entities.get_mut(entity) else {
            return Err(EntityNotFoundError::new(entity).into());
        };

        let location = archetype_id.as_ref().copied().into();
        let new_archetype_id = archetypes.remove_bundle_with::<B>(components, entity, location)?;

        *archetype_id = new_archetype_id;
        Ok(())
    }

    #[inline]
    pub fn remove_bundle_exact<B>(&mut self, entity: Entity) -> Result<B, RemoveBundleExactError>
    where
        B: Bundle,
    {
        let Self {
            entities,
            components,
            archetypes,
            ..
        } = self;

        let Some(archetype_id) = entities.get_mut(entity) else {
            return Err(EntityNotFoundError::new(entity).into());
        };

        let location = archetype_id.as_ref().copied().into();
        let (value, new_archetype_id) =
            archetypes.remove_bundle_exact_with::<B>(components, entity, location)?;
        let Some(value) = value else {
            return Err(EntityHasNoDataError::new(entity).into());
        };

        *archetype_id = new_archetype_id;
        Ok(value)
    }
}
