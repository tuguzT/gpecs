use std::{alloc::Layout, mem::MaybeUninit};

use arrayvec::ArrayVec;
use gpecs_soa_erased::{
    ErasedSoa, data::ErasedRef, ptr::slice::CoreSliceItemPtrs, soa::field::FieldLayouts,
    storage::AlignedStorageSlice,
};

#[cfg(feature = "alloc")]
use gpecs_soa_erased::{data::BoxedErased, storage::BoxedAlignedUninitStorage};

use crate::common::ArrayLayouts;

type ArrayErasedSoa<T, const CAP: usize> =
    ErasedSoa<T, ArrayLayouts<Layout, CAP>, CoreSliceItemPtrs<MaybeUninit<u8>>>;

#[test]
#[cfg(feature = "alloc")]
fn value() {
    type Value = ((), String, u32, u128, u8);

    let context = Default::default();

    let i1 = 1;
    let i2 = 2;
    let i3 = 3;
    let str = "hello";
    let value = ((), str.to_owned(), i1, i2, i3);

    let mut bytes = [0_u8; size_of::<Value>() * 2];
    let bytes = unsafe {
        let (_, bytes, _) = bytes.align_to_mut::<Value>();
        let (_, bytes, _) = bytes.align_to_mut();
        bytes
    };

    let bytes = AlignedStorageSlice::new(bytes, Layout::new::<Value>()).unwrap();
    let erased_value =
        ArrayErasedSoa::<_, 5>::try_from_storage_value::<Value, _>(bytes, &context, value).unwrap();

    let layouts = [
        Layout::new::<()>(),
        Layout::new::<u8>(),
        Layout::new::<u32>(),
        Layout::new::<String>(),
        Layout::new::<u128>(),
    ];
    itertools::assert_equal(erased_value.field_layouts().iter().copied(), layouts);

    let erased_refs = erased_value.as_refs();
    assert_eq!(erased_refs.iter().len(), 5);

    let field_ref = erased_refs.iter().nth(0).unwrap();
    assert_eq!(
        unsafe { field_ref.downcast::<()>() }.expect("layouts should match"),
        &(),
    );

    let (_, field_ref_bytes, _) = unsafe { field_ref.into_buffer().align_to::<u8>() };
    assert_eq!(
        field_ref_bytes,
        ErasedRef::<*const _>::try_from(&()).unwrap().into_buffer(),
    );

    let field_ref = erased_refs.iter().nth(1).unwrap();
    assert_eq!(
        unsafe { field_ref.downcast::<u8>() }.expect("layouts should match"),
        &i3,
    );

    let (_, field_ref_bytes, _) = unsafe { field_ref.into_buffer().align_to::<u8>() };
    assert_eq!(
        field_ref_bytes,
        ErasedRef::<*const _>::try_from(&i3).unwrap().into_buffer(),
    );

    let field_ref = erased_refs.iter().nth(2).unwrap();
    assert_eq!(
        unsafe { field_ref.downcast::<u32>() }.expect("layouts should match"),
        &i1,
    );

    let (_, field_ref_bytes, _) = unsafe { field_ref.into_buffer().align_to::<u8>() };
    assert_eq!(
        field_ref_bytes,
        ErasedRef::<*const _>::try_from(&i1).unwrap().into_buffer(),
    );

    let field_ref = erased_refs.iter().nth(3).unwrap();
    assert_eq!(
        unsafe { field_ref.downcast::<String>() }.expect("layouts should match"),
        &str,
    );

    let field_ref = erased_refs.iter().nth(4).unwrap();
    assert_eq!(
        unsafe { field_ref.downcast::<u128>() }.expect("layouts should match"),
        &i2,
    );

    let (_, field_ref_bytes, _) = unsafe { field_ref.into_buffer().align_to::<u8>() };
    assert_eq!(
        field_ref_bytes,
        ErasedRef::<*const _>::try_from(&i2).unwrap().into_buffer(),
    );

    let field_refs = [
        ErasedRef::<*const _>::new(layouts[0], bytemuck::bytes_of(&())).unwrap(),
        ErasedRef::<*const _>::new(layouts[1], bytemuck::bytes_of(&i3)).unwrap(),
        ErasedRef::<*const _>::new(layouts[2], bytemuck::bytes_of(&i1)).unwrap(),
        ErasedRef::<*const _>::new(layouts[4], bytemuck::bytes_of(&i2)).unwrap(),
    ];
    itertools::assert_equal(
        erased_refs
            .iter()
            .enumerate()
            .filter_map(|(i, item)| (i != 3).then_some(item))
            .map(|item| unsafe { item.into_buffer().align_to::<u8>().1 }),
        field_refs.into_iter().map(ErasedRef::into_buffer),
    );

    let mut fields = erased_value
        .into_fields()
        .collect::<Result<ArrayVec<_, 5>, _>>()
        .expect("allocation of small byte array should succeed");
    let field: BoxedErased<_> = fields.remove(3);
    assert_eq!(
        unsafe { field.downcast::<String>() }.expect("layouts should match"),
        str,
    );

    let fields_with_layouts = fields.into_iter().map(BoxedErased::into_parts);
    let erased_value =
        ArrayErasedSoa::<BoxedAlignedUninitStorage, 4>::try_from_fields_with_layouts(
            fields_with_layouts,
        )
        .expect("all the fields should be valid here");

    itertools::assert_equal(
        erased_value
            .iter()
            .map(|item| unsafe { item.into_buffer().align_to::<u8>().1 }),
        field_refs.into_iter().map(ErasedRef::into_buffer),
    );

    let context = Default::default();
    let value = unsafe { erased_value.downcast::<((), u32, u128, u8), _>(&context) }
        .expect("all the fields should be valid here");
    assert_eq!(value, ((), i1, i2, i3));
}

#[test]
fn value_zst() {
    let context = ();
    let value = ();

    let bytes = [MaybeUninit::zeroed(); size_of::<()>() * 2];
    let bytes = AlignedStorageSlice::new(bytes, Layout::new::<()>()).unwrap();
    let erased_value =
        ArrayErasedSoa::<_, 1>::try_from_storage_value::<(), _>(bytes, &context, value).unwrap();

    let layouts = [];
    itertools::assert_equal(erased_value.field_layouts().iter().copied(), layouts);

    let field_refs = [];
    itertools::assert_equal(
        erased_value
            .iter()
            .map(|item| unsafe { item.into_buffer().align_to::<u8>().1 }),
        field_refs
            .into_iter()
            .map(ErasedRef::<*const _>::into_buffer),
    );

    let value = unsafe { erased_value.downcast::<(), _>(&context) }
        .expect("all the fields should be valid here");
    assert_eq!(value, ());
}
