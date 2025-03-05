use std::alloc::Layout;

use gpecs_soa::{
    prelude::*,
    r#dyn::{DynSoa, DynSoaContext},
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
    assert_eq!(dyn_value.as_refs(&dyn_context).as_ref(), [[]]);

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
        [i3_bytes, i2_bytes, i1_bytes],
    );
}
