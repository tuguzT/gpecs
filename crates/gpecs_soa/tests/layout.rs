use std::{alloc::Layout, ptr, slice};

use gpecs_soa::{
    erased::{
        ErasedFieldRef, ErasedFieldSlice, ErasedSoa, ErasedSoaContext, ErasedSoaRefs,
        ErasedSoaSlices,
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
#[should_panic = "input alignment must be less than or equal to 1, but got 2"]
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
    assert_eq!(erased_value.layouts(), [Layout::new::<()>()]);
    assert_eq!(
        erased_value.as_refs().as_ref(),
        [ErasedFieldRef::new(Layout::new::<()>(), [].as_slice())],
    );

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
    assert_eq!(erased_value.layouts(), optimized_layout);

    let erased_refs = erased_value.as_refs();
    assert_eq!(erased_refs.as_ref().len(), 3);

    assert_eq!(unsafe { erased_refs.as_ref()[0].into::<u8>() }, &i3);
    assert_eq!(erased_refs.as_ref()[0], ErasedFieldRef::from(&i3));

    assert_eq!(unsafe { erased_refs.as_ref()[1].into::<u16>() }, &i2);
    assert_eq!(erased_refs.as_ref()[1], ErasedFieldRef::from(&i2));

    assert_eq!(unsafe { erased_refs.as_ref()[2].into::<u32>() }, &i1);
    assert_eq!(erased_refs.as_ref()[2], ErasedFieldRef::from(&i1));

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
    assert_eq!(
        erased_refs.as_ref(),
        [
            ErasedFieldRef::new(optimized_layout[0], i3_bytes),
            ErasedFieldRef::new(optimized_layout[1], i2_bytes),
            ErasedFieldRef::new(optimized_layout[2], i1_bytes),
        ],
    );

    let erased_fields = erased_value.into_fields();
    assert_eq!(
        erased_fields.as_ref(),
        [
            (optimized_layout[0], i3_bytes.into()),
            (optimized_layout[1], i2_bytes.into()),
            (optimized_layout[2], i1_bytes.into()),
        ],
    );

    let erased_value = ErasedSoa::new(
        erased_fields
            .iter()
            .map(|(field_layout, field)| (*field_layout, field.as_ref())),
    );
    assert_eq!(
        erased_value.as_refs().as_ref(),
        [
            ErasedFieldRef::new(optimized_layout[0], i3_bytes),
            ErasedFieldRef::new(optimized_layout[1], i2_bytes),
            ErasedFieldRef::new(optimized_layout[2], i1_bytes),
        ],
    );

    let value = unsafe { erased_value.into::<(u32, u16, u8)>(&context) };
    assert_eq!(value, (i1, i2, i3));

    let refs = (&i1, &i2, &i3);
    let erased_refs = ErasedSoaRefs::from::<(u32, u16, u8)>(&context, refs);
    assert_eq!(erased_refs.as_ref().len(), 3);

    assert_eq!(unsafe { erased_refs.as_ref()[0].into::<u8>() }, &i3);
    assert_eq!(erased_refs.as_ref()[0], ErasedFieldRef::from(&i3));

    assert_eq!(unsafe { erased_refs.as_ref()[1].into::<u16>() }, &i2);
    assert_eq!(erased_refs.as_ref()[1], ErasedFieldRef::from(&i2));

    assert_eq!(unsafe { erased_refs.as_ref()[2].into::<u32>() }, &i1);
    assert_eq!(erased_refs.as_ref()[2], ErasedFieldRef::from(&i1));

    assert_eq!(
        erased_refs.as_ref(),
        [
            ErasedFieldRef::new(optimized_layout[0], i3_bytes),
            ErasedFieldRef::new(optimized_layout[1], i2_bytes),
            ErasedFieldRef::new(optimized_layout[2], i1_bytes),
        ],
    );

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
    assert_eq!(erased_slices.as_ref().len(), 3);

    assert_eq!(
        unsafe { erased_slices.as_ref()[0].into::<u8>() },
        i789_slices,
    );
    assert_eq!(
        erased_slices.as_ref()[0],
        ErasedFieldSlice::from(i789_slices),
    );
    for (idx, r#ref) in erased_slices.as_ref()[0].into_iter().enumerate() {
        assert_eq!(unsafe { r#ref.into::<u8>() }, &i789[idx]);
        assert_eq!(r#ref, ErasedFieldRef::from(&i789[idx]));
    }

    assert_eq!(
        unsafe { erased_slices.as_ref()[1].into::<u16>() },
        i456_slices,
    );
    assert_eq!(
        erased_slices.as_ref()[1],
        ErasedFieldSlice::from(i456_slices),
    );
    for (idx, r#ref) in erased_slices.as_ref()[1].into_iter().enumerate() {
        assert_eq!(unsafe { r#ref.into::<u16>() }, &i456[idx]);
        assert_eq!(r#ref, ErasedFieldRef::from(&i456[idx]));
    }

    assert_eq!(
        unsafe { erased_slices.as_ref()[2].into::<u32>() },
        i123_slices,
    );
    assert_eq!(
        erased_slices.as_ref()[2],
        ErasedFieldSlice::from(i123_slices),
    );
    for (idx, r#ref) in erased_slices.as_ref()[2].into_iter().enumerate() {
        assert_eq!(unsafe { r#ref.into::<u32>() }, &i123[idx]);
        assert_eq!(r#ref, ErasedFieldRef::from(&i123[idx]));
    }

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
    assert_eq!(
        erased_slices.as_ref(),
        [
            ErasedFieldSlice::new(optimized_layout[0], i789_bytes),
            ErasedFieldSlice::new(optimized_layout[1], i456_bytes),
            ErasedFieldSlice::new(optimized_layout[2], i123_bytes),
        ],
    );

    let slices = unsafe { erased_slices.into::<(u32, u16, u8)>(&()) };
    assert_eq!(slices, (i123_slices, i456_slices, i789_slices));
}
