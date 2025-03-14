use std::{
    alloc::Layout,
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Debug},
    iter,
};

use gpecs_sparse::set::EpochSparseSet;

use crate::{
    bundle::{Bundle, DuplicateComponentError},
    component::{ComponentId, ComponentRegistry},
    entity::Entity,
    soa::erased::{ErasedSoa, ErasedSoaContext, ErasedSoaRefs, ErasedSoaRefsMut},
};

use super::error::{
    ExclusiveComponentError, IncompatibleBundleError, IncompatibleBundleExactError,
    IncompatibleBundleValueError, TooFewComponentsError,
};

pub struct ArchetypeStorage {
    component_ids: BTreeSet<ComponentId>,
    erased_storage: Box<dyn ErasedStorage>,
}

type SparseSet<V> = EpochSparseSet<Entity, V>;

impl ArchetypeStorage {
    #[inline]
    pub fn of<B>(
        components: &mut ComponentRegistry,
        context: B::Context,
    ) -> Result<Self, DuplicateComponentError>
    where
        B: Bundle,
    {
        let component_ids = B::component_ids(&context, components)?
            .into_iter()
            .collect();
        let storage = SparseSet::<B>::with_context(context);

        let this = Self {
            component_ids,
            erased_storage: Box::new(storage),
        };
        Ok(this)
    }

    #[inline]
    pub fn entities(&self) -> &[Entity] {
        let Self { erased_storage, .. } = self;
        erased_storage.entities()
    }

    #[inline]
    pub fn get<B>(
        &self,
        components: &mut ComponentRegistry,
        context: &B::Context,
        entity: Entity,
    ) -> Result<Option<B::Refs<'_>>, IncompatibleBundleError>
    where
        B: Bundle,
    {
        let Self {
            component_ids,
            erased_storage,
        } = self;

        let mut target_component_ids = B::component_ids(context, components)?.into_iter();
        if let Some(component_id) = target_component_ids.find(|id| !component_ids.contains(&id)) {
            return Err(ExclusiveComponentError { component_id }.into());
        }

        let Some(fields) = erased_storage.get(components, entity) else {
            return Ok(None);
        };
        let refs = from_erased_field_refs::<B>(components, context, fields);
        Ok(Some(refs))
    }

    #[inline]
    pub fn get_mut<B>(
        &mut self,
        components: &mut ComponentRegistry,
        context: &B::Context,
        entity: Entity,
    ) -> Result<Option<B::RefsMut<'_>>, IncompatibleBundleError>
    where
        B: Bundle,
    {
        let Self {
            component_ids,
            erased_storage,
        } = self;

        let mut target_component_ids = B::component_ids(context, components)?.into_iter();
        if let Some(component_id) = target_component_ids.find(|id| !component_ids.contains(&id)) {
            return Err(ExclusiveComponentError { component_id }.into());
        }

        let Some(fields) = erased_storage.get_mut(components, entity) else {
            return Ok(None);
        };
        let refs = from_erased_field_refs_mut::<B>(components, context, fields);
        Ok(Some(refs))
    }

    #[inline]
    pub fn insert<B>(
        &mut self,
        components: &mut ComponentRegistry,
        context: &B::Context,
        entity: Entity,
        value: B,
    ) -> Result<Option<B>, IncompatibleBundleValueError<B>>
    where
        B: Bundle,
    {
        let Self {
            component_ids,
            erased_storage,
        } = self;

        let mut target_component_ids_count = 0;
        let mut target_component_ids = B::component_ids(context, components)
            .expect("components of the bundle should be unique")
            .into_iter()
            .inspect(|_| target_component_ids_count += 1);
        if let Some(component_id) = target_component_ids.find(|id| !component_ids.contains(&id)) {
            let reason = ExclusiveComponentError { component_id }.into();
            return Err(IncompatibleBundleValueError { value, reason });
        }

        target_component_ids.for_each(drop);
        if target_component_ids_count != component_ids.len() {
            let reason = TooFewComponentsError.into();
            return Err(IncompatibleBundleValueError { value, reason });
        }

        let fields = into_erased_fields::<B>(components, context, value);
        let Some(fields) = erased_storage.insert(components, entity, fields) else {
            return Ok(None);
        };
        let value = from_erased_fields::<B>(components, context, fields);
        Ok(Some(value))
    }

    #[inline]
    pub fn remove<B>(
        &mut self,
        components: &mut ComponentRegistry,
        context: &B::Context,
        entity: Entity,
    ) -> Result<Option<B>, IncompatibleBundleExactError>
    where
        B: Bundle,
    {
        let Self {
            component_ids,
            erased_storage,
        } = self;

        let mut target_component_ids_count = 0;
        let mut target_component_ids = B::component_ids(context, components)?
            .into_iter()
            .inspect(|_| target_component_ids_count += 1);
        if let Some(component_id) = target_component_ids.find(|id| !component_ids.contains(&id)) {
            return Err(ExclusiveComponentError { component_id }.into());
        }

        target_component_ids.for_each(drop);
        if target_component_ids_count != component_ids.len() {
            return Err(TooFewComponentsError.into());
        }

        let Some(fields) = erased_storage.remove(components, entity) else {
            return Ok(None);
        };
        let value = from_erased_fields::<B>(components, context, fields);
        Ok(Some(value))
    }
}

impl Debug for ArchetypeStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { component_ids, .. } = self;

        f.debug_struct("ArchetypeStorage")
            .field("component_ids", component_ids)
            .finish_non_exhaustive()
    }
}

type ErasedComponents<T> = BTreeMap<ComponentId, (Layout, T)>;

type ErasedField = Box<[u8]>;
type ErasedFieldRef<'a> = &'a [u8];
type ErasedFieldRefMut<'a> = &'a mut [u8];

trait ErasedStorage {
    fn entities(&self) -> &[Entity];

    #[track_caller]
    fn insert(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
        fields: ErasedComponents<ErasedField>,
    ) -> Option<ErasedComponents<ErasedField>>;

    #[track_caller]
    fn remove(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
    ) -> Option<ErasedComponents<ErasedField>>;

    #[track_caller]
    fn get(
        &self,
        components: &mut ComponentRegistry,
        entity: Entity,
    ) -> Option<ErasedComponents<ErasedFieldRef<'_>>>;

    #[track_caller]
    fn get_mut(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
    ) -> Option<ErasedComponents<ErasedFieldRefMut<'_>>>;
}

impl<B> ErasedStorage for SparseSet<B>
where
    B: Bundle,
{
    #[inline]
    fn entities(&self) -> &[Entity] {
        self.as_keys_slice()
    }

    #[inline]
    fn insert(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
        fields: ErasedComponents<ErasedField>,
    ) -> Option<ErasedComponents<ErasedField>> {
        let value = from_erased_fields::<B>(components, self.context(), fields);
        let value = SparseSet::insert(self, entity, value)?;
        let fields = into_erased_fields::<B>(components, self.context(), value);
        Some(fields)
    }

    #[inline]
    fn remove(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
    ) -> Option<ErasedComponents<ErasedField>> {
        let value = SparseSet::remove(self, entity)?;
        let fields = into_erased_fields::<B>(components, self.context(), value);
        Some(fields)
    }

    #[inline]
    fn get(
        &self,
        components: &mut ComponentRegistry,
        entity: Entity,
    ) -> Option<ErasedComponents<ErasedFieldRef<'_>>> {
        let refs = SparseSet::get(self, entity)?;
        let refs = into_erased_field_refs::<B>(components, self.context(), refs);
        Some(refs)
    }

    #[inline]
    fn get_mut(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
    ) -> Option<ErasedComponents<ErasedFieldRefMut<'_>>> {
        let (context, refs) = self.as_mut_view().into_get_mut_with_context(entity);
        let refs = into_erased_field_refs_mut::<B>(components, context, refs?);
        Some(refs)
    }
}

#[allow(unsafe_code)]
#[inline]
fn from_erased_fields<B>(
    components: &mut ComponentRegistry,
    context: &B::Context,
    mut fields: ErasedComponents<ErasedField>,
) -> B
where
    B: Bundle,
{
    let len = fields.len();
    let fields: Box<[_]> = B::component_ids(context, components)
        .expect("components of the bundle should be unique")
        .into_iter()
        .map(|id| {
            fields
                .remove(&id)
                .expect("field with given component id should present")
        })
        .collect();
    assert_eq!(fields.len(), len);

    let erased_context = ErasedSoaContext::<B::Fields>::new(
        fields.iter().map(|(field_layout, _)| field_layout),
        None,
    );
    let erased_value = ErasedSoa::<B::Fields>::new(
        &erased_context,
        fields.iter().map(|(_, field)| field.as_ref()),
    );
    unsafe { erased_value.into::<B>(context) }
}

#[inline]
fn into_erased_fields<B>(
    components: &mut ComponentRegistry,
    context: &B::Context,
    value: B,
) -> ErasedComponents<ErasedField>
where
    B: Bundle,
{
    let component_ids = B::component_ids(context, components)
        .expect("components of the bundle should be unique")
        .into_iter();

    let erased_context = ErasedSoaContext::<B::Fields>::new(B::field_layouts(context), None);
    let erased_value = ErasedSoa::from(context, value);

    component_ids
        .zip(erased_value.into_fields(&erased_context))
        .collect()
}

#[allow(unsafe_code)]
#[inline]
fn from_erased_field_refs<'a, B>(
    components: &mut ComponentRegistry,
    context: &B::Context,
    mut fields: ErasedComponents<ErasedFieldRef<'a>>,
) -> B::Refs<'a>
where
    B: Bundle,
{
    let fields: Box<[_]> = B::component_ids(context, components)
        .expect("components of the bundle should be unique")
        .into_iter()
        .map(|id| {
            fields
                .remove(&id)
                .expect("field with given component id should present")
        })
        .collect();

    let erased_context = ErasedSoaContext::<B::Fields>::new(
        fields.iter().map(|(field_layout, _)| field_layout),
        None,
    );
    let erased_refs = ErasedSoaRefs::<B::Fields>::new(
        &erased_context,
        fields
            .into_vec()
            .into_iter()
            .map(|(_, field)| field.as_ref()),
    );
    unsafe { erased_refs.into::<B>(context) }
}

#[inline]
fn into_erased_field_refs<'a, B>(
    components: &mut ComponentRegistry,
    context: &B::Context,
    refs: B::Refs<'a>,
) -> ErasedComponents<ErasedFieldRef<'a>>
where
    B: Bundle,
{
    let component_ids: Box<[ComponentId]> = B::component_ids(context, components)
        .expect("components of the bundle should be unique")
        .into_iter()
        .collect();

    let erased_refs = ErasedSoaRefs::from::<B>(context, refs);
    assert_eq!(component_ids.len(), erased_refs.as_ref().len());

    iter::zip(component_ids, erased_refs).collect()
}

#[allow(unsafe_code)]
#[inline]
fn from_erased_field_refs_mut<'a, B>(
    components: &mut ComponentRegistry,
    context: &B::Context,
    mut fields: ErasedComponents<ErasedFieldRefMut<'a>>,
) -> B::RefsMut<'a>
where
    B: Bundle,
{
    let fields: Box<[_]> = B::component_ids(context, components)
        .expect("components of the bundle should be unique")
        .into_iter()
        .map(|id| {
            fields
                .remove(&id)
                .expect("field with given component id should present")
        })
        .collect();

    let erased_context = ErasedSoaContext::<B::Fields>::new(
        fields.iter().map(|(field_layout, _)| field_layout),
        None,
    );
    let erased_refs = ErasedSoaRefsMut::<B::Fields>::new(
        &erased_context,
        fields
            .into_vec()
            .into_iter()
            .map(|(_, field)| field.as_mut()),
    );
    unsafe { erased_refs.into::<B>(context) }
}

#[inline]
fn into_erased_field_refs_mut<'a, B>(
    components: &mut ComponentRegistry,
    context: &B::Context,
    refs: B::RefsMut<'a>,
) -> ErasedComponents<ErasedFieldRefMut<'a>>
where
    B: Bundle,
{
    let component_ids: Box<[ComponentId]> = B::component_ids(context, components)
        .expect("components of the bundle should be unique")
        .into_iter()
        .collect();

    let erased_refs = ErasedSoaRefsMut::from::<B>(context, refs);
    assert_eq!(component_ids.len(), erased_refs.as_ref().len());

    iter::zip(component_ids, erased_refs).collect()
}

#[cfg(test)]
mod tests {
    use crate::{component::Component, entity::EntityRegistry};

    use super::*;

    #[test]
    fn storage_unit() {
        let mut components = ComponentRegistry::new();
        let mut storage = ArchetypeStorage::of::<()>(&mut components, ())
            .expect("creation of storage for empty archetype should succeed");
        assert_eq!(storage.entities(), []);

        let mut entities = EntityRegistry::new();
        let entity = entities.spawn();

        let value = storage
            .insert::<()>(&mut components, &(), entity, ())
            .expect("archetype storage should store unit");
        assert_eq!(value, None);
        assert_eq!(storage.entities(), [entity]);

        let refs = storage
            .get::<()>(&mut components, &(), entity)
            .expect("components by given entity should exist");
        assert_eq!(refs, Some(&()));
        assert_eq!(storage.entities(), [entity]);

        let value = storage
            .remove::<()>(&mut components, &(), entity)
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
        value: u16,
    }

    impl Component for Position {}
    impl Component for Mass {}

    #[test]
    fn storage_tuple() {
        let mut components = ComponentRegistry::new();

        let error = ArchetypeStorage::of::<(Position, Position)>(&mut components, ())
            .expect_err("creation of storage for bundle `(Position, Position)` should fail");
        assert_eq!(
            error.component_id(),
            components.register_component::<Position>(),
        );

        let mut storage = ArchetypeStorage::of::<(Position, Mass)>(&mut components, ())
            .expect("creation of storage for bundle `(Position, Mass)` should succeed");
        assert_eq!(storage.entities(), []);

        let mut entities = EntityRegistry::new();
        let entity = entities.spawn();

        let mut position = Position {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let IncompatibleBundleValueError { value, reason } = storage
            .insert::<(Position,)>(&mut components, &(), entity, (position,))
            .expect_err("insertion of just `Position` should fail");
        assert_eq!(value, (position,));
        assert_eq!(reason, TooFewComponentsError.into());

        let mut mass = Mass { value: 4 };
        let IncompatibleBundleValueError { value, reason } = storage
            .insert::<(Position, Mass, ())>(&mut components, &(), entity, (position, mass, ()))
            .expect_err("insertion of `Position`, `Mass` and `()` should fail");
        assert_eq!(value, (position, mass, ()));
        assert_eq!(
            reason,
            ExclusiveComponentError::new(components.register_component::<()>()).into(),
        );

        let value = storage
            .insert::<(Mass, Position)>(&mut components, &(), entity, (mass, position))
            .expect("insertion of `Mass` and `Position` should succeed");
        assert_eq!(value, None);
        assert_eq!(storage.entities(), [entity]);

        let refs = storage
            .get::<(Position,)>(&mut components, &(), entity)
            .expect("retrieval of just `Position` should succeed");
        assert_eq!(refs, Some((&position,)));
        assert_eq!(storage.entities(), [entity]);

        let error = storage
            .get::<(Position, Mass, ())>(&mut components, &(), entity)
            .expect_err("retrieval of `Position`, `Mass` and `()` should fail");
        assert_eq!(
            error,
            ExclusiveComponentError::new(components.register_component::<()>()).into(),
        );

        let refs = storage
            .get::<(Mass, Position)>(&mut components, &(), entity)
            .expect("retrieval of `Mass` and `Position` should succeed");
        assert_eq!(refs, Some((&mass, &position)));
        assert_eq!(storage.entities(), [entity]);

        let refs_mut = storage
            .get_mut::<(Position,)>(&mut components, &(), entity)
            .expect("retrieval of just `Position` should succeed");
        assert_eq!(refs_mut, Some((&mut position,)));
        assert_eq!(storage.entities(), [entity]);

        let error = storage
            .get_mut::<(Position, Mass, ())>(&mut components, &(), entity)
            .expect_err("retrieval of `Position`, `Mass` and `()` should fail");
        assert_eq!(
            error,
            ExclusiveComponentError::new(components.register_component::<()>()).into(),
        );

        let refs_mut = storage
            .get_mut::<(Mass, Position)>(&mut components, &(), entity)
            .expect("retrieval of `Mass` and `Position` should succeed");
        assert_eq!(refs_mut, Some((&mut mass, &mut position)));
        assert_eq!(storage.entities(), [entity]);

        let error = storage
            .remove::<(Position,)>(&mut components, &(), entity)
            .expect_err("removal of just `Position` should fail");
        assert_eq!(error, TooFewComponentsError.into());

        let error = storage
            .remove::<(Position, Mass, ())>(&mut components, &(), entity)
            .expect_err("removal of `Position`, `Mass` and `()` should fail");
        assert_eq!(
            error,
            ExclusiveComponentError::new(components.register_component::<()>()).into(),
        );

        let value = storage
            .remove::<(Mass, Position)>(&mut components, &(), entity)
            .expect("removal of `Mass` and `Position` should succeed");
        assert_eq!(value, Some((mass, position)));
        assert_eq!(storage.entities(), []);
    }
}
