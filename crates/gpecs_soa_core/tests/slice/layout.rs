use gpecs_soa_core::{prelude::*, ptrs, slices};

type Item = (u32, u16, u8);

#[test]
#[cfg_attr(miri, ignore)]
fn slices_npo() {
    type ViewPtrs<'ctx> = SoaViewPtrs<'ctx, Item>;
    type ViewMutPtrs<'ctx> = SoaViewMutPtrs<'ctx, Item>;

    assert_eq!(size_of::<Option<ViewPtrs>>(), size_of::<ViewPtrs>());
    assert_eq!(size_of::<Option<ViewMutPtrs>>(), size_of::<ViewMutPtrs>());

    type View<'ctx, 'a> = SoaView<'ctx, 'a, Item>;
    type ViewMut<'ctx, 'a> = SoaViewMut<'ctx, 'a, Item>;

    assert_eq!(size_of::<Option<View>>(), size_of::<View>());
    assert_eq!(size_of::<Option<ViewMut>>(), size_of::<ViewMut>());
}

#[test]
#[cfg_attr(miri, ignore)]
fn iter_npo() {
    type IterPtrs<'ctx> = ptrs::IterPtrs<'ctx, Item>;
    type IterMutPtrs<'ctx> = ptrs::IterMutPtrs<'ctx, Item>;

    assert_eq!(size_of::<Option<IterPtrs>>(), size_of::<IterPtrs>());
    assert_eq!(size_of::<Option<IterMutPtrs>>(), size_of::<IterMutPtrs>());

    type Iter<'ctx, 'a> = slices::Iter<'ctx, 'a, Item>;
    type IterMut<'ctx, 'a> = slices::IterMut<'ctx, 'a, Item>;

    assert_eq!(size_of::<Option<Iter>>(), size_of::<Iter>());
    assert_eq!(size_of::<Option<IterMut>>(), size_of::<IterMut>());
}
