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
    soa::erased::{
        ErasedSoa, ErasedSoaRefs, ErasedSoaRefsMut, ErasedSoaSlices, ErasedSoaSlicesMut,
    },
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
            component_ids,
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
            component_ids,
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
type ErasedFieldSlice<'a> = &'a [u8];
type ErasedFieldSliceMut<'a> = &'a mut [u8];

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
        fields: ErasedComponents<ErasedField>,
    ) -> Option<ErasedComponents<ErasedField>>;

    fn remove(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
    ) -> Option<ErasedComponents<ErasedField>>;

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
fn validate_component<B>(components: &mut ComponentRegistry, id: ComponentId, layout: B)
where
    B: Borrow<Layout>,
{
    let info = components
        .get_info(id)
        .ok_or_else(|| format!("info of component {id:?} should be present"))
        .unwrap();
    assert_eq!(info.layout(), *layout.borrow());
}

#[inline]
#[track_caller]
fn validate_components<'components, 'context, B>(
    components: &'components mut ComponentRegistry,
    context: &'context B::Context,
) -> impl Iterator<Item = ComponentId> + 'context
where
    'components: 'context,
    B: Bundle,
{
    B::component_ids(context, components)
        .expect("components of the bundle should be unique")
        .into_iter()
        .zip(B::field_layouts(context))
        .map(|(id, layout)| {
            validate_component(components, id, layout);
            id
        })
}

#[inline]
#[track_caller]
fn reorder_fields<'components, 'context, B, F>(
    components: &'components mut ComponentRegistry,
    context: &'context B::Context,
    mut fields: ErasedComponents<F>,
) -> impl Iterator<Item = (Layout, F)> + 'context
where
    'components: 'context,
    B: Bundle,
    F: 'context,
{
    B::component_ids(context, components)
        .expect("components of the bundle should be unique")
        .into_iter()
        .zip(B::field_layouts(context))
        .map(move |(id, layout)| {
            validate_component(components, id, layout);
            fields
                .remove(&id)
                .ok_or_else(|| format!("field of component {id:?} should be present"))
                .unwrap()
        })
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
    let fields = reorder_fields::<B, _>(components, context, fields);
    let erased_value = ErasedSoa::<B::Fields>::new(fields);
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
    let erased_value = ErasedSoa::from(context, value);
    validate_components::<B>(components, context)
        .zip(erased_value.into_fields())
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
    let erased_refs = ErasedSoaRefs::from::<B>(context, refs);
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
    let erased_refs = ErasedSoaRefsMut::<B::Fields>::new(refs);
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
    let erased_refs = ErasedSoaRefsMut::from::<B>(context, refs);
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
    let erased_slices = ErasedSoaSlices::<B::Fields>::new(len, slices);
    unsafe { erased_slices.into::<B>(context) }
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
    let erased_slices = ErasedSoaSlices::from::<B>(context, slices);

    let len = erased_slices.len();
    let fields = validate_components::<B>(components, context)
        .zip(erased_slices)
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
    let erased_slices = ErasedSoaSlicesMut::<B::Fields>::new(len, slices);
    unsafe { erased_slices.into::<B>(context) }
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
    let erased_slices = ErasedSoaSlicesMut::from::<B>(context, slices);

    let len = erased_slices.len();
    let fields = validate_components::<B>(components, context)
        .zip(erased_slices)
        .collect();
    (len, fields)
}
