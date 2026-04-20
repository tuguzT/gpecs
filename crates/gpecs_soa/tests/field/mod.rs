use std::alloc::Layout;

use gpecs_soa::{
    field::{FieldLayouts, buffer_layout, buffer_offsets},
    identity::Identity,
    traits::AllocSoaContext,
};

use crate::common::{ZST1, ZST2, ZST3};

#[test]
fn unit() {
    let context = ();

    let context_layouts = FieldLayouts::<()>::field_layouts(&context);
    let layouts = [];
    itertools::assert_equal(context_layouts, layouts);

    let capacity = 5;
    let from_layouts = buffer_layout(layouts, capacity).unwrap();
    let from_context = AllocSoaContext::<()>::buffer_layout(&context, capacity).unwrap();
    assert_eq!(from_layouts, from_context);

    let mut offsets = buffer_offsets(layouts, capacity);
    assert_eq!(offsets.len(), layouts.len());
    assert_eq!(offsets.buffer_layout(), Layout::new::<()>());
    assert_eq!(offsets.capacity(), capacity);

    let offset = offsets.next();
    assert!(offset.is_none());

    let from_offsets = offsets.into_buffer_layout();
    assert_eq!(from_layouts, from_offsets);
}

#[test]
fn identity() {
    let context = ();

    let context_layouts = FieldLayouts::<Identity<u128>>::field_layouts(&context);
    let layouts = [Layout::new::<u128>()];
    itertools::assert_equal(context_layouts, layouts);

    let capacity = 5;
    let from_layouts = buffer_layout(layouts, capacity).unwrap();
    let from_context =
        AllocSoaContext::<Identity<u128>>::buffer_layout(&context, capacity).unwrap();
    assert_eq!(from_layouts, from_context);

    let mut offsets = buffer_offsets(layouts, capacity);
    assert_eq!(offsets.len(), layouts.len());
    assert_eq!(offsets.buffer_layout(), Layout::new::<()>());
    assert_eq!(offsets.capacity(), capacity);

    let offset = offsets.next().unwrap().unwrap();
    assert_eq!(offset.offset, 0);
    assert_eq!(offset.desc, Layout::new::<u128>());
    assert_eq!(
        offsets.buffer_layout(),
        Layout::array::<u128>(capacity).unwrap(),
    );

    let offset = offsets.next();
    assert!(offset.is_none());

    let from_offsets = offsets.into_buffer_layout();
    assert_eq!(from_layouts, from_offsets);
}

#[test]
fn tuple() {
    let context = ();

    let context_layouts = FieldLayouts::<(u32, u128, u8, ())>::field_layouts(&context);
    let layouts = [
        Layout::new::<u8>(),
        Layout::new::<()>(),
        Layout::new::<u32>(),
        Layout::new::<u128>(),
    ];
    itertools::assert_equal(context_layouts, layouts);

    let capacity = 5;
    let from_layouts = buffer_layout(layouts, capacity).unwrap();
    let from_context =
        AllocSoaContext::<(u32, u128, u8, ())>::buffer_layout(&context, capacity).unwrap();
    assert_eq!(from_layouts, from_context);

    let mut offsets = buffer_offsets(layouts, capacity);
    assert_eq!(offsets.len(), layouts.len());
    assert_eq!(offsets.buffer_layout(), Layout::new::<()>());
    assert_eq!(offsets.capacity(), capacity);

    let offset = offsets.next().unwrap().unwrap();
    assert_eq!(offset.offset, 0);
    assert_eq!(offset.desc, Layout::new::<u8>());
    assert_eq!(
        offsets.buffer_layout(),
        Layout::from_size_align(5, 1).unwrap(),
    );

    let offset = offsets.next().unwrap().unwrap();
    assert_eq!(offset.offset, 5);
    assert_eq!(offset.desc, Layout::new::<()>());
    assert_eq!(
        offsets.buffer_layout(),
        Layout::from_size_align(5, 1).unwrap(),
    );

    let offset = offsets.next().unwrap().unwrap();
    assert_eq!(offset.offset, 8);
    assert_eq!(offset.desc, Layout::new::<u32>());
    assert_eq!(
        offsets.buffer_layout(),
        Layout::from_size_align(28, 4).unwrap(),
    );

    let offset = offsets.next().unwrap().unwrap();
    assert_eq!(offset.offset, 32);
    assert_eq!(offset.desc, Layout::new::<u128>());
    assert_eq!(
        offsets.buffer_layout(),
        Layout::from_size_align(112, 16).unwrap(),
    );

    let offset = offsets.next();
    assert!(offset.is_none());

    let from_offsets = offsets.into_buffer_layout();
    assert_eq!(from_layouts, from_offsets);
}

#[test]
fn zst_tuple() {
    let context = ();

    let context_layouts = FieldLayouts::<(ZST1, ZST2, ZST3)>::field_layouts(&context);
    let layouts = [
        Layout::new::<ZST2>(),
        Layout::new::<ZST3>(),
        Layout::new::<ZST1>(),
    ];
    itertools::assert_equal(context_layouts, layouts);

    let capacity = 5;
    let from_layouts = buffer_layout(layouts, capacity).unwrap();
    let from_context =
        AllocSoaContext::<(ZST1, ZST2, ZST3)>::buffer_layout(&context, capacity).unwrap();
    assert_eq!(from_layouts, from_context);

    let mut offsets = buffer_offsets(layouts, capacity);
    assert_eq!(offsets.len(), layouts.len());
    assert_eq!(offsets.buffer_layout(), Layout::new::<()>());
    assert_eq!(offsets.capacity(), capacity);

    let offset = offsets.next().unwrap().unwrap();
    assert_eq!(offset.offset, 0);
    assert_eq!(offset.desc, Layout::new::<ZST2>());
    assert_eq!(offsets.buffer_layout(), Layout::new::<ZST2>());

    let offset = offsets.next().unwrap().unwrap();
    assert_eq!(offset.offset, 0);
    assert_eq!(offset.desc, Layout::new::<ZST3>());
    assert_eq!(offsets.buffer_layout(), Layout::new::<ZST3>());

    let offset = offsets.next().unwrap().unwrap();
    assert_eq!(offset.offset, 0);
    assert_eq!(offset.desc, Layout::new::<ZST1>());
    assert_eq!(offsets.buffer_layout(), Layout::new::<ZST1>());

    let offset = offsets.next();
    assert!(offset.is_none());

    let from_offsets = offsets.into_buffer_layout();
    assert_eq!(from_layouts, from_offsets);
}
