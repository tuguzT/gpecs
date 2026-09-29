use std::{array, cmp::Reverse, convert::identity, hash::BuildHasher};

use gpecs_soa_core::{prelude::*, slices};
use itertools::assert_equal;
use rustc_hash::FxBuildHasher;

use crate::common::{ZST1, ZST2, ZST3};

#[test]
fn empty() {
    type Item = (u32, u128, u8, ());
    type View<'ctx, 'a> = SoaView<'ctx, 'a, Item>;
    type ViewMut<'ctx, 'a> = SoaViewMut<'ctx, 'a, Item>;

    let context = Default::default();

    let view = View::new(&context, (&[], &[], &[], &[]));
    assert!(view.is_empty());
    assert_eq!(view.get(0), None);
    assert_eq!(view.as_ref(), &view);
    assert_eq!(view, View::empty(&context));
    assert_eq!(format!("{view:?}"), "SoaView(([], [], [], []))");

    let mut iter = view.into_iter();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    assert_equal(view, &view);

    let slices = view.into_slices();
    assert_eq!(
        slices,
        ([].as_slice(), [].as_slice(), [].as_slice(), [].as_slice()),
    );

    let mut view_mut = ViewMut::new(&context, (&mut [], &mut [], &mut [], &mut []));
    assert!(view_mut.is_empty());
    assert_eq!(view_mut.get_mut(0), None);
    assert_eq!(view_mut.as_ref(), &view_mut);
    assert_eq!(view_mut, ViewMut::empty(&context));
    assert_eq!(view_mut.as_mut(), &mut ViewMut::empty(&context));
    assert_eq!(format!("{view_mut:?}"), "SoaViewMut(([], [], [], []))");

    let mut iter = view_mut.iter_mut();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    let eq_mut = [];
    assert_equal(&mut view_mut, eq_mut);

    let slices_mut = view_mut.into_mut_slices();
    assert_eq!(
        slices_mut,
        (
            [].as_mut_slice(),
            [].as_mut_slice(),
            [].as_mut_slice(),
            [].as_mut_slice(),
        ),
    );

    let mut view_mut = ViewMut::new(&context, slices_mut);

    let permutation: [_; 0] = array::from_fn(identity);
    view_mut.sort_unstable_with_permutation(permutation);
    assert!(view_mut.is_empty());

    let view = view_mut.into_view();
    assert!(view.is_empty());
    assert_eq!(view.get(0), None);
    assert_eq!(format!("{view:?}"), "SoaView(([], [], [], []))");
}

#[test]
fn empty_unit() {
    type View<'ctx, 'a> = SoaView<'ctx, 'a, ()>;
    type ViewMut<'ctx, 'a> = SoaViewMut<'ctx, 'a, ()>;

    let context = Default::default();

    let view = View::new(&context, &[]);
    assert!(view.is_empty());
    assert_eq!(view.get(0), None);
    assert_eq!(view.as_ref(), []);
    assert_eq!(view, View::empty(&context));
    assert_eq!(format!("{view:?}"), "SoaView([])");

    let mut iter = view.into_iter();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    assert_equal(view, &view);

    let slices = view.into_slices();
    assert_eq!(slices, []);

    let mut view_mut = ViewMut::new(&context, &mut []);
    assert!(view_mut.is_empty());
    assert_eq!(view_mut.get_mut(0), None);
    assert_eq!(view_mut.as_ref(), []);
    assert_eq!(view_mut, ViewMut::empty(&context));
    assert_eq!(format!("{view_mut:?}"), "SoaViewMut([])");

    let mut iter = view_mut.iter_mut();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    let eq_mut = &mut [];
    assert_equal(&mut view_mut, eq_mut);

    let slices_mut = view_mut.into_mut_slices();
    assert_eq!(slices_mut, []);

    let mut view_mut = ViewMut::new(&context, slices_mut);

    let permutation: [_; 0] = array::from_fn(identity);
    view_mut.sort_unstable_with_permutation(permutation);
    assert!(view_mut.is_empty());

    let view = view_mut.into_view();
    assert!(view.is_empty());
    assert_eq!(view.get(0), None);
    assert_eq!(format!("{view:?}"), "SoaView([])");
}

#[test]
fn empty_identity() {
    type Item = Identity<u128>;
    type View<'ctx, 'a> = SoaView<'ctx, 'a, Item>;
    type ViewMut<'ctx, 'a> = SoaViewMut<'ctx, 'a, Item>;

    let context = Default::default();

    let view = View::new(&context, &[]);
    assert!(view.is_empty());
    assert_eq!(view.get(0), None);
    assert_eq!(view.as_ref(), []);
    assert_eq!(view, View::empty(&context));
    assert_eq!(format!("{view:?}"), "SoaView([])");

    let mut iter = view.into_iter();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    assert_equal(view, &view);

    let slices = view.into_slices();
    assert_eq!(slices, []);

    let mut view_mut = ViewMut::new(&context, &mut []);
    assert!(view_mut.is_empty());
    assert_eq!(view_mut.get_mut(0), None);
    assert_eq!(view_mut.as_mut(), []);
    assert_eq!(view_mut, ViewMut::empty(&context));
    assert_eq!(format!("{view_mut:?}"), "SoaViewMut([])");

    let mut iter = view_mut.iter_mut();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    let eq_mut: [&mut Identity<_>; _] = [];
    assert_equal(&mut view_mut, eq_mut);

    let slices_mut = view_mut.into_mut_slices();
    assert_eq!(slices_mut, []);

    let mut view_mut = ViewMut::new(&context, slices_mut);

    let permutation: [_; 0] = array::from_fn(identity);
    view_mut.sort_unstable_with_permutation(permutation);
    assert!(view_mut.is_empty());

    let view = view_mut.into_view();
    assert!(view.is_empty());
    assert_eq!(view.get(0), None);
    assert_eq!(format!("{view:?}"), "SoaView([])");
}

#[test]
fn empty_zst() {
    type View<'ctx, 'a> = SoaView<'ctx, 'a, (ZST1, ZST2, ZST3)>;
    type ViewMut<'ctx, 'a> = SoaViewMut<'ctx, 'a, (ZST1, ZST2, ZST3)>;

    let context = Default::default();

    let view = View::new(&context, (&[], &[], &[]));
    assert!(view.is_empty());
    assert_eq!(view.get(0), None);
    assert_eq!(view.as_ref(), &view);
    assert_eq!(view, View::empty(&context));
    assert_eq!(format!("{view:?}"), "SoaView(([], [], []))");

    let mut iter = view.into_iter();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    assert_equal(view, &view);

    let slices = view.into_slices();
    assert_eq!(slices, ([].as_slice(), [].as_slice(), [].as_slice()));

    let mut view_mut = ViewMut::new(&context, (&mut [], &mut [], &mut []));
    assert!(view_mut.is_empty());
    assert_eq!(view_mut.get_mut(0), None);
    assert_eq!(view_mut.as_ref(), &view_mut);
    assert_eq!(view_mut, ViewMut::empty(&context));
    assert_eq!(view_mut.as_mut(), &mut ViewMut::empty(&context));
    assert_eq!(format!("{view_mut:?}"), "SoaViewMut(([], [], []))");

    let mut iter = view_mut.iter_mut();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    let eq_mut = [];
    assert_equal(&mut view_mut, eq_mut);

    let slices_mut = view_mut.into_mut_slices();
    assert_eq!(
        slices_mut,
        ([].as_mut_slice(), [].as_mut_slice(), [].as_mut_slice()),
    );

    let mut view_mut = ViewMut::new(&context, slices_mut);

    let permutation: [_; 0] = array::from_fn(identity);
    view_mut.sort_unstable_with_permutation(permutation);
    assert!(view_mut.is_empty());

    let view = view_mut.into_view();
    assert!(view.is_empty());
    assert_eq!(view.get(0), None);
    assert_eq!(format!("{view:?}"), "SoaView(([], [], []))");
}

#[test]
fn one_item() {
    type Item = (u32, u128, u8, ());
    type View<'ctx, 'a> = SoaView<'ctx, 'a, Item>;
    type ViewMut<'ctx, 'a> = SoaViewMut<'ctx, 'a, Item>;

    let context = Default::default();
    let mut u8s = [1];
    let mut u64s = [2];
    let mut u16s = [3];
    let mut units = [()];

    let view = View::new(&context, (&u8s, &u64s, &u16s, &units));
    assert_eq!(view.len(), 1);
    assert!(view.contains((&1, &2, &3, &())));

    assert_eq!(
        view.as_slices(),
        (
            u8s.as_slice(),
            u64s.as_slice(),
            u16s.as_slice(),
            units.as_slice(),
        ),
    );
    assert_eq!(
        view.into_index(..),
        (
            u8s.as_slice(),
            u64s.as_slice(),
            u16s.as_slice(),
            units.as_slice(),
        ),
    );
    assert_eq!(
        view.index(0..),
        (
            u8s.as_slice(),
            u64s.as_slice(),
            u16s.as_slice(),
            units.as_slice(),
        ),
    );
    assert_eq!(
        view.index(..0),
        ([].as_slice(), [].as_slice(), [].as_slice(), [].as_slice()),
    );
    assert_eq!(view.index(0), (&1, &2, &3, &()));
    assert_eq!(view.as_ref(), &view);
    assert_ne!(view, View::empty(&context));
    assert_eq!(format!("{view:?}"), "SoaView(([1], [2], [3], [()]))");

    let mut iter = view.into_iter();
    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some((&1, &2, &3, &())));
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    assert_equal(view, &view);

    let mut view_mut = ViewMut::new(&context, (&mut u8s, &mut u64s, &mut u16s, &mut units));
    assert_eq!(view_mut.len(), 1);
    assert_eq!(
        view_mut.index_mut(..),
        (
            [1].as_mut_slice(),
            [2].as_mut_slice(),
            [3].as_mut_slice(),
            [()].as_mut_slice(),
        ),
    );
    assert_eq!(
        view_mut.index_mut(..0),
        (
            [].as_mut_slice(),
            [].as_mut_slice(),
            [].as_mut_slice(),
            [].as_mut_slice(),
        ),
    );
    assert_eq!(view_mut.index_mut(0), (&mut 1, &mut 2, &mut 3, &mut ()));
    assert!(view_mut.contains((&1, &2, &3, &())));

    let eq_mut = [(&mut 1, &mut 2, &mut 3, &mut ())];
    assert_equal(&mut view_mut, eq_mut);

    view_mut.copy_from_slices(&View::new(&context, (&[0], &[0], &[0], &[()])));
    assert_eq!(view_mut.len(), 1);
    assert_eq!(view_mut.index_mut(0), (&mut 0, &mut 0, &mut 0, &mut ()));
    assert!(!view_mut.contains((&1, &2, &3, &())));
    assert_eq!(
        view_mut.as_mut_slices(),
        (
            [0].as_mut_slice(),
            [0].as_mut_slice(),
            [0].as_mut_slice(),
            [()].as_mut_slice(),
        ),
    );
    assert_eq!(view_mut.as_ref(), &view_mut);
    assert_ne!(view_mut, ViewMut::empty(&context));
    assert_ne!(view_mut.as_mut(), &mut ViewMut::empty(&context));
    assert_eq!(format!("{view_mut:?}"), "SoaViewMut(([0], [0], [0], [()]))",);

    let permutation: [_; 1] = array::from_fn(identity);
    view_mut.sort_unstable_with_permutation(permutation);
    assert_eq!(
        view_mut.as_mut_slices(),
        (
            [0].as_mut_slice(),
            [0].as_mut_slice(),
            [0].as_mut_slice(),
            [()].as_mut_slice(),
        ),
    );

    let view = view_mut.into_view();
    assert_eq!(view.len(), 1);
    assert_eq!(view.index(0), (&0, &0, &0, &()));
    assert_eq!(
        view.as_slices(),
        (
            [0].as_slice(),
            [0].as_slice(),
            [0].as_slice(),
            [()].as_slice(),
        ),
    );

    let mut iter = view.into_iter();
    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some((&0, &0, &0, &())));
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    assert_equal(view, &view);

    assert_eq!(u8s, [0]);
    assert_eq!(u64s, [0]);
    assert_eq!(u16s, [0]);
    assert_eq!(units, [()]);
}

#[test]
fn one_item_unit() {
    type View<'ctx, 'a> = SoaView<'ctx, 'a, ()>;
    type ViewMut<'ctx, 'a> = SoaViewMut<'ctx, 'a, ()>;

    let context = Default::default();
    let mut units = [()];

    let view = View::new(&context, &units);
    assert_eq!(view.len(), 1);
    assert!(view.contains(&()));

    assert_eq!(view.as_slices(), units);
    assert_eq!(view.into_index(..), units);
    assert_eq!(view.index(0..), units);
    assert_eq!(view.index(..0), []);
    assert_eq!(view.get(0), Some(&()));
    assert_ne!(view, View::empty(&context));
    assert_eq!(format!("{view:?}"), "SoaView([()])");

    let mut iter = view.into_iter();
    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some(&()));
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    assert_equal(view, &view);

    let mut view_mut = ViewMut::new(&context, &mut units);
    assert_eq!(view_mut.len(), 1);
    assert_eq!(view_mut.index_mut(..), [()]);
    assert_eq!(view_mut.index_mut(..0), []);
    assert_eq!(view_mut.index_mut(0), &mut ());
    assert!(view_mut.contains(&()));

    let permutation: [_; 1] = array::from_fn(identity);
    view_mut.sort_unstable_with_permutation(permutation);
    assert_eq!(view_mut.as_mut_slices(), [()]);

    let eq_mut = &mut [()];
    assert_equal(&mut view_mut, eq_mut);

    view_mut.copy_from_slices(&View::new(&context, &[()]));
    assert_eq!(view_mut.len(), 1);
    assert_eq!(view_mut.as_mut_slices(), [()]);
    assert_eq!(view_mut.index_mut(0), &mut ());
    assert_ne!(view_mut, ViewMut::empty(&context));
    assert_eq!(format!("{view_mut:?}"), "SoaViewMut([()])");

    let view = view_mut.into_view();
    assert_eq!(view.len(), 1);
    assert_eq!(view.index(0), &());
    assert_eq!(view.as_slices(), [()]);

    let mut iter = view.into_iter();
    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some(&()));
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    assert_equal(view, &view);

    assert_eq!(units, [()]);
}

#[test]
fn one_item_identity() {
    type Item = Identity<u128>;
    type View<'ctx, 'a> = SoaView<'ctx, 'a, Item>;
    type ViewMut<'ctx, 'a> = SoaViewMut<'ctx, 'a, Item>;

    let context = Default::default();
    let mut data = [1.into()];

    let view = View::new(&context, &data);
    assert_eq!(view.len(), 1);
    assert!(view.contains(&1.into()));

    assert_eq!(view.as_slices(), data);
    assert_eq!(view[0..], data);
    assert_eq!(view[..0], []);
    assert_eq!(&view[0], &1.into());
    assert_ne!(view, View::empty(&context));
    assert_eq!(format!("{view:?}"), "SoaView([Identity(1)])");

    let mut iter = view.into_iter();
    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some(&1.into()));
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    assert_equal(view, &view);

    let mut view_mut = ViewMut::new(&context, &mut data);
    assert_eq!(view_mut.len(), 1);
    assert_eq!(view_mut[..0], []);
    assert_eq!(&mut view_mut[0], &mut 1.into());
    assert!(view_mut.contains(&1.into()));

    let permutation: [_; 1] = array::from_fn(identity);
    view_mut.sort_unstable_with_permutation(permutation);
    assert_eq!(view_mut.as_ref(), [1.into()]);

    let eq_mut = [&mut 1.into()];
    assert_equal(&mut view_mut, eq_mut);

    view_mut.copy_from_slices(&View::new(&context, &[0.into()]));
    assert_eq!(view_mut.len(), 1);
    assert_eq!(&mut view_mut[0], &mut 0.into());
    assert!(!view_mut.contains(&1.into()));

    assert_eq!(view_mut.as_mut_slices(), [0.into()]);
    assert_ne!(view_mut, ViewMut::empty(&context));
    assert_eq!(format!("{view_mut:?}"), "SoaViewMut([Identity(0)])");

    let view = view_mut.into_view();
    assert_eq!(view.len(), 1);
    assert_eq!(&view[0], &0.into());
    assert_eq!(view.as_slices(), [0.into()]);

    let mut iter = view.into_iter();
    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some(&0.into()));
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    assert_equal(view, &view);

    assert_eq!(data, [0.into()]);
}

#[test]
fn one_item_zst() {
    type View<'ctx, 'a> = SoaView<'ctx, 'a, (ZST1, ZST2, ZST3)>;
    type ViewMut<'ctx, 'a> = SoaViewMut<'ctx, 'a, (ZST1, ZST2, ZST3)>;

    let context = Default::default();
    let mut zst1s = [ZST1];
    let mut zst2s = [ZST2(())];
    let mut zst3s = [ZST3 { empty: () }];

    let view = View::new(&context, (&zst1s, &zst2s, &zst3s));
    assert_eq!(view.len(), 1);
    assert!(view.contains((&ZST1, &ZST2(()), &ZST3 { empty: () })));

    assert_eq!(
        view.as_slices(),
        (zst1s.as_slice(), zst2s.as_slice(), zst3s.as_slice()),
    );
    assert_eq!(
        view.into_index(..),
        (zst1s.as_slice(), zst2s.as_slice(), zst3s.as_slice()),
    );
    assert_eq!(
        view.index(0..),
        (zst1s.as_slice(), zst2s.as_slice(), zst3s.as_slice()),
    );
    assert_eq!(
        view.index(..0),
        ([].as_slice(), [].as_slice(), [].as_slice()),
    );
    assert_eq!(view.get(0), Some((&ZST1, &ZST2(()), &ZST3 { empty: () })));
    assert_eq!(view.as_ref(), &view);
    assert_ne!(view, View::empty(&context));
    assert_eq!(
        format!("{view:?}"),
        "SoaView(([ZST1], [ZST2(())], [ZST3 { empty: () }]))",
    );

    let mut iter = view.into_iter();
    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some((&ZST1, &ZST2(()), &ZST3 { empty: () })));
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    assert_equal(view, &view);

    let mut view_mut = ViewMut::new(&context, (&mut zst1s, &mut zst2s, &mut zst3s));
    assert_eq!(view_mut.len(), 1);
    assert_eq!(
        view_mut.index_mut(..),
        (
            [ZST1; 1].as_mut_slice(),
            [ZST2(()); 1].as_mut_slice(),
            [ZST3 { empty: () }; 1].as_mut_slice(),
        ),
    );
    assert_eq!(
        view_mut.index_mut(..0),
        ([].as_mut_slice(), [].as_mut_slice(), [].as_mut_slice()),
    );
    assert_eq!(
        view_mut.index_mut(0),
        (&mut ZST1, &mut ZST2(()), &mut ZST3 { empty: () }),
    );
    assert!(view_mut.contains((&ZST1, &ZST2(()), &ZST3 { empty: () })));

    let permutation: [_; 1] = array::from_fn(identity);
    view_mut.sort_unstable_with_permutation(permutation);
    assert_eq!(
        view_mut.as_mut_slices(),
        (
            [ZST1].as_mut_slice(),
            [ZST2(())].as_mut_slice(),
            [ZST3 { empty: () }].as_mut_slice(),
        ),
    );

    let eq_mut = [(&mut ZST1, &mut ZST2(()), &mut ZST3 { empty: () })];
    assert_equal(&mut view_mut, eq_mut);

    view_mut.copy_from_slices(&View::new(
        &context,
        (&[ZST1], &[ZST2(())], &[ZST3 { empty: () }]),
    ));
    assert_eq!(view_mut.len(), 1);
    assert_eq!(
        view_mut.as_mut_slices(),
        (
            [ZST1].as_mut_slice(),
            [ZST2(())].as_mut_slice(),
            [ZST3 { empty: () }].as_mut_slice(),
        ),
    );
    assert_eq!(
        view_mut.index_mut(0),
        (&mut ZST1, &mut ZST2(()), &mut ZST3 { empty: () }),
    );
    assert_eq!(view_mut.as_ref(), &view_mut);
    assert_ne!(view_mut, ViewMut::empty(&context));
    assert_ne!(view_mut.as_mut(), &mut ViewMut::empty(&context));
    assert_eq!(
        format!("{view_mut:?}"),
        "SoaViewMut(([ZST1], [ZST2(())], [ZST3 { empty: () }]))",
    );

    let view = view_mut.into_view();
    assert_eq!(view.len(), 1);
    assert_eq!(view.index(0), (&ZST1, &ZST2(()), &ZST3 { empty: () }));
    assert_eq!(
        view.as_slices(),
        (
            [ZST1].as_slice(),
            [ZST2(())].as_slice(),
            [ZST3 { empty: () }].as_slice(),
        ),
    );

    let mut iter = view.into_iter();
    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some((&ZST1, &ZST2(()), &ZST3 { empty: () })));
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    assert_equal(view, &view);

    assert_eq!(zst1s, [ZST1]);
    assert_eq!(zst2s, [ZST2(())]);
    assert_eq!(zst3s, [ZST3 { empty: () }]);
}

#[test]
fn three_items() {
    type Item = (u16, String, u128, ());
    type View<'ctx, 'a> = SoaView<'ctx, 'a, Item>;
    type ViewMut<'ctx, 'a> = SoaViewMut<'ctx, 'a, Item>;

    let context = Default::default();
    let mut u8s = [1, 2, 3];
    let mut strings = ["4".into(), "5".into(), "6".into()];
    let mut u64s = [7, 8, 9];
    let mut units = [(), (), ()];

    let mut view_mut = ViewMut::new(&context, (&mut u8s, &mut strings, &mut u64s, &mut units));
    assert_eq!(view_mut.len(), 3);
    assert_eq!(
        view_mut.index_mut(0),
        (&mut 1, &mut "4".into(), &mut 7, &mut ()),
    );
    assert_eq!(
        view_mut.index_mut(1),
        (&mut 2, &mut "5".into(), &mut 8, &mut ()),
    );
    assert_eq!(
        view_mut.index_mut(2),
        (&mut 3, &mut "6".into(), &mut 9, &mut ()),
    );
    assert_eq!(view_mut.get_mut(3), None);

    assert_eq!(
        view_mut.as_mut_slices(),
        (
            [1, 2, 3].as_mut_slice(),
            ["4".into(), "5".into(), "6".into()].as_mut_slice(),
            [7, 8, 9].as_mut_slice(),
            [(), (), ()].as_mut_slice(),
        ),
    );
    assert_eq!(
        view_mut.index_mut(..),
        (
            [1, 2, 3].as_mut_slice(),
            ["4".into(), "5".into(), "6".into()].as_mut_slice(),
            [7, 8, 9].as_mut_slice(),
            [(), (), ()].as_mut_slice(),
        ),
    );
    assert_eq!(
        view_mut.index_mut(..1),
        (
            [1].as_mut_slice(),
            ["4".into()].as_mut_slice(),
            [7].as_mut_slice(),
            [()].as_mut_slice(),
        ),
    );
    assert_eq!(
        view_mut.index(1..),
        (
            [2, 3].as_slice(),
            ["5".into(), "6".into()].as_slice(),
            [8, 9].as_slice(),
            [(), ()].as_slice(),
        ),
    );
    assert_eq!(view_mut.as_ref(), &view_mut);
    assert_ne!(view_mut, ViewMut::empty(&context));
    assert_ne!(view_mut.as_mut(), &mut ViewMut::empty(&context));
    assert_eq!(
        format!("{view_mut:?}"),
        r#"SoaViewMut(([1, 2, 3], ["4", "5", "6"], [7, 8, 9], [(), (), ()]))"#,
    );

    let mut iter = view_mut.iter_mut();
    assert_eq!(iter.len(), 3);
    assert_eq!(
        iter.next(),
        Some((&mut 1, &mut "4".into(), &mut 7, &mut ())),
    );

    assert_eq!(iter.len(), 2);
    assert_eq!(
        iter.next_back(),
        Some((&mut 3, &mut "6".into(), &mut 9, &mut ())),
    );

    assert_eq!(iter.len(), 1);
    assert_eq!(
        iter.next(),
        Some((&mut 2, &mut "5".into(), &mut 8, &mut ())),
    );

    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);

    let eq_mut = [
        (&mut 1, &mut "4".into(), &mut 7, &mut ()),
        (&mut 2, &mut "5".into(), &mut 8, &mut ()),
        (&mut 3, &mut "6".into(), &mut 9, &mut ()),
    ];
    assert_equal(&mut view_mut, eq_mut);

    let first = View::new(&context, view_mut.index(..=1));
    let second = View::new(&context, view_mut.index(1..));

    assert_ne!(first.as_slices(), second.as_slices());
    assert_ne!(first, second);

    assert!(first.as_slices() < second.as_slices());
    assert!(first < second);

    assert_eq!(
        first.cmp(&second),
        first.as_slices().cmp(&second.as_slices()),
    );

    let hasher = FxBuildHasher::default();
    assert_ne!(
        hasher.hash_one(first.as_slices()),
        hasher.hash_one(second.as_slices()),
    );
    assert_ne!(hasher.hash_one(&first), hasher.hash_one(&second));

    assert_eq!(hasher.hash_one(first.as_slices()), hasher.hash_one(&first));
    assert_eq!(
        hasher.hash_one(second.as_slices()),
        hasher.hash_one(&second),
    );

    let mut sub_view = ViewMut::new(&context, view_mut.index_mut(1..));
    assert_eq!(sub_view.len(), 2);
    assert_eq!(sub_view.index(0), (&2, &"5".into(), &8, &()));
    assert_eq!(sub_view.index(1), (&3, &"6".into(), &9, &()));
    assert_eq!(sub_view.get(2), None);
    assert_eq!(
        sub_view.as_slices(),
        (
            [2, 3].as_slice(),
            ["5".into(), "6".into()].as_slice(),
            [8, 9].as_slice(),
            [(), ()].as_slice(),
        ),
    );

    let mut gr_u8s = [2, 3];
    let mut gr_strings = ["5".into(), "6".into()];
    let mut gr_u64s = [8, 42]; // the last one is greater
    let mut gr_units = [(), ()];
    let mut gr_slices = ViewMut::new(
        &context,
        (&mut gr_u8s, &mut gr_strings, &mut gr_u64s, &mut gr_units),
    );

    assert_ne!(sub_view.as_mut_slices(), gr_slices.as_mut_slices());
    assert_ne!(sub_view, gr_slices);

    assert!(sub_view.as_mut_slices() < gr_slices.as_mut_slices());
    assert!(sub_view < gr_slices);

    assert_eq!(
        sub_view.cmp(&gr_slices),
        sub_view.as_mut_slices().cmp(&gr_slices.as_mut_slices()),
    );

    let hasher = FxBuildHasher::default();
    assert_ne!(
        hasher.hash_one(sub_view.as_mut_slices()),
        hasher.hash_one(gr_slices.as_mut_slices()),
    );
    assert_ne!(hasher.hash_one(&sub_view), hasher.hash_one(&gr_slices));

    assert_eq!(
        hasher.hash_one(sub_view.as_mut_slices()),
        hasher.hash_one(&sub_view),
    );
    assert_eq!(
        hasher.hash_one(gr_slices.as_mut_slices()),
        hasher.hash_one(&gr_slices),
    );

    sub_view.clone_from_slices(&View::new(
        &context,
        (&[0, 0], &["0".into(), "0".into()], &[0, 0], &[(), ()]),
    ));
    assert_eq!(sub_view.len(), 2);
    assert_eq!(
        sub_view.as_slices(),
        (
            [0, 0].as_slice(),
            ["0".into(), "0".into()].as_slice(),
            [0, 0].as_slice(),
            [(), ()].as_slice(),
        ),
    );

    assert_eq!(view_mut.index(0), (&1, &"4".into(), &7, &()));
    assert_eq!(view_mut.index(1), (&0, &"0".into(), &0, &()));
    assert_eq!(view_mut.index(2), (&0, &"0".into(), &0, &()));
    assert_eq!(view_mut.get(3), None);
    assert_eq!(
        view_mut.as_mut_slices(),
        (
            [1, 0, 0].as_mut_slice(),
            ["4".into(), "0".into(), "0".into()].as_mut_slice(),
            [7, 0, 0].as_mut_slice(),
            [(), (), ()].as_mut_slice(),
        ),
    );
    assert_eq!(
        format!("{view_mut:?}"),
        r#"SoaViewMut(([1, 0, 0], ["4", "0", "0"], [7, 0, 0], [(), (), ()]))"#,
    );

    let permutation: [_; 3] = array::from_fn(identity);
    view_mut.sort_unstable_with_permutation_by_key(permutation, |(_, _, _, &key)| key);
    assert_eq!(
        view_mut.as_mut_slices(),
        (
            [1, 0, 0].as_mut_slice(),
            ["4".into(), "0".into(), "0".into()].as_mut_slice(),
            [7, 0, 0].as_mut_slice(),
            [(), (), ()].as_mut_slice(),
        ),
    );

    view_mut.sort_unstable_with_permutation(permutation);
    assert_eq!(
        view_mut.as_mut_slices(),
        (
            [0, 0, 1].as_mut_slice(),
            ["0".into(), "0".into(), "4".into()].as_mut_slice(),
            [0, 0, 7].as_mut_slice(),
            [(), (), ()].as_mut_slice(),
        ),
    );

    let sub_slices = unsafe { view_mut.get_unchecked(..=0) };
    let sub_slices = unsafe { slices::from_slice_ptrs::<Item>(&context, sub_slices) };
    assert_eq!(
        sub_slices,
        (
            [0].as_slice(),
            ["0".into()].as_slice(),
            [0].as_slice(),
            [()].as_slice(),
        ),
    );

    let sub_slices = unsafe { view_mut.get_unchecked_mut(..=1) };
    let sub_slices = unsafe { slices::from_mut_slice_ptrs::<Item>(&context, sub_slices) };
    assert_eq!(
        sub_slices,
        (
            [0, 0].as_mut_slice(),
            ["0".into(), "0".into()].as_mut_slice(),
            [0, 0].as_mut_slice(),
            [(), ()].as_mut_slice(),
        ),
    );

    let view = view_mut.into_view();
    assert_eq!(view.len(), 3);
    assert_eq!(view.index(0), (&0, &"0".into(), &0, &()));
    assert_eq!(view.index(1), (&0, &"0".into(), &0, &()));
    assert_eq!(view.index(2), (&1, &"4".into(), &7, &()));
    assert_eq!(view.get(3), None);
    assert_eq!(
        view.as_slices(),
        (
            [0, 0, 1].as_slice(),
            ["0".into(), "0".into(), "4".into()].as_slice(),
            [0, 0, 7].as_slice(),
            [(), (), ()].as_slice(),
        ),
    );
    assert_eq!(view.as_ref(), &view);
    assert_ne!(view, View::empty(&context));
    assert_eq!(
        format!("{view:?}"),
        r#"SoaView(([0, 0, 1], ["0", "0", "4"], [0, 0, 7], [(), (), ()]))"#,
    );

    let sub_slices = unsafe { view.into_get_unchecked(..1) };
    let sub_slices = unsafe { slices::from_slice_ptrs::<Item>(&context, sub_slices) };
    assert_eq!(
        sub_slices,
        (
            [0].as_slice(),
            ["0".into()].as_slice(),
            [0].as_slice(),
            [()].as_slice(),
        ),
    );

    let mut iter = view.into_iter();
    assert_eq!(iter.len(), 3);
    assert_eq!(iter.next(), Some((&0, &"0".into(), &0, &())));

    assert_eq!(iter.len(), 2);
    assert_eq!(iter.next_back(), Some((&1, &"4".into(), &7, &())));

    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some((&0, &"0".into(), &0, &())));

    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);

    assert_equal(view, &view);

    assert_eq!(u8s, [0, 0, 1]);
    assert_eq!(strings, ["0", "0", "4"]);
    assert_eq!(u64s, [0, 0, 7]);
    assert_eq!(units, [(), (), ()]);
}

#[test]
fn three_items_unit() {
    type Item = ();
    type View<'ctx, 'a> = SoaView<'ctx, 'a, Item>;
    type ViewMut<'ctx, 'a> = SoaViewMut<'ctx, 'a, Item>;

    let context = Default::default();
    let mut units = [(); 3];

    let mut view_mut = ViewMut::new(&context, &mut units);
    assert_eq!(view_mut.len(), 3);
    assert_eq!(view_mut[0], ());
    assert_eq!(&view_mut[1], &());
    assert_eq!(&mut view_mut[2], &mut ());
    assert_eq!(view_mut.get_mut(3), None);

    assert_eq!(view_mut.as_mut_slices(), [(); 3]);
    assert_eq!(&mut view_mut[..], [(); 3]);
    assert_eq!(view_mut[..1], [(); 1]);
    assert_eq!(&view_mut[1..], [(); 2]);
    assert_eq!(view_mut.as_mut(), [(); 3]);
    assert_ne!(view_mut, ViewMut::empty(&context));
    assert_eq!(format!("{view_mut:?}"), "SoaViewMut([(), (), ()])");

    let mut iter = view_mut.iter_mut();
    assert_eq!(iter.len(), 3);
    assert_eq!(iter.next(), Some(&mut ()));

    assert_eq!(iter.len(), 2);
    assert_eq!(iter.next_back(), Some(&mut ()));

    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some(&mut ()));

    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);

    let eq_mut = [&mut (), &mut (), &mut ()];
    assert_equal(&mut view_mut, eq_mut);

    let first = View::new(&context, view_mut.index(..=1));
    let second = View::new(&context, view_mut.index(1..));

    assert_eq!(first.as_slices(), second.as_slices());
    assert_eq!(first, second);

    assert_eq!(
        first.cmp(&second),
        first.as_slices().cmp(&second.as_slices()),
    );

    let hasher = FxBuildHasher::default();
    assert_eq!(
        hasher.hash_one(first.as_slices()),
        hasher.hash_one(second.as_slices()),
    );
    assert_eq!(hasher.hash_one(&first), hasher.hash_one(&second));

    assert_eq!(hasher.hash_one(first.as_slices()), hasher.hash_one(&first));
    assert_eq!(
        hasher.hash_one(second.as_slices()),
        hasher.hash_one(&second),
    );

    let mut sub_view = ViewMut::new(&context, &mut view_mut[1..]);
    assert_eq!(sub_view.len(), 2);
    assert_eq!(sub_view[0], ());
    assert_eq!(sub_view[1], ());
    assert_eq!(sub_view.get(2), None);
    assert_eq!(sub_view.as_slices(), [(); 2]);

    let mut gr_data = [(); 2]; // the last one is greater
    let mut gr_slices = ViewMut::new(&context, &mut gr_data);

    assert_eq!(sub_view.as_mut_slices(), gr_slices.as_mut_slices());
    assert_eq!(sub_view, gr_slices);

    assert_eq!(
        sub_view.cmp(&gr_slices),
        sub_view.as_mut_slices().cmp(&gr_slices.as_mut_slices()),
    );

    let hasher = FxBuildHasher::default();
    assert_eq!(
        hasher.hash_one(sub_view.as_mut_slices()),
        hasher.hash_one(gr_slices.as_mut_slices()),
    );
    assert_eq!(hasher.hash_one(&sub_view), hasher.hash_one(&gr_slices));

    assert_eq!(
        hasher.hash_one(sub_view.as_mut_slices()),
        hasher.hash_one(&sub_view),
    );
    assert_eq!(
        hasher.hash_one(gr_slices.as_mut_slices()),
        hasher.hash_one(&gr_slices),
    );

    sub_view.clone_from_slices(&View::new(&context, &[(); 2]));
    assert_eq!(sub_view.len(), 2);
    assert_eq!(sub_view.as_slices(), [(); 2]);

    assert_eq!(view_mut[0], ());
    assert_eq!(&view_mut[1], &());
    assert_eq!(&mut view_mut[2], &mut ());
    assert_eq!(view_mut.get_mut(3), None);
    assert_eq!(view_mut.as_mut_slices(), [(); 3]);
    assert_eq!(format!("{view_mut:?}"), "SoaViewMut([(), (), ()])");

    let permutation: [_; 3] = array::from_fn(identity);
    view_mut.sort_unstable_with_permutation_by_key(permutation, |&item| Reverse(item));
    assert_eq!(view_mut.as_mut_slices(), [(); 3]);

    view_mut.sort_unstable_with_permutation(permutation);
    assert_eq!(view_mut.as_mut_slices(), [(); 3]);

    let sub_slices = unsafe { view_mut.get_unchecked(..=0) };
    let sub_slices = unsafe { slices::from_slice_ptrs::<Item>(&context, sub_slices) };
    assert_eq!(sub_slices, [()]);

    let sub_slices_mut = unsafe { view_mut.get_unchecked_mut(..=1) };
    let sub_slices_mut = unsafe { slices::from_mut_slice_ptrs::<Item>(&context, sub_slices_mut) };
    assert_eq!(sub_slices_mut, [(); 2]);

    let view = view_mut.into_view();
    assert_eq!(view.len(), 3);
    assert_eq!(view[0], ());
    assert_eq!(view[1], ());
    assert_eq!(view[2], ());
    assert_eq!(view.get(3), None);
    assert_eq!(view.as_ref(), [(); 3]);
    assert_ne!(view, View::empty(&context));
    assert_eq!(format!("{view:?}"), "SoaView([(), (), ()])");

    let sub_slices = unsafe { view.into_get_unchecked(..1) };
    let sub_slices = unsafe { slices::from_slice_ptrs::<Item>(&context, sub_slices) };
    assert_eq!(sub_slices, [()]);

    let mut iter = view.into_iter();
    assert_eq!(iter.len(), 3);
    assert_eq!(iter.next(), Some(&()));

    assert_eq!(iter.len(), 2);
    assert_eq!(iter.next_back(), Some(&()));

    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some(&()));

    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);

    assert_equal(view, &view);

    assert_eq!(units, [(); 3]);
}

#[test]
fn three_items_identity() {
    type Item = Identity<u128>;
    type View<'ctx, 'a> = SoaView<'ctx, 'a, Item>;
    type ViewMut<'ctx, 'a> = SoaViewMut<'ctx, 'a, Item>;

    let context = Default::default();
    let mut data = [1.into(), 2.into(), 3.into()];

    let mut view_mut = ViewMut::new(&context, &mut data);
    assert_eq!(view_mut.len(), 3);
    assert_eq!(view_mut[0], 1.into());
    assert_eq!(&view_mut[1], &2.into());
    assert_eq!(&mut view_mut[2], &mut 3.into());
    assert_eq!(view_mut.get_mut(3), None);

    assert_eq!(view_mut.as_mut_slices(), [1.into(), 2.into(), 3.into()]);
    assert_eq!(&mut view_mut[..], [1.into(), 2.into(), 3.into()]);
    assert_eq!(view_mut[..1], [1.into()]);
    assert_eq!(&view_mut[1..], [2.into(), 3.into()]);
    assert_eq!(view_mut.as_mut(), [1.into(), 2.into(), 3.into()]);
    assert_ne!(view_mut, ViewMut::empty(&context));
    assert_eq!(
        format!("{view_mut:?}"),
        "SoaViewMut([Identity(1), Identity(2), Identity(3)])",
    );

    let mut iter = view_mut.iter_mut();
    assert_eq!(iter.len(), 3);
    assert_eq!(iter.next(), Some(&mut 1.into()));

    assert_eq!(iter.len(), 2);
    assert_eq!(iter.next_back(), Some(&mut 3.into()));

    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some(&mut 2.into()));

    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);

    let eq_mut = [&mut 1.into(), &mut 2.into(), &mut 3.into()];
    assert_equal(&mut view_mut, eq_mut);

    let first = View::new(&context, view_mut.index(..=1));
    let second = View::new(&context, view_mut.index(1..));

    assert_ne!(first.as_slices(), second.as_slices());
    assert_ne!(first, second);

    assert!(first.as_slices() < second.as_slices());
    assert!(first < second);

    assert_eq!(
        first.cmp(&second),
        first.as_slices().cmp(&second.as_slices()),
    );

    let hasher = FxBuildHasher::default();
    assert_ne!(
        hasher.hash_one(first.as_slices()),
        hasher.hash_one(second.as_slices()),
    );
    assert_ne!(hasher.hash_one(&first), hasher.hash_one(&second));

    assert_eq!(hasher.hash_one(first.as_slices()), hasher.hash_one(&first));
    assert_eq!(
        hasher.hash_one(second.as_slices()),
        hasher.hash_one(&second),
    );

    let mut sub_view = ViewMut::new(&context, &mut view_mut[1..]);
    assert_eq!(sub_view.len(), 2);
    assert_eq!(sub_view[0], 2.into());
    assert_eq!(sub_view[1], 3.into());
    assert_eq!(sub_view.get(2), None);
    assert_eq!(sub_view.as_slices(), [2.into(), 3.into()]);

    let mut gr_data = [2.into(), 42.into()]; // the last one is greater
    let mut gr_slices = ViewMut::new(&context, &mut gr_data);

    assert_ne!(sub_view.as_mut_slices(), gr_slices.as_mut_slices());
    assert_ne!(sub_view, gr_slices);

    assert!(sub_view.as_mut_slices() < gr_slices.as_mut_slices());
    assert!(sub_view < gr_slices);

    assert_eq!(
        sub_view.cmp(&gr_slices),
        sub_view.as_mut_slices().cmp(&gr_slices.as_mut_slices()),
    );

    let hasher = FxBuildHasher::default();
    assert_ne!(
        hasher.hash_one(sub_view.as_mut_slices()),
        hasher.hash_one(gr_slices.as_mut_slices()),
    );
    assert_ne!(hasher.hash_one(&sub_view), hasher.hash_one(&gr_slices));

    assert_eq!(
        hasher.hash_one(sub_view.as_mut_slices()),
        hasher.hash_one(&sub_view),
    );
    assert_eq!(
        hasher.hash_one(gr_slices.as_mut_slices()),
        hasher.hash_one(&gr_slices),
    );

    sub_view.clone_from_slices(&View::new(&context, &[4.into(), 2.into()]));
    assert_eq!(sub_view.len(), 2);
    assert_eq!(sub_view.as_slices(), [4.into(), 2.into()]);

    assert_eq!(view_mut[0], 1.into());
    assert_eq!(&view_mut[1], &4.into());
    assert_eq!(&mut view_mut[2], &mut 2.into());
    assert_eq!(view_mut.get_mut(3), None);
    assert_eq!(view_mut.as_mut_slices(), [1.into(), 4.into(), 2.into()]);
    assert_eq!(
        format!("{view_mut:?}"),
        "SoaViewMut([Identity(1), Identity(4), Identity(2)])",
    );

    let permutation: [_; 3] = array::from_fn(identity);
    view_mut.sort_unstable_with_permutation_by_key(permutation, |&item| Reverse(item));
    assert_eq!(view_mut.as_mut_slices(), [4.into(), 2.into(), 1.into()]);

    view_mut.sort_unstable_with_permutation(permutation);
    assert_eq!(view_mut.as_mut_slices(), [1.into(), 2.into(), 4.into()]);

    let sub_slices = unsafe { view_mut.get_unchecked(..=0) };
    let sub_slices = unsafe { slices::from_slice_ptrs::<Item>(&context, sub_slices) };
    assert_eq!(sub_slices, [1.into()]);

    let sub_slices_mut = unsafe { view_mut.get_unchecked_mut(..=1) };
    let sub_slices_mut = unsafe { slices::from_mut_slice_ptrs::<Item>(&context, sub_slices_mut) };
    assert_eq!(sub_slices_mut, [1.into(), 2.into()]);

    let view = view_mut.into_view();
    assert_eq!(view.len(), 3);
    assert_eq!(view[0], 1.into());
    assert_eq!(view[1], 2.into());
    assert_eq!(view[2], 4.into());
    assert_eq!(view.get(3), None);
    assert_eq!(view.as_ref(), [1.into(), 2.into(), 4.into()]);
    assert_ne!(view, View::empty(&context));
    assert_eq!(
        format!("{view:?}"),
        "SoaView([Identity(1), Identity(2), Identity(4)])",
    );

    let sub_slices = unsafe { view.into_get_unchecked(..1) };
    let sub_slices = unsafe { slices::from_slice_ptrs::<Item>(&context, sub_slices) };
    assert_eq!(sub_slices, [1.into()]);

    let mut iter = view.into_iter();
    assert_eq!(iter.len(), 3);
    assert_eq!(iter.next(), Some(&1.into()));

    assert_eq!(iter.len(), 2);
    assert_eq!(iter.next_back(), Some(&4.into()));

    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some(&2.into()));

    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);

    assert_equal(view, &view);

    assert_eq!(data, [1.into(), 2.into(), 4.into()]);
}

#[test]
fn three_items_zst() {
    type Item = (ZST1, ZST2, ZST3);
    type View<'ctx, 'a> = SoaView<'ctx, 'a, Item>;
    type ViewMut<'ctx, 'a> = SoaViewMut<'ctx, 'a, Item>;

    let context = Default::default();
    let mut zst1s = [ZST1; 3];
    let mut zst2s = [ZST2(()); 3];
    let mut zst3s = [ZST3 { empty: () }; 3];

    let mut view_mut = ViewMut::new(&context, (&mut zst1s, &mut zst2s, &mut zst3s));
    assert_eq!(view_mut.len(), 3);
    assert_eq!(
        view_mut.index_mut(0),
        (&mut ZST1, &mut ZST2(()), &mut ZST3 { empty: () }),
    );
    assert_eq!(
        view_mut.index_mut(1),
        (&mut ZST1, &mut ZST2(()), &mut ZST3 { empty: () }),
    );
    assert_eq!(
        view_mut.index_mut(2),
        (&mut ZST1, &mut ZST2(()), &mut ZST3 { empty: () }),
    );
    assert_eq!(view_mut.get_mut(3), None);

    assert_eq!(
        view_mut.as_mut_slices(),
        (
            [ZST1; 3].as_mut_slice(),
            [ZST2(()); 3].as_mut_slice(),
            [ZST3 { empty: () }; 3].as_mut_slice(),
        ),
    );
    assert_eq!(
        view_mut.index_mut(..),
        (
            [ZST1; 3].as_mut_slice(),
            [ZST2(()); 3].as_mut_slice(),
            [ZST3 { empty: () }; 3].as_mut_slice(),
        ),
    );
    assert_eq!(
        view_mut.index_mut(..1),
        (
            [ZST1; 1].as_mut_slice(),
            [ZST2(()); 1].as_mut_slice(),
            [ZST3 { empty: () }; 1].as_mut_slice(),
        ),
    );
    assert_eq!(
        view_mut.index(1..),
        (
            [ZST1; 2].as_slice(),
            [ZST2(()); 2].as_slice(),
            [ZST3 { empty: () }; 2].as_slice(),
        ),
    );
    assert_eq!(view_mut.as_ref(), &view_mut);
    assert_ne!(view_mut, ViewMut::empty(&context));
    assert_ne!(view_mut.as_mut(), &mut ViewMut::empty(&context));
    assert_eq!(
        format!("{view_mut:?}"),
        r#"SoaViewMut(([ZST1, ZST1, ZST1], [ZST2(()), ZST2(()), ZST2(())], [ZST3 { empty: () }, ZST3 { empty: () }, ZST3 { empty: () }]))"#,
    );

    let mut iter = view_mut.iter_mut();
    assert_eq!(iter.len(), 3);
    assert_eq!(
        iter.next(),
        Some((&mut ZST1, &mut ZST2(()), &mut ZST3 { empty: () })),
    );

    assert_eq!(iter.len(), 2);
    assert_eq!(
        iter.next_back(),
        Some((&mut ZST1, &mut ZST2(()), &mut ZST3 { empty: () })),
    );

    assert_eq!(iter.len(), 1);
    assert_eq!(
        iter.next(),
        Some((&mut ZST1, &mut ZST2(()), &mut ZST3 { empty: () })),
    );

    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);

    let eq_mut = [
        (&mut ZST1, &mut ZST2(()), &mut ZST3 { empty: () }),
        (&mut ZST1, &mut ZST2(()), &mut ZST3 { empty: () }),
        (&mut ZST1, &mut ZST2(()), &mut ZST3 { empty: () }),
    ];
    assert_equal(&mut view_mut, eq_mut);

    let mut sub_view = ViewMut::new(&context, view_mut.index_mut(1..));
    assert_eq!(sub_view.len(), 2);
    assert_eq!(sub_view.index(0), (&ZST1, &ZST2(()), &ZST3 { empty: () }));
    assert_eq!(sub_view.index(1), (&ZST1, &ZST2(()), &ZST3 { empty: () }));
    assert_eq!(sub_view.get(2), None);
    assert_eq!(
        sub_view.as_slices(),
        (
            [ZST1; 2].as_slice(),
            [ZST2(()); 2].as_slice(),
            [ZST3 { empty: () }; 2].as_slice(),
        ),
    );

    sub_view.clone_from_slices(&View::new(
        &context,
        (
            [ZST1; 2].as_slice(),
            [ZST2(()); 2].as_slice(),
            [ZST3 { empty: () }; 2].as_slice(),
        ),
    ));
    assert_eq!(sub_view.len(), 2);
    assert_eq!(
        sub_view.as_slices(),
        (
            [ZST1; 2].as_slice(),
            [ZST2(()); 2].as_slice(),
            [ZST3 { empty: () }; 2].as_slice(),
        ),
    );

    assert_eq!(view_mut.index(0), (&ZST1, &ZST2(()), &ZST3 { empty: () }));
    assert_eq!(view_mut.index(1), (&ZST1, &ZST2(()), &ZST3 { empty: () }));
    assert_eq!(view_mut.index(2), (&ZST1, &ZST2(()), &ZST3 { empty: () }));
    assert_eq!(view_mut.get(3), None);
    assert_eq!(
        view_mut.as_mut_slices(),
        (
            [ZST1; 3].as_mut_slice(),
            [ZST2(()); 3].as_mut_slice(),
            [ZST3 { empty: () }; 3].as_mut_slice(),
        ),
    );
    assert_eq!(
        format!("{view_mut:?}"),
        r#"SoaViewMut(([ZST1, ZST1, ZST1], [ZST2(()), ZST2(()), ZST2(())], [ZST3 { empty: () }, ZST3 { empty: () }, ZST3 { empty: () }]))"#,
    );

    let permutation: [_; 3] = array::from_fn(identity);
    view_mut.sort_unstable_with_permutation_by_key(permutation, |(_, &key, _)| Reverse(key));
    assert_eq!(
        view_mut.as_mut_slices(),
        (
            [ZST1; 3].as_mut_slice(),
            [ZST2(()); 3].as_mut_slice(),
            [ZST3 { empty: () }; 3].as_mut_slice(),
        ),
    );

    view_mut.sort_unstable_with_permutation(permutation);
    assert_eq!(
        view_mut.as_mut_slices(),
        (
            [ZST1; 3].as_mut_slice(),
            [ZST2(()); 3].as_mut_slice(),
            [ZST3 { empty: () }; 3].as_mut_slice(),
        ),
    );

    let sub_slices = unsafe { view_mut.get_unchecked(..=0) };
    let sub_slices = unsafe { slices::from_slice_ptrs::<Item>(&context, sub_slices) };
    assert_eq!(
        sub_slices,
        (
            [ZST1; 1].as_slice(),
            [ZST2(()); 1].as_slice(),
            [ZST3 { empty: () }; 1].as_slice(),
        ),
    );

    let sub_slices = unsafe { view_mut.get_unchecked_mut(..=1) };
    let sub_slices = unsafe { slices::from_mut_slice_ptrs::<Item>(&context, sub_slices) };
    assert_eq!(
        sub_slices,
        (
            [ZST1; 2].as_mut_slice(),
            [ZST2(()); 2].as_mut_slice(),
            [ZST3 { empty: () }; 2].as_mut_slice(),
        ),
    );

    let view = view_mut.into_view();
    assert_eq!(view.len(), 3);
    assert_eq!(view.index(0), (&ZST1, &ZST2(()), &ZST3 { empty: () }));
    assert_eq!(view.index(1), (&ZST1, &ZST2(()), &ZST3 { empty: () }));
    assert_eq!(view.index(2), (&ZST1, &ZST2(()), &ZST3 { empty: () }));
    assert_eq!(view.get(3), None);
    assert_eq!(
        view.as_slices(),
        (
            [ZST1; 3].as_slice(),
            [ZST2(()); 3].as_slice(),
            [ZST3 { empty: () }; 3].as_slice(),
        ),
    );
    assert_eq!(view.as_ref(), &view);
    assert_ne!(view, View::empty(&context));
    assert_eq!(
        format!("{view:?}"),
        r#"SoaView(([ZST1, ZST1, ZST1], [ZST2(()), ZST2(()), ZST2(())], [ZST3 { empty: () }, ZST3 { empty: () }, ZST3 { empty: () }]))"#,
    );

    let sub_slices = unsafe { view.into_get_unchecked(..1) };
    let sub_slices = unsafe { slices::from_slice_ptrs::<Item>(&context, sub_slices) };
    assert_eq!(
        sub_slices,
        (
            [ZST1; 1].as_slice(),
            [ZST2(()); 1].as_slice(),
            [ZST3 { empty: () }; 1].as_slice(),
        ),
    );

    let mut iter = view.into_iter();
    assert_eq!(iter.len(), 3);
    assert_eq!(iter.next(), Some((&ZST1, &ZST2(()), &ZST3 { empty: () })));

    assert_eq!(iter.len(), 2);
    assert_eq!(
        iter.next_back(),
        Some((&ZST1, &ZST2(()), &ZST3 { empty: () })),
    );

    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some((&ZST1, &ZST2(()), &ZST3 { empty: () })));

    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);

    assert_equal(view, &view);

    assert_eq!(zst1s, [ZST1; 3]);
    assert_eq!(zst2s, [ZST2(()); 3]);
    assert_eq!(zst3s, [ZST3 { empty: () }; 3]);
}
