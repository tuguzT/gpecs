use std::{borrow::Borrow, mem::MaybeUninit};

use gpecs_soa_erased::{BoxedErasedSoa, ErasedSoaMutRefs, ptr::slice::CoreSliceItemPtrs};
use itertools::{Itertools, zip_eq};

use crate::{
    bundle::Bundle,
    component::{
        erased::{
            ErasedComponent, ErasedComponentMutRef, ErasedComponentMutSlice, ErasedComponentRef,
            ErasedComponentSlice,
        },
        registry::{ComponentId, ComponentRegistry, DropFn},
    },
    hash::IndexSet,
    soa::{
        field::{FieldDescriptor, FieldDescriptors},
        traits::{
            AllocSoa, RawSoa, RawSoaContext, Refs, RefsMut, Slices, SlicesMut, Soa, SoaContext,
            SoaRead, SoaWrite,
        },
    },
};

// TODO: convert this whole very unsafe code into some type which implements `Soa` trait & provides its guarantees

pub type ErasedBundle = BoxedErasedSoa<CoreSliceItemPtrs<MaybeUninit<u8>>>;
pub type ErasedBundleRef<'a, D> = ErasedSoaMutRefs<'a, D, *mut MaybeUninit<u8>>;

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
pub unsafe fn from_erased_fields<'a, T>(
    components: &ComponentRegistry,
    context: &T::Context,
    component_ids: impl IntoIterator<Item = ComponentId>,
    fields: IndexSet<ErasedComponent>,
) -> T
where
    T: AllocSoa + Soa<'a> + SoaRead,
{
    let fields_with_descriptors =
        reorder_fields::<T, _, _>(components, context, component_ids, fields).map(|field| {
            let (_, field) = field.into_parts();
            let (storage, layout) = field.into_parts();
            (storage, FieldDescriptor::new(layout))
        });
    let erased_value = ErasedBundle::try_from_fields_with_descriptors(fields_with_descriptors)
        .expect("all the fields should be valid");
    unsafe { erased_value.downcast::<T>(context) }.expect("all the fields should be valid")
}

#[inline]
pub unsafe fn into_erased_fields<'a, T>(
    components: &ComponentRegistry,
    context: &T::Context,
    component_ids: impl IntoIterator<Item = ComponentId>,
    value: T,
) -> IndexSet<ErasedComponent>
where
    T: AllocSoa + Soa<'a> + SoaWrite,
{
    let erased_value = BoxedErasedSoa::try_from(context, value)
        .unwrap()
        .into_fields()
        .collect::<Result<Box<[_]>, _>>()
        .unwrap();
    validate_components::<T, _>(components, context, component_ids)
        .zip_eq(erased_value)
        .map(|(id, field)| unsafe { ErasedComponent::from_parts(id, field) })
        .collect()
}

#[inline]
pub unsafe fn from_erased_refs<'a, B>(
    components: &ComponentRegistry,
    fields: IndexSet<ErasedComponentRef<'a>>,
) -> Refs<'static, 'a, B>
where
    B: Bundle,
{
    let iter = fields
        .into_iter()
        .map(|component| component.as_component_ptr());
    let ptrs = B::ptrs_from_erased(components, iter)
        .expect("all the components should be present in the right order");
    unsafe { B::CONTEXT.ptrs_to_refs(ptrs) }
}

#[inline]
pub unsafe fn from_erased_refs_mut<'a, B>(
    components: &ComponentRegistry,
    fields: IndexSet<ErasedComponentMutRef<'a>>,
) -> RefsMut<'static, 'a, B>
where
    B: Bundle,
{
    let iter = fields
        .into_iter()
        .map(|mut component| component.as_mut_component_ptr());
    let ptrs = B::mut_ptrs_from_erased(components, iter)
        .expect("all the components should be present in the right order");
    unsafe { B::CONTEXT.mut_ptrs_to_mut_refs(ptrs) }
}

#[inline]
pub unsafe fn from_erased_slices<'a, B>(
    components: &ComponentRegistry,
    len: usize,
    fields: IndexSet<ErasedComponentSlice<'a>>,
) -> Slices<'static, 'a, B>
where
    B: Bundle,
{
    let iter = fields
        .into_iter()
        .map(|components| components.as_component_ptr());
    let ptrs = B::ptrs_from_erased(components, iter)
        .expect("all the components should be present in the right order");
    let slices = B::CONTEXT.slice_ptrs_from_raw_parts(ptrs, len);
    unsafe { B::CONTEXT.slice_ptrs_to_slices(slices) }
}

#[inline]
pub unsafe fn from_erased_mut_slices<'a, B>(
    components: &ComponentRegistry,
    len: usize,
    fields: IndexSet<ErasedComponentMutSlice<'a>>,
) -> SlicesMut<'static, 'a, B>
where
    B: Bundle,
{
    let iter = fields
        .into_iter()
        .map(|mut components| components.as_mut_component_ptr());
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
