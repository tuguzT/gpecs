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

    assert!(erased_value
        .as_refs()
        .field_refs()
        .into_iter()
        .copied()
        .map(ErasedFieldRef::into_buffer)
        .eq([ErasedFieldRef::new(FieldDescriptor::of::<()>(), [].as_slice()).into_buffer()]));

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

    assert_eq!(unsafe { erased_refs.field_refs()[0].into::<()>() }, &());
    assert_eq!(
        erased_refs.field_refs()[0].into_buffer(),
        ErasedFieldRef::from(&()).into_buffer(),
    );

    assert_eq!(unsafe { erased_refs.field_refs()[1].into::<u8>() }, &i3);
    assert_eq!(
        erased_refs.field_refs()[1].into_buffer(),
        ErasedFieldRef::from(&i3).into_buffer(),
    );

    assert_eq!(unsafe { erased_refs.field_refs()[2].into::<u16>() }, &i2);
    assert_eq!(
        erased_refs.field_refs()[2].into_buffer(),
        ErasedFieldRef::from(&i2).into_buffer(),
    );

    assert_eq!(unsafe { erased_refs.field_refs()[3].into::<u32>() }, &i1);
    assert_eq!(
        erased_refs.field_refs()[3].into_buffer(),
        ErasedFieldRef::from(&i1).into_buffer(),
    );

    assert_eq!(
        unsafe { erased_refs.field_refs()[4].into::<String>() },
        &str
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
    assert!(erased_refs.field_refs()[..4]
        .into_iter()
        .copied()
        .map(ErasedFieldRef::into_buffer)
        .eq([
            ErasedFieldRef::new(descriptors[0], unit_bytes).into_buffer(),
            ErasedFieldRef::new(descriptors[1], i3_bytes).into_buffer(),
            ErasedFieldRef::new(descriptors[2], i2_bytes).into_buffer(),
            ErasedFieldRef::new(descriptors[3], i1_bytes).into_buffer(),
        ]));

    let mut fields = erased_value.into_fields().into_vec();
    let field = fields.pop().expect("string field should exist");
    assert_eq!(unsafe { field.into::<String>() }, str);

    let erased_value = ErasedSoa::new(fields.into_iter().map(ErasedField::into_parts));
    assert!(erased_value
        .as_refs()
        .field_refs()
        .into_iter()
        .copied()
        .map(ErasedFieldRef::into_buffer)
        .eq([
            ErasedFieldRef::new(descriptors[0], unit_bytes).into_buffer(),
            ErasedFieldRef::new(descriptors[1], i3_bytes).into_buffer(),
            ErasedFieldRef::new(descriptors[2], i2_bytes).into_buffer(),
            ErasedFieldRef::new(descriptors[3], i1_bytes).into_buffer(),
        ]));

    let value = unsafe { erased_value.into::<((), u32, u16, u8)>(&context) };
    assert_eq!(value, ((), i1, i2, i3));

    let refs = (&(), &str.to_owned(), &i1, &i2, &i3);
    let erased_refs = ErasedSoaRefs::from::<((), String, u32, u16, u8)>(&context, refs);
    assert_eq!(erased_refs.field_refs().len(), 5);

    assert_eq!(unsafe { erased_refs.field_refs()[0].into::<()>() }, &());
    assert_eq!(
        erased_refs.field_refs()[0].into_buffer(),
        ErasedFieldRef::from(&()).into_buffer(),
    );

    assert_eq!(unsafe { erased_refs.field_refs()[1].into::<u8>() }, &i3);
    assert_eq!(
        erased_refs.field_refs()[1].into_buffer(),
        ErasedFieldRef::from(&i3).into_buffer(),
    );

    assert_eq!(unsafe { erased_refs.field_refs()[2].into::<u16>() }, &i2);
    assert_eq!(
        erased_refs.field_refs()[2].into_buffer(),
        ErasedFieldRef::from(&i2).into_buffer(),
    );

    assert_eq!(unsafe { erased_refs.field_refs()[3].into::<u32>() }, &i1);
    assert_eq!(
        erased_refs.field_refs()[3].into_buffer(),
        ErasedFieldRef::from(&i1).into_buffer(),
    );

    assert_eq!(
        unsafe { erased_refs.field_refs()[4].into::<String>() },
        &str
    );

    assert!(erased_refs.field_refs()[..4]
        .into_iter()
        .copied()
        .map(ErasedFieldRef::into_buffer)
        .eq([
            ErasedFieldRef::new(descriptors[0], [].as_slice()).into_buffer(),
            ErasedFieldRef::new(descriptors[1], i3_bytes).into_buffer(),
            ErasedFieldRef::new(descriptors[2], i2_bytes).into_buffer(),
            ErasedFieldRef::new(descriptors[3], i1_bytes).into_buffer(),
        ]));

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

    assert_eq!(
        unsafe { erased_slices.field_slices()[0].into::<()>() },
        units_slices,
    );
    assert_eq!(
        erased_slices.field_slices()[0].into_buffer(),
        ErasedFieldSlice::from([(); 0].as_slice()).into_buffer(),
    );
    for (idx, r#ref) in erased_slices.field_slices()[0].iter().enumerate().rev() {
        assert_eq!(unsafe { r#ref.into::<()>() }, &units[idx]);
        assert_eq!(
            r#ref.into_buffer(),
            ErasedFieldRef::from(&units[idx]).into_buffer(),
        );
    }

    assert_eq!(
        unsafe { erased_slices.field_slices()[1].into::<u8>() },
        i789_slices,
    );
    assert_eq!(
        erased_slices.field_slices()[1].into_buffer(),
        ErasedFieldSlice::from(i789_slices).into_buffer(),
    );
    for (idx, r#ref) in erased_slices.field_slices()[1].iter().enumerate().rev() {
        assert_eq!(unsafe { r#ref.into::<u8>() }, &i789[idx]);
        assert_eq!(
            r#ref.into_buffer(),
            ErasedFieldRef::from(&i789[idx]).into_buffer(),
        );
    }
    assert_eq!(
        erased_slices.field_slices()[1]
            .iter()
            .fold(0, |acc, item| acc + unsafe { item.into::<u8>() }),
        24,
    );

    assert_eq!(
        unsafe { erased_slices.field_slices()[2].into::<u16>() },
        i456_slices,
    );
    assert_eq!(
        erased_slices.field_slices()[2].into_buffer(),
        ErasedFieldSlice::from(i456_slices).into_buffer(),
    );
    for (idx, r#ref) in erased_slices.field_slices()[2].iter().enumerate().rev() {
        assert_eq!(unsafe { r#ref.into::<u16>() }, &i456[idx]);
        assert_eq!(
            r#ref.into_buffer(),
            ErasedFieldRef::from(&i456[idx]).into_buffer(),
        );
    }
    assert_eq!(
        erased_slices.field_slices()[2]
            .iter()
            .fold(0, |acc, item| acc + unsafe { item.into::<u16>() }),
        15,
    );

    assert_eq!(
        unsafe { erased_slices.field_slices()[3].into::<u32>() },
        i123_slices,
    );
    assert_eq!(
        erased_slices.field_slices()[3].into_buffer(),
        ErasedFieldSlice::from(i123_slices).into_buffer(),
    );
    for (idx, r#ref) in erased_slices.field_slices()[3].iter().enumerate().rev() {
        assert_eq!(unsafe { r#ref.into::<u32>() }, &i123[idx]);
        assert_eq!(
            r#ref.into_buffer(),
            ErasedFieldRef::from(&i123[idx]).into_buffer(),
        );
    }
    assert_eq!(
        erased_slices.field_slices()[3]
            .iter()
            .fold(0, |acc, item| acc + unsafe { item.into::<u32>() }),
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
    assert!(erased_slices
        .field_slices()
        .into_iter()
        .copied()
        .map(ErasedFieldSlice::into_buffer)
        .eq([
            ErasedFieldSlice::new(descriptors[0], units_bytes, units.len()).into_buffer(),
            ErasedFieldSlice::new(descriptors[1], i789_bytes, i789.len()).into_buffer(),
            ErasedFieldSlice::new(descriptors[2], i456_bytes, i456.len()).into_buffer(),
            ErasedFieldSlice::new(descriptors[3], i123_bytes, i123.len()).into_buffer(),
        ]));

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
