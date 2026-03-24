use std::{array, cmp::Reverse, convert::identity, hash::BuildHasher};

use gpecs_soa::prelude::*;
use itertools::assert_equal;
use rustc_hash::FxBuildHasher;

use crate::common::{ZST1, ZST2, ZST3};

#[test]
fn empty() {
    type Item = (u32, u128, u8, ());
    type Slices<'ctx, 'a> = SoaSlices<'ctx, 'a, Item>;
    type SlicesMut<'ctx, 'a> = SoaSlicesMut<'ctx, 'a, Item>;

    let context = Default::default();

    let slices = Slices::new(&context, (&[], &[], &[], &[]));
    assert!(slices.is_empty());
    assert_eq!(slices.get(0), None);
    assert_eq!(slices.as_ref(), &slices);
    assert_eq!(slices, Slices::from(&context));
    assert_eq!(format!("{slices:?}"), "SoaSlices(([], [], [], []))");

    let mut iter = slices.into_iter();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    assert_equal(slices, &slices);

    let slices = slices.into_slices();
    assert_eq!(
        slices,
        ([].as_slice(), [].as_slice(), [].as_slice(), [].as_slice()),
    );

    let mut slices_mut = SlicesMut::new(&context, (&mut [], &mut [], &mut [], &mut []));
    assert!(slices_mut.is_empty());
    assert_eq!(slices_mut.get_mut(0), None);
    assert_eq!(slices_mut.as_ref(), &slices_mut);
    assert_eq!(slices_mut, SlicesMut::from(&context));
    assert_eq!(slices_mut.as_mut(), &mut SlicesMut::from(&context));
    assert_eq!(format!("{slices_mut:?}"), "SoaSlicesMut(([], [], [], []))");

    let mut iter = slices_mut.iter_mut();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    let eq_mut = [];
    assert_equal(&mut slices_mut, eq_mut);

    let slices_mut = slices_mut.into_slices();
    assert_eq!(
        slices_mut,
        (
            [].as_mut_slice(),
            [].as_mut_slice(),
            [].as_mut_slice(),
            [].as_mut_slice(),
        ),
    );

    let mut slices_mut = SlicesMut::new(&context, slices_mut);

    let permutation: [_; 0] = array::from_fn(identity);
    slices_mut.sort_unstable_with_permutation(permutation);
    assert!(slices_mut.is_empty());

    let slices = Slices::from(slices_mut);
    assert!(slices.is_empty());
    assert_eq!(slices.get(0), None);
    assert_eq!(format!("{slices:?}"), "SoaSlices(([], [], [], []))");
}

#[test]
fn empty_unit() {
    type Slices<'ctx, 'a> = SoaSlices<'ctx, 'a, ()>;
    type SlicesMut<'ctx, 'a> = SoaSlicesMut<'ctx, 'a, ()>;

    let context = Default::default();

    let slices = Slices::new(&context, &[]);
    assert!(slices.is_empty());
    assert_eq!(slices.get(0), None);
    assert_eq!(slices.as_ref(), []);
    assert_eq!(slices, Slices::from(&context));
    assert_eq!(format!("{slices:?}"), "SoaSlices([])");

    let mut iter = slices.into_iter();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    assert_equal(slices, &slices);

    let slices = slices.into_slices();
    assert_eq!(slices, []);

    let mut slices_mut = SlicesMut::new(&context, &mut []);
    assert!(slices_mut.is_empty());
    assert_eq!(slices_mut.get_mut(0), None);
    assert_eq!(slices_mut.as_ref(), []);
    assert_eq!(slices_mut, SlicesMut::from(&context));
    assert_eq!(format!("{slices_mut:?}"), "SoaSlicesMut([])");

    let mut iter = slices_mut.iter_mut();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    let eq_mut = &mut [];
    assert_equal(&mut slices_mut, eq_mut);

    let slices_mut = slices_mut.into_slices();
    assert_eq!(slices_mut, []);

    let mut slices_mut = SlicesMut::new(&context, slices_mut);

    let permutation: [_; 0] = array::from_fn(identity);
    slices_mut.sort_unstable_with_permutation(permutation);
    assert!(slices_mut.is_empty());

    let slices = Slices::from(slices_mut);
    assert!(slices.is_empty());
    assert_eq!(slices.get(0), None);
    assert_eq!(format!("{slices:?}"), "SoaSlices([])");
}

#[test]
fn empty_identity() {
    type Item = Identity<u128>;
    type Slices<'ctx, 'a> = SoaSlices<'ctx, 'a, Item>;
    type SlicesMut<'ctx, 'a> = SoaSlicesMut<'ctx, 'a, Item>;

    let context = Default::default();

    let slices = Slices::new(&context, &[]);
    assert!(slices.is_empty());
    assert_eq!(slices.get(0), None);
    assert_eq!(slices.as_ref(), []);
    assert_eq!(slices, Slices::from(&context));
    assert_eq!(format!("{slices:?}"), "SoaSlices([])");

    let mut iter = slices.into_iter();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    assert_equal(slices, &slices);

    let slices = slices.into_slices();
    assert_eq!(slices, []);

    let mut slices_mut = SlicesMut::new(&context, &mut []);
    assert!(slices_mut.is_empty());
    assert_eq!(slices_mut.get_mut(0), None);
    assert_eq!(slices_mut.as_mut(), []);
    assert_eq!(slices_mut, SlicesMut::from(&context));
    assert_eq!(format!("{slices_mut:?}"), "SoaSlicesMut([])");

    let mut iter = slices_mut.iter_mut();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    let eq_mut: [&mut Identity<_>; _] = [];
    assert_equal(&mut slices_mut, eq_mut);

    let slices_mut = slices_mut.into_slices();
    assert_eq!(slices_mut, []);

    let mut slices_mut = SlicesMut::new(&context, slices_mut);

    let permutation: [_; 0] = array::from_fn(identity);
    slices_mut.sort_unstable_with_permutation(permutation);
    assert!(slices_mut.is_empty());

    let slices = Slices::from(slices_mut);
    assert!(slices.is_empty());
    assert_eq!(slices.get(0), None);
    assert_eq!(format!("{slices:?}"), "SoaSlices([])");
}

#[test]
fn empty_zst() {
    type Slices<'ctx, 'a> = SoaSlices<'ctx, 'a, (ZST1, ZST2, ZST3)>;
    type SlicesMut<'ctx, 'a> = SoaSlicesMut<'ctx, 'a, (ZST1, ZST2, ZST3)>;

    let context = Default::default();

    let slices = Slices::new(&context, (&[], &[], &[]));
    assert!(slices.is_empty());
    assert_eq!(slices.get(0), None);
    assert_eq!(slices.as_ref(), &slices);
    assert_eq!(slices, Slices::from(&context));
    assert_eq!(format!("{slices:?}"), "SoaSlices(([], [], []))");

    let mut iter = slices.into_iter();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    assert_equal(slices, &slices);

    let slices = slices.into_slices();
    assert_eq!(slices, ([].as_slice(), [].as_slice(), [].as_slice()));

    let mut slices_mut = SlicesMut::new(&context, (&mut [], &mut [], &mut []));
    assert!(slices_mut.is_empty());
    assert_eq!(slices_mut.get_mut(0), None);
    assert_eq!(slices_mut.as_ref(), &slices_mut);
    assert_eq!(slices_mut, SlicesMut::from(&context));
    assert_eq!(slices_mut.as_mut(), &mut SlicesMut::from(&context));
    assert_eq!(format!("{slices_mut:?}"), "SoaSlicesMut(([], [], []))");

    let mut iter = slices_mut.iter_mut();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    let eq_mut = [];
    assert_equal(&mut slices_mut, eq_mut);

    let slices_mut = slices_mut.into_slices();
    assert_eq!(
        slices_mut,
        ([].as_mut_slice(), [].as_mut_slice(), [].as_mut_slice()),
    );

    let mut slices_mut = SlicesMut::new(&context, slices_mut);

    let permutation: [_; 0] = array::from_fn(identity);
    slices_mut.sort_unstable_with_permutation(permutation);
    assert!(slices_mut.is_empty());

    let slices = Slices::from(slices_mut);
    assert!(slices.is_empty());
    assert_eq!(slices.get(0), None);
    assert_eq!(format!("{slices:?}"), "SoaSlices(([], [], []))");
}

#[test]
fn one_item() {
    type Item = (u32, u128, u8, ());
    type Slices<'ctx, 'a> = SoaSlices<'ctx, 'a, Item>;
    type SlicesMut<'ctx, 'a> = SoaSlicesMut<'ctx, 'a, Item>;

    let context = Default::default();
    let mut u8s = [1];
    let mut u64s = [2];
    let mut u16s = [3];
    let mut units = [()];

    let slices = Slices::new(&context, (&u8s, &u64s, &u16s, &units));
    assert_eq!(slices.len(), 1);
    assert!(slices.contains((&1, &2, &3, &())));

    assert_eq!(
        slices.as_slices(),
        (
            u8s.as_slice(),
            u64s.as_slice(),
            u16s.as_slice(),
            units.as_slice(),
        ),
    );
    assert_eq!(
        slices.into_index(..),
        (
            u8s.as_slice(),
            u64s.as_slice(),
            u16s.as_slice(),
            units.as_slice(),
        ),
    );
    assert_eq!(
        slices.index(0..),
        (
            u8s.as_slice(),
            u64s.as_slice(),
            u16s.as_slice(),
            units.as_slice(),
        ),
    );
    assert_eq!(
        slices.index(..0),
        ([].as_slice(), [].as_slice(), [].as_slice(), [].as_slice()),
    );
    assert_eq!(slices.index(0), (&1, &2, &3, &()));
    assert_eq!(slices.as_ref(), &slices);
    assert_ne!(slices, Slices::from(&context));
    assert_eq!(format!("{slices:?}"), "SoaSlices(([1], [2], [3], [()]))");

    let mut iter = slices.into_iter();
    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some((&1, &2, &3, &())));
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    assert_equal(slices, &slices);

    let mut slices_mut = SlicesMut::new(&context, (&mut u8s, &mut u64s, &mut u16s, &mut units));
    assert_eq!(slices_mut.len(), 1);
    assert_eq!(
        slices_mut.index_mut(..),
        (
            [1].as_mut_slice(),
            [2].as_mut_slice(),
            [3].as_mut_slice(),
            [()].as_mut_slice(),
        ),
    );
    assert_eq!(
        slices_mut.index_mut(..0),
        (
            [].as_mut_slice(),
            [].as_mut_slice(),
            [].as_mut_slice(),
            [].as_mut_slice(),
        ),
    );
    assert_eq!(slices_mut.index_mut(0), (&mut 1, &mut 2, &mut 3, &mut ()));
    assert!(slices_mut.contains((&1, &2, &3, &())));

    let eq_mut = [(&mut 1, &mut 2, &mut 3, &mut ())];
    assert_equal(&mut slices_mut, eq_mut);

    slices_mut.copy_from_slices(&Slices::new(&context, (&[0], &[0], &[0], &[()])));
    assert_eq!(slices_mut.len(), 1);
    assert_eq!(slices_mut.index_mut(0), (&mut 0, &mut 0, &mut 0, &mut ()));
    assert!(!slices_mut.contains((&1, &2, &3, &())));
    assert_eq!(
        slices_mut.as_mut_slices(),
        (
            [0].as_mut_slice(),
            [0].as_mut_slice(),
            [0].as_mut_slice(),
            [()].as_mut_slice(),
        ),
    );
    assert_eq!(slices_mut.as_ref(), &slices_mut);
    assert_ne!(slices_mut, SlicesMut::from(&context));
    assert_ne!(slices_mut.as_mut(), &mut SlicesMut::from(&context));
    assert_eq!(
        format!("{slices_mut:?}"),
        "SoaSlicesMut(([0], [0], [0], [()]))",
    );

    let permutation: [_; 1] = array::from_fn(identity);
    slices_mut.sort_unstable_with_permutation(permutation);
    assert_eq!(
        slices_mut.as_mut_slices(),
        (
            [0].as_mut_slice(),
            [0].as_mut_slice(),
            [0].as_mut_slice(),
            [()].as_mut_slice(),
        ),
    );

    let slices = Slices::from(slices_mut);
    assert_eq!(slices.len(), 1);
    assert_eq!(slices.index(0), (&0, &0, &0, &()));
    assert_eq!(
        slices.as_slices(),
        (
            [0].as_slice(),
            [0].as_slice(),
            [0].as_slice(),
            [()].as_slice(),
        ),
    );

    let mut iter = slices.into_iter();
    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some((&0, &0, &0, &())));
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    assert_equal(slices, &slices);

    assert_eq!(u8s, [0]);
    assert_eq!(u64s, [0]);
    assert_eq!(u16s, [0]);
    assert_eq!(units, [()]);
}

#[test]
fn one_item_unit() {
    type Slices<'ctx, 'a> = SoaSlices<'ctx, 'a, ()>;
    type SlicesMut<'ctx, 'a> = SoaSlicesMut<'ctx, 'a, ()>;

    let context = Default::default();
    let mut units = [()];

    let slices = Slices::new(&context, &units);
    assert_eq!(slices.len(), 1);
    assert!(slices.contains(&()));

    assert_eq!(slices.as_slices(), units);
    assert_eq!(slices.into_index(..), units);
    assert_eq!(slices.index(0..), units);
    assert_eq!(slices.index(..0), []);
    assert_eq!(slices.get(0), Some(&()));
    assert_ne!(slices, Slices::from(&context));
    assert_eq!(format!("{slices:?}"), "SoaSlices([()])");

    let mut iter = slices.into_iter();
    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some(&()));
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    assert_equal(slices, &slices);

    let mut slices_mut = SlicesMut::new(&context, &mut units);
    assert_eq!(slices_mut.len(), 1);
    assert_eq!(slices_mut.index_mut(..), [()]);
    assert_eq!(slices_mut.index_mut(..0), []);
    assert_eq!(slices_mut.index_mut(0), &mut ());
    assert!(slices_mut.contains(&()));

    let permutation: [_; 1] = array::from_fn(identity);
    slices_mut.sort_unstable_with_permutation(permutation);
    assert_eq!(slices_mut.as_mut_slices(), [()]);

    let eq_mut = &mut [()];
    assert_equal(&mut slices_mut, eq_mut);

    slices_mut.copy_from_slices(&Slices::new(&context, &[()]));
    assert_eq!(slices_mut.len(), 1);
    assert_eq!(slices_mut.as_mut_slices(), [()]);
    assert_eq!(slices_mut.index_mut(0), &mut ());
    assert_ne!(slices_mut, SlicesMut::from(&context));
    assert_eq!(format!("{slices_mut:?}"), "SoaSlicesMut([()])");

    let slices = Slices::from(slices_mut);
    assert_eq!(slices.len(), 1);
    assert_eq!(slices.index(0), &());
    assert_eq!(slices.as_slices(), [()]);

    let mut iter = slices.into_iter();
    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some(&()));
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    assert_equal(slices, &slices);

    assert_eq!(units, [()]);
}

#[test]
fn one_item_identity() {
    type Item = Identity<u128>;
    type Slices<'ctx, 'a> = SoaSlices<'ctx, 'a, Item>;
    type SlicesMut<'ctx, 'a> = SoaSlicesMut<'ctx, 'a, Item>;

    let context = Default::default();
    let mut data = [1.into()];

    let slices = Slices::new(&context, &data);
    assert_eq!(slices.len(), 1);
    assert!(slices.contains(&1.into()));

    assert_eq!(slices.as_slices(), data);
    assert_eq!(slices[0..], data);
    assert_eq!(slices[..0], []);
    assert_eq!(&slices[0], &1.into());
    assert_ne!(slices, Slices::from(&context));
    assert_eq!(format!("{slices:?}"), "SoaSlices([Identity(1)])");

    let mut iter = slices.into_iter();
    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some(&1.into()));
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    assert_equal(slices, &slices);

    let mut slices_mut = SlicesMut::new(&context, &mut data);
    assert_eq!(slices_mut.len(), 1);
    assert_eq!(slices_mut[..0], []);
    assert_eq!(&mut slices_mut[0], &mut 1.into());
    assert!(slices_mut.contains(&1.into()));

    let permutation: [_; 1] = array::from_fn(identity);
    slices_mut.sort_unstable_with_permutation(permutation);
    assert_eq!(slices_mut.as_ref(), [1.into()]);

    let eq_mut = [&mut 1.into()];
    assert_equal(&mut slices_mut, eq_mut);

    slices_mut.copy_from_slices(&Slices::new(&context, &[0.into()]));
    assert_eq!(slices_mut.len(), 1);
    assert_eq!(&mut slices_mut[0], &mut 0.into());
    assert!(!slices_mut.contains(&1.into()));

    assert_eq!(slices_mut.as_mut_slices(), [0.into()]);
    assert_ne!(slices_mut, SlicesMut::from(&context));
    assert_eq!(format!("{slices_mut:?}"), "SoaSlicesMut([Identity(0)])");

    let slices = Slices::from(slices_mut);
    assert_eq!(slices.len(), 1);
    assert_eq!(&slices[0], &0.into());
    assert_eq!(slices.as_slices(), [0.into()]);

    let mut iter = slices.into_iter();
    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some(&0.into()));
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    assert_equal(slices, &slices);

    assert_eq!(data, [0.into()]);
}

#[test]
fn one_item_zst() {
    type Slices<'ctx, 'a> = SoaSlices<'ctx, 'a, (ZST1, ZST2, ZST3)>;
    type SlicesMut<'ctx, 'a> = SoaSlicesMut<'ctx, 'a, (ZST1, ZST2, ZST3)>;

    let context = Default::default();
    let mut zst1s = [ZST1];
    let mut zst2s = [ZST2(())];
    let mut zst3s = [ZST3 { empty: () }];

    let slices = Slices::new(&context, (&zst1s, &zst2s, &zst3s));
    assert_eq!(slices.len(), 1);
    assert!(slices.contains((&ZST1, &ZST2(()), &ZST3 { empty: () })));

    assert_eq!(
        slices.as_slices(),
        (zst1s.as_slice(), zst2s.as_slice(), zst3s.as_slice()),
    );
    assert_eq!(
        slices.into_index(..),
        (zst1s.as_slice(), zst2s.as_slice(), zst3s.as_slice()),
    );
    assert_eq!(
        slices.index(0..),
        (zst1s.as_slice(), zst2s.as_slice(), zst3s.as_slice()),
    );
    assert_eq!(
        slices.index(..0),
        ([].as_slice(), [].as_slice(), [].as_slice()),
    );
    assert_eq!(slices.get(0), Some((&ZST1, &ZST2(()), &ZST3 { empty: () })));
    assert_eq!(slices.as_ref(), &slices);
    assert_ne!(slices, Slices::from(&context));
    assert_eq!(
        format!("{slices:?}"),
        "SoaSlices(([ZST1], [ZST2(())], [ZST3 { empty: () }]))",
    );

    let mut iter = slices.into_iter();
    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some((&ZST1, &ZST2(()), &ZST3 { empty: () })));
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    assert_equal(slices, &slices);

    let mut slices_mut = SlicesMut::new(&context, (&mut zst1s, &mut zst2s, &mut zst3s));
    assert_eq!(slices_mut.len(), 1);
    assert_eq!(
        slices_mut.index_mut(..),
        (
            [ZST1; 1].as_mut_slice(),
            [ZST2(()); 1].as_mut_slice(),
            [ZST3 { empty: () }; 1].as_mut_slice(),
        ),
    );
    assert_eq!(
        slices_mut.index_mut(..0),
        ([].as_mut_slice(), [].as_mut_slice(), [].as_mut_slice()),
    );
    assert_eq!(
        slices_mut.index_mut(0),
        (&mut ZST1, &mut ZST2(()), &mut ZST3 { empty: () }),
    );
    assert!(slices_mut.contains((&ZST1, &ZST2(()), &ZST3 { empty: () })));

    let permutation: [_; 1] = array::from_fn(identity);
    slices_mut.sort_unstable_with_permutation(permutation);
    assert_eq!(
        slices_mut.as_mut_slices(),
        (
            [ZST1].as_mut_slice(),
            [ZST2(())].as_mut_slice(),
            [ZST3 { empty: () }].as_mut_slice(),
        ),
    );

    let eq_mut = [(&mut ZST1, &mut ZST2(()), &mut ZST3 { empty: () })];
    assert_equal(&mut slices_mut, eq_mut);

    slices_mut.copy_from_slices(&Slices::new(
        &context,
        (&[ZST1], &[ZST2(())], &[ZST3 { empty: () }]),
    ));
    assert_eq!(slices_mut.len(), 1);
    assert_eq!(
        slices_mut.as_mut_slices(),
        (
            [ZST1].as_mut_slice(),
            [ZST2(())].as_mut_slice(),
            [ZST3 { empty: () }].as_mut_slice(),
        ),
    );
    assert_eq!(
        slices_mut.index_mut(0),
        (&mut ZST1, &mut ZST2(()), &mut ZST3 { empty: () }),
    );
    assert_eq!(slices_mut.as_ref(), &slices_mut);
    assert_ne!(slices_mut, SlicesMut::from(&context));
    assert_ne!(slices_mut.as_mut(), &mut SlicesMut::from(&context));
    assert_eq!(
        format!("{slices_mut:?}"),
        "SoaSlicesMut(([ZST1], [ZST2(())], [ZST3 { empty: () }]))",
    );

    let slices = Slices::from(slices_mut);
    assert_eq!(slices.len(), 1);
    assert_eq!(slices.index(0), (&ZST1, &ZST2(()), &ZST3 { empty: () }));
    assert_eq!(
        slices.as_slices(),
        (
            [ZST1].as_slice(),
            [ZST2(())].as_slice(),
            [ZST3 { empty: () }].as_slice(),
        ),
    );

    let mut iter = slices.into_iter();
    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some((&ZST1, &ZST2(()), &ZST3 { empty: () })));
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    assert_equal(slices, &slices);

    assert_eq!(zst1s, [ZST1]);
    assert_eq!(zst2s, [ZST2(())]);
    assert_eq!(zst3s, [ZST3 { empty: () }]);
}

#[test]
fn three_items() {
    type Item = (u16, String, u128, ());
    type Slices<'ctx, 'a> = SoaSlices<'ctx, 'a, Item>;
    type SlicesMut<'ctx, 'a> = SoaSlicesMut<'ctx, 'a, Item>;

    let context = Default::default();
    let mut u8s = [1, 2, 3];
    let mut strings = ["4".into(), "5".into(), "6".into()];
    let mut u64s = [7, 8, 9];
    let mut units = [(), (), ()];

    let mut slices_mut = SlicesMut::new(&context, (&mut u8s, &mut strings, &mut u64s, &mut units));
    assert_eq!(slices_mut.len(), 3);
    assert_eq!(
        slices_mut.index_mut(0),
        (&mut 1, &mut "4".into(), &mut 7, &mut ()),
    );
    assert_eq!(
        slices_mut.index_mut(1),
        (&mut 2, &mut "5".into(), &mut 8, &mut ()),
    );
    assert_eq!(
        slices_mut.index_mut(2),
        (&mut 3, &mut "6".into(), &mut 9, &mut ()),
    );
    assert_eq!(slices_mut.get_mut(3), None);

    assert_eq!(
        slices_mut.as_mut_slices(),
        (
            [1, 2, 3].as_mut_slice(),
            ["4".into(), "5".into(), "6".into()].as_mut_slice(),
            [7, 8, 9].as_mut_slice(),
            [(), (), ()].as_mut_slice(),
        ),
    );
    assert_eq!(
        slices_mut.index_mut(..),
        (
            [1, 2, 3].as_mut_slice(),
            ["4".into(), "5".into(), "6".into()].as_mut_slice(),
            [7, 8, 9].as_mut_slice(),
            [(), (), ()].as_mut_slice(),
        ),
    );
    assert_eq!(
        slices_mut.index_mut(..1),
        (
            [1].as_mut_slice(),
            ["4".into()].as_mut_slice(),
            [7].as_mut_slice(),
            [()].as_mut_slice(),
        ),
    );
    assert_eq!(
        slices_mut.index(1..),
        (
            [2, 3].as_slice(),
            ["5".into(), "6".into()].as_slice(),
            [8, 9].as_slice(),
            [(), ()].as_slice(),
        ),
    );
    assert_eq!(slices_mut.as_ref(), &slices_mut);
    assert_ne!(slices_mut, SlicesMut::from(&context));
    assert_ne!(slices_mut.as_mut(), &mut SlicesMut::from(&context));
    assert_eq!(
        format!("{slices_mut:?}"),
        r#"SoaSlicesMut(([1, 2, 3], ["4", "5", "6"], [7, 8, 9], [(), (), ()]))"#,
    );

    let mut iter = slices_mut.iter_mut();
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
    assert_equal(&mut slices_mut, eq_mut);

    let first = Slices::new(&context, slices_mut.index(..=1));
    let second = Slices::new(&context, slices_mut.index(1..));

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

    let mut sub_slices = SlicesMut::new(&context, slices_mut.index_mut(1..));
    assert_eq!(sub_slices.len(), 2);
    assert_eq!(sub_slices.index(0), (&2, &"5".into(), &8, &()));
    assert_eq!(sub_slices.index(1), (&3, &"6".into(), &9, &()));
    assert_eq!(sub_slices.get(2), None);
    assert_eq!(
        sub_slices.as_slices(),
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
    let mut gr_slices = SlicesMut::new(
        &context,
        (&mut gr_u8s, &mut gr_strings, &mut gr_u64s, &mut gr_units),
    );

    assert_ne!(sub_slices.as_mut_slices(), gr_slices.as_mut_slices());
    assert_ne!(sub_slices, gr_slices);

    assert!(sub_slices.as_mut_slices() < gr_slices.as_mut_slices());
    assert!(sub_slices < gr_slices);

    assert_eq!(
        sub_slices.cmp(&gr_slices),
        sub_slices.as_mut_slices().cmp(&gr_slices.as_mut_slices()),
    );

    let hasher = FxBuildHasher::default();
    assert_ne!(
        hasher.hash_one(sub_slices.as_mut_slices()),
        hasher.hash_one(gr_slices.as_mut_slices()),
    );
    assert_ne!(hasher.hash_one(&sub_slices), hasher.hash_one(&gr_slices));

    assert_eq!(
        hasher.hash_one(sub_slices.as_mut_slices()),
        hasher.hash_one(&sub_slices),
    );
    assert_eq!(
        hasher.hash_one(gr_slices.as_mut_slices()),
        hasher.hash_one(&gr_slices),
    );

    sub_slices.clone_from_slices(&Slices::new(
        &context,
        (&[0, 0], &["0".into(), "0".into()], &[0, 0], &[(), ()]),
    ));
    assert_eq!(sub_slices.len(), 2);
    assert_eq!(
        sub_slices.as_slices(),
        (
            [0, 0].as_slice(),
            ["0".into(), "0".into()].as_slice(),
            [0, 0].as_slice(),
            [(), ()].as_slice(),
        ),
    );

    assert_eq!(slices_mut.index(0), (&1, &"4".into(), &7, &()));
    assert_eq!(slices_mut.index(1), (&0, &"0".into(), &0, &()));
    assert_eq!(slices_mut.index(2), (&0, &"0".into(), &0, &()));
    assert_eq!(slices_mut.get(3), None);
    assert_eq!(
        slices_mut.as_mut_slices(),
        (
            [1, 0, 0].as_mut_slice(),
            ["4".into(), "0".into(), "0".into()].as_mut_slice(),
            [7, 0, 0].as_mut_slice(),
            [(), (), ()].as_mut_slice(),
        ),
    );
    assert_eq!(
        format!("{slices_mut:?}"),
        r#"SoaSlicesMut(([1, 0, 0], ["4", "0", "0"], [7, 0, 0], [(), (), ()]))"#,
    );

    let permutation: [_; 3] = array::from_fn(identity);
    slices_mut.sort_unstable_with_permutation_by_key(permutation, |(_, _, _, &key)| key);
    assert_eq!(
        slices_mut.as_mut_slices(),
        (
            [1, 0, 0].as_mut_slice(),
            ["4".into(), "0".into(), "0".into()].as_mut_slice(),
            [7, 0, 0].as_mut_slice(),
            [(), (), ()].as_mut_slice(),
        ),
    );

    slices_mut.sort_unstable_with_permutation(permutation);
    assert_eq!(
        slices_mut.as_mut_slices(),
        (
            [0, 0, 1].as_mut_slice(),
            ["0".into(), "0".into(), "4".into()].as_mut_slice(),
            [0, 0, 7].as_mut_slice(),
            [(), (), ()].as_mut_slice(),
        ),
    );

    let sub_slices = unsafe { slices_mut.get_unchecked(..=0) };
    let sub_slices = unsafe { SoaContext::<Item>::slice_ptrs_to_slices(&context, sub_slices) };
    assert_eq!(
        sub_slices,
        (
            [0].as_slice(),
            ["0".into()].as_slice(),
            [0].as_slice(),
            [()].as_slice(),
        ),
    );

    let sub_slices = unsafe { slices_mut.get_unchecked_mut(..=1) };
    let sub_slices =
        unsafe { SoaContext::<Item>::mut_slice_ptrs_to_mut_slices(&context, sub_slices) };
    assert_eq!(
        sub_slices,
        (
            [0, 0].as_mut_slice(),
            ["0".into(), "0".into()].as_mut_slice(),
            [0, 0].as_mut_slice(),
            [(), ()].as_mut_slice(),
        ),
    );

    let slices = Slices::from(slices_mut);
    assert_eq!(slices.len(), 3);
    assert_eq!(slices.index(0), (&0, &"0".into(), &0, &()));
    assert_eq!(slices.index(1), (&0, &"0".into(), &0, &()));
    assert_eq!(slices.index(2), (&1, &"4".into(), &7, &()));
    assert_eq!(slices.get(3), None);
    assert_eq!(
        slices.as_slices(),
        (
            [0, 0, 1].as_slice(),
            ["0".into(), "0".into(), "4".into()].as_slice(),
            [0, 0, 7].as_slice(),
            [(), (), ()].as_slice(),
        ),
    );
    assert_eq!(slices.as_ref(), &slices);
    assert_ne!(slices, Slices::from(&context));
    assert_eq!(
        format!("{slices:?}"),
        r#"SoaSlices(([0, 0, 1], ["0", "0", "4"], [0, 0, 7], [(), (), ()]))"#,
    );

    let sub_slices = unsafe { slices.into_get_unchecked(..1) };
    let sub_slices = unsafe { SoaContext::<Item>::slice_ptrs_to_slices(&context, sub_slices) };
    assert_eq!(
        sub_slices,
        (
            [0].as_slice(),
            ["0".into()].as_slice(),
            [0].as_slice(),
            [()].as_slice(),
        ),
    );

    let mut iter = slices.into_iter();
    assert_eq!(iter.len(), 3);
    assert_eq!(iter.next(), Some((&0, &"0".into(), &0, &())));

    assert_eq!(iter.len(), 2);
    assert_eq!(iter.next_back(), Some((&1, &"4".into(), &7, &())));

    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some((&0, &"0".into(), &0, &())));

    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);

    assert_equal(slices, &slices);

    assert_eq!(u8s, [0, 0, 1]);
    assert_eq!(strings, ["0", "0", "4"]);
    assert_eq!(u64s, [0, 0, 7]);
    assert_eq!(units, [(), (), ()]);
}

#[test]
fn three_items_unit() {
    type Item = ();
    type Slices<'ctx, 'a> = SoaSlices<'ctx, 'a, Item>;
    type SlicesMut<'ctx, 'a> = SoaSlicesMut<'ctx, 'a, Item>;

    let context = Default::default();
    let mut units = [(); 3];

    let mut slices_mut = SlicesMut::new(&context, &mut units);
    assert_eq!(slices_mut.len(), 3);
    assert_eq!(slices_mut[0], ());
    assert_eq!(&slices_mut[1], &());
    assert_eq!(&mut slices_mut[2], &mut ());
    assert_eq!(slices_mut.get_mut(3), None);

    assert_eq!(slices_mut.as_mut_slices(), [(); 3]);
    assert_eq!(&mut slices_mut[..], [(); 3]);
    assert_eq!(slices_mut[..1], [(); 1]);
    assert_eq!(&slices_mut[1..], [(); 2]);
    assert_eq!(slices_mut.as_mut(), [(); 3]);
    assert_ne!(slices_mut, SlicesMut::from(&context));
    assert_eq!(format!("{slices_mut:?}"), "SoaSlicesMut([(), (), ()])");

    let mut iter = slices_mut.iter_mut();
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
    assert_equal(&mut slices_mut, eq_mut);

    let first = Slices::new(&context, slices_mut.index(..=1));
    let second = Slices::new(&context, slices_mut.index(1..));

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

    let mut sub_slices = SlicesMut::new(&context, &mut slices_mut[1..]);
    assert_eq!(sub_slices.len(), 2);
    assert_eq!(sub_slices[0], ());
    assert_eq!(sub_slices[1], ());
    assert_eq!(sub_slices.get(2), None);
    assert_eq!(sub_slices.as_slices(), [(); 2]);

    let mut gr_data = [(); 2]; // the last one is greater
    let mut gr_slices = SlicesMut::new(&context, &mut gr_data);

    assert_eq!(sub_slices.as_mut_slices(), gr_slices.as_mut_slices());
    assert_eq!(sub_slices, gr_slices);

    assert_eq!(
        sub_slices.cmp(&gr_slices),
        sub_slices.as_mut_slices().cmp(&gr_slices.as_mut_slices()),
    );

    let hasher = FxBuildHasher::default();
    assert_eq!(
        hasher.hash_one(sub_slices.as_mut_slices()),
        hasher.hash_one(gr_slices.as_mut_slices()),
    );
    assert_eq!(hasher.hash_one(&sub_slices), hasher.hash_one(&gr_slices));

    assert_eq!(
        hasher.hash_one(sub_slices.as_mut_slices()),
        hasher.hash_one(&sub_slices),
    );
    assert_eq!(
        hasher.hash_one(gr_slices.as_mut_slices()),
        hasher.hash_one(&gr_slices),
    );

    sub_slices.clone_from_slices(&Slices::new(&context, &[(); 2]));
    assert_eq!(sub_slices.len(), 2);
    assert_eq!(sub_slices.as_slices(), [(); 2]);

    assert_eq!(slices_mut[0], ());
    assert_eq!(&slices_mut[1], &());
    assert_eq!(&mut slices_mut[2], &mut ());
    assert_eq!(slices_mut.get_mut(3), None);
    assert_eq!(slices_mut.as_mut_slices(), [(); 3]);
    assert_eq!(format!("{slices_mut:?}"), "SoaSlicesMut([(), (), ()])");

    let permutation: [_; 3] = array::from_fn(identity);
    slices_mut.sort_unstable_with_permutation_by_key(permutation, |&item| Reverse(item));
    assert_eq!(slices_mut.as_mut_slices(), [(); 3]);

    slices_mut.sort_unstable_with_permutation(permutation);
    assert_eq!(slices_mut.as_mut_slices(), [(); 3]);

    let sub_slices = unsafe { slices_mut.get_unchecked(..=0) };
    let sub_slices = unsafe { SoaContext::<Item>::slice_ptrs_to_slices(&context, sub_slices) };
    assert_eq!(sub_slices, [()]);

    let sub_slices_mut = unsafe { slices_mut.get_unchecked_mut(..=1) };
    let sub_slices_mut =
        unsafe { SoaContext::<Item>::mut_slice_ptrs_to_mut_slices(&context, sub_slices_mut) };
    assert_eq!(sub_slices_mut, [(); 2]);

    let slices = Slices::from(slices_mut);
    assert_eq!(slices.len(), 3);
    assert_eq!(slices[0], ());
    assert_eq!(slices[1], ());
    assert_eq!(slices[2], ());
    assert_eq!(slices.get(3), None);
    assert_eq!(slices.as_ref(), [(); 3]);
    assert_ne!(slices, Slices::from(&context));
    assert_eq!(format!("{slices:?}"), "SoaSlices([(), (), ()])");

    let sub_slices = unsafe { slices.into_get_unchecked(..1) };
    let sub_slices = unsafe { SoaContext::<Item>::slice_ptrs_to_slices(&context, sub_slices) };
    assert_eq!(sub_slices, [()]);

    let mut iter = slices.into_iter();
    assert_eq!(iter.len(), 3);
    assert_eq!(iter.next(), Some(&()));

    assert_eq!(iter.len(), 2);
    assert_eq!(iter.next_back(), Some(&()));

    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some(&()));

    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);

    assert_equal(slices, &slices);

    assert_eq!(units, [(); 3]);
}

#[test]
fn three_items_identity() {
    type Item = Identity<u128>;
    type Slices<'ctx, 'a> = SoaSlices<'ctx, 'a, Item>;
    type SlicesMut<'ctx, 'a> = SoaSlicesMut<'ctx, 'a, Item>;

    let context = Default::default();
    let mut data = [1.into(), 2.into(), 3.into()];

    let mut slices_mut = SlicesMut::new(&context, &mut data);
    assert_eq!(slices_mut.len(), 3);
    assert_eq!(slices_mut[0], 1.into());
    assert_eq!(&slices_mut[1], &2.into());
    assert_eq!(&mut slices_mut[2], &mut 3.into());
    assert_eq!(slices_mut.get_mut(3), None);

    assert_eq!(slices_mut.as_mut_slices(), [1.into(), 2.into(), 3.into()]);
    assert_eq!(&mut slices_mut[..], [1.into(), 2.into(), 3.into()]);
    assert_eq!(slices_mut[..1], [1.into()]);
    assert_eq!(&slices_mut[1..], [2.into(), 3.into()]);
    assert_eq!(slices_mut.as_mut(), [1.into(), 2.into(), 3.into()]);
    assert_ne!(slices_mut, SlicesMut::from(&context));
    assert_eq!(
        format!("{slices_mut:?}"),
        "SoaSlicesMut([Identity(1), Identity(2), Identity(3)])",
    );

    let mut iter = slices_mut.iter_mut();
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
    assert_equal(&mut slices_mut, eq_mut);

    let first = Slices::new(&context, slices_mut.index(..=1));
    let second = Slices::new(&context, slices_mut.index(1..));

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

    let mut sub_slices = SlicesMut::new(&context, &mut slices_mut[1..]);
    assert_eq!(sub_slices.len(), 2);
    assert_eq!(sub_slices[0], 2.into());
    assert_eq!(sub_slices[1], 3.into());
    assert_eq!(sub_slices.get(2), None);
    assert_eq!(sub_slices.as_slices(), [2.into(), 3.into()]);

    let mut gr_data = [2.into(), 42.into()]; // the last one is greater
    let mut gr_slices = SlicesMut::new(&context, &mut gr_data);

    assert_ne!(sub_slices.as_mut_slices(), gr_slices.as_mut_slices());
    assert_ne!(sub_slices, gr_slices);

    assert!(sub_slices.as_mut_slices() < gr_slices.as_mut_slices());
    assert!(sub_slices < gr_slices);

    assert_eq!(
        sub_slices.cmp(&gr_slices),
        sub_slices.as_mut_slices().cmp(&gr_slices.as_mut_slices()),
    );

    let hasher = FxBuildHasher::default();
    assert_ne!(
        hasher.hash_one(sub_slices.as_mut_slices()),
        hasher.hash_one(gr_slices.as_mut_slices()),
    );
    assert_ne!(hasher.hash_one(&sub_slices), hasher.hash_one(&gr_slices));

    assert_eq!(
        hasher.hash_one(sub_slices.as_mut_slices()),
        hasher.hash_one(&sub_slices),
    );
    assert_eq!(
        hasher.hash_one(gr_slices.as_mut_slices()),
        hasher.hash_one(&gr_slices),
    );

    sub_slices.clone_from_slices(&Slices::new(&context, &[4.into(), 2.into()]));
    assert_eq!(sub_slices.len(), 2);
    assert_eq!(sub_slices.as_slices(), [4.into(), 2.into()]);

    assert_eq!(slices_mut[0], 1.into());
    assert_eq!(&slices_mut[1], &4.into());
    assert_eq!(&mut slices_mut[2], &mut 2.into());
    assert_eq!(slices_mut.get_mut(3), None);
    assert_eq!(slices_mut.as_mut_slices(), [1.into(), 4.into(), 2.into()]);
    assert_eq!(
        format!("{slices_mut:?}"),
        "SoaSlicesMut([Identity(1), Identity(4), Identity(2)])",
    );

    let permutation: [_; 3] = array::from_fn(identity);
    slices_mut.sort_unstable_with_permutation_by_key(permutation, |&item| Reverse(item));
    assert_eq!(slices_mut.as_mut_slices(), [4.into(), 2.into(), 1.into()]);

    slices_mut.sort_unstable_with_permutation(permutation);
    assert_eq!(slices_mut.as_mut_slices(), [1.into(), 2.into(), 4.into()]);

    let sub_slices = unsafe { slices_mut.get_unchecked(..=0) };
    let sub_slices = unsafe { SoaContext::<Item>::slice_ptrs_to_slices(&context, sub_slices) };
    assert_eq!(sub_slices, [1.into()]);

    let sub_slices_mut = unsafe { slices_mut.get_unchecked_mut(..=1) };
    let sub_slices_mut =
        unsafe { SoaContext::<Item>::mut_slice_ptrs_to_mut_slices(&context, sub_slices_mut) };
    assert_eq!(sub_slices_mut, [1.into(), 2.into()]);

    let slices = Slices::from(slices_mut);
    assert_eq!(slices.len(), 3);
    assert_eq!(slices[0], 1.into());
    assert_eq!(slices[1], 2.into());
    assert_eq!(slices[2], 4.into());
    assert_eq!(slices.get(3), None);
    assert_eq!(slices.as_ref(), [1.into(), 2.into(), 4.into()]);
    assert_ne!(slices, Slices::from(&context));
    assert_eq!(
        format!("{slices:?}"),
        "SoaSlices([Identity(1), Identity(2), Identity(4)])",
    );

    let sub_slices = unsafe { slices.into_get_unchecked(..1) };
    let sub_slices = unsafe { SoaContext::<Item>::slice_ptrs_to_slices(&context, sub_slices) };
    assert_eq!(sub_slices, [1.into()]);

    let mut iter = slices.into_iter();
    assert_eq!(iter.len(), 3);
    assert_eq!(iter.next(), Some(&1.into()));

    assert_eq!(iter.len(), 2);
    assert_eq!(iter.next_back(), Some(&4.into()));

    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some(&2.into()));

    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);

    assert_equal(slices, &slices);

    assert_eq!(data, [1.into(), 2.into(), 4.into()]);
}

#[test]
fn three_items_zst() {
    type Item = (ZST1, ZST2, ZST3);
    type Slices<'ctx, 'a> = SoaSlices<'ctx, 'a, Item>;
    type SlicesMut<'ctx, 'a> = SoaSlicesMut<'ctx, 'a, Item>;

    let context = Default::default();
    let mut zst1s = [ZST1; 3];
    let mut zst2s = [ZST2(()); 3];
    let mut zst3s = [ZST3 { empty: () }; 3];

    let mut slices_mut = SlicesMut::new(&context, (&mut zst1s, &mut zst2s, &mut zst3s));
    assert_eq!(slices_mut.len(), 3);
    assert_eq!(
        slices_mut.index_mut(0),
        (&mut ZST1, &mut ZST2(()), &mut ZST3 { empty: () }),
    );
    assert_eq!(
        slices_mut.index_mut(1),
        (&mut ZST1, &mut ZST2(()), &mut ZST3 { empty: () }),
    );
    assert_eq!(
        slices_mut.index_mut(2),
        (&mut ZST1, &mut ZST2(()), &mut ZST3 { empty: () }),
    );
    assert_eq!(slices_mut.get_mut(3), None);

    assert_eq!(
        slices_mut.as_mut_slices(),
        (
            [ZST1; 3].as_mut_slice(),
            [ZST2(()); 3].as_mut_slice(),
            [ZST3 { empty: () }; 3].as_mut_slice(),
        ),
    );
    assert_eq!(
        slices_mut.index_mut(..),
        (
            [ZST1; 3].as_mut_slice(),
            [ZST2(()); 3].as_mut_slice(),
            [ZST3 { empty: () }; 3].as_mut_slice(),
        ),
    );
    assert_eq!(
        slices_mut.index_mut(..1),
        (
            [ZST1; 1].as_mut_slice(),
            [ZST2(()); 1].as_mut_slice(),
            [ZST3 { empty: () }; 1].as_mut_slice(),
        ),
    );
    assert_eq!(
        slices_mut.index(1..),
        (
            [ZST1; 2].as_slice(),
            [ZST2(()); 2].as_slice(),
            [ZST3 { empty: () }; 2].as_slice(),
        ),
    );
    assert_eq!(slices_mut.as_ref(), &slices_mut);
    assert_ne!(slices_mut, SlicesMut::from(&context));
    assert_ne!(slices_mut.as_mut(), &mut SlicesMut::from(&context));
    assert_eq!(
        format!("{slices_mut:?}"),
        r#"SoaSlicesMut(([ZST1, ZST1, ZST1], [ZST2(()), ZST2(()), ZST2(())], [ZST3 { empty: () }, ZST3 { empty: () }, ZST3 { empty: () }]))"#,
    );

    let mut iter = slices_mut.iter_mut();
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
    assert_equal(&mut slices_mut, eq_mut);

    let mut sub_slices = SlicesMut::new(&context, slices_mut.index_mut(1..));
    assert_eq!(sub_slices.len(), 2);
    assert_eq!(sub_slices.index(0), (&ZST1, &ZST2(()), &ZST3 { empty: () }));
    assert_eq!(sub_slices.index(1), (&ZST1, &ZST2(()), &ZST3 { empty: () }));
    assert_eq!(sub_slices.get(2), None);
    assert_eq!(
        sub_slices.as_slices(),
        (
            [ZST1; 2].as_slice(),
            [ZST2(()); 2].as_slice(),
            [ZST3 { empty: () }; 2].as_slice(),
        ),
    );

    sub_slices.clone_from_slices(&Slices::new(
        &context,
        (
            [ZST1; 2].as_slice(),
            [ZST2(()); 2].as_slice(),
            [ZST3 { empty: () }; 2].as_slice(),
        ),
    ));
    assert_eq!(sub_slices.len(), 2);
    assert_eq!(
        sub_slices.as_slices(),
        (
            [ZST1; 2].as_slice(),
            [ZST2(()); 2].as_slice(),
            [ZST3 { empty: () }; 2].as_slice(),
        ),
    );

    assert_eq!(slices_mut.index(0), (&ZST1, &ZST2(()), &ZST3 { empty: () }));
    assert_eq!(slices_mut.index(1), (&ZST1, &ZST2(()), &ZST3 { empty: () }));
    assert_eq!(slices_mut.index(2), (&ZST1, &ZST2(()), &ZST3 { empty: () }));
    assert_eq!(slices_mut.get(3), None);
    assert_eq!(
        slices_mut.as_mut_slices(),
        (
            [ZST1; 3].as_mut_slice(),
            [ZST2(()); 3].as_mut_slice(),
            [ZST3 { empty: () }; 3].as_mut_slice(),
        ),
    );
    assert_eq!(
        format!("{slices_mut:?}"),
        r#"SoaSlicesMut(([ZST1, ZST1, ZST1], [ZST2(()), ZST2(()), ZST2(())], [ZST3 { empty: () }, ZST3 { empty: () }, ZST3 { empty: () }]))"#,
    );

    let permutation: [_; 3] = array::from_fn(identity);
    slices_mut.sort_unstable_with_permutation_by_key(permutation, |(_, &key, _)| Reverse(key));
    assert_eq!(
        slices_mut.as_mut_slices(),
        (
            [ZST1; 3].as_mut_slice(),
            [ZST2(()); 3].as_mut_slice(),
            [ZST3 { empty: () }; 3].as_mut_slice(),
        ),
    );

    slices_mut.sort_unstable_with_permutation(permutation);
    assert_eq!(
        slices_mut.as_mut_slices(),
        (
            [ZST1; 3].as_mut_slice(),
            [ZST2(()); 3].as_mut_slice(),
            [ZST3 { empty: () }; 3].as_mut_slice(),
        ),
    );

    let sub_slices = unsafe { slices_mut.get_unchecked(..=0) };
    let sub_slices = unsafe { SoaContext::<Item>::slice_ptrs_to_slices(&context, sub_slices) };
    assert_eq!(
        sub_slices,
        (
            [ZST1; 1].as_slice(),
            [ZST2(()); 1].as_slice(),
            [ZST3 { empty: () }; 1].as_slice(),
        ),
    );

    let sub_slices = unsafe { slices_mut.get_unchecked_mut(..=1) };
    let sub_slices =
        unsafe { SoaContext::<Item>::mut_slice_ptrs_to_mut_slices(&context, sub_slices) };
    assert_eq!(
        sub_slices,
        (
            [ZST1; 2].as_mut_slice(),
            [ZST2(()); 2].as_mut_slice(),
            [ZST3 { empty: () }; 2].as_mut_slice(),
        ),
    );

    let slices = Slices::from(slices_mut);
    assert_eq!(slices.len(), 3);
    assert_eq!(slices.index(0), (&ZST1, &ZST2(()), &ZST3 { empty: () }));
    assert_eq!(slices.index(1), (&ZST1, &ZST2(()), &ZST3 { empty: () }));
    assert_eq!(slices.index(2), (&ZST1, &ZST2(()), &ZST3 { empty: () }));
    assert_eq!(slices.get(3), None);
    assert_eq!(
        slices.as_slices(),
        (
            [ZST1; 3].as_slice(),
            [ZST2(()); 3].as_slice(),
            [ZST3 { empty: () }; 3].as_slice(),
        ),
    );
    assert_eq!(slices.as_ref(), &slices);
    assert_ne!(slices, Slices::from(&context));
    assert_eq!(
        format!("{slices:?}"),
        r#"SoaSlices(([ZST1, ZST1, ZST1], [ZST2(()), ZST2(()), ZST2(())], [ZST3 { empty: () }, ZST3 { empty: () }, ZST3 { empty: () }]))"#,
    );

    let sub_slices = unsafe { slices.into_get_unchecked(..1) };
    let sub_slices = unsafe { SoaContext::<Item>::slice_ptrs_to_slices(&context, sub_slices) };
    assert_eq!(
        sub_slices,
        (
            [ZST1; 1].as_slice(),
            [ZST2(()); 1].as_slice(),
            [ZST3 { empty: () }; 1].as_slice(),
        ),
    );

    let mut iter = slices.into_iter();
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

    assert_equal(slices, &slices);

    assert_eq!(zst1s, [ZST1; 3]);
    assert_eq!(zst2s, [ZST2(()); 3]);
    assert_eq!(zst3s, [ZST3 { empty: () }; 3]);
}
