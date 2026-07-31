use gpecs_soa_core::{prelude::*, slice};

type Item = (u32, u16, u8);

#[test]
#[cfg_attr(miri, ignore)]
fn slices_npo() {
    type SlicePtrs<'ctx> = SoaSlicePtrs<'ctx, Item>;
    type SliceMutPtrs<'ctx> = SoaSliceMutPtrs<'ctx, Item>;

    assert_eq!(size_of::<Option<SlicePtrs>>(), size_of::<SlicePtrs>());
    assert_eq!(size_of::<Option<SliceMutPtrs>>(), size_of::<SliceMutPtrs>());

    type Slices<'ctx, 'a> = SoaSlices<'ctx, 'a, Item>;
    type SlicesMut<'ctx, 'a> = SoaSlicesMut<'ctx, 'a, Item>;

    assert_eq!(size_of::<Option<Slices>>(), size_of::<Slices>());
    assert_eq!(size_of::<Option<SlicesMut>>(), size_of::<SlicesMut>());
}

#[test]
#[cfg_attr(miri, ignore)]
fn iter_npo() {
    type RawIter<'ctx> = slice::RawIter<'ctx, Item>;
    type RawIterMut<'ctx> = slice::RawIterMut<'ctx, Item>;

    assert_eq!(size_of::<Option<RawIter>>(), size_of::<RawIter>());
    assert_eq!(size_of::<Option<RawIterMut>>(), size_of::<RawIterMut>());

    type Iter<'ctx, 'a> = slice::Iter<'ctx, 'a, Item>;
    type IterMut<'ctx, 'a> = slice::IterMut<'ctx, 'a, Item>;

    assert_eq!(size_of::<Option<Iter>>(), size_of::<Iter>());
    assert_eq!(size_of::<Option<IterMut>>(), size_of::<IterMut>());
}
