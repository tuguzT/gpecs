use std::{ptr, slice};

use gpecs_soa_erased::{
    erased::ErasedSoa,
    field::{ErasedField, ErasedFieldRef},
    soa::traits::FieldDescriptor,
};

#[test]
fn value() {
    let context = ();

    let i1 = 1;
    let i2 = 2;
    let i3 = 3;
    let str = "hello";
    let value = ((), str.to_owned(), i1, i2, i3);
    let erased_value = ErasedSoa::from(&context, value);

    let descriptors = [
        FieldDescriptor::of::<()>(),
        FieldDescriptor::of::<u8>(),
        FieldDescriptor::of::<u16>(),
        FieldDescriptor::of::<u32>(),
        FieldDescriptor::of::<String>(),
    ];
    assert!(
        erased_value
            .field_descriptors()
            .iter()
            .map(FieldDescriptor::layout)
            .eq(descriptors.iter().map(FieldDescriptor::layout))
    );

    let erased_refs = erased_value.as_refs();
    assert_eq!(erased_refs.into_iter().len(), 5);

    let field_ref = erased_refs.into_iter().nth(0).unwrap();
    assert_eq!(
        unsafe { field_ref.into::<()>() }.expect("layouts should match"),
        &(),
    );
    assert_eq!(
        field_ref.into_buffer(),
        ErasedFieldRef::from(&()).into_buffer(),
    );

    let field_ref = erased_refs.into_iter().nth(1).unwrap();
    assert_eq!(
        unsafe { field_ref.into::<u8>() }.expect("layouts should match"),
        &i3,
    );
    assert_eq!(
        field_ref.into_buffer(),
        ErasedFieldRef::from(&i3).into_buffer(),
    );

    let field_ref = erased_refs.into_iter().nth(2).unwrap();
    assert_eq!(
        unsafe { field_ref.into::<u16>() }.expect("layouts should match"),
        &i2,
    );
    assert_eq!(
        field_ref.into_buffer(),
        ErasedFieldRef::from(&i2).into_buffer(),
    );

    let field_ref = erased_refs.into_iter().nth(3).unwrap();
    assert_eq!(
        unsafe { field_ref.into::<u32>() }.expect("layouts should match"),
        &i1,
    );
    assert_eq!(
        field_ref.into_buffer(),
        ErasedFieldRef::from(&i1).into_buffer(),
    );

    let field_ref = erased_refs.into_iter().nth(4).unwrap();
    assert_eq!(
        unsafe { field_ref.into::<String>() }.expect("layouts should match"),
        &str,
    );

    let unit_bytes = [0u8; 0].as_slice();
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
        ErasedFieldRef::new(descriptors[0], unit_bytes).expect("incorrect inputs"),
        ErasedFieldRef::new(descriptors[1], i3_bytes).expect("incorrect inputs"),
        ErasedFieldRef::new(descriptors[2], i2_bytes).expect("incorrect inputs"),
        ErasedFieldRef::new(descriptors[3], i1_bytes).expect("incorrect inputs"),
    ];
    assert!(
        erased_refs
            .into_iter()
            .take(4)
            .map(ErasedFieldRef::into_buffer)
            .eq(field_refs.into_iter().map(ErasedFieldRef::into_buffer))
    );

    let mut fields = erased_value.into_fields().into_vec();
    let field = fields.pop().expect("string field should exist");
    assert_eq!(
        unsafe { field.into::<String>() }.expect("layouts should match"),
        str,
    );

    let erased_value = ErasedSoa::new(fields.into_iter().map(ErasedField::into_parts))
        .expect("all the fields should be valid");
    assert!(
        erased_value
            .as_refs()
            .into_iter()
            .map(ErasedFieldRef::into_buffer)
            .eq(field_refs.into_iter().map(ErasedFieldRef::into_buffer))
    );

    let value = unsafe { erased_value.into::<((), u32, u16, u8)>(&context) }
        .expect("all the fields should be valid");
    assert_eq!(value, ((), i1, i2, i3));
}

#[test]
fn value_zst() {
    let context = ();

    let value = ();
    let erased_value = ErasedSoa::from(&context, value);

    let descriptors = [FieldDescriptor::of::<()>()];
    assert!(
        erased_value
            .field_descriptors()
            .iter()
            .map(FieldDescriptor::layout)
            .eq(descriptors.iter().map(FieldDescriptor::layout))
    );

    let field_refs = [
        ErasedFieldRef::new(FieldDescriptor::of::<()>(), [].as_slice()).expect("incorrect inputs"),
    ];
    assert!(
        erased_value
            .as_refs()
            .into_iter()
            .map(ErasedFieldRef::into_buffer)
            .eq(field_refs.into_iter().map(ErasedFieldRef::into_buffer))
    );

    let value =
        unsafe { erased_value.into::<()>(&context) }.expect("all the fields should be valid");
    assert_eq!(value, ());
}
