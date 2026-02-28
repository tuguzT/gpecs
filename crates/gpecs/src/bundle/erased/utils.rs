use std::{borrow::Borrow, mem::MaybeUninit};

use gpecs_soa_erased::{BoxedErasedSoa, ptr::slice::CoreSliceItemPtrs};
use itertools::{Itertools, zip_eq};

use crate::{
    component::{
        erased::ErasedComponent,
        registry::{ComponentId, ComponentRegistry},
    },
    hash::IndexSet,
    soa::{
        field::{FieldDescriptor, FieldDescriptors},
        traits::{AllocSoa, RawSoa, Soa, SoaRead, SoaWrite},
    },
};

// TODO: convert this whole very unsafe code into some type which implements `Soa` trait & provides its guarantees

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
    T: RawSoa + Soa<'a> + ?Sized,
    T::Context: FieldDescriptors<'ctx>,
    I: IntoIterator<Item = ComponentId>,
{
    zip_eq(component_ids, context.field_descriptors())
        .inspect(|(id, desc)| validate_component(components, *id, desc))
        .map(|(id, _)| id)
}

#[inline]
#[track_caller]
fn reorder_fields<'a, 'components, 'ctx, T, I, F>(
    components: &'components ComponentRegistry,
    context: &'ctx T::Context,
    component_ids: I,
    mut fields: IndexSet<F>,
) -> impl Iterator<Item = F> + use<'components, 'ctx, T, I, F>
where
    T: RawSoa + Soa<'a>,
    T::Context: FieldDescriptors<'ctx>,
    I: IntoIterator<Item = ComponentId>,
    F: Borrow<ComponentId>,
{
    #[cold]
    #[track_caller]
    #[inline(never)]
    fn remove_field_fail(component_id: ComponentId) -> ! {
        panic!("field of {component_id} should be present")
    }

    zip_eq(component_ids, context.field_descriptors())
        .inspect(|(id, desc)| validate_component(components, *id, desc))
        .map(move |(id, _)| {
            fields
                .swap_take(&id)
                .unwrap_or_else(|| remove_field_fail(id))
        })
}

#[inline]
pub unsafe fn from_erased_fields<'ctx, 'a, T, R>(
    components: &ComponentRegistry,
    context: &'ctx T::Context,
    component_ids: impl IntoIterator<Item = ComponentId>,
    fields: IndexSet<ErasedComponent>,
) -> R
where
    T: AllocSoa + Soa<'a> + SoaRead<'ctx, R>,
{
    type ErasedSoa = BoxedErasedSoa<CoreSliceItemPtrs<MaybeUninit<u8>>>;

    let fields_with_descriptors =
        reorder_fields::<T, _, _>(components, context, component_ids, fields).map(|field| {
            let (_, field, _) = field.into_parts();
            let (storage, layout) = field.into_parts();
            (storage, FieldDescriptor::new(layout))
        });
    let erased_value = ErasedSoa::try_from_fields_with_descriptors(fields_with_descriptors)
        .expect("all the fields should be valid");
    unsafe { erased_value.downcast::<T, R>(context) }.expect("all the fields should be valid")
}

#[inline]
pub unsafe fn into_erased_fields<'a, T, W>(
    components: &ComponentRegistry,
    context: &T::Context,
    component_ids: impl IntoIterator<Item = ComponentId>,
    value: W,
) -> IndexSet<ErasedComponent>
where
    T: AllocSoa + Soa<'a> + SoaWrite<W> + ?Sized,
{
    let erased_value = BoxedErasedSoa::try_from::<T, W>(context, value)
        .expect("the value should be valid for the given context");
    validate_components::<T, _>(components, context, component_ids)
        .zip_eq(erased_value.into_fields())
        .map(|(id, field)| {
            let field = field.expect("field should be created successfully");
            let info = components
                .get_component_info(id)
                .unwrap_or_else(|| get_component_info_fail(id));
            unsafe { ErasedComponent::from_parts(id, field, info.drop_fn()) }
        })
        .collect()
}
