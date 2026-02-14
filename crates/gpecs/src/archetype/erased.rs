use std::{iter::zip, mem::MaybeUninit};

use gpecs_soa_erased::{
    erased::{BoxedErasedSoa, ErasedSoaRefsMut},
    field::{
        BoxedErasedField, ErasedField, ErasedFieldRef, ErasedFieldRefMut, ErasedFieldSlice,
        ErasedFieldSliceMut,
    },
    slice_item_ptr::CoreSliceItemPtrs,
};

use crate::{
    bundle::Bundle,
    component::registry::{ComponentId, ComponentRegistry, DropFn},
    hash::IndexMap,
    soa::{
        field::{FieldDescriptor, FieldDescriptors},
        traits::{
            AllocSoa, RawSoa, RawSoaContext, Refs, RefsMut, Slices, SlicesMut, Soa, SoaContext,
            SoaRead, SoaWrite,
        },
    },
};

// TODO: convert this whole very unsafe code into some type which implements `Soa` trait & provides its guarantees

pub type ErasedComponents<T> = IndexMap<ComponentId, T>;

pub type ErasedBundle = BoxedErasedSoa<CoreSliceItemPtrs<MaybeUninit<u8>>>;
pub type ErasedBundleRef<'a, D> = ErasedSoaRefsMut<'a, D, *mut MaybeUninit<u8>>;

pub type ErasedComponent = BoxedErasedField<CoreSliceItemPtrs<MaybeUninit<u8>>>;
pub type ErasedComponentRef<'a> = ErasedFieldRef<'a, *const MaybeUninit<u8>>;
pub type ErasedComponentRefMut<'a> = ErasedFieldRefMut<'a, *mut MaybeUninit<u8>>;
pub type ErasedComponentSlice<'a> = ErasedFieldSlice<'a, *const MaybeUninit<u8>>;
pub type ErasedComponentSliceMut<'a> = ErasedFieldSliceMut<'a, *mut MaybeUninit<u8>>;

#[cold]
#[track_caller]
#[inline(never)]
pub fn get_component_info_fail(component_id: ComponentId) -> ! {
    panic!("info of {component_id} should be present")
}

#[inline]
#[track_caller]
fn validate_component<D>(components: &ComponentRegistry, id: ComponentId, desc: D)
where
    D: AsRef<FieldDescriptor>,
{
    let info = components
        .get_component_info(id)
        .unwrap_or_else(|| get_component_info_fail(id));
    assert_eq!(info.descriptor().layout(), desc.as_ref().layout());
}

#[inline]
#[track_caller]
pub fn validate_components<'a, 'components, 'ctx, T, I>(
    components: &'components ComponentRegistry,
    context: &'ctx T::Context,
    component_ids: I,
) -> impl Iterator<Item = ComponentId> + use<'components, 'ctx, T, I>
where
    T: RawSoa + Soa<'a>,
    T::Context: FieldDescriptors<'ctx>,
    I: IntoIterator<Item = ComponentId>,
{
    zip(component_ids, context.field_descriptors())
        .inspect(|(id, desc)| validate_component(components, *id, desc))
        .map(|(id, _)| id)
}

#[inline]
#[track_caller]
fn reorder_fields<'a, 'components, 'ctx, T, I, F>(
    components: &'components ComponentRegistry,
    context: &'ctx T::Context,
    component_ids: I,
    mut fields: ErasedComponents<F>,
) -> impl Iterator<Item = F> + use<'components, 'ctx, T, I, F>
where
    T: RawSoa + Soa<'a>,
    T::Context: FieldDescriptors<'ctx>,
    I: IntoIterator<Item = ComponentId>,
{
    #[cold]
    #[track_caller]
    #[inline(never)]
    fn remove_field_fail(component_id: ComponentId) -> ! {
        panic!("field of {component_id} should be present")
    }

    zip(component_ids, context.field_descriptors())
        .inspect(|(id, desc)| validate_component(components, *id, desc))
        .map(move |(id, _)| {
            fields
                .swap_remove(&id)
                .unwrap_or_else(|| remove_field_fail(id))
        })
}

#[inline]
pub unsafe fn from_erased_fields<'a, T>(
    components: &ComponentRegistry,
    context: &T::Context,
    component_ids: impl IntoIterator<Item = ComponentId>,
    fields: ErasedComponents<ErasedComponent>,
) -> T
where
    T: AllocSoa + Soa<'a> + SoaRead,
{
    let fields_with_descriptors =
        reorder_fields::<T, _, _>(components, context, component_ids, fields)
            .map(ErasedField::into_parts);
    let erased_value = ErasedBundle::try_from_fields_with_descriptors(fields_with_descriptors)
        .expect("all the fields should be valid");
    unsafe { erased_value.try_into::<T>(context) }.expect("all the fields should be valid")
}

#[inline]
pub fn into_erased_fields<'a, T>(
    components: &ComponentRegistry,
    context: &T::Context,
    component_ids: impl IntoIterator<Item = ComponentId>,
    value: T,
) -> ErasedComponents<ErasedComponent>
where
    T: AllocSoa + Soa<'a> + SoaWrite,
{
    let erased_value = BoxedErasedSoa::try_from(context, value)
        .unwrap()
        .into_fields()
        .collect::<Result<Box<[_]>, _>>()
        .unwrap();
    validate_components::<T, _>(components, context, component_ids)
        .zip(erased_value)
        .collect()
}

#[inline]
pub unsafe fn from_erased_refs<'a, B>(
    components: &ComponentRegistry,
    fields: ErasedComponents<ErasedComponentRef<'a>>,
) -> Refs<'static, 'a, B>
where
    B: Bundle,
{
    let iter = fields
        .into_iter()
        .map(|(component_id, field)| (component_id, field.as_field_ptr()));
    let ptrs = B::ptrs_from_erased(components, iter)
        .expect("all the components should be present in the right order");
    unsafe { B::CONTEXT.ptrs_to_refs(ptrs) }
}

#[inline]
pub unsafe fn from_erased_refs_mut<'a, B>(
    components: &ComponentRegistry,
    fields: ErasedComponents<ErasedComponentRefMut<'a>>,
) -> RefsMut<'static, 'a, B>
where
    B: Bundle,
{
    let iter = fields
        .into_iter()
        .map(|(component_id, mut field)| (component_id, field.as_mut_field_ptr()));
    let ptrs = B::mut_ptrs_from_erased(components, iter)
        .expect("all the components should be present in the right order");
    unsafe { B::CONTEXT.mut_ptrs_to_mut_refs(ptrs) }
}

#[inline]
pub unsafe fn from_erased_slices<'a, B>(
    components: &ComponentRegistry,
    len: usize,
    fields: ErasedComponents<ErasedComponentSlice<'a>>,
) -> Slices<'static, 'a, B>
where
    B: Bundle,
{
    let iter = fields
        .into_iter()
        .map(|(component_id, slice)| (component_id, slice.as_field_ptr()));
    let ptrs = B::ptrs_from_erased(components, iter)
        .expect("all the components should be present in the right order");
    let slices = B::CONTEXT.slice_ptrs_from_raw_parts(ptrs, len);
    unsafe { B::CONTEXT.slice_ptrs_to_slices(slices) }
}

#[inline]
pub unsafe fn from_erased_mut_slices<'a, B>(
    components: &ComponentRegistry,
    len: usize,
    fields: ErasedComponents<ErasedComponentSliceMut<'a>>,
) -> SlicesMut<'static, 'a, B>
where
    B: Bundle,
{
    let iter = fields
        .into_iter()
        .map(|(component_id, mut slice)| (component_id, slice.as_mut_field_ptr()));
    let ptrs = B::mut_ptrs_from_erased(components, iter)
        .expect("all the components should be present in the right order");
    let slices = B::CONTEXT.mut_slice_ptrs_from_raw_parts(ptrs, len);
    unsafe { B::CONTEXT.mut_slice_ptrs_to_mut_slices(slices) }
}

#[inline]
pub unsafe fn drop_erased_in_place<I, F>(fields: I)
where
    I: IntoIterator<Item = (F, Option<DropFn>)>,
    F: AsMut<[u8]>,
{
    fields.into_iter().for_each(|(mut field, drop_fn)| {
        let Some(drop_fn) = drop_fn else { return };
        unsafe { drop_fn(field.as_mut().as_mut_ptr()) }
    });
}
