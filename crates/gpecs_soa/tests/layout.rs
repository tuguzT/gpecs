use std::{ptr, slice};

use gpecs_soa::{
    erased::{
        field::{ErasedField, ErasedFieldRef, ErasedFieldSlice},
        ErasedSoa, ErasedSoaContext, ErasedSoaRefs, ErasedSoaSlices,
    },
    prelude::*,
    slice::{Iter as SoaIter, IterMut as SoaIterMut},
    traits::FieldDescriptor,
    vec::IntoIter as SoaIntoIter,
};

#[test]
#[cfg_attr(miri, ignore)]
fn vec_null_opt() {
    type Vec = SoaVec<(u32, u16, u8)>;

    assert_eq!(size_of::<Option<Vec>>(), size_of::<Vec>());
}

#[test]
#[cfg_attr(miri, ignore)]
fn slice_null_opt() {
    type Item = (u32, u16, u8);
    type Slice = SoaSlice<Item>;

    assert_eq!(size_of::<&Slice>(), size_of::<&[Item]>());
    assert_eq!(size_of::<Option<&Slice>>(), size_of::<&Slice>());

    assert_eq!(size_of::<&mut Slice>(), size_of::<&mut [Item]>());
    assert_eq!(size_of::<Option<&mut Slice>>(), size_of::<&mut Slice>());
}

#[test]
#[cfg_attr(miri, ignore)]
fn iter_null_opt() {
    type Iter<'a> = SoaIter<'a, (u32, u16, u8)>;

    assert_eq!(size_of::<Option<Iter>>(), size_of::<Iter>());
}

#[test]
#[cfg_attr(miri, ignore)]
fn iter_mut_null_opt() {
    type IterMut<'a> = SoaIterMut<'a, (u32, u16, u8)>;

    assert_eq!(size_of::<Option<IterMut>>(), size_of::<IterMut>());
}

#[test]
#[cfg_attr(miri, ignore)]
fn into_iter_null_opt() {
    type IntoIter = SoaIntoIter<(u32, u16, u8)>;

    assert_eq!(size_of::<Option<IntoIter>>(), size_of::<IntoIter>());
}

#[test]
#[cfg_attr(miri, ignore)]
fn erased_context() {
    let descriptors = [FieldDescriptor::of::<u8>(), FieldDescriptor::of::<i16>()];
    let _context = ErasedSoaContext::<i16>::new(descriptors);
}

#[test]
#[should_panic = "input alignment 2 must be less than or equal to 1"]
#[cfg_attr(miri, ignore)]
fn erased_context_fail() {
    let descriptors = [FieldDescriptor::of::<u8>(), FieldDescriptor::of::<i16>()];
    let _context = ErasedSoaContext::<u8>::new(descriptors);
}

#[test]
#[cfg_attr(miri, ignore)]
fn erased_context_of() {
    let context = ErasedSoaContext::of::<()>(());
    let descriptors = [FieldDescriptor::of::<()>()];
    assert!(context
        .field_descriptors()
        .iter()
        .map(FieldDescriptor::layout)
        .eq(descriptors.iter().map(FieldDescriptor::layout)));

    let context = ErasedSoaContext::of::<(u32, u16, u8)>(());
    let descriptors = [
        FieldDescriptor::of::<u8>(),
        FieldDescriptor::of::<u16>(),
        FieldDescriptor::of::<u32>(),
    ];
    assert!(context
        .field_descriptors()
        .iter()
        .map(FieldDescriptor::layout)
        .eq(descriptors.iter().map(FieldDescriptor::layout)));
}

#[test]
fn erased_value() {
    let context = ();

    let value = ();
    let erased_value = ErasedSoa::from(&context, value);

    let descriptors = [FieldDescriptor::of::<()>()];
    assert!(erased_value
        .field_descriptors()
        .iter()
        .map(FieldDescriptor::layout)
        .eq(descriptors.iter().map(FieldDescriptor::layout)));

    let field_refs = [
        ErasedFieldRef::new(FieldDescriptor::of::<()>(), [].as_slice()).expect("incorrect inputs"),
    ];
    assert!(erased_value
        .as_refs()
        .field_refs()
        .into_iter()
        .copied()
        .map(ErasedFieldRef::into_buffer)
        .eq(field_refs.into_iter().map(ErasedFieldRef::into_buffer)));

    let value = unsafe { erased_value.into::<()>(&context) };
    assert_eq!(value, ());

    let i1 = 1;
    let i2 = 2;
    let i3 = 3;
    let str = "hello";
    let value = ((), str.to_owned(), i1, i2, i3);
    let erased_value = ErasedSoa::from(&(), value);

    let descriptors = [
        FieldDescriptor::of::<()>(),
        FieldDescriptor::of::<u8>(),
        FieldDescriptor::of::<u16>(),
        FieldDescriptor::of::<u32>(),
        FieldDescriptor::of::<String>(),
    ];
    assert!(erased_value
        .field_descriptors()
        .iter()
        .map(FieldDescriptor::layout)
        .eq(descriptors.iter().map(FieldDescriptor::layout)));

    let erased_refs = erased_value.as_refs();
    assert_eq!(erased_refs.field_refs().len(), 5);

    let field_ref = erased_refs.field_refs()[0];
    assert_eq!(
        unsafe { field_ref.into::<()>() }.expect("layouts should match"),
        &(),
    );
    assert_eq!(
        field_ref.into_buffer(),
        ErasedFieldRef::from(&()).into_buffer(),
    );

    let field_ref = erased_refs.field_refs()[1];
    assert_eq!(
        unsafe { field_ref.into::<u8>() }.expect("layouts should match"),
        &i3,
    );
    assert_eq!(
        field_ref.into_buffer(),
        ErasedFieldRef::from(&i3).into_buffer(),
    );

    let field_ref = erased_refs.field_refs()[2];
    assert_eq!(
        unsafe { field_ref.into::<u16>() }.expect("layouts should match"),
        &i2,
    );
    assert_eq!(
        field_ref.into_buffer(),
        ErasedFieldRef::from(&i2).into_buffer(),
    );

    let field_ref = erased_refs.field_refs()[3];
    assert_eq!(
        unsafe { field_ref.into::<u32>() }.expect("layouts should match"),
        &i1,
    );
    assert_eq!(
        field_ref.into_buffer(),
        ErasedFieldRef::from(&i1).into_buffer(),
    );

    let field_ref = erased_refs.field_refs()[4];
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
    assert!(erased_refs.field_refs()[..4]
        .into_iter()
        .copied()
        .map(ErasedFieldRef::into_buffer)
        .eq(field_refs.into_iter().map(ErasedFieldRef::into_buffer)));

    let mut fields = erased_value.into_fields().into_vec();
    let field = fields.pop().expect("string field should exist");
    assert_eq!(
        unsafe { field.into::<String>() }.expect("layouts should match"),
        str,
    );

    let erased_value = ErasedSoa::new(fields.into_iter().map(ErasedField::into_parts))
        .expect("all the fields should have the same length");
    assert!(erased_value
        .as_refs()
        .field_refs()
        .into_iter()
        .copied()
        .map(ErasedFieldRef::into_buffer)
        .eq(field_refs.into_iter().map(ErasedFieldRef::into_buffer)));

    let value = unsafe { erased_value.into::<((), u32, u16, u8)>(&context) };
    assert_eq!(value, ((), i1, i2, i3));

    let refs = (&(), &str.to_owned(), &i1, &i2, &i3);
    let erased_refs = ErasedSoaRefs::from::<((), String, u32, u16, u8)>(&context, refs);
    assert_eq!(erased_refs.field_refs().len(), 5);

    let field_ref = erased_refs.field_refs()[0];
    assert_eq!(
        unsafe { field_ref.into::<()>() }.expect("layouts should match"),
        &(),
    );
    assert_eq!(
        field_ref.into_buffer(),
        ErasedFieldRef::from(&()).into_buffer(),
    );

    let field_ref = erased_refs.field_refs()[1];
    assert_eq!(
        unsafe { field_ref.into::<u8>() }.expect("layouts should match"),
        &i3,
    );
    assert_eq!(
        field_ref.into_buffer(),
        ErasedFieldRef::from(&i3).into_buffer(),
    );

    let field_ref = erased_refs.field_refs()[2];
    assert_eq!(
        unsafe { field_ref.into::<u16>() }.expect("layouts should match"),
        &i2,
    );
    assert_eq!(
        field_ref.into_buffer(),
        ErasedFieldRef::from(&i2).into_buffer(),
    );

    let field_ref = erased_refs.field_refs()[3];
    assert_eq!(
        unsafe { field_ref.into::<u32>() }.expect("layouts should match"),
        &i1,
    );
    assert_eq!(
        field_ref.into_buffer(),
        ErasedFieldRef::from(&i1).into_buffer(),
    );

    let field_ref = erased_refs.field_refs()[4];
    assert_eq!(
        unsafe { field_ref.into::<String>() }.expect("layouts should match"),
        &str,
    );

    assert!(erased_refs.field_refs()[..4]
        .into_iter()
        .copied()
        .map(ErasedFieldRef::into_buffer)
        .eq(field_refs.into_iter().map(ErasedFieldRef::into_buffer)));

    let refs = unsafe { erased_refs.into::<((), String, u32, u16, u8)>(&context) };
    assert_eq!(refs, (&(), &str.to_owned(), &i1, &i2, &i3));

    let units = [(), (), ()];
    let i123 = [1, 2, 3];
    let i456 = [4, 5, 6];
    let i789 = [7, 8, 9];

    let units_slices = units.as_slice();
    let i123_slices = i123.as_slice();
    let i456_slices = i456.as_slice();
    let i789_slices = i789.as_slice();

    let slices = (units_slices, i123_slices, i456_slices, i789_slices);
    let erased_slices = ErasedSoaSlices::from::<((), u32, u16, u8)>(&(), slices);
    assert_eq!(erased_slices.field_slices().len(), 4);

    let field_slice = erased_slices.field_slices()[0];
    assert_eq!(
        unsafe { field_slice.into::<()>() }.expect("layouts should match"),
        units_slices,
    );
    assert_eq!(
        field_slice.into_buffer(),
        ErasedFieldSlice::from([(); 3].as_slice()).into_buffer(),
    );
    for (idx, r#ref) in field_slice.iter().enumerate().rev() {
        assert_eq!(
            unsafe { r#ref.into::<()>() }.expect("layouts should match"),
            &units[idx],
        );
        assert_eq!(
            r#ref.into_buffer(),
            ErasedFieldRef::from(&units[idx]).into_buffer(),
        );
    }

    let field_slice = erased_slices.field_slices()[1];
    assert_eq!(
        unsafe { field_slice.into::<u8>() }.expect("layouts should match"),
        i789_slices,
    );
    assert_eq!(
        field_slice.into_buffer(),
        ErasedFieldSlice::from(i789_slices).into_buffer(),
    );
    for (idx, r#ref) in field_slice.iter().enumerate().rev() {
        assert_eq!(
            unsafe { r#ref.into::<u8>() }.expect("layouts should match"),
            &i789[idx],
        );
        assert_eq!(
            r#ref.into_buffer(),
            ErasedFieldRef::from(&i789[idx]).into_buffer(),
        );
    }
    assert_eq!(
        field_slice.iter().fold(0, |acc, item| {
            let item = unsafe { item.into::<u8>() }.expect("layouts should match");
            acc + item
        }),
        24,
    );

    let field_slice = erased_slices.field_slices()[2];
    assert_eq!(
        unsafe { field_slice.into::<u16>() }.expect("layouts should match"),
        i456_slices,
    );
    assert_eq!(
        field_slice.into_buffer(),
        ErasedFieldSlice::from(i456_slices).into_buffer(),
    );
    for (idx, r#ref) in field_slice.iter().enumerate().rev() {
        assert_eq!(
            unsafe { r#ref.into::<u16>() }.expect("layouts should match"),
            &i456[idx],
        );
        assert_eq!(
            r#ref.into_buffer(),
            ErasedFieldRef::from(&i456[idx]).into_buffer(),
        );
    }
    assert_eq!(
        field_slice.iter().fold(0, |acc, item| {
            let item = unsafe { item.into::<u16>() }.expect("layouts should match");
            acc + item
        }),
        15,
    );

    let field_slice = erased_slices.field_slices()[3];
    assert_eq!(
        unsafe { field_slice.into::<u32>() }.expect("layouts should match"),
        i123_slices,
    );
    assert_eq!(
        field_slice.into_buffer(),
        ErasedFieldSlice::from(i123_slices).into_buffer(),
    );
    for (idx, r#ref) in field_slice.iter().enumerate().rev() {
        assert_eq!(
            unsafe { r#ref.into::<u32>() }.expect("layouts should match"),
            &i123[idx],
        );
        assert_eq!(
            r#ref.into_buffer(),
            ErasedFieldRef::from(&i123[idx]).into_buffer(),
        );
    }
    assert_eq!(
        field_slice.iter().fold(0, |acc, item| {
            let item = unsafe { item.into::<u32>() }.expect("layouts should match");
            acc + item
        }),
        6,
    );

    let units_bytes = unsafe {
        let data = ptr::from_ref(&units).cast();
        let len = size_of_val(&units);
        slice::from_raw_parts(data, len)
    };
    let i123_bytes = unsafe {
        let data = ptr::from_ref(&i123).cast();
        let len = size_of_val(&i123);
        slice::from_raw_parts(data, len)
    };
    let i456_bytes = unsafe {
        let data = ptr::from_ref(&i456).cast();
        let len = size_of_val(&i456);
        slice::from_raw_parts(data, len)
    };
    let i789_bytes = unsafe {
        let data = ptr::from_ref(&i789).cast();
        let len = size_of_val(&i789);
        slice::from_raw_parts(data, len)
    };
    let field_slices = [
        ErasedFieldSlice::new(descriptors[0], units_bytes, units.len()).expect("incorrect inputs"),
        ErasedFieldSlice::new(descriptors[1], i789_bytes, i789.len()).expect("incorrect inputs"),
        ErasedFieldSlice::new(descriptors[2], i456_bytes, i456.len()).expect("incorrect inputs"),
        ErasedFieldSlice::new(descriptors[3], i123_bytes, i123.len()).expect("incorrect inputs"),
    ];
    assert!(erased_slices
        .field_slices()
        .into_iter()
        .copied()
        .map(ErasedFieldSlice::into_buffer)
        .eq(field_slices.into_iter().map(ErasedFieldSlice::into_buffer)));

    for (idx, refs) in erased_slices.iter().enumerate().rev() {
        let target_refs = ErasedSoaRefs::from::<((), u32, u16, u8)>(
            &context,
            (&units[idx], &i123[idx], &i456[idx], &i789[idx]),
        );
        let target_fields = target_refs
            .field_refs()
            .into_iter()
            .copied()
            .map(ErasedFieldRef::into_buffer);
        assert!(refs
            .field_refs()
            .into_iter()
            .copied()
            .map(ErasedFieldRef::into_buffer)
            .eq(target_fields));

        assert_eq!(
            unsafe { refs.into::<((), u32, u16, u8)>(&context) },
            (&units[idx], &i123[idx], &i456[idx], &i789[idx]),
        );
    }

    let slices = unsafe { erased_slices.into::<((), u32, u16, u8)>(&()) };
    assert_eq!(
        slices,
        (units_slices, i123_slices, i456_slices, i789_slices),
    );
}
