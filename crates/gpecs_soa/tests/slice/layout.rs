use gpecs_soa::{
    prelude::*,
    slice::{Iter as SoaIter, IterMut as SoaIterMut},
};

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
    type Iter<'ctx, 'a> = SoaIter<'ctx, 'a, (u32, u16, u8)>;

    assert_eq!(size_of::<Option<Iter>>(), size_of::<Iter>());
}

#[test]
#[cfg_attr(miri, ignore)]
fn iter_mut_null_opt() {
    type IterMut<'ctx, 'a> = SoaIterMut<'ctx, 'a, (u32, u16, u8)>;

    assert_eq!(size_of::<Option<IterMut>>(), size_of::<IterMut>());
}
