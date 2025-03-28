use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Debug},
};

use gpecs_soa_erased::{
    align::Unaligned,
    erased::{ErasedSoa, ErasedSoaRefs, ErasedSoaRefsMut, ErasedSoaSlices, ErasedSoaSlicesMut},
    field::{
        ErasedField, ErasedFieldRef, ErasedFieldRefMut, ErasedFieldSlice, ErasedFieldSliceMut,
    },
};
use gpecs_sparse::set::EpochSparseSet;

use crate::{
    bundle::{error::DuplicateComponentError, Bundle},
    component::registry::{ComponentId, ComponentRegistry},
    entity::Entity,
    soa::traits::FieldDescriptor,
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
    pub fn components<B>(
        &self,
        components: &mut ComponentRegistry,
        context: &B::Context,
    ) -> Result<B::Slices<'_>, IncompatibleBundleError>
    where
        B: Bundle,
    {
        let Self {
            ref component_ids,
            erased_storage,
        } = self;

        let mut target_component_ids = B::component_ids(context, components)?.into_iter();
        if let Some(component_id) = target_component_ids.find(|id| !component_ids.contains(&id)) {
            return Err(ExclusiveComponentError { component_id }.into());
        }

        let (len, fields) = erased_storage.components(components);
        let slices = from_erased_field_slices::<B>(components, context, len, fields);
        Ok(slices)
    }

    #[inline]
    pub fn components_mut<B>(
        &mut self,
        components: &mut ComponentRegistry,
        context: &B::Context,
    ) -> Result<B::SlicesMut<'_>, IncompatibleBundleError>
    where
        B: Bundle,
    {
        let Self {
            ref component_ids,
            erased_storage,
        } = self;

        let mut target_component_ids = B::component_ids(context, components)?.into_iter();
        if let Some(component_id) = target_component_ids.find(|id| !component_ids.contains(&id)) {
            return Err(ExclusiveComponentError { component_id }.into());
        }

        let (len, fields) = erased_storage.components_mut(components);
        let slices = from_erased_field_slices_mut::<B>(components, context, len, fields);
        Ok(slices)
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
            ref component_ids,
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
            ref component_ids,
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
            ref component_ids,
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
            ref component_ids,
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

type ErasedComponents<T> = BTreeMap<ComponentId, T>;

trait ErasedStorage {
    fn entities(&self) -> &[Entity];

    fn components(
        &self,
        components: &mut ComponentRegistry,
    ) -> (usize, ErasedComponents<ErasedFieldSlice<'_>>);

    fn components_mut(
        &mut self,
        components: &mut ComponentRegistry,
    ) -> (usize, ErasedComponents<ErasedFieldSliceMut<'_>>);

    fn insert(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
        fields: ErasedComponents<ErasedField<Unaligned>>,
    ) -> Option<ErasedComponents<ErasedField<Unaligned>>>;

    fn remove(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
    ) -> Option<ErasedComponents<ErasedField<Unaligned>>>;

    fn get(
        &self,
        components: &mut ComponentRegistry,
        entity: Entity,
    ) -> Option<ErasedComponents<ErasedFieldRef<'_>>>;

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
    fn components(
        &self,
        components: &mut ComponentRegistry,
    ) -> (usize, ErasedComponents<ErasedFieldSlice<'_>>) {
        let (context, slices) = self.as_view().into_slices_with_context();
        into_erased_field_slices::<B>(components, context, slices)
    }

    #[inline]
    fn components_mut(
        &mut self,
        components: &mut ComponentRegistry,
    ) -> (usize, ErasedComponents<ErasedFieldSliceMut<'_>>) {
        let (context, slices) = self.as_mut_view().into_slices_with_context();
        into_erased_field_slices_mut::<B>(components, context, slices)
    }

    #[inline]
    fn insert(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
        fields: ErasedComponents<ErasedField<Unaligned>>,
    ) -> Option<ErasedComponents<ErasedField<Unaligned>>> {
        let value = from_erased_fields::<B>(components, self.context(), fields);
        let value = SparseSet::insert(self, entity, value).unwrap()?;
        let fields = into_erased_fields::<B>(components, self.context(), value);
        Some(fields)
    }

    #[inline]
    fn remove(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
    ) -> Option<ErasedComponents<ErasedField<Unaligned>>> {
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
        let (context, refs) = self.as_view().into_get_with_context(entity);
        let refs = into_erased_field_refs::<B>(components, context, refs?);
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

#[inline]
#[track_caller]
fn validate_component<D>(components: &mut ComponentRegistry, id: ComponentId, desc: D)
where
    D: AsRef<FieldDescriptor>,
{
    let info = components
        .get_info(id)
        .unwrap_or_else(|| panic!("info of component {id:?} should be present"));
    assert_eq!(info.descriptor().layout(), desc.as_ref().layout());
}

#[inline]
#[track_caller]
fn validate_components<'components, 'context, B>(
    components: &'components mut ComponentRegistry,
    context: &'context B::Context,
) -> impl Iterator<Item = ComponentId> + use<'components, 'context, B>
where
    B: Bundle,
{
    B::component_ids(context, components)
        .expect("components of the bundle should be unique")
        .into_iter()
        .zip(B::field_descriptors(context))
        .inspect(|(id, desc)| validate_component(components, *id, desc))
        .map(|(id, _)| id)
}

#[inline]
#[track_caller]
fn reorder_fields<'components, 'context, B, F>(
    components: &'components mut ComponentRegistry,
    context: &'context B::Context,
    mut fields: ErasedComponents<F>,
) -> impl Iterator<Item = F> + use<'components, 'context, B, F>
where
    B: Bundle,
{
    B::component_ids(context, components)
        .expect("components of the bundle should be unique")
        .into_iter()
        .zip(B::field_descriptors(context))
        .inspect(|(id, desc)| validate_component(components, *id, desc))
        .map(move |(id, _)| {
            fields
                .remove(&id)
                .unwrap_or_else(|| panic!("field of component {id:?} should be present"))
        })
}

#[allow(unsafe_code)]
#[inline]
fn from_erased_fields<B>(
    components: &mut ComponentRegistry,
    context: &B::Context,
    fields: ErasedComponents<ErasedField<Unaligned>>,
) -> B
where
    B: Bundle,
{
    let fields = reorder_fields::<B, _>(components, context, fields).map(ErasedField::into_parts);
    let erased_value = ErasedSoa::<B::Fields>::new(fields).expect("all the fields should be valid");
    unsafe { erased_value.into::<B>(context) }.expect("all the fields should be valid")
}

#[inline]
fn into_erased_fields<B>(
    components: &mut ComponentRegistry,
    context: &B::Context,
    value: B,
) -> ErasedComponents<ErasedField<Unaligned>>
where
    B: Bundle,
{
    let erased_value = ErasedSoa::from(context, value)
        .expect("all the fields should be valid")
        .into_fields();
    validate_components::<B>(components, context)
        .zip(erased_value)
        .map(|(component_id, field)| (component_id, field.into_unaligned()))
        .collect()
}

#[allow(unsafe_code)]
#[inline]
fn from_erased_field_refs<'a, B>(
    components: &mut ComponentRegistry,
    context: &B::Context,
    fields: ErasedComponents<ErasedFieldRef<'a>>,
) -> B::Refs<'a>
where
    B: Bundle,
{
    let refs = reorder_fields::<B, _>(components, context, fields);
    let erased_refs =
        ErasedSoaRefs::<B::Fields>::new(refs).expect("all the fields should be valid");
    unsafe { erased_refs.into::<B>(context) }.expect("all the fields should be valid")
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
    let erased_refs = ErasedSoaRefs::from::<B>(context, refs)
        .expect("all the fields should be valid")
        .into_field_refs();
    validate_components::<B>(components, context)
        .zip(erased_refs)
        .collect()
}

#[allow(unsafe_code)]
#[inline]
fn from_erased_field_refs_mut<'a, B>(
    components: &mut ComponentRegistry,
    context: &B::Context,
    fields: ErasedComponents<ErasedFieldRefMut<'a>>,
) -> B::RefsMut<'a>
where
    B: Bundle,
{
    let refs = reorder_fields::<B, _>(components, context, fields);
    let erased_refs =
        ErasedSoaRefsMut::<B::Fields>::new(refs).expect("all the fields should be valid");
    unsafe { erased_refs.into::<B>(context) }.expect("all the fields should be valid")
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
    let erased_refs = ErasedSoaRefsMut::from::<B>(context, refs)
        .expect("all the fields should be valid")
        .into_field_refs();
    validate_components::<B>(components, context)
        .zip(erased_refs)
        .collect()
}

#[allow(unsafe_code)]
#[inline]
fn from_erased_field_slices<'a, B>(
    components: &mut ComponentRegistry,
    context: &B::Context,
    len: usize,
    fields: ErasedComponents<ErasedFieldSlice<'a>>,
) -> B::Slices<'a>
where
    B: Bundle,
{
    let slices = reorder_fields::<B, _>(components, context, fields);
    let erased_slices =
        ErasedSoaSlices::<B::Fields>::new(len, slices).expect("all the fields should be valid");
    unsafe { erased_slices.into::<B>(context) }.expect("all the fields should be valid")
}

#[inline]
fn into_erased_field_slices<'a, B>(
    components: &mut ComponentRegistry,
    context: &B::Context,
    slices: B::Slices<'a>,
) -> (usize, ErasedComponents<ErasedFieldSlice<'a>>)
where
    B: Bundle,
{
    let erased_slices =
        ErasedSoaSlices::from::<B>(context, slices).expect("all the fields should be valid");
    let len = erased_slices.len();
    let fields = validate_components::<B>(components, context)
        .zip(erased_slices.into_field_slices())
        .collect();
    (len, fields)
}

#[allow(unsafe_code)]
#[inline]
fn from_erased_field_slices_mut<'a, B>(
    components: &mut ComponentRegistry,
    context: &B::Context,
    len: usize,
    fields: ErasedComponents<ErasedFieldSliceMut<'a>>,
) -> B::SlicesMut<'a>
where
    B: Bundle,
{
    let slices = reorder_fields::<B, _>(components, context, fields);
    let erased_slices =
        ErasedSoaSlicesMut::<B::Fields>::new(len, slices).expect("all the fields should be valid");
    unsafe { erased_slices.into::<B>(context) }.expect("all the fields should be valid")
}

#[inline]
fn into_erased_field_slices_mut<'a, B>(
    components: &mut ComponentRegistry,
    context: &B::Context,
    slices: B::SlicesMut<'a>,
) -> (usize, ErasedComponents<ErasedFieldSliceMut<'a>>)
where
    B: Bundle,
{
    let erased_slices =
        ErasedSoaSlicesMut::from::<B>(context, slices).expect("all the fields should be valid");
    let len = erased_slices.len();
    let fields = validate_components::<B>(components, context)
        .zip(erased_slices.into_field_slices())
        .collect();
    (len, fields)
}
