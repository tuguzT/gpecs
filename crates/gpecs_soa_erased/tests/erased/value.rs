use std::{alloc::Layout, mem::MaybeUninit};

#[cfg(feature = "alloc")]
use std::{ptr, slice};

use arrayvec::ArrayVec;
use gpecs_soa_erased::{
    erased::ErasedSoa,
    field::ErasedFieldRef,
    slice_item_ptr::gpu::{GpuSliceItemPtr, GpuSliceItemPtrs},
    soa::field::{FieldDescriptor, FieldDescriptors},
    storage::AlignedUninitStorage,
};

#[cfg(feature = "alloc")]
use gpecs_soa_erased::{
    field::{BoxedErasedField, ErasedField},
    storage::BoxedAlignedUninitStorage,
};

use crate::common::ArrayDescriptors;

#[test]
#[cfg(feature = "alloc")]
fn value() {
    type Value = ((), String, u32, u16, u8);

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

    let bytes = AlignedUninitStorage::new(bytes, Layout::new::<Value>()).unwrap();
    let erased_value =
        ErasedSoa::<_, ArrayDescriptors<FieldDescriptor, 5>, GpuSliceItemPtrs, _>::try_from_storage_value(
            bytes, &context, value,
        )
        .unwrap();

    let descriptors = [
        FieldDescriptor::of::<()>(),
        FieldDescriptor::of::<u8>(),
        FieldDescriptor::of::<u16>(),
        FieldDescriptor::of::<u32>(),
        FieldDescriptor::of::<String>(),
    ];
    itertools::assert_equal(
        erased_value
            .field_descriptors()
            .iter()
            .copied()
            .map(FieldDescriptor::layout),
        descriptors.map(FieldDescriptor::layout),
    );

    let erased_refs = erased_value.as_fields();
    assert_eq!(erased_refs.into_iter().len(), 5);

    let field_ref = erased_refs.into_iter().nth(0).unwrap();
    assert_eq!(
        unsafe { field_ref.try_into::<()>() }.expect("layouts should match"),
        &(),
    );
    assert_eq!(
        field_ref.into_buffer(),
        ErasedFieldRef::<GpuSliceItemPtr<*const [MaybeUninit<_>]>>::try_from(&())
            .unwrap()
            .into_buffer(),
    );

    let field_ref = erased_refs.into_iter().nth(1).unwrap();
    assert_eq!(
        unsafe { field_ref.try_into::<u8>() }.expect("layouts should match"),
        &i3,
    );
    assert_eq!(
        field_ref.into_buffer(),
        ErasedFieldRef::<GpuSliceItemPtr<*const [MaybeUninit<_>]>>::try_from(&i3)
            .unwrap()
            .into_buffer(),
    );

    let field_ref = erased_refs.into_iter().nth(2).unwrap();
    assert_eq!(
        unsafe { field_ref.try_into::<u16>() }.expect("layouts should match"),
        &i2,
    );
    assert_eq!(
        field_ref.into_buffer(),
        ErasedFieldRef::<GpuSliceItemPtr<*const [MaybeUninit<_>]>>::try_from(&i2)
            .unwrap()
            .into_buffer(),
    );

    let field_ref = erased_refs.into_iter().nth(3).unwrap();
    assert_eq!(
        unsafe { field_ref.try_into::<u32>() }.expect("layouts should match"),
        &i1,
    );
    assert_eq!(
        field_ref.into_buffer(),
        ErasedFieldRef::<GpuSliceItemPtr<*const [MaybeUninit<_>]>>::try_from(&i1)
            .unwrap()
            .into_buffer(),
    );

    let field_ref = erased_refs.into_iter().nth(4).unwrap();
    assert_eq!(
        unsafe { field_ref.try_into::<String>() }.expect("layouts should match"),
        &str,
    );

    let unit_bytes = [0u8; size_of::<()>()].as_slice();
    let i1_bytes = unsafe {
        let data = ptr::from_ref(&i1).cast();
        let len = size_of_val(&i1);
        slice::from_raw_parts(data, len)
    };
    let i2_bytes = unsafe {
        let data = ptr::from_ref(&i2).cast();
        let len = size_of_val(&i2);
        slice::from_raw_parts(data, len)
    };
    let i3_bytes = unsafe {
        let data = ptr::from_ref(&i3).cast();
        let len = size_of_val(&i3);
        slice::from_raw_parts(data, len)
    };
    let field_refs = [
        ErasedFieldRef::<GpuSliceItemPtr<*const [MaybeUninit<_>]>>::new(descriptors[0], unit_bytes)
            .expect("incorrect inputs"),
        ErasedFieldRef::<GpuSliceItemPtr<*const [MaybeUninit<_>]>>::new(descriptors[1], i3_bytes)
            .expect("incorrect inputs"),
        ErasedFieldRef::<GpuSliceItemPtr<*const [MaybeUninit<_>]>>::new(descriptors[2], i2_bytes)
            .expect("incorrect inputs"),
        ErasedFieldRef::<GpuSliceItemPtr<*const [MaybeUninit<_>]>>::new(descriptors[3], i1_bytes)
            .expect("incorrect inputs"),
    ];
    assert!(
        erased_refs
            .into_iter()
            .take(4)
            .map(ErasedFieldRef::into_buffer)
            .eq(field_refs.into_iter().map(ErasedFieldRef::into_buffer)),
    );

    let mut fields = erased_value
        .into_fields()
        .collect::<Result<ArrayVec<_, 5>, _>>()
        .expect("allocation of small byte array should succeed");
    let field: BoxedErasedField<_> = fields.pop().expect("string field should exist");
    assert_eq!(
        unsafe { field.try_into::<String>() }.expect("layouts should match"),
        str,
    );

    let (descriptors, fields): (ArrayDescriptors<FieldDescriptor, 4>, ArrayVec<_, 4>) =
        fields.into_iter().map(ErasedField::into_parts).unzip();
    let erased_value = ErasedSoa::<BoxedAlignedUninitStorage, _, GpuSliceItemPtrs, _>::try_from_fields_descriptors(
        fields,
        descriptors,
    )
    .expect("all the fields should be valid here");

    let erased_value_refs = erased_value.as_fields();
    itertools::assert_equal(
        erased_value_refs
            .into_iter()
            .map(ErasedFieldRef::into_buffer),
        field_refs.into_iter().map(ErasedFieldRef::into_buffer),
    );

    let context = Default::default();
    let value = unsafe { erased_value.try_into::<((), u32, u16, u8)>(&context) }
        .expect("all the fields should be valid here");
    assert_eq!(value, ((), i1, i2, i3));
}

#[test]
fn value_zst() {
    let context = ();
    let value = ();

    let bytes = [MaybeUninit::zeroed(); size_of::<()>() * 2];
    let bytes = AlignedUninitStorage::new(bytes, Layout::new::<()>()).unwrap();
    let erased_value =
        ErasedSoa::<_, ArrayDescriptors<FieldDescriptor, 1>, GpuSliceItemPtrs, _>::try_from_storage_value(
            bytes, &context, value,
        )
        .unwrap();

    let descriptors = [FieldDescriptor::of::<()>()];
    itertools::assert_equal(
        erased_value
            .field_descriptors()
            .iter()
            .copied()
            .map(FieldDescriptor::layout),
        descriptors.map(FieldDescriptor::layout),
    );

    let field_refs = [
        ErasedFieldRef::<GpuSliceItemPtr<*const [MaybeUninit<_>]>>::new(
            FieldDescriptor::of::<()>(),
            [].as_slice(),
        )
        .expect("incorrect inputs"),
    ];
    itertools::assert_equal(
        erased_value
            .as_fields()
            .into_iter()
            .map(ErasedFieldRef::into_buffer),
        field_refs.into_iter().map(ErasedFieldRef::into_buffer),
    );

    let value = unsafe { erased_value.try_into::<()>(&context) }
        .expect("all the fields should be valid here");
    assert_eq!(value, ());
}
