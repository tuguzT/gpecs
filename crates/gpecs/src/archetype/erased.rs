use gpecs_soa_erased::{
    erased::{ErasedSoa, ErasedSoaRefs, ErasedSoaRefsMut, ErasedSoaSlices, ErasedSoaSlicesMut},
    field::{
        ErasedField, ErasedFieldRef, ErasedFieldRefMut, ErasedFieldSlice, ErasedFieldSliceMut,
    },
};
use indexmap::IndexMap;

use crate::{
    component::registry::{ComponentId, ComponentRegistry, DropFn},
    soa::{traits::FieldDescriptor, Soa},
};

pub type ErasedComponents<T> = IndexMap<ComponentId, T>;

#[cold]
#[track_caller]
#[inline(never)]
pub fn get_component_info_fail(component_id: &ComponentId) -> ! {
    panic!("info of component {component_id:?} should be present")
}

#[inline]
#[track_caller]
fn validate_component<D>(components: &ComponentRegistry, id: ComponentId, desc: D)
where
    D: AsRef<FieldDescriptor>,
{
    let info = components
        .get_component_info(id)
        .unwrap_or_else(|| get_component_info_fail(&id));
    assert_eq!(info.descriptor().layout(), desc.as_ref().layout());
}

#[inline]
#[track_caller]
fn validate_components<'components, 'context, T, I>(
    components: &'components ComponentRegistry,
    context: &'context T::Context,
    component_ids: I,
) -> impl Iterator<Item = ComponentId> + use<'components, 'context, T, I>
where
    T: Soa,
    I: IntoIterator<Item = ComponentId>,
{
    component_ids
        .into_iter()
        .zip(T::field_descriptors(context))
        .inspect(|(id, desc)| validate_component(components, *id, desc))
        .map(|(id, _)| id)
}

#[inline]
#[track_caller]
fn reorder_fields<'components, 'context, T, I, F>(
    components: &'components ComponentRegistry,
    context: &'context T::Context,
    component_ids: I,
    mut fields: ErasedComponents<F>,
) -> impl Iterator<Item = F> + use<'components, 'context, T, I, F>
where
    T: Soa,
    I: IntoIterator<Item = ComponentId>,
{
    #[cold]
    #[track_caller]
    #[inline(never)]
    fn remove_field_fail(component_id: &ComponentId) -> ! {
        panic!("field of component {component_id:?} should be present")
    }

    let remove_field = move |(id, _)| {
        fields
            .swap_remove(&id)
            .unwrap_or_else(|| remove_field_fail(&id))
    };
    component_ids
        .into_iter()
        .zip(T::field_descriptors(context))
        .inspect(|(id, desc)| validate_component(components, *id, desc))
        .map(remove_field)
}

#[inline]
#[allow(unsafe_code)]
pub unsafe fn from_erased_fields<T>(
    components: &ComponentRegistry,
    context: &T::Context,
    component_ids: impl IntoIterator<Item = ComponentId>,
    fields: ErasedComponents<ErasedField>,
) -> T
where
    T: Soa,
{
    let fields = reorder_fields::<T, _, _>(components, context, component_ids, fields)
        .map(ErasedField::into_parts);
    let erased_value = ErasedSoa::new(fields).expect("all the fields should be valid");
    unsafe { erased_value.into::<T>(context) }.expect("all the fields should be valid")
}

#[inline]
pub fn into_erased_fields<T>(
    components: &ComponentRegistry,
    context: &T::Context,
    component_ids: impl IntoIterator<Item = ComponentId>,
    value: T,
) -> ErasedComponents<ErasedField>
where
    T: Soa,
{
    let erased_value = ErasedSoa::from(context, value).into_fields();
    validate_components::<T, _>(components, context, component_ids)
        .zip(erased_value)
        .collect()
}

#[inline]
#[allow(unsafe_code)]
pub unsafe fn from_erased_refs<'a, T>(
    components: &ComponentRegistry,
    context: &T::Context,
    component_ids: impl IntoIterator<Item = ComponentId>,
    fields: ErasedComponents<ErasedFieldRef<'a>>,
) -> T::Refs<'a>
where
    T: Soa,
{
    let refs = reorder_fields::<T, _, _>(components, context, component_ids, fields);
    let erased_refs = ErasedSoaRefs::new(refs);
    unsafe { erased_refs.into::<T>(context) }.expect("all the fields should be valid")
}

#[inline]
pub fn into_erased_refs<'a, T>(
    components: &ComponentRegistry,
    context: &T::Context,
    component_ids: impl IntoIterator<Item = ComponentId>,
    refs: T::Refs<'a>,
) -> ErasedComponents<ErasedFieldRef<'a>>
where
    T: Soa,
{
    let erased_refs = ErasedSoaRefs::from::<T>(context, refs).into_field_refs();
    validate_components::<T, _>(components, context, component_ids)
        .zip(erased_refs)
        .collect()
}

#[inline]
#[allow(unsafe_code)]
pub unsafe fn from_erased_refs_mut<'a, T>(
    components: &ComponentRegistry,
    context: &T::Context,
    component_ids: impl IntoIterator<Item = ComponentId>,
    fields: ErasedComponents<ErasedFieldRefMut<'a>>,
) -> T::RefsMut<'a>
where
    T: Soa,
{
    let refs = reorder_fields::<T, _, _>(components, context, component_ids, fields);
    let erased_refs = ErasedSoaRefsMut::new(refs);
    unsafe { erased_refs.into::<T>(context) }.expect("all the fields should be valid")
}

#[inline]
pub fn into_erased_refs_mut<'a, T>(
    components: &ComponentRegistry,
    context: &T::Context,
    component_ids: impl IntoIterator<Item = ComponentId>,
    refs: T::RefsMut<'a>,
) -> ErasedComponents<ErasedFieldRefMut<'a>>
where
    T: Soa,
{
    let erased_refs = ErasedSoaRefsMut::from::<T>(context, refs).into_field_refs();
    validate_components::<T, _>(components, context, component_ids)
        .zip(erased_refs)
        .collect()
}

#[inline]
#[allow(unsafe_code)]
pub unsafe fn from_erased_slices<'a, T>(
    components: &ComponentRegistry,
    context: &T::Context,
    component_ids: impl IntoIterator<Item = ComponentId>,
    len: usize,
    fields: ErasedComponents<ErasedFieldSlice<'a>>,
) -> T::Slices<'a>
where
    T: Soa,
{
    let slices = reorder_fields::<T, _, _>(components, context, component_ids, fields);
    let erased_slices = ErasedSoaSlices::new(len, slices).expect("all the fields should be valid");
    unsafe { erased_slices.into::<T>(context) }.expect("all the fields should be valid")
}

#[inline]
pub fn into_erased_slices<'a, T>(
    components: &ComponentRegistry,
    context: &T::Context,
    component_ids: impl IntoIterator<Item = ComponentId>,
    slices: T::Slices<'a>,
) -> (usize, ErasedComponents<ErasedFieldSlice<'a>>)
where
    T: Soa,
{
    let erased_slices = ErasedSoaSlices::from::<T>(context, slices);
    let len = erased_slices.len();
    let fields = validate_components::<T, _>(components, context, component_ids)
        .zip(erased_slices.into_field_slices())
        .collect();
    (len, fields)
}

#[inline]
#[allow(unsafe_code)]
pub unsafe fn from_erased_slices_mut<'a, T>(
    components: &ComponentRegistry,
    context: &T::Context,
    component_ids: impl IntoIterator<Item = ComponentId>,
    len: usize,
    fields: ErasedComponents<ErasedFieldSliceMut<'a>>,
) -> T::SlicesMut<'a>
where
    T: Soa,
{
    let slices = reorder_fields::<T, _, _>(components, context, component_ids, fields);
    let erased_slices =
        ErasedSoaSlicesMut::new(len, slices).expect("all the fields should be valid");
    unsafe { erased_slices.into::<T>(context) }.expect("all the fields should be valid")
}

#[inline]
pub fn into_erased_slices_mut<'a, T>(
    components: &ComponentRegistry,
    context: &T::Context,
    component_ids: impl IntoIterator<Item = ComponentId>,
    slices: T::SlicesMut<'a>,
) -> (usize, ErasedComponents<ErasedFieldSliceMut<'a>>)
where
    T: Soa,
{
    let erased_slices = ErasedSoaSlicesMut::from::<T>(context, slices);
    let len = erased_slices.len();
    let fields = validate_components::<T, _>(components, context, component_ids)
        .zip(erased_slices.into_field_slices())
        .collect();
    (len, fields)
}

#[inline]
#[allow(unsafe_code)]
pub unsafe fn drop_erased_in_place<I, F>(fields: I)
where
    I: IntoIterator<Item = (F, Option<DropFn>)>,
    F: AsMut<[u8]>,
{
    fields.into_iter().for_each(|(mut field, drop_fn)| {
        let Some(drop_fn) = drop_fn else { return };
        unsafe { drop_fn(field.as_mut().as_mut_ptr()) }
    })
}
