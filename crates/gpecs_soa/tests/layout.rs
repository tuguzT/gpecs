use std::{alloc::Layout, ptr, slice};

use gpecs_soa::{
    prelude::*,
    r#dyn::{DynSoa, DynSoaContext, DynSoaRefs, DynSoaSlices},
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
fn dyn_context() {
    let field_layouts = [Layout::new::<u8>(), Layout::new::<i16>()];
    let _context = DynSoaContext::<i16>::new(field_layouts);
}

#[test]
#[should_panic = "input alignment must be less than or equal to 1, but got 2"]
#[cfg_attr(miri, ignore)]
fn dyn_context_fail() {
    let field_layouts = [Layout::new::<u8>(), Layout::new::<i16>()];
    let _context = DynSoaContext::<u8>::new(field_layouts);
}

#[test]
#[cfg_attr(miri, ignore)]
fn dyn_context_of() {
    let context = DynSoaContext::of::<()>(&());
    assert_eq!(context.layouts(), [Layout::new::<()>()]);

    let context = DynSoaContext::of::<(u32, u16, u8)>(&());
    let optimized_layout = [
        Layout::new::<u8>(),
        Layout::new::<u16>(),
        Layout::new::<u32>(),
    ];
    assert_eq!(context.layouts(), optimized_layout);
}

#[test]
#[cfg_attr(miri, ignore)]
fn dyn_value() {
    let context = ();
    let dyn_context = DynSoaContext::of::<()>(&context);

    let value = ();
    let dyn_value = DynSoa::from(&context, value);
    assert_eq!(dyn_value.layouts(), [Layout::new::<()>()]);
    assert_eq!(
        dyn_value.as_refs(&dyn_context).as_ref(),
        [(Layout::new::<()>(), [].as_slice())],
    );

    let value = unsafe { dyn_value.into::<()>(&context) };
    assert_eq!(value, ());

    let dyn_context = DynSoaContext::of::<(u32, u16, u8)>(&());

    let i1 = 1u32;
    let i2 = 2u16;
    let i3 = 3u8;
    let value = (i1, i2, i3);
    let dyn_value = DynSoa::from(&(), value);

    let optimized_layout = [
        Layout::new::<u8>(),
        Layout::new::<u16>(),
        Layout::new::<u32>(),
    ];
    assert_eq!(dyn_value.layouts(), optimized_layout);

    let i1_bytes = i1.to_ne_bytes();
    let i2_bytes = i2.to_ne_bytes();
    let i3_bytes = i3.to_ne_bytes();

    let i1_bytes = i1_bytes.as_slice();
    let i2_bytes = i2_bytes.as_slice();
    let i3_bytes = i3_bytes.as_slice();
    assert_eq!(
        dyn_value.as_refs(&dyn_context).as_ref(),
        [
            (optimized_layout[0], i3_bytes),
            (optimized_layout[1], i2_bytes),
            (optimized_layout[2], i1_bytes),
        ],
    );

    let value = unsafe { dyn_value.into::<(u32, u16, u8)>(&context) };
    assert_eq!(value, (i1, i2, i3));

    let refs = (&i1, &i2, &i3);
    let refs = DynSoaRefs::from::<(u32, u16, u8)>(&context, refs);
    assert_eq!(
        refs.as_ref(),
        [
            (optimized_layout[0], i3_bytes),
            (optimized_layout[1], i2_bytes),
            (optimized_layout[2], i1_bytes),
        ],
    );

    let refs = unsafe { refs.into::<(u32, u16, u8)>(&context) };
    assert_eq!(refs, (&i1, &i2, &i3));

    let i123 = [1u32, 2, 3];
    let i456 = [4u16, 5, 6];
    let i789 = [7u8, 8, 9];

    let i123_slices = i123.as_slice();
    let i456_slices = i456.as_slice();
    let i789_slices = i789.as_slice();

    let slices = (i123_slices, i456_slices, i789_slices);
    let slices = DynSoaSlices::from::<(u32, u16, u8)>(&(), slices);

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
        slices.as_ref(),
        [
            (optimized_layout[0], i789_bytes),
            (optimized_layout[1], i456_bytes),
            (optimized_layout[2], i123_bytes),
        ],
    );

    let slices = unsafe { slices.into::<(u32, u16, u8)>(&()) };
    assert_eq!(slices, (i123_slices, i456_slices, i789_slices));
}
