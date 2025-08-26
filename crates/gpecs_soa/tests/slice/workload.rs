use gpecs_soa::prelude::*;

use crate::common::{ZST1, ZST2, ZST3};

#[test]
fn empty() {
    type Slices<'c, 'a> = SoaSlices<'c, 'a, (u8, u64, u16, ())>;
    type SlicesMut<'c, 'a> = SoaSlicesMut<'c, 'a, (u8, u64, u16, ())>;

    let context = ();

    let slices = Slices::new(&context, (&[], &[], &[], &[]));
    assert!(slices.is_empty());
    assert_eq!(slices.get(0), None);
    assert_eq!(format!("{slices:?}"), "SoaSlices(([], [], [], []))");

    let mut iter = slices.into_iter();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    let mut slices_mut = SlicesMut::new(&context, (&mut [], &mut [], &mut [], &mut []));
    assert!(slices_mut.is_empty());
    assert_eq!(slices_mut.get_mut(0), None);
    assert_eq!(format!("{slices_mut:?}"), "SoaSlicesMut(([], [], [], []))");

    let mut iter = slices_mut.iter_mut();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    let slices = Slices::from(slices_mut);
    assert!(slices.is_empty());
    assert_eq!(slices.get(0), None);
    assert_eq!(format!("{slices:?}"), "SoaSlices(([], [], [], []))");
}

#[test]
fn empty_zst() {
    type Slices<'c, 'a> = SoaSlices<'c, 'a, (ZST1, ZST2, ZST3)>;
    type SlicesMut<'c, 'a> = SoaSlicesMut<'c, 'a, (ZST1, ZST2, ZST3)>;

    let context = ();

    let slices = Slices::new(&context, (&[], &[], &[]));
    assert!(slices.is_empty());
    assert_eq!(slices.get(0), None);
    assert_eq!(format!("{slices:?}"), "SoaSlices(([], [], []))");

    let mut iter = slices.into_iter();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    let mut slices_mut = SlicesMut::new(&context, (&mut [], &mut [], &mut []));
    assert!(slices_mut.is_empty());
    assert_eq!(slices_mut.get_mut(0), None);
    assert_eq!(format!("{slices_mut:?}"), "SoaSlicesMut(([], [], []))");

    let mut iter = slices_mut.iter_mut();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    let slices = Slices::from(slices_mut);
    assert!(slices.is_empty());
    assert_eq!(slices.get(0), None);
    assert_eq!(format!("{slices:?}"), "SoaSlices(([], [], []))");
}

#[test]
fn one_item() {
    type Slices<'c, 'a> = SoaSlices<'c, 'a, (u8, u64, u16, ())>;
    type SlicesMut<'c, 'a> = SoaSlicesMut<'c, 'a, (u8, u64, u16, ())>;

    let context = ();
    let mut u8s = [1];
    let mut u64s = [2];
    let mut u16s = [3];
    let mut units = [()];

    let slices = Slices::new(&context, (&u8s, &u64s, &u16s, &units));
    assert_eq!(slices.len(), 1);
    assert!(slices.contains(&(&1, &2, &3, &())));

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
    assert_eq!(format!("{slices:?}"), "SoaSlices(([1], [2], [3], [()]))");

    let mut iter = slices.into_iter();
    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some((&1, &2, &3, &())));
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    let mut slices_mut = SlicesMut::new(&context, (&mut u8s, &mut u64s, &mut u16s, &mut units));
    assert_eq!(slices_mut.len(), 1);
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
    assert!(slices_mut.contains(&(&1, &2, &3, &())));

    slices_mut.copy_from_slices(&Slices::new(&context, (&[0], &[0], &[0], &[()])));
    assert_eq!(slices_mut.len(), 1);
    assert_eq!(slices_mut.index_mut(0), (&mut 0, &mut 0, &mut 0, &mut ()));
    assert!(!slices_mut.contains(&(&1, &2, &3, &())));
    assert_eq!(
        slices_mut.as_mut_slices(),
        (
            [0].as_mut_slice(),
            [0].as_mut_slice(),
            [0].as_mut_slice(),
            [()].as_mut_slice(),
        ),
    );
    assert_eq!(
        format!("{slices_mut:?}"),
        "SoaSlicesMut(([0], [0], [0], [()]))",
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

    assert_eq!(u8s, [0]);
    assert_eq!(u64s, [0]);
    assert_eq!(u16s, [0]);
    assert_eq!(units, [()]);
}

#[test]
fn one_item_zst() {
    type Slices<'c, 'a> = SoaSlices<'c, 'a, (ZST1, ZST2, ZST3)>;
    type SlicesMut<'c, 'a> = SoaSlicesMut<'c, 'a, (ZST1, ZST2, ZST3)>;

    let context = ();
    let mut zst1s = [ZST1];
    let mut zst2s = [ZST2(())];
    let mut zst3s = [ZST3 { empty: () }];

    let slices = Slices::new(&context, (&zst1s, &zst2s, &zst3s));
    assert_eq!(slices.len(), 1);
    assert!(slices.contains(&(&ZST1, &ZST2(()), &ZST3 { empty: () })));

    assert_eq!(
        slices.as_slices(),
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
    assert_eq!(
        format!("{slices:?}"),
        "SoaSlices(([ZST1], [ZST2(())], [ZST3 { empty: () }]))",
    );

    let mut iter = slices.into_iter();
    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some((&ZST1, &ZST2(()), &ZST3 { empty: () })));
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);

    let mut slices_mut = SlicesMut::new(&context, (&mut zst1s, &mut zst2s, &mut zst3s));
    assert_eq!(slices_mut.len(), 1);
    assert_eq!(
        slices_mut.index_mut(..0),
        ([].as_mut_slice(), [].as_mut_slice(), [].as_mut_slice()),
    );
    assert_eq!(
        slices_mut.index_mut(0),
        (&mut ZST1, &mut ZST2(()), &mut ZST3 { empty: () }),
    );
    assert!(slices_mut.contains(&(&ZST1, &ZST2(()), &ZST3 { empty: () })));

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

    assert_eq!(zst1s, [ZST1]);
    assert_eq!(zst2s, [ZST2(())]);
    assert_eq!(zst3s, [ZST3 { empty: () }]);
}

#[test]
fn three_items() {
    type Slices<'c, 'a> = SoaSlices<'c, 'a, (u8, String, u64, ())>;
    type SlicesMut<'c, 'a> = SoaSlicesMut<'c, 'a, (u8, String, u64, ())>;

    let context = ();
    let mut u8s = [1, 2, 3];
    let mut strings = ["4".to_owned(), "5".to_owned(), "6".to_owned()];
    let mut u64s = [7, 8, 9];
    let mut units = [(), (), ()];

    let mut slices_mut = SlicesMut::new(&context, (&mut u8s, &mut strings, &mut u64s, &mut units));
    assert_eq!(slices_mut.len(), 3);
    assert_eq!(
        slices_mut.index_mut(0),
        (&mut 1, &mut "4".to_owned(), &mut 7, &mut ()),
    );
    assert_eq!(
        slices_mut.index_mut(1),
        (&mut 2, &mut "5".to_owned(), &mut 8, &mut ()),
    );
    assert_eq!(
        slices_mut.index_mut(2),
        (&mut 3, &mut "6".to_owned(), &mut 9, &mut ()),
    );
    assert_eq!(slices_mut.get_mut(3), None);

    assert_eq!(
        slices_mut.as_mut_slices(),
        (
            [1, 2, 3].as_mut_slice(),
            ["4".to_owned(), "5".to_owned(), "6".to_owned()].as_mut_slice(),
            [7, 8, 9].as_mut_slice(),
            [(), (), ()].as_mut_slice(),
        ),
    );
    assert_eq!(
        slices_mut.index_mut(..1),
        (
            [1].as_mut_slice(),
            ["4".to_owned()].as_mut_slice(),
            [7].as_mut_slice(),
            [()].as_mut_slice(),
        ),
    );
    assert_eq!(
        slices_mut.index(1..),
        (
            [2, 3].as_slice(),
            ["5".to_owned(), "6".to_owned()].as_slice(),
            [8, 9].as_slice(),
            [(), ()].as_slice(),
        ),
    );
    assert_eq!(
        format!("{slices_mut:?}"),
        r#"SoaSlicesMut(([1, 2, 3], ["4", "5", "6"], [7, 8, 9], [(), (), ()]))"#,
    );

    let mut iter = slices_mut.iter_mut();
    assert_eq!(iter.len(), 3);
    assert_eq!(
        iter.next(),
        Some((&mut 1, &mut "4".to_owned(), &mut 7, &mut ())),
    );

    assert_eq!(iter.len(), 2);
    assert_eq!(
        iter.next_back(),
        Some((&mut 3, &mut "6".to_owned(), &mut 9, &mut ())),
    );

    assert_eq!(iter.len(), 1);
    assert_eq!(
        iter.next(),
        Some((&mut 2, &mut "5".to_owned(), &mut 8, &mut ())),
    );

    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);

    let mut sub_slices = SlicesMut::new(&context, slices_mut.index_mut(1..));
    assert_eq!(sub_slices.len(), 2);
    assert_eq!(sub_slices.index(0), (&2, &"5".to_owned(), &8, &()));
    assert_eq!(sub_slices.index(1), (&3, &"6".to_owned(), &9, &()));
    assert_eq!(sub_slices.get(2), None);
    assert_eq!(
        sub_slices.as_slices(),
        (
            [2, 3].as_slice(),
            ["5".to_owned(), "6".to_owned()].as_slice(),
            [8, 9].as_slice(),
            [(), ()].as_slice(),
        ),
    );

    sub_slices.clone_from_slices(&Slices::new(
        &context,
        (
            &[0, 0],
            &["0".to_owned(), "0".to_owned()],
            &[0, 0],
            &[(), ()],
        ),
    ));
    assert_eq!(sub_slices.len(), 2);
    assert_eq!(
        sub_slices.as_slices(),
        (
            [0, 0].as_slice(),
            ["0".to_owned(), "0".to_owned()].as_slice(),
            [0, 0].as_slice(),
            [(), ()].as_slice(),
        ),
    );

    assert_eq!(slices_mut.index(0), (&1, &"4".to_owned(), &7, &()));
    assert_eq!(slices_mut.index(1), (&0, &"0".to_owned(), &0, &()));
    assert_eq!(slices_mut.index(2), (&0, &"0".to_owned(), &0, &()));
    assert_eq!(slices_mut.get(3), None);
    assert_eq!(
        slices_mut.as_mut_slices(),
        (
            [1, 0, 0].as_mut_slice(),
            ["4".to_owned(), "0".to_owned(), "0".to_owned()].as_mut_slice(),
            [7, 0, 0].as_mut_slice(),
            [(), (), ()].as_mut_slice(),
        ),
    );
    assert_eq!(
        format!("{slices_mut:?}"),
        r#"SoaSlicesMut(([1, 0, 0], ["4", "0", "0"], [7, 0, 0], [(), (), ()]))"#,
    );

    let slices = Slices::from(slices_mut);
    assert_eq!(slices.len(), 3);
    assert_eq!(slices.index(0), (&1, &"4".to_owned(), &7, &()));
    assert_eq!(slices.index(1), (&0, &"0".to_owned(), &0, &()));
    assert_eq!(slices.index(2), (&0, &"0".to_owned(), &0, &()));
    assert_eq!(slices.get(3), None);
    assert_eq!(
        slices.as_slices(),
        (
            [1, 0, 0].as_slice(),
            ["4".to_owned(), "0".to_owned(), "0".to_owned()].as_slice(),
            [7, 0, 0].as_slice(),
            [(), (), ()].as_slice(),
        ),
    );
    assert_eq!(
        format!("{slices:?}"),
        r#"SoaSlices(([1, 0, 0], ["4", "0", "0"], [7, 0, 0], [(), (), ()]))"#,
    );

    let mut iter = slices.into_iter();
    assert_eq!(iter.len(), 3);
    assert_eq!(iter.next(), Some((&1, &"4".to_owned(), &7, &())));

    assert_eq!(iter.len(), 2);
    assert_eq!(iter.next_back(), Some((&0, &"0".to_owned(), &0, &())));

    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some((&0, &"0".to_owned(), &0, &())));

    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);

    assert_eq!(u8s, [1, 0, 0]);
    assert_eq!(strings, ["4".to_owned(), "0".to_owned(), "0".to_owned()]);
    assert_eq!(u64s, [7, 0, 0]);
    assert_eq!(units, [(), (), ()]);
}

#[test]
fn three_items_zst() {
    type Slices<'c, 'a> = SoaSlices<'c, 'a, (ZST1, ZST2, ZST3)>;
    type SlicesMut<'c, 'a> = SoaSlicesMut<'c, 'a, (ZST1, ZST2, ZST3)>;

    let context = ();
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
    assert_eq!(
        format!("{slices:?}"),
        r#"SoaSlices(([ZST1, ZST1, ZST1], [ZST2(()), ZST2(()), ZST2(())], [ZST3 { empty: () }, ZST3 { empty: () }, ZST3 { empty: () }]))"#,
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

    assert_eq!(zst1s, [ZST1; 3]);
    assert_eq!(zst2s, [ZST2(()); 3]);
    assert_eq!(zst3s, [ZST3 { empty: () }; 3]);
}
