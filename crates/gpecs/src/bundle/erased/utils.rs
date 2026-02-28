use std::borrow::Borrow;

use gpecs_soa_erased::BoxedErasedSoa;
use itertools::zip_eq;

use crate::{
    component::{
        erased::ErasedComponent,
        registry::{ComponentId, ComponentRegistry},
    },
    hash::IndexSet,
    soa::{
        field::{FieldDescriptor, FieldDescriptors},
        traits::{AllocSoa, RawSoa, Soa, SoaWrite},
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
fn assert_component<D>(components: &ComponentRegistry, id: ComponentId, desc: D)
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
fn validated_components<'a, 'components, 'ctx, T, I>(
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
        .inspect(|(id, desc)| assert_component(components, *id, desc))
        .map(|(id, _)| id)
}

#[inline]
#[track_caller]
pub fn reorder_fields<I, F>(mut fields: IndexSet<F>, order: I) -> impl Iterator<Item = F>
where
    I: IntoIterator<Item = ComponentId>,
    F: Borrow<ComponentId>,
{
    #[cold]
    #[track_caller]
    #[inline(never)]
    fn remove_field_fail(component_id: ComponentId) -> ! {
        panic!("field of {component_id} should be present")
    }

    order.into_iter().map(move |id| {
        fields
            .swap_take(&id)
            .unwrap_or_else(|| remove_field_fail(id))
    })
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
    let component_ids = validated_components::<T, _>(components, context, component_ids);
    let fields = BoxedErasedSoa::try_from::<T, W>(context, value)
        .expect("the value should be valid for the given context");

    zip_eq(fields, component_ids)
        .map(|(field, id)| {
            let field = field.expect("field should be created successfully");
            let info = components
                .get_component_info(id)
                .unwrap_or_else(|| get_component_info_fail(id));
            unsafe { ErasedComponent::from_parts(id, field, info.drop_fn()) }
        })
        .collect()
}
