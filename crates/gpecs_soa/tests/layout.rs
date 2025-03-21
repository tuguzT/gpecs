use std::{alloc::Layout, ptr, slice};

use gpecs_soa::{
    erased::{
        field::{ErasedFieldRef, ErasedFieldSlice},
        ErasedSoa, ErasedSoaContext, ErasedSoaRefs, ErasedSoaSlices,
    },
    prelude::*,
    slice::{Iter as SoaIter, IterMut as SoaIterMut},
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
    let field_layouts = [Layout::new::<u8>(), Layout::new::<i16>()];
    let _context = ErasedSoaContext::<i16>::new(field_layouts, None);
}

#[test]
#[should_panic = "input alignment 2 must be less than or equal to 1"]
#[cfg_attr(miri, ignore)]
fn erased_context_fail() {
    let field_layouts = [Layout::new::<u8>(), Layout::new::<i16>()];
    let _context = ErasedSoaContext::<u8>::new(field_layouts, None);
}

#[test]
#[cfg_attr(miri, ignore)]
fn erased_context_of() {
    let context = ErasedSoaContext::of::<()>(());
    assert_eq!(context.field_layouts(), [Layout::new::<()>()]);

    let context = ErasedSoaContext::of::<(u32, u16, u8)>(());
    let optimized_layout = [
        Layout::new::<u8>(),
        Layout::new::<u16>(),
        Layout::new::<u32>(),
    ];
    assert_eq!(context.field_layouts(), optimized_layout);
}

#[test]
fn erased_value() {
    let context = ();

    let value = ();
    let erased_value = ErasedSoa::from(&context, value);
    assert_eq!(erased_value.field_layouts(), [Layout::new::<()>()]);

    assert!(erased_value
        .as_refs()
        .fields()
        .into_iter()
        .copied()
        .map(ErasedFieldRef::into_parts)
        .eq([ErasedFieldRef::new(Layout::new::<()>(), [].as_slice()).into_parts()]));

    let value = unsafe { erased_value.into::<()>(&context) };
    assert_eq!(value, ());

    let i1 = 1;
    let i2 = 2;
    let i3 = 3;
    let value = (i1, i2, i3);
    let erased_value = ErasedSoa::from(&(), value);

    let optimized_layout = [
        Layout::new::<u8>(),
        Layout::new::<u16>(),
        Layout::new::<u32>(),
    ];
    assert_eq!(erased_value.field_layouts(), optimized_layout);

    let erased_refs = erased_value.as_refs();
    assert_eq!(erased_refs.fields().len(), 3);

    assert_eq!(unsafe { erased_refs.fields()[0].into::<u8>() }, &i3);
    assert_eq!(
        erased_refs.fields()[0].into_parts(),
        ErasedFieldRef::from(&i3).into_parts(),
    );

    assert_eq!(unsafe { erased_refs.fields()[1].into::<u16>() }, &i2);
    assert_eq!(
        erased_refs.fields()[1].into_parts(),
        ErasedFieldRef::from(&i2).into_parts(),
    );

    assert_eq!(unsafe { erased_refs.fields()[2].into::<u32>() }, &i1);
    assert_eq!(
        erased_refs.fields()[2].into_parts(),
        ErasedFieldRef::from(&i1).into_parts(),
    );

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
    assert!(erased_refs
        .fields()
        .into_iter()
        .copied()
        .map(ErasedFieldRef::into_parts)
        .eq([
            ErasedFieldRef::new(optimized_layout[0], i3_bytes).into_parts(),
            ErasedFieldRef::new(optimized_layout[1], i2_bytes).into_parts(),
            ErasedFieldRef::new(optimized_layout[2], i1_bytes).into_parts(),
        ]));

    let erased_value = ErasedSoa::new(
        erased_value
            .into_fields()
            .iter()
            .map(|field| (field.layout(), field.buffer())),
    );
    assert!(erased_value
        .as_refs()
        .fields()
        .into_iter()
        .copied()
        .map(ErasedFieldRef::into_parts)
        .eq([
            ErasedFieldRef::new(optimized_layout[0], i3_bytes).into_parts(),
            ErasedFieldRef::new(optimized_layout[1], i2_bytes).into_parts(),
            ErasedFieldRef::new(optimized_layout[2], i1_bytes).into_parts(),
        ]));

    let value = unsafe { erased_value.into::<(u32, u16, u8)>(&context) };
    assert_eq!(value, (i1, i2, i3));

    let refs = (&i1, &i2, &i3);
    let erased_refs = ErasedSoaRefs::from::<(u32, u16, u8)>(&context, refs);
    assert_eq!(erased_refs.fields().len(), 3);

    assert_eq!(unsafe { erased_refs.fields()[0].into::<u8>() }, &i3);
    assert_eq!(
        erased_refs.fields()[0].into_parts(),
        ErasedFieldRef::from(&i3).into_parts(),
    );

    assert_eq!(unsafe { erased_refs.fields()[1].into::<u16>() }, &i2);
    assert_eq!(
        erased_refs.fields()[1].into_parts(),
        ErasedFieldRef::from(&i2).into_parts(),
    );

    assert_eq!(unsafe { erased_refs.fields()[2].into::<u32>() }, &i1);
    assert_eq!(
        erased_refs.fields()[2].into_parts(),
        ErasedFieldRef::from(&i1).into_parts(),
    );

    assert!(erased_refs
        .fields()
        .into_iter()
        .copied()
        .map(ErasedFieldRef::into_parts)
        .eq([
            ErasedFieldRef::new(optimized_layout[0], i3_bytes).into_parts(),
            ErasedFieldRef::new(optimized_layout[1], i2_bytes).into_parts(),
            ErasedFieldRef::new(optimized_layout[2], i1_bytes).into_parts(),
        ]));

    let refs = unsafe { erased_refs.into::<(u32, u16, u8)>(&context) };
    assert_eq!(refs, (&i1, &i2, &i3));

    let i123 = [1, 2, 3];
    let i456 = [4, 5, 6];
    let i789 = [7, 8, 9];

    let i123_slices = i123.as_slice();
    let i456_slices = i456.as_slice();
    let i789_slices = i789.as_slice();

    let slices = (i123_slices, i456_slices, i789_slices);
    let erased_slices = ErasedSoaSlices::from::<(u32, u16, u8)>(&(), slices);
    assert_eq!(erased_slices.fields().len(), 3);

    assert_eq!(
        unsafe { erased_slices.fields()[0].into::<u8>() },
        i789_slices,
    );
    assert_eq!(
        erased_slices.fields()[0].into_parts(),
        ErasedFieldSlice::from(i789_slices).into_parts(),
    );
    for (idx, r#ref) in erased_slices.fields()[0].iter().enumerate().rev() {
        assert_eq!(unsafe { r#ref.into::<u8>() }, &i789[idx]);
        assert_eq!(
            r#ref.into_parts(),
            ErasedFieldRef::from(&i789[idx]).into_parts(),
        );
    }
    assert_eq!(
        erased_slices.fields()[0]
            .iter()
            .fold(0, |acc, item| acc + unsafe { item.into::<u8>() }),
        24,
    );

    assert_eq!(
        unsafe { erased_slices.fields()[1].into::<u16>() },
        i456_slices,
    );
    assert_eq!(
        erased_slices.fields()[1].into_parts(),
        ErasedFieldSlice::from(i456_slices).into_parts(),
    );
    for (idx, r#ref) in erased_slices.fields()[1].iter().enumerate().rev() {
        assert_eq!(unsafe { r#ref.into::<u16>() }, &i456[idx]);
        assert_eq!(
            r#ref.into_parts(),
            ErasedFieldRef::from(&i456[idx]).into_parts(),
        );
    }
    assert_eq!(
        erased_slices.fields()[1]
            .iter()
            .fold(0, |acc, item| acc + unsafe { item.into::<u16>() }),
        15,
    );

    assert_eq!(
        unsafe { erased_slices.fields()[2].into::<u32>() },
        i123_slices,
    );
    assert_eq!(
        erased_slices.fields()[2].into_parts(),
        ErasedFieldSlice::from(i123_slices).into_parts(),
    );
    for (idx, r#ref) in erased_slices.fields()[2].iter().enumerate().rev() {
        assert_eq!(unsafe { r#ref.into::<u32>() }, &i123[idx]);
        assert_eq!(
            r#ref.into_parts(),
            ErasedFieldRef::from(&i123[idx]).into_parts(),
        );
    }
    assert_eq!(
        erased_slices.fields()[2]
            .iter()
            .fold(0, |acc, item| acc + unsafe { item.into::<u32>() }),
        6,
    );

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
        .fields()
        .into_iter()
        .copied()
        .map(ErasedFieldSlice::into_parts)
        .eq([
            ErasedFieldSlice::new(optimized_layout[0], i789_bytes).into_parts(),
            ErasedFieldSlice::new(optimized_layout[1], i456_bytes).into_parts(),
            ErasedFieldSlice::new(optimized_layout[2], i123_bytes).into_parts(),
        ]));

    for (idx, refs) in erased_slices.iter().enumerate().rev() {
        let target_refs =
            ErasedSoaRefs::from::<(u32, u16, u8)>(&context, (&i123[idx], &i456[idx], &i789[idx]));
        let target_fields = target_refs
            .fields()
            .into_iter()
            .copied()
            .map(ErasedFieldRef::into_parts);
        assert!(refs
            .fields()
            .into_iter()
            .copied()
            .map(ErasedFieldRef::into_parts)
            .eq(target_fields));

        assert_eq!(
            unsafe { refs.into::<(u32, u16, u8)>(&context) },
            (&i123[idx], &i456[idx], &i789[idx]),
        );
    }

    let slices = unsafe { erased_slices.into::<(u32, u16, u8)>(&()) };
    assert_eq!(slices, (i123_slices, i456_slices, i789_slices));
}
