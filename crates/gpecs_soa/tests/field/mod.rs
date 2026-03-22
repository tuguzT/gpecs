use std::alloc::Layout;

use gpecs_soa::{
    field::{FieldDescriptor, FieldDescriptors, buffer_layout, buffer_offsets},
    identity::IdentityContext,
    traits::{AllocSoaContext, TupleContext},
};

use crate::common::{ZST1, ZST2, ZST3};

#[test]
fn unit() {
    let context = ();

    let context_descriptors = context.field_descriptors();
    let descriptors = [];
    itertools::assert_equal(
        context_descriptors.map(FieldDescriptor::layout),
        descriptors.map(FieldDescriptor::layout),
    );

    let capacity = 5;
    let from_descriptors = buffer_layout(descriptors, capacity).unwrap();
    let from_context = context.buffer_layout(capacity).unwrap();
    assert_eq!(from_descriptors, from_context);

    let mut offsets = buffer_offsets(descriptors, capacity);
    assert_eq!(offsets.len(), descriptors.len());
    assert_eq!(offsets.layout(), Layout::new::<()>());
    assert_eq!(offsets.capacity(), capacity);

    let offset = offsets.next();
    assert!(offset.is_none());

    let from_offsets = offsets.into_layout();
    assert_eq!(from_descriptors, from_offsets);
}

#[test]
fn identity() {
    let context = IdentityContext::<u128>::new();

    let context_descriptors = context.field_descriptors();
    let descriptors = [FieldDescriptor::of::<u128>()];
    itertools::assert_equal(
        context_descriptors.map(FieldDescriptor::layout),
        descriptors.map(FieldDescriptor::layout),
    );

    let capacity = 5;
    let from_descriptors = buffer_layout(descriptors, capacity).unwrap();
    let from_context = context.buffer_layout(capacity).unwrap();
    assert_eq!(from_descriptors, from_context);

    let mut offsets = buffer_offsets(descriptors, capacity);
    assert_eq!(offsets.len(), descriptors.len());
    assert_eq!(offsets.layout(), Layout::new::<()>());
    assert_eq!(offsets.capacity(), capacity);

    let offset = offsets.next().unwrap().unwrap();
    assert_eq!(offset.offset, 0);
    assert_eq!(offset.desc.layout(), Layout::new::<u128>());
    assert_eq!(offset.layout, Layout::array::<u128>(capacity).unwrap());
    assert_eq!(offsets.layout(), Layout::array::<u128>(capacity).unwrap());

    let offset = offsets.next();
    assert!(offset.is_none());

    let from_offsets = offsets.into_layout();
    assert_eq!(from_descriptors, from_offsets);
}

#[test]
fn tuple() {
    let context = TupleContext::<(u32, u128, u8, ())>::new();

    let context_descriptors = context.field_descriptors();
    let descriptors = [
        FieldDescriptor::of::<u8>(),
        FieldDescriptor::of::<()>(),
        FieldDescriptor::of::<u32>(),
        FieldDescriptor::of::<u128>(),
    ];
    itertools::assert_equal(
        context_descriptors.map(FieldDescriptor::layout),
        descriptors.map(FieldDescriptor::layout),
    );

    let capacity = 5;
    let from_descriptors = buffer_layout(descriptors, capacity).unwrap();
    let from_context = context.buffer_layout(capacity).unwrap();
    assert_eq!(from_descriptors, from_context);

    let mut offsets = buffer_offsets(descriptors, capacity);
    assert_eq!(offsets.len(), descriptors.len());
    assert_eq!(offsets.layout(), Layout::new::<()>());
    assert_eq!(offsets.capacity(), capacity);

    let offset = offsets.next().unwrap().unwrap();
    assert_eq!(offset.offset, 0);
    assert_eq!(offset.desc.layout(), Layout::new::<u8>());
    assert_eq!(offset.layout, Layout::array::<u8>(capacity).unwrap());
    assert_eq!(offsets.layout(), Layout::from_size_align(5, 1).unwrap());

    let offset = offsets.next().unwrap().unwrap();
    assert_eq!(offset.offset, 5);
    assert_eq!(offset.desc.layout(), Layout::new::<()>());
    assert_eq!(offset.layout, Layout::array::<()>(capacity).unwrap());
    assert_eq!(offsets.layout(), Layout::from_size_align(5, 1).unwrap());

    let offset = offsets.next().unwrap().unwrap();
    assert_eq!(offset.offset, 8);
    assert_eq!(offset.desc.layout(), Layout::new::<u32>());
    assert_eq!(offset.layout, Layout::array::<u32>(capacity).unwrap());
    assert_eq!(offsets.layout(), Layout::from_size_align(28, 4).unwrap());

    let offset = offsets.next().unwrap().unwrap();
    assert_eq!(offset.offset, 32);
    assert_eq!(offset.desc.layout(), Layout::new::<u128>());
    assert_eq!(offset.layout, Layout::array::<u128>(capacity).unwrap());
    assert_eq!(offsets.layout(), Layout::from_size_align(112, 16).unwrap());

    let offset = offsets.next();
    assert!(offset.is_none());

    let from_offsets = offsets.into_layout();
    assert_eq!(from_descriptors, from_offsets);
}

#[test]
fn zst_tuple() {
    let context = TupleContext::<(ZST1, ZST2, ZST3)>::new();

    let context_descriptors = context.field_descriptors();
    let descriptors = [
        FieldDescriptor::of::<ZST2>(),
        FieldDescriptor::of::<ZST3>(),
        FieldDescriptor::of::<ZST1>(),
    ];
    itertools::assert_equal(
        context_descriptors.map(FieldDescriptor::layout),
        descriptors.map(FieldDescriptor::layout),
    );

    let capacity = 5;
    let from_descriptors = buffer_layout(descriptors, capacity).unwrap();
    let from_context = context.buffer_layout(capacity).unwrap();
    assert_eq!(from_descriptors, from_context);

    let mut offsets = buffer_offsets(descriptors, capacity);
    assert_eq!(offsets.len(), descriptors.len());
    assert_eq!(offsets.layout(), Layout::new::<()>());
    assert_eq!(offsets.capacity(), capacity);

    let offset = offsets.next().unwrap().unwrap();
    assert_eq!(offset.offset, 0);
    assert_eq!(offset.desc.layout(), Layout::new::<ZST2>());
    assert_eq!(offset.layout, Layout::array::<ZST2>(capacity).unwrap());
    assert_eq!(offsets.layout(), Layout::from_size_align(0, 1).unwrap());

    let offset = offsets.next().unwrap().unwrap();
    assert_eq!(offset.offset, 0);
    assert_eq!(offset.desc.layout(), Layout::new::<ZST3>());
    assert_eq!(offset.layout, Layout::array::<ZST3>(capacity).unwrap());
    assert_eq!(offsets.layout(), Layout::from_size_align(0, 4).unwrap());

    let offset = offsets.next().unwrap().unwrap();
    assert_eq!(offset.offset, 0);
    assert_eq!(offset.desc.layout(), Layout::new::<ZST1>());
    assert_eq!(offset.layout, Layout::array::<ZST1>(capacity).unwrap());
    assert_eq!(offsets.layout(), Layout::from_size_align(0, 16).unwrap());

    let offset = offsets.next();
    assert!(offset.is_none());

    let from_offsets = offsets.into_layout();
    assert_eq!(from_descriptors, from_offsets);
}
