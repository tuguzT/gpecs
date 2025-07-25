use gpecs_soa_erased::{
    erased::ErasedSoa,
    field::{
        ErasedField, ErasedFieldRef, ErasedFieldRefMut, ErasedFieldSlice, ErasedFieldSliceMut,
    },
};
use indexmap::IndexMap;

use crate::{
    bundle::Bundle,
    component::registry::{ComponentId, ComponentRegistry, DropFn},
    soa::traits::{FieldDescriptor, Soa},
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
pub fn validate_components<'components, 'context, T, I>(
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
pub unsafe fn from_erased_refs<'a, B>(
    components: &ComponentRegistry,
    fields: ErasedComponents<ErasedFieldRef<'a>>,
) -> B::Refs<'static, 'a>
where
    B: Bundle,
{
    let iter = fields
        .into_iter()
        .map(|(component_id, r#ref)| (component_id, r#ref.as_field_ptr().cast_mut()));
    let ptrs = unsafe { B::ptrs_from_iter(components, iter) };
    let ptrs = B::ptrs_cast_const(B::CONTEXT, ptrs);
    unsafe { B::ptrs_to_refs(B::CONTEXT, ptrs) }
}

#[inline]
#[allow(unsafe_code)]
pub unsafe fn from_erased_refs_mut<'a, B>(
    components: &ComponentRegistry,
    fields: ErasedComponents<ErasedFieldRefMut<'a>>,
) -> B::RefsMut<'static, 'a>
where
    B: Bundle,
{
    let iter = fields
        .into_iter()
        .map(|(component_id, mut r#ref)| (component_id, r#ref.as_field_mut_ptr()));
    let ptrs = unsafe { B::ptrs_from_iter(components, iter) };
    unsafe { B::ptrs_to_refs_mut(B::CONTEXT, ptrs) }
}

#[inline]
#[allow(unsafe_code)]
pub unsafe fn from_erased_slices<'a, B>(
    components: &ComponentRegistry,
    len: usize,
    fields: ErasedComponents<ErasedFieldSlice<'a>>,
) -> B::Slices<'static, 'a>
where
    B: Bundle,
{
    let iter = fields
        .into_iter()
        .map(|(component_id, slice)| (component_id, slice.as_field_ptr().cast_mut()));
    let ptrs = unsafe { B::ptrs_from_iter(components, iter) };
    let ptrs = B::ptrs_cast_const(B::CONTEXT, ptrs);
    let slices = B::slices_from_raw_parts(B::CONTEXT, ptrs, len);
    unsafe { B::slice_ptrs_to_slices(B::CONTEXT, slices) }
}

#[inline]
#[allow(unsafe_code)]
pub unsafe fn from_erased_slices_mut<'a, B>(
    components: &ComponentRegistry,
    len: usize,
    fields: ErasedComponents<ErasedFieldSliceMut<'a>>,
) -> B::SlicesMut<'static, 'a>
where
    B: Bundle,
{
    let iter = fields
        .into_iter()
        .map(|(component_id, mut slice)| (component_id, slice.as_field_mut_ptr()));
    let ptrs = unsafe { B::ptrs_from_iter(components, iter) };
    let slices = B::slices_from_raw_parts_mut(B::CONTEXT, ptrs, len);
    unsafe { B::slice_mut_ptrs_to_slices(B::CONTEXT, slices) }
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
