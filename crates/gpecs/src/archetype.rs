use std::{
    alloc::Layout,
    borrow::Borrow,
    collections::{BTreeMap, BTreeSet},
    iter,
};

use as_any::{AsAny, Downcast};
use gpecs_sparse::set::EpochSparseSet;
use gpecs_utils::permutation::apply as apply_permutation;

use crate::{
    component::{ComponentId, ComponentRegistry},
    entity::Entity,
    prelude::Component,
    soa::{
        erased::{ErasedSoa, ErasedSoaContext},
        traits::Soa,
    },
};

#[allow(unsafe_code)]
pub unsafe trait Archetype: Soa + 'static {
    // order of component ids should be the same as the order of layouts returned by `field_layouts` method
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

        let fields = into_erased_fields(components, context, value);
        let Some(fields) = erased_storage.insert(components, entity, fields) else {
            return Ok(None);
        };
        let value = from_erased_fields(components, context, fields);
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
type ErasedFields = BTreeMap<ComponentId, (Layout, ErasedField)>;

trait ErasedStorage: AsAny {
    fn entities(&self) -> &[Entity];

    fn insert(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
        fields: ErasedFields,
    ) -> Option<ErasedFields>;
}

impl<T> ErasedStorage for SparseSet<T>
where
    T: Archetype,
{
    fn entities(&self) -> &[Entity] {
        self.as_keys_slice()
    }

    fn insert(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
        fields: ErasedFields,
    ) -> Option<ErasedFields> {
        let value = from_erased_fields(components, self.context(), fields);
        let value = self.insert(entity, value)?;
        let fields = into_erased_fields(components, self.context(), value);
        Some(fields)
    }
}

#[allow(unsafe_code)]
fn from_erased_fields<T>(
    components: &mut ComponentRegistry,
    context: &T::Context,
    mut fields: ErasedFields,
) -> T
where
    T: Archetype,
{
    let len = fields.len();
    let mut fields: Box<[_]> = T::component_ids(components)
        .into_iter()
        .map(|id| {
            fields
                .remove(&id)
                .expect("field with given component id should present")
        })
        .collect();
    assert_eq!(fields.len(), len);

    let mut permutation: Box<[_]> = T::field_permutation(context).into_iter().collect();
    apply_permutation(&mut permutation, &mut fields);

    let erased_context = ErasedSoaContext::<T::Fields>::new(
        fields.iter().map(|(field_layout, _)| field_layout),
        None,
    );
    let erased_value = ErasedSoa::<T::Fields>::new(
        &erased_context,
        fields.iter().map(|(_, field)| field.as_ref()),
    );
    unsafe { erased_value.into(context) }
}

fn into_erased_fields<T>(
    components: &mut ComponentRegistry,
    context: &T::Context,
    value: T,
) -> ErasedFields
where
    T: Archetype,
{
    let mut field_metadata: Box<[(Layout, ComponentId)]> =
        iter::zip(T::field_layouts(context), T::component_ids(components))
            .map(|(item, component_id)| (item.borrow().clone(), component_id))
            .collect();

    let mut permutation: Box<[_]> = T::field_permutation(context).into_iter().collect();
    apply_permutation(&mut permutation, &mut field_metadata);

    let erased_context = ErasedSoaContext::<T::Fields>::new(
        field_metadata.iter().map(|(field_layout, _)| field_layout),
        None,
    );
    let erased_value = ErasedSoa::from(context, value);

    iter::zip(field_metadata, erased_value.into_fields(&erased_context))
        .map(|((_, component_id), field)| (component_id, field))
        .collect()
}

#[allow(unsafe_code)]
unsafe impl Archetype for () {
    fn component_ids(components: &mut ComponentRegistry) -> impl IntoIterator<Item = ComponentId> {
        [components.register_component::<Self>()]
    }
}

#[allow(unsafe_code)]
unsafe impl<A> Archetype for (A,)
where
    A: Component,
{
    fn component_ids(components: &mut ComponentRegistry) -> impl IntoIterator<Item = ComponentId> {
        [components.register_component::<A>()]
    }
}

#[allow(unsafe_code)]
unsafe impl<A, B> Archetype for (A, B)
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

#[allow(unsafe_code)]
unsafe impl<A, B, C> Archetype for (A, B, C)
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
            .insert::<(Mass, Position)>(&mut components, entity, &(), (mass, position))
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
