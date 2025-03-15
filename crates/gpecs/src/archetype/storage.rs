use std::{
    alloc::Layout,
    borrow::Borrow,
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Debug},
};

use gpecs_sparse::set::EpochSparseSet;

use crate::{
    bundle::{error::DuplicateComponentError, Bundle},
    component::registry::{ComponentId, ComponentRegistry},
    entity::Entity,
    soa::erased::{ErasedSoa, ErasedSoaRefs, ErasedSoaRefsMut},
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
        let mut target_component_ids = match B::component_ids(context, components) {
            Ok(target_component_ids) => target_component_ids
                .into_iter()
                .inspect(|_| target_component_ids_count += 1),
            Err(error) => {
                let reason = error.into();
                return Err(IncompatibleBundleValueError { value, reason });
            }
        };
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

fn validate_component<B>(components: &mut ComponentRegistry, id: ComponentId, layout: B)
where
    B: Borrow<Layout>,
{
    let info = components
        .get_info(id)
        .expect("component info should present");
    assert_eq!(info.layout(), *layout.borrow());
}

#[allow(unsafe_code)]
#[inline]
fn from_erased_fields<B>(
    components: &mut ComponentRegistry,
    context: &B::Context,
    fields: ErasedComponents<ErasedField>,
) -> B
where
    B: Bundle,
{
    let fields = B::component_ids(context, components)
        .expect("components of the bundle should be unique")
        .into_iter()
        .zip(B::field_layouts(context))
        .map(|(id, layout)| {
            validate_component(components, id, layout);
            fields
                .get(&id)
                .expect("field with given component id should present")
        });

    let erased_value = ErasedSoa::<B::Fields>::new(
        fields.map(|(field_layout, field)| (*field_layout, field.as_ref())),
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
        .into_iter()
        .zip(B::field_layouts(context))
        .map(|(id, layout)| {
            validate_component(components, id, layout);
            id
        });

    let erased_value = ErasedSoa::from(context, value);
    component_ids.zip(erased_value.into_fields()).collect()
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
    let refs = B::component_ids(context, components)
        .expect("components of the bundle should be unique")
        .into_iter()
        .zip(B::field_layouts(context))
        .map(|(id, layout)| {
            validate_component(components, id, layout);
            fields
                .remove(&id)
                .expect("field with given component id should present")
        });

    let erased_refs = ErasedSoaRefs::<B::Fields>::new(refs);
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
    let component_ids = B::component_ids(context, components)
        .expect("components of the bundle should be unique")
        .into_iter()
        .zip(B::field_layouts(context))
        .map(|(id, layout)| {
            validate_component(components, id, layout);
            id
        });

    let erased_refs = ErasedSoaRefs::from::<B>(context, refs);
    component_ids.zip(erased_refs).collect()
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
    let fields = B::component_ids(context, components)
        .expect("components of the bundle should be unique")
        .into_iter()
        .zip(B::field_layouts(context))
        .map(|(id, layout)| {
            validate_component(components, id, layout);
            fields
                .remove(&id)
                .expect("field with given component id should present")
        });

    let erased_refs = ErasedSoaRefsMut::<B::Fields>::new(fields);
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
    let component_ids = B::component_ids(context, components)
        .expect("components of the bundle should be unique")
        .into_iter()
        .zip(B::field_layouts(context))
        .map(|(id, layout)| {
            validate_component(components, id, layout);
            id
        });

    let erased_refs = ErasedSoaRefsMut::from::<B>(context, refs);
    component_ids.zip(erased_refs).collect()
}
