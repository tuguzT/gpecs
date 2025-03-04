use std::{alloc::Layout, iter, ops::Not, ptr, slice, u64};

use gpecs_soa::{
    r#dyn::{DynSoa, DynSoaContext, DynSoaRefs, DynSoaSlices},
    vec::SoaVec,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct ZST1;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct ZST2(());

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct ZST3 {
    empty: (),
}

#[test]
fn new() {
    type Vec = SoaVec<(u32, u16, u8)>;

    let vec = Vec::from_iter([]);
    assert!(vec.is_empty());

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    assert_eq!(format!("{vec:?}"), "SoaVec(([], [], []))");

    let slice = vec.as_slice();
    assert!(slice.is_empty());

    assert_eq!(format!("{slice:?}"), "SoaSlice(([], [], []))");

    assert_eq!(vec, slice);
    assert!(vec >= slice);
    assert!(slice <= vec);

    assert_eq!(slice.to_owned(), vec.clone());

    let (context, vecs) = vec.into_vecs();
    assert_eq!(vecs, (vec![], vec![], vec![]));

    let vec = Vec::from_vecs(context, vecs);
    assert!(vec.is_empty());

    let boxed_slice = vec.into_boxed_slice();
    assert!(boxed_slice.is_empty());

    let vec = boxed_slice.into_vec();
    assert!(vec.is_empty());

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let boxed_slice = vec.into_boxed_slice();
    assert!(boxed_slice.is_empty());

    let into_iter = boxed_slice.into_iter();
    assert!(into_iter.is_empty());

    assert_eq!(format!("{into_iter:?}"), "IntoIter(([], [], []))");
}

#[test]
fn new_zst() {
    type Vec = SoaVec<(ZST1, ZST2, ZST3)>;

    let vec = Vec::from_iter([]);
    assert!(vec.is_empty());
    assert_eq!(vec.capacity(), usize::MAX);

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    assert_eq!(format!("{vec:?}"), "SoaVec(([], [], []))");

    let slice = vec.as_slice();
    assert!(slice.is_empty());
    assert_eq!(slice.capacity(), usize::MAX);

    assert_eq!(format!("{slice:?}"), "SoaSlice(([], [], []))");

    assert_eq!(vec, slice);
    assert!(vec >= slice);
    assert!(slice <= vec);

    assert_eq!(slice.to_owned(), vec.clone());

    let (context, vecs) = vec.into_vecs();
    assert_eq!(vecs, (vec![], vec![], vec![]));

    let vec = Vec::from_vecs(context, vecs);
    assert!(vec.is_empty());
    assert_eq!(vec.capacity(), usize::MAX);

    let boxed_slice = vec.into_boxed_slice();
    assert!(boxed_slice.is_empty());
    assert_eq!(boxed_slice.capacity(), usize::MAX);

    let vec = boxed_slice.into_vec();
    assert!(vec.is_empty());
    assert_eq!(vec.capacity(), usize::MAX);

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let boxed_slice = vec.into_boxed_slice();
    assert!(boxed_slice.is_empty());
    assert_eq!(boxed_slice.capacity(), usize::MAX);

    let into_iter = boxed_slice.into_iter();
    assert!(into_iter.is_empty());

    assert_eq!(format!("{into_iter:?}"), "IntoIter(([], [], []))");
}

#[test]
fn new_dyn() {
    type Vec = SoaVec<DynSoa<(u8, u64, u16)>>;

    let context = DynSoaContext::new([
        Layout::new::<u8>(),
        Layout::new::<u64>(),
        Layout::new::<u16>(),
    ]);
    let vec = Vec::with_context(context);
    assert!(vec.is_empty());

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    assert_eq!(format!("{vec:?}"), "SoaVec(DynSoaSlices([[], [], []]))");

    let slice = vec.as_slice();
    assert!(slice.is_empty());

    assert_eq!(format!("{slice:?}"), "SoaSlice(DynSoaSlices([[], [], []]))");

    assert_eq!(vec, slice);
    assert!(vec >= slice);
    assert!(slice <= vec);

    assert!(slice.to_owned().is_empty());

    let (context, vecs) = vec.into_vecs();
    // assert_eq!(vecs, (vec![], vec![], vec![]));

    let vec = Vec::from_vecs(context, vecs);
    assert!(vec.is_empty());

    let boxed_slice = vec.into_boxed_slice();
    assert!(boxed_slice.is_empty());

    let vec = boxed_slice.into_vec();
    assert!(vec.is_empty());

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let boxed_slice = vec.into_boxed_slice();
    assert!(boxed_slice.is_empty());

    let into_iter = boxed_slice.into_iter();
    assert!(into_iter.is_empty());

    assert_eq!(
        format!("{into_iter:?}"),
        "IntoIter(DynSoaSlices([[], [], []]))",
    );
}

#[test]
fn with_capacity() {
    type Vec = SoaVec<(u8, u64, u16)>;

    let vec = Vec::with_capacity(10);
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 10);

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    assert_eq!(format!("{vec:?}"), "SoaVec(([], [], []))");

    let slice = vec.as_slice();
    assert!(slice.is_empty());
    assert!(slice.capacity() >= 10);

    assert_eq!(format!("{slice:?}"), "SoaSlice(([], [], []))");

    assert_eq!(vec, slice);
    assert!(vec >= slice);
    assert!(slice <= vec);

    assert_eq!(slice.to_owned(), vec.clone());

    let (context, vecs) = vec.into_vecs();
    assert_eq!(vecs, (vec![], vec![], vec![]));

    let vec = Vec::from_vecs(context, vecs);
    assert!(vec.is_empty());

    let boxed_slice = vec.into_boxed_slice();
    assert!(boxed_slice.is_empty());

    let vec = boxed_slice.into_vec();
    assert!(vec.is_empty());

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let boxed_slice = vec.into_boxed_slice();
    assert!(boxed_slice.is_empty());

    let into_iter = boxed_slice.into_iter();
    assert!(into_iter.is_empty());

    assert_eq!(format!("{into_iter:?}"), "IntoIter(([], [], []))");
}

#[test]
fn with_capacity_zst() {
    type Vec = SoaVec<(ZST1, ZST2, ZST3)>;

    let vec = Vec::with_capacity(10);
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 10);

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    assert_eq!(format!("{vec:?}"), "SoaVec(([], [], []))");

    let slice = vec.as_slice();
    assert!(slice.is_empty());
    assert!(slice.capacity() >= 10);

    assert_eq!(format!("{slice:?}"), "SoaSlice(([], [], []))");

    assert_eq!(vec, slice);
    assert!(vec >= slice);
    assert!(slice <= vec);

    assert_eq!(slice.to_owned(), vec.clone());

    let (context, vecs) = vec.into_vecs();
    assert_eq!(vecs, (vec![], vec![], vec![]));

    let vec = Vec::from_vecs(context, vecs);
    assert!(vec.is_empty());

    let boxed_slice = vec.into_boxed_slice();
    assert!(boxed_slice.is_empty());
    assert_eq!(boxed_slice.capacity(), usize::MAX);

    let vec = boxed_slice.into_vec();
    assert!(vec.is_empty());
    assert_eq!(vec.capacity(), usize::MAX);

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let boxed_slice = vec.into_boxed_slice();
    assert!(boxed_slice.is_empty());
    assert_eq!(boxed_slice.capacity(), usize::MAX);

    let into_iter = boxed_slice.into_iter();
    assert!(into_iter.is_empty());

    assert_eq!(format!("{into_iter:?}"), "IntoIter(([], [], []))");
}

#[test]
fn with_capacity_dyn() {
    type Vec = SoaVec<DynSoa<(u8, u64, u16)>>;

    let context = DynSoaContext::new([
        Layout::new::<u8>(),
        Layout::new::<u64>(),
        Layout::new::<u16>(),
    ]);
    let vec = Vec::with_context_and_capacity(context, 10);
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 10);

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    assert_eq!(format!("{vec:?}"), "SoaVec(DynSoaSlices([[], [], []]))");

    let slice = vec.as_slice();
    assert!(slice.is_empty());
    assert!(slice.capacity() >= 10);

    assert_eq!(format!("{slice:?}"), "SoaSlice(DynSoaSlices([[], [], []]))");

    assert_eq!(vec, slice);
    assert!(vec >= slice);
    assert!(slice <= vec);

    assert!(slice.to_owned().is_empty());

    let (context, vecs) = vec.into_vecs();
    // assert_eq!(vecs, (vec![], vec![], vec![]));

    let vec = Vec::from_vecs(context, vecs);
    assert!(vec.is_empty());

    let boxed_slice = vec.into_boxed_slice();
    assert!(boxed_slice.is_empty());

    let vec = boxed_slice.into_vec();
    assert!(vec.is_empty());

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let boxed_slice = vec.into_boxed_slice();
    assert!(boxed_slice.is_empty());

    let into_iter = boxed_slice.into_iter();
    assert!(into_iter.is_empty());

    assert_eq!(
        format!("{into_iter:?}"),
        "IntoIter(DynSoaSlices([[], [], []]))",
    );
}

#[test]
fn one_item() {
    type Vec = SoaVec<(u8, u32, u16)>;

    let mut vec = Vec::new();
    vec.push((1, 2, 3));
    assert_eq!(vec.len(), 1);
    assert!(vec.capacity() >= 1);
    assert!(vec.contains_by_refs((&1, &2, &3)));

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    assert_eq!(format!("{vec:?}"), "SoaVec(([1], [2], [3]))");

    let slice = vec.as_slice();
    assert_eq!(slice.len(), 1);
    assert!(slice.capacity() >= 1);
    assert_eq!(
        slice.as_slices(),
        ([1].as_slice(), [2].as_slice(), [3].as_slice()),
    );
    assert_eq!(slice.get(0), Some((&1, &2, &3)));

    assert_eq!(format!("{slice:?}"), "SoaSlice(([1], [2], [3]))");

    assert_eq!(vec, slice);
    assert!(vec >= slice);
    assert!(slice <= vec);

    assert_eq!(slice.to_owned(), vec.clone());

    let (context, vecs) = vec.into_vecs();
    assert_eq!(vecs, (vec![1], vec![2], vec![3]));

    let mut vec = Vec::from_vecs(context, vecs);
    assert_eq!(vec.len(), 1);
    assert!(vec.capacity() >= 1);
    assert_eq!(
        vec.as_slices(),
        ([1].as_slice(), [2].as_slice(), [3].as_slice()),
    );

    let mut iter = vec.iter();
    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some((&1, &2, &3)));
    assert_eq!(iter.next(), None);

    vec.extend_from_within(..0);
    assert_eq!(
        vec.as_slices(),
        ([1].as_slice(), [2].as_slice(), [3].as_slice()),
    );

    vec.extend_from_within(..);
    assert_eq!(
        vec.as_slices(),
        ([1; 2].as_slice(), [2; 2].as_slice(), [3; 2].as_slice()),
    );
    assert_eq!(format!("{vec:?}"), "SoaVec(([1, 1], [2, 2], [3, 3]))");

    vec.clone_from_slice(Vec::from_iter([Default::default(); 2]).as_slice());
    assert_eq!(
        vec.as_slices(),
        (
            [Default::default(); 2].as_slice(),
            [Default::default(); 2].as_slice(),
            [Default::default(); 2].as_slice(),
        ),
    );
    assert_eq!(format!("{vec:?}"), "SoaVec(([0, 0], [0, 0], [0, 0]))");

    vec.copy_from_slice(Vec::from_iter([(1, 2, 3); 2]).as_slice());
    assert_eq!(
        vec.as_slices(),
        ([1; 2].as_slice(), [2; 2].as_slice(), [3; 2].as_slice()),
    );

    let (t, u, v) = vec.remove(0);
    assert_eq!((t, u, v), (1, 2, 3));
    assert_eq!(vec.len(), 1);
    assert!(vec.capacity() >= 1);
    assert_eq!(vec.get(0), Some((&1, &2, &3)));

    let (t, u, v) = vec.pop().expect("vector should not be empty");
    assert_eq!((t, u, v), (1, 2, 3));
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 1);
    assert_eq!(vec.get(0), None);

    let clone = vec.clone();
    vec.copy_from_slice(clone.as_slice());
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 1);
    assert_eq!(vec.get(0), None);

    vec.extend_from_slice(vec.clone().as_slice());
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 1);
    assert_eq!(vec.get(0), None);

    let boxed_slice = vec.into_boxed_slice();
    assert!(boxed_slice.is_empty());

    let vec = boxed_slice.into_vec();
    assert!(vec.is_empty());

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let boxed_slice = vec.into_boxed_slice();
    assert!(boxed_slice.is_empty());

    let into_iter = boxed_slice.into_iter();
    assert!(into_iter.is_empty());
}

#[test]
fn one_item_zst() {
    type Vec = SoaVec<(ZST1, ZST2, ZST3)>;

    let mut vec = Vec::new();
    vec.push((ZST1, ZST2(()), ZST3 { empty: () }));
    assert_eq!(vec.len(), 1);
    assert!(vec.capacity() >= 1);
    assert!(vec.contains_by_refs((&ZST1, &ZST2(()), &ZST3 { empty: () })));

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    assert_eq!(
        format!("{vec:?}"),
        "SoaVec(([ZST1], [ZST2(())], [ZST3 { empty: () }]))",
    );

    let slice = vec.as_slice();
    assert_eq!(slice.len(), 1);
    assert!(slice.capacity() >= 1);
    assert_eq!(
        slice.as_slices(),
        (
            [ZST1].as_slice(),
            [ZST2(())].as_slice(),
            [ZST3 { empty: () }].as_slice(),
        ),
    );
    assert_eq!(slice.get(0), Some((&ZST1, &ZST2(()), &ZST3 { empty: () })));

    assert_eq!(
        format!("{slice:?}"),
        "SoaSlice(([ZST1], [ZST2(())], [ZST3 { empty: () }]))",
    );

    assert_eq!(vec, slice);
    assert!(vec >= slice);
    assert!(slice <= vec);

    assert_eq!(slice.to_owned(), vec.clone());

    let (context, vecs) = vec.into_vecs();
    assert_eq!(vecs, (vec![ZST1], vec![ZST2(())], vec![ZST3 { empty: () }]));

    let mut vec = Vec::from_vecs(context, vecs);
    assert_eq!(vec.len(), 1);
    assert!(vec.capacity() >= 1);
    assert_eq!(
        vec.as_slices(),
        (
            [ZST1].as_slice(),
            [ZST2(())].as_slice(),
            [ZST3 { empty: () }].as_slice(),
        ),
    );

    let mut iter = vec.iter();
    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some((&ZST1, &ZST2(()), &ZST3 { empty: () })));
    assert_eq!(iter.next(), None);

    vec.extend_from_within(..0);
    assert_eq!(
        vec.as_slices(),
        (
            [ZST1].as_slice(),
            [ZST2(())].as_slice(),
            [ZST3 { empty: () }].as_slice(),
        ),
    );

    vec.extend_from_within(..);
    assert_eq!(
        vec.as_slices(),
        (
            [ZST1; 2].as_slice(),
            [ZST2(()); 2].as_slice(),
            [ZST3 { empty: () }; 2].as_slice(),
        ),
    );

    assert_eq!(
        format!("{vec:?}"),
        "SoaVec(([ZST1, ZST1], [ZST2(()), ZST2(())], [ZST3 { empty: () }, ZST3 { empty: () }]))",
    );

    let (t, u, v) = vec.remove(0);
    assert_eq!((t, u, v), (ZST1, ZST2(()), ZST3 { empty: () }));
    assert_eq!(vec.len(), 1);
    assert!(vec.capacity() >= 1);
    assert_eq!(vec.get(0), Some((&ZST1, &ZST2(()), &ZST3 { empty: () })));

    let (t, u, v) = vec.pop().expect("vector should not be empty");
    assert_eq!((t, u, v), (ZST1, ZST2(()), ZST3 { empty: () }));
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 1);
    assert_eq!(vec.get(0), None);

    vec.extend_from_slice(vec.clone().as_slice());
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 1);
    assert_eq!(vec.get(0), None);

    let (context, vecs) = vec.into_vecs();
    assert_eq!(vecs, (vec![], vec![], vec![]));

    let vec = Vec::from_vecs(context, vecs);
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 1);
    assert_eq!(
        vec.as_slices(),
        ([].as_slice(), [].as_slice(), [].as_slice()),
    );

    let boxed_slice = vec.into_boxed_slice();
    assert!(boxed_slice.is_empty());
    assert_eq!(boxed_slice.capacity(), usize::MAX);

    let vec = boxed_slice.into_vec();
    assert!(vec.is_empty());
    assert_eq!(vec.capacity(), usize::MAX);

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let boxed_slice = vec.into_boxed_slice();
    assert!(boxed_slice.is_empty());
    assert_eq!(boxed_slice.capacity(), usize::MAX);

    let into_iter = boxed_slice.into_iter();
    assert!(into_iter.is_empty());
}

#[test]
fn one_item_dyn() {
    type Vec = SoaVec<DynSoa<(u8, u64, u16)>>;

    let field_layouts = [
        Layout::new::<u8>(),
        Layout::new::<u64>(),
        Layout::new::<u16>(),
    ];

    let u8 = 1u8;
    let u64 = 2u64;
    let u16 = 3u16;

    let u8_slice = unsafe {
        let data = ptr::from_ref(&u8).cast();
        let len = size_of_val(&u8);
        slice::from_raw_parts(data, len)
    };
    let u64_slice = unsafe {
        let data = ptr::from_ref(&u64).cast();
        let len = size_of_val(&u64);
        slice::from_raw_parts(data, len)
    };
    let u16_slice = unsafe {
        let data = ptr::from_ref(&u16).cast();
        let len = size_of_val(&u16);
        slice::from_raw_parts(data, len)
    };

    let fields = [
        (u8_slice, field_layouts[0]),
        (u64_slice, field_layouts[1]),
        (u16_slice, field_layouts[2]),
    ];
    let value = DynSoa::new(fields);
    assert_eq!(
        format!("{value:?}"),
        format!("DynSoa([{u8_slice:?}, {u64_slice:?}, {u16_slice:?}])"),
    );

    let context = DynSoaContext::new(field_layouts);
    let mut vec = Vec::with_context(context);

    vec.push(value);
    assert_eq!(vec.len(), 1);
    assert!(vec.capacity() >= 1);

    let refs = DynSoaRefs::new([u8_slice, u64_slice, u16_slice]);
    assert_eq!(vec.get(0), Some(refs.clone()));
    assert!(vec.contains_by_refs(refs.clone()));

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    assert_eq!(
        format!("{vec:?}"),
        format!("SoaVec(DynSoaSlices([{u8_slice:?}, {u64_slice:?}, {u16_slice:?}]))"),
    );

    let slice = vec.as_slice();
    assert_eq!(slice.len(), 1);
    assert!(slice.capacity() >= 1);
    assert_eq!(
        slice.as_slices(),
        DynSoaSlices::new([u8_slice, u64_slice, u16_slice]),
    );
    assert_eq!(slice.get(0), Some(refs.clone()));

    assert_eq!(
        format!("{slice:?}"),
        format!("SoaSlice(DynSoaSlices([{u8_slice:?}, {u64_slice:?}, {u16_slice:?}]))"),
    );

    assert_eq!(vec, slice);
    assert!(vec >= slice);
    assert!(slice <= vec);

    assert!(slice.to_owned().is_empty().not());

    let (context, vecs) = vec.into_vecs();
    // assert_eq!(vecs, (vec![ZST1], vec![ZST2(())], vec![ZST3 { empty: () }]));

    let mut vec = Vec::from_vecs(context, vecs);
    assert_eq!(vec.len(), 1);
    assert!(vec.capacity() >= 1);
    assert_eq!(
        vec.as_slices(),
        DynSoaSlices::new([u8_slice, u64_slice, u16_slice]),
    );

    let mut iter = vec.iter();
    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next_back(), Some(refs.clone()));
    assert_eq!(iter.next(), None);

    let value = vec.pop().expect("vector should not be empty");
    assert_eq!(value, DynSoa::new(fields));
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 1);
    assert_eq!(vec.get(0), None);

    assert_eq!(
        format!("{vec:?}"),
        format!("SoaVec(DynSoaSlices([[], [], []]))"),
    );

    let boxed_slice = vec.into_boxed_slice();
    assert!(boxed_slice.is_empty());

    let vec = boxed_slice.into_vec();
    assert!(vec.is_empty());

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let boxed_slice = vec.into_boxed_slice();
    assert!(boxed_slice.is_empty());

    let into_iter = boxed_slice.into_iter();
    assert!(into_iter.is_empty());
}

#[test]
fn three_items() {
    type Vec = SoaVec<(u16, String, u128)>;

    let mut vec = Vec::from_iter(iter::repeat((0, "0".to_owned(), 0)).take(3));
    assert_eq!(vec.len(), 3);
    assert!(vec.capacity() >= 3);
    assert!(vec.contains_by_refs((&0, &"0".to_owned(), &0)));

    assert_eq!(
        format!("{vec:?}"),
        r#"SoaVec(([0, 0, 0], ["0", "0", "0"], [0, 0, 0]))"#,
    );

    vec.truncate(0);
    vec.insert(0, (1, "2".to_owned(), 3));
    vec.insert(0, (4, "5".to_owned(), 6));
    vec.insert(1, (7, "8".to_owned(), 9));

    assert_eq!(vec.len(), 3);
    assert!(vec.capacity() >= 3);
    assert!(vec.contains_by_refs((&4, &"5".to_owned(), &6)));

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    assert_eq!(
        format!("{vec:?}"),
        r#"SoaVec(([4, 7, 1], ["5", "8", "2"], [6, 9, 3]))"#,
    );

    let slice = vec.as_slice();
    assert_eq!(slice.len(), 3);
    assert!(slice.capacity() >= 3);
    assert_eq!(
        slice.as_slices(),
        (
            [4, 7, 1].as_slice(),
            ["5".to_owned(), "8".to_owned(), "2".to_owned()].as_slice(),
            [6, 9, 3].as_slice(),
        ),
    );
    assert_eq!(slice.get(0), Some((&4, &"5".to_owned(), &6)));
    assert_eq!(slice.get(1), Some((&7, &"8".to_owned(), &9)));
    assert_eq!(slice.get(2), Some((&1, &"2".to_owned(), &3)));
    assert_eq!(
        slice.get(0..),
        Some((
            [4, 7, 1].as_slice(),
            ["5".to_owned(), "8".to_owned(), "2".to_owned()].as_slice(),
            [6, 9, 3].as_slice(),
        )),
    );

    assert_eq!(
        format!("{slice:?}"),
        r#"SoaSlice(([4, 7, 1], ["5", "8", "2"], [6, 9, 3]))"#,
    );

    assert_eq!(vec, slice);
    assert!(vec >= slice);
    assert!(slice <= vec);

    assert_eq!(slice.to_owned(), vec.clone());

    let mut clone = vec.clone();
    for (t, _, _) in &mut clone {
        *t += 1;
    }
    vec.clone_from_slice(clone.as_slice());
    assert_eq!(vec.len(), 3);
    assert!(vec.capacity() >= 3);
    assert_eq!(
        vec.as_slices(),
        (
            [5, 8, 2].as_slice(),
            ["5".to_owned(), "8".to_owned(), "2".to_owned()].as_slice(),
            [6, 9, 3].as_slice(),
        ),
    );

    let (context, vecs) = vec.into_vecs();
    assert_eq!(
        vecs,
        (
            vec![5, 8, 2],
            vec!["5".to_owned(), "8".to_owned(), "2".to_owned()],
            vec![6, 9, 3],
        ),
    );

    let mut vec = Vec::from_vecs(context, vecs);
    assert_eq!(vec.len(), 3);
    assert!(vec.capacity() >= 3);
    assert_eq!(
        vec.as_slices(),
        (
            [5, 8, 2].as_slice(),
            ["5".to_owned(), "8".to_owned(), "2".to_owned()].as_slice(),
            [6, 9, 3].as_slice(),
        ),
    );

    let mut iter = vec.iter_mut();
    assert_eq!(iter.len(), 3);

    assert_eq!(iter.next(), Some((&mut 5, &mut "5".to_owned(), &mut 6)));
    assert_eq!(iter.len(), 2);

    assert_eq!(
        iter.next_back(),
        Some((&mut 2, &mut "2".to_owned(), &mut 3)),
    );
    assert_eq!(iter.len(), 1);

    assert_eq!(iter.next(), Some((&mut 8, &mut "8".to_owned(), &mut 9)));
    assert_eq!(iter.len(), 0);

    assert_eq!(iter.next_back(), None);

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    vec.extend_from_within(1..);
    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);
    assert_eq!(
        vec.as_slices(),
        (
            [5, 8, 2, 8, 2].as_slice(),
            [
                "5".to_owned(),
                "8".to_owned(),
                "2".to_owned(),
                "8".to_owned(),
                "2".to_owned(),
            ]
            .as_slice(),
            [6, 9, 3, 9, 3].as_slice(),
        ),
    );
    assert_eq!(vec.get(0), Some((&5, &"5".to_owned(), &6)));
    assert_eq!(vec.get(1), Some((&8, &"8".to_owned(), &9)));
    assert_eq!(vec.get(2), Some((&2, &"2".to_owned(), &3)));
    assert_eq!(vec.get(3), Some((&8, &"8".to_owned(), &9)));
    assert_eq!(vec.get(4), Some((&2, &"2".to_owned(), &3)));

    {
        let mut drain = vec.drain(2..4);
        assert_eq!(drain.len(), 2);
        assert_eq!(
            drain.as_slices(),
            (
                [2, 8].as_slice(),
                ["2".to_owned(), "8".to_owned()].as_slice(),
                [3, 9].as_slice(),
            )
        );

        assert_eq!(drain.next_back(), Some((8, "8".to_owned(), 9)));
        assert_eq!(drain.len(), 1);

        assert_eq!(drain.next(), Some((2, "2".to_owned(), 3)));
        assert_eq!(drain.len(), 0);

        assert_eq!(drain.next(), None);
        assert_eq!(drain.next_back(), None);
    }

    assert_eq!(vec.len(), 3);
    assert!(vec.capacity() >= 5);

    let (t, u, v) = vec.swap_remove(1);
    assert_eq!((t, u, v), (8, "8".to_owned(), 9));
    assert_eq!(vec.len(), 2);
    assert!(vec.capacity() >= 3);
    assert!(!vec.contains_by_refs((&8, &"8".to_owned(), &9)));

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let (t, u, v) = vec.pop().expect("vector should not be empty");
    assert_eq!((t, u, v), (2, "2".to_owned(), 3));
    assert_eq!(vec.len(), 1);
    assert!(vec.capacity() >= 3);
    assert!(!vec.contains_by_refs((&2, &"2".to_owned(), &3)));

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let (t, u, v) = vec.remove(0);
    assert_eq!((t, u, v), (5, "5".to_owned(), 6));
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 3);
    assert!(!vec.contains_by_refs((&5, &"5".to_owned(), &6)));

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let clone = vec.clone();
    vec.clone_from_slice(clone.as_slice());
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 1);
    assert_eq!(vec.get(0), None);

    vec.extend_from_slice(vec.clone().as_slice());
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 1);
    assert_eq!(vec.get(0), None);

    vec.extend_from_within(..);
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 1);
    assert_eq!(vec.get(0), None);

    vec.extend(iter::repeat((0, "0".to_owned(), 0)).take(3));
    vec.reserve(1);
    assert!(vec.capacity() >= 4);
    vec.reserve_exact(6);
    assert!(vec.capacity() >= 9);

    let (context, vecs) = vec.into_vecs();
    assert_eq!(
        vecs,
        (
            vec![0, 0, 0],
            vec!["0".to_owned(), "0".to_owned(), "0".to_owned()],
            vec![0, 0, 0],
        ),
    );

    let vec = Vec::from_vecs(context, vecs);
    assert_eq!(vec.len(), 3);
    assert!(vec.capacity() >= 3);
    assert_eq!(
        vec.as_slices(),
        (
            [0, 0, 0].as_slice(),
            ["0".to_owned(), "0".to_owned(), "0".to_owned()].as_slice(),
            [0, 0, 0].as_slice(),
        ),
    );

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    vec.reserve(1);
    assert!(vec.capacity() >= 4);
    vec.reserve_exact(6);
    assert!(vec.capacity() >= 9);

    vec.shrink_to(6);
    assert!(vec.capacity() >= 6);
    vec.shrink_to(0);
    assert!(vec.capacity() >= 3);

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    vec.truncate(1);
    assert_eq!(vec.len(), 1);
    assert!(vec.capacity() >= 3);

    vec.clear();
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 3);

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    vec.push((1, "2".to_owned(), 3));
    for _ in 0..10 {
        vec.push((4, "5".to_owned(), 6));
        vec.push((7, "8".to_owned(), 9));
    }
    vec.retain_mut(|(x, _, _)| {
        if *x <= 3 {
            *x += 1;
            true
        } else {
            false
        }
    });
    assert_eq!(vec.len(), 1);
    assert!(vec.capacity() >= (1 + 2 * 10));
    assert_eq!(
        vec.as_slices(),
        ([2].as_slice(), ["2".to_owned()].as_slice(), [3].as_slice()),
    );

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let boxed_slice = vec.into_boxed_slice();
    assert_eq!(boxed_slice.len(), 1);
    assert!(boxed_slice.capacity() >= 1);
    assert_eq!(boxed_slice.get(0), Some((&2, &"2".to_owned(), &3)));

    let vec = boxed_slice.into_vec();
    assert_eq!(vec.len(), 1);
    assert!(vec.capacity() >= 1);
    assert_eq!(vec.get(0), Some((&2, &"2".to_owned(), &3)));

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let boxed_slice = vec.into_boxed_slice();
    assert_eq!(boxed_slice.len(), 1);
    assert!(boxed_slice.capacity() >= 1);
    assert_eq!(boxed_slice.get(0), Some((&2, &"2".to_owned(), &3)));

    let mut into_iter = boxed_slice.into_iter();
    assert_eq!(into_iter.len(), 1);
    assert_eq!(into_iter.next_back(), Some((2, "2".to_owned(), 3)));
    assert_eq!(into_iter.next(), None);
    assert_eq!(into_iter.next_back(), None);
}

#[test]
fn three_items_zst() {
    type Vec = SoaVec<(ZST1, ZST2, ZST3)>;

    let vec = Vec::from_iter([(ZST1, ZST2(()), ZST3 { empty: () }); 3]);
    assert_eq!(vec.len(), 3);
    assert!(vec.capacity() >= 3);
    assert!(vec.contains_by_refs((&ZST1, &ZST2(()), &ZST3 { empty: () })));

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let slice = vec.as_slice();
    assert_eq!(slice.len(), 3);
    assert!(slice.capacity() >= 3);
    assert_eq!(
        slice.as_slices(),
        (
            [ZST1; 3].as_slice(),
            [ZST2(()); 3].as_slice(),
            [ZST3 { empty: () }; 3].as_slice(),
        ),
    );
    assert_eq!(slice.get(0), Some((&ZST1, &ZST2(()), &ZST3 { empty: () })));
    assert_eq!(slice.get(1), Some((&ZST1, &ZST2(()), &ZST3 { empty: () })));
    assert_eq!(slice.get(2), Some((&ZST1, &ZST2(()), &ZST3 { empty: () })));
    assert_eq!(
        slice.get(0..),
        Some((
            [ZST1; 3].as_slice(),
            [ZST2(()); 3].as_slice(),
            [ZST3 { empty: () }; 3].as_slice(),
        )),
    );

    assert_eq!(vec, slice);
    assert!(vec >= slice);
    assert!(slice <= vec);

    assert_eq!(slice.to_owned(), vec.clone());

    let (context, vecs) = vec.into_vecs();
    assert_eq!(
        vecs,
        (
            vec![ZST1; 3],
            vec![ZST2(()); 3],
            vec![ZST3 { empty: () }; 3],
        ),
    );

    let mut vec = Vec::from_vecs(context, vecs);
    assert_eq!(vec.len(), 3);
    assert!(vec.capacity() >= 3);
    assert_eq!(
        vec.as_slices(),
        (
            [ZST1; 3].as_slice(),
            [ZST2(()); 3].as_slice(),
            [ZST3 { empty: () }; 3].as_slice(),
        ),
    );

    let mut iter = vec.iter_mut();
    assert_eq!(iter.len(), 3);

    assert!(iter.next().is_some());
    assert_eq!(iter.len(), 2);

    assert!(iter.next_back().is_some());
    assert_eq!(iter.len(), 1);

    assert!(iter.next().is_some());
    assert_eq!(iter.len(), 0);

    assert!(iter.next_back().is_none());

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    vec.extend_from_within(1..);
    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);
    assert_eq!(
        vec.as_slices(),
        (
            [ZST1; 5].as_slice(),
            [ZST2(()); 5].as_slice(),
            [ZST3 { empty: () }; 5].as_slice(),
        ),
    );
    assert_eq!(vec.get(0), Some((&ZST1, &ZST2(()), &ZST3 { empty: () })));
    assert_eq!(vec.get(1), Some((&ZST1, &ZST2(()), &ZST3 { empty: () })));
    assert_eq!(vec.get(2), Some((&ZST1, &ZST2(()), &ZST3 { empty: () })));
    assert_eq!(vec.get(3), Some((&ZST1, &ZST2(()), &ZST3 { empty: () })));
    assert_eq!(vec.get(4), Some((&ZST1, &ZST2(()), &ZST3 { empty: () })));

    let (context, vecs) = vec.into_vecs();
    assert_eq!(
        vecs,
        (
            vec![ZST1; 5],
            vec![ZST2(()); 5],
            vec![ZST3 { empty: () }; 5],
        ),
    );

    let mut vec = Vec::from_vecs(context, vecs);
    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);
    assert_eq!(
        vec.as_slices(),
        (
            [ZST1; 5].as_slice(),
            [ZST2(()); 5].as_slice(),
            [ZST3 { empty: () }; 5].as_slice(),
        ),
    );

    {
        let mut drain = vec.drain(2..4);
        assert_eq!(drain.len(), 2);
        assert_eq!(
            drain.as_slices(),
            (
                [ZST1; 2].as_slice(),
                [ZST2(()); 2].as_slice(),
                [ZST3 { empty: () }; 2].as_slice(),
            ),
        );

        assert!(drain.next_back().is_some());
        assert_eq!(drain.len(), 1);

        assert!(drain.next().is_some());
        assert_eq!(drain.len(), 0);

        assert!(drain.next().is_none());
        assert!(drain.next_back().is_none());
    }

    assert_eq!(vec.len(), 3);
    assert!(vec.capacity() >= 5);

    let (t, u, v) = vec.swap_remove(1);
    assert_eq!((t, u, v), (ZST1, ZST2(()), ZST3 { empty: () }));
    assert_eq!(vec.len(), 2);
    assert!(vec.capacity() >= 3);

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let (t, u, v) = vec.pop().expect("vector should not be empty");
    assert_eq!((t, u, v), (ZST1, ZST2(()), ZST3 { empty: () }));
    assert_eq!(vec.len(), 1);
    assert!(vec.capacity() >= 3);

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let (t, u, v) = vec.remove(0);
    assert_eq!((t, u, v), (ZST1, ZST2(()), ZST3 { empty: () }));
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 3);

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let clone = vec.clone();
    vec.copy_from_slice(clone.as_slice());
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 1);
    assert_eq!(vec.get(0), None);

    vec.extend_from_slice(vec.clone().as_slice());
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 1);
    assert_eq!(vec.get(0), None);

    vec.extend_from_within(..);
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 1);
    assert_eq!(vec.get(0), None);

    vec.extend(iter::repeat((ZST1, ZST2(()), ZST3 { empty: () })).take(3));
    vec.reserve(1);
    assert!(vec.capacity() >= 4);
    vec.reserve_exact(6);
    assert!(vec.capacity() >= 9);

    let (context, vecs) = vec.into_vecs();
    assert_eq!(
        vecs,
        (
            vec![ZST1; 3],
            vec![ZST2(()); 3],
            vec![ZST3 { empty: () }; 3],
        ),
    );

    let vec = Vec::from_vecs(context, vecs);
    assert_eq!(vec.len(), 3);
    assert!(vec.capacity() >= 3);
    assert_eq!(
        vec.as_slices(),
        (
            [ZST1; 3].as_slice(),
            [ZST2(()); 3].as_slice(),
            [ZST3 { empty: () }; 3].as_slice(),
        ),
    );

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    vec.shrink_to(6);
    assert!(vec.capacity() >= 6);
    vec.shrink_to(0);
    assert!(vec.capacity() >= 3);

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    vec.truncate(1);
    assert_eq!(vec.len(), 1);
    assert!(vec.capacity() >= 3);

    vec.clear();
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 3);
    assert!(!vec.contains_by_refs((&ZST1, &ZST2(()), &ZST3 { empty: () })));

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    vec.push((ZST1, ZST2(()), ZST3 { empty: () }));
    vec.push((ZST1, ZST2(()), ZST3 { empty: () }));
    vec.push((ZST1, ZST2(()), ZST3 { empty: () }));

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let mut idx = 0;
    vec.retain(|_| {
        let current = idx;
        idx += 1;
        current % 2 == 0
    });
    assert_eq!(vec.len(), 2);
    assert!(vec.capacity() >= 3);

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let boxed_slice = vec.into_boxed_slice();
    assert_eq!(boxed_slice.len(), 2);
    assert_eq!(boxed_slice.capacity(), usize::MAX);
    assert_eq!(
        boxed_slice.get(..),
        Some((
            [ZST1; 2].as_slice(),
            [ZST2(()); 2].as_slice(),
            [ZST3 { empty: () }; 2].as_slice(),
        )),
    );

    let vec = boxed_slice.into_vec();
    assert_eq!(vec.len(), 2);
    assert!(vec.capacity() >= 2);

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let boxed_slice = vec.into_boxed_slice();
    assert_eq!(boxed_slice.len(), 2);
    assert_eq!(boxed_slice.capacity(), usize::MAX);

    let mut into_iter = boxed_slice.into_iter();
    assert_eq!(into_iter.len(), 2);
    assert_eq!(
        into_iter.next_back(),
        Some((ZST1, ZST2(()), ZST3 { empty: () })),
    );
    assert_eq!(into_iter.next(), Some((ZST1, ZST2(()), ZST3 { empty: () })));
    assert_eq!(into_iter.next_back(), None);
    assert_eq!(into_iter.next(), None);
}

#[test]
fn three_items_dyn() {
    type Vec = SoaVec<DynSoa<(u8, u64, u16)>>;

    let field_layouts = [
        Layout::new::<u8>(),
        Layout::new::<u64>(),
        Layout::new::<u16>(),
    ];

    let context = DynSoaContext::new(field_layouts);
    let mut vec = Vec::with_context(context);

    let i0_u8 = 0u8;
    let i0_u64 = 0u64;
    let i0_u16 = 0u16;

    let i0_u8_bytes = unsafe {
        let data = ptr::from_ref(&i0_u8).cast();
        let len = size_of_val(&i0_u8);
        slice::from_raw_parts(data, len)
    };
    let i0_u64_bytes = unsafe {
        let data = ptr::from_ref(&i0_u64).cast();
        let len = size_of_val(&i0_u64);
        slice::from_raw_parts(data, len)
    };
    let i0_u16_bytes = unsafe {
        let data = ptr::from_ref(&i0_u16).cast();
        let len = size_of_val(&i0_u16);
        slice::from_raw_parts(data, len)
    };

    let fields = [
        (i0_u8_bytes, field_layouts[0]),
        (i0_u64_bytes, field_layouts[1]),
        (i0_u16_bytes, field_layouts[2]),
    ];
    vec.extend(iter::repeat_with(|| DynSoa::new(fields)).take(3));
    assert_eq!(vec.len(), 3);
    assert!(vec.capacity() >= 3);
    assert!(vec.contains_by_refs(DynSoaRefs::new([i0_u8_bytes, i0_u64_bytes, i0_u16_bytes])));

    let i0_u8s = [i0_u8; 3];
    let i0_u64s = [i0_u64; 3];
    let i0_u16s = [i0_u16; 3];

    let i0_u8s_bytes = unsafe {
        let data = ptr::from_ref(&i0_u8s).cast();
        let len = size_of_val(&i0_u8s);
        slice::from_raw_parts(data, len)
    };
    let i0_u64s_bytes = unsafe {
        let data = ptr::from_ref(&i0_u64s).cast();
        let len = size_of_val(&i0_u64s);
        slice::from_raw_parts(data, len)
    };
    let i0_u16s_bytes = unsafe {
        let data = ptr::from_ref(&i0_u16s).cast();
        let len = size_of_val(&i0_u16s);
        slice::from_raw_parts(data, len)
    };

    assert_eq!(
        vec.as_slices(),
        DynSoaSlices::new([i0_u8s_bytes, i0_u64s_bytes, i0_u16s_bytes]),
    );
    assert_eq!(
        format!("{vec:?}"),
        format!("SoaVec(DynSoaSlices([{i0_u8s_bytes:?}, {i0_u64s_bytes:?}, {i0_u16s_bytes:?}]))"),
    );

    vec.truncate(0);

    let i1 = 1u8;
    let i2 = 2u64;
    let i3 = 3u16;

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

    let fields = [
        (i1_bytes, field_layouts[0]),
        (i2_bytes, field_layouts[1]),
        (i3_bytes, field_layouts[2]),
    ];
    vec.insert(0, DynSoa::new(fields));

    let i4 = 4u8;
    let i5 = 5u64;
    let i6 = 6u16;

    let i4_bytes = unsafe {
        let data = ptr::from_ref(&i4).cast();
        let len = size_of_val(&i4);
        slice::from_raw_parts(data, len)
    };
    let i5_bytes = unsafe {
        let data = ptr::from_ref(&i5).cast();
        let len = size_of_val(&i5);
        slice::from_raw_parts(data, len)
    };
    let i6_bytes = unsafe {
        let data = ptr::from_ref(&i6).cast();
        let len = size_of_val(&i6);
        slice::from_raw_parts(data, len)
    };

    let fields = [
        (i4_bytes, field_layouts[0]),
        (i5_bytes, field_layouts[1]),
        (i6_bytes, field_layouts[2]),
    ];
    vec.insert(0, DynSoa::new(fields));

    let i7 = 7u8;
    let i8 = 8u64;
    let i9 = 9u16;

    let i7_bytes = unsafe {
        let data = ptr::from_ref(&i7).cast();
        let len = size_of_val(&i7);
        slice::from_raw_parts(data, len)
    };
    let i8_bytes = unsafe {
        let data = ptr::from_ref(&i8).cast();
        let len = size_of_val(&i8);
        slice::from_raw_parts(data, len)
    };
    let i9_bytes = unsafe {
        let data = ptr::from_ref(&i9).cast();
        let len = size_of_val(&i9);
        slice::from_raw_parts(data, len)
    };

    let fields = [
        (i7_bytes, field_layouts[0]),
        (i8_bytes, field_layouts[1]),
        (i9_bytes, field_layouts[2]),
    ];
    vec.insert(1, DynSoa::new(fields));

    assert_eq!(vec.len(), 3);
    assert!(vec.capacity() >= 3);

    let i471 = [i4, i7, i1];
    let i582 = [i5, i8, i2];
    let i693 = [i6, i9, i3];

    let i471_bytes = unsafe {
        let data = ptr::from_ref(&i471).cast();
        let len = size_of_val(&i471);
        slice::from_raw_parts(data, len)
    };
    let i582_bytes = unsafe {
        let data = ptr::from_ref(&i582).cast();
        let len = size_of_val(&i582);
        slice::from_raw_parts(data, len)
    };
    let i693_bytes = unsafe {
        let data = ptr::from_ref(&i693).cast();
        let len = size_of_val(&i693);
        slice::from_raw_parts(data, len)
    };

    assert_eq!(
        vec.as_slices(),
        DynSoaSlices::new([i471_bytes, i582_bytes, i693_bytes]),
    );
    assert_eq!(
        format!("{vec:?}"),
        format!("SoaVec(DynSoaSlices([{i471_bytes:?}, {i582_bytes:?}, {i693_bytes:?}]))"),
    );

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let slice = vec.as_slice();
    assert_eq!(slice.len(), 3);
    assert!(slice.capacity() >= 3);

    let slices = DynSoaSlices::new([i471_bytes, i582_bytes, i693_bytes]);
    assert_eq!(slice.as_slices(), slices.clone());

    assert_eq!(
        slice.get(0),
        Some(DynSoaRefs::new([i4_bytes, i5_bytes, i6_bytes])),
    );
    assert_eq!(
        slice.get(1),
        Some(DynSoaRefs::new([i7_bytes, i8_bytes, i9_bytes])),
    );
    assert_eq!(
        slice.get(2),
        Some(DynSoaRefs::new([i1_bytes, i2_bytes, i3_bytes])),
    );
    assert_eq!(slice.get(0..), Some(slices.clone()));

    assert_eq!(
        format!("{slice:?}"),
        format!("SoaSlice(DynSoaSlices([{i471_bytes:?}, {i582_bytes:?}, {i693_bytes:?}]))"),
    );

    assert_eq!(vec, slice);
    assert!(vec >= slice);
    assert!(slice <= vec);

    assert!(slice.to_owned().is_empty().not());

    let (context, vecs) = vec.into_vecs();
    // assert_eq!(vecs, ..);

    let mut vec = Vec::from_vecs(context, vecs);
    assert_eq!(vec.len(), 3);
    assert!(vec.capacity() >= 3);
    assert_eq!(vec.as_slices(), slices.clone());

    for mut refs in &mut vec {
        let [_, u64_bytes, u16_bytes] = refs.as_mut() else {
            panic!("expected 3 fields");
        };
        assert_eq!(u64_bytes.len(), 8);

        let u64 = unsafe { &mut *u64_bytes.as_mut_ptr().cast::<u64>() };
        let u16 = unsafe { &*u16_bytes.as_mut_ptr().cast::<u16>() };
        *u64 += u64::from(*u16);
    }

    let mut iter = vec.iter();
    assert_eq!(iter.len(), 3);

    let i5_plus_6 = 11_u64;
    let i5_plus_6_bytes = unsafe {
        let data = ptr::from_ref(&i5_plus_6).cast();
        let len = size_of_val(&i5_plus_6);
        slice::from_raw_parts(data, len)
    };
    assert_eq!(
        iter.next(),
        Some(DynSoaRefs::new([i4_bytes, i5_plus_6_bytes, i6_bytes])),
    );
    assert_eq!(iter.len(), 2);

    let i2_plus_3 = 5_u64;
    let i2_plus_3_bytes = unsafe {
        let data = ptr::from_ref(&i2_plus_3).cast();
        let len = size_of_val(&i2_plus_3);
        slice::from_raw_parts(data, len)
    };
    assert_eq!(
        iter.next_back(),
        Some(DynSoaRefs::new([i1_bytes, i2_plus_3_bytes, i3_bytes])),
    );
    assert_eq!(iter.len(), 1);

    let i8_plus_9 = 17_u64;
    let i8_plus_9_bytes = unsafe {
        let data = ptr::from_ref(&i8_plus_9).cast();
        let len = size_of_val(&i8_plus_9);
        slice::from_raw_parts(data, len)
    };
    assert_eq!(
        iter.next(),
        Some(DynSoaRefs::new([i7_bytes, i8_plus_9_bytes, i9_bytes])),
    );
    assert_eq!(iter.len(), 0);

    assert_eq!(iter.next_back(), None);

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    vec.push(DynSoa::new([
        (i7_bytes, field_layouts[0]),
        (i8_plus_9_bytes, field_layouts[1]),
        (i9_bytes, field_layouts[2]),
    ]));
    vec.push(DynSoa::new([
        (i1_bytes, field_layouts[0]),
        (i2_plus_3_bytes, field_layouts[1]),
        (i3_bytes, field_layouts[2]),
    ]));
    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);

    assert_eq!(
        vec.get(0),
        Some(DynSoaRefs::new([i4_bytes, i5_plus_6_bytes, i6_bytes])),
    );
    assert_eq!(
        vec.get(1),
        Some(DynSoaRefs::new([i7_bytes, i8_plus_9_bytes, i9_bytes])),
    );
    assert_eq!(
        vec.get(2),
        Some(DynSoaRefs::new([i1_bytes, i2_plus_3_bytes, i3_bytes])),
    );
    assert_eq!(
        vec.get(3),
        Some(DynSoaRefs::new([i7_bytes, i8_plus_9_bytes, i9_bytes])),
    );
    assert_eq!(
        vec.get(4),
        Some(DynSoaRefs::new([i1_bytes, i2_plus_3_bytes, i3_bytes])),
    );

    {
        let mut drain = vec.drain(2..4);
        assert_eq!(drain.len(), 2);

        assert_eq!(
            drain.next_back(),
            Some(DynSoa::new([
                (i7_bytes, field_layouts[0]),
                (i8_plus_9_bytes, field_layouts[1]),
                (i9_bytes, field_layouts[2]),
            ])),
        );
        assert_eq!(drain.len(), 1);

        assert_eq!(
            drain.next(),
            Some(DynSoa::new([
                (i1_bytes, field_layouts[0]),
                (i2_plus_3_bytes, field_layouts[1]),
                (i3_bytes, field_layouts[2]),
            ])),
        );
        assert_eq!(drain.len(), 0);

        assert_eq!(drain.next(), None);
        assert_eq!(drain.next_back(), None);
    }

    assert_eq!(vec.len(), 3);
    assert!(vec.capacity() >= 5);

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let value = vec.swap_remove(1);
    assert_eq!(
        value,
        DynSoa::new([
            (i7_bytes, field_layouts[0]),
            (i8_plus_9_bytes, field_layouts[1]),
            (i9_bytes, field_layouts[2]),
        ]),
    );
    assert_eq!(vec.len(), 2);
    assert!(vec.capacity() >= 3);
    assert!(!vec.contains_by_refs(DynSoaRefs::new([i7_bytes, i8_plus_9_bytes, i9_bytes])));

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let value = vec.pop().expect("vector should not be empty");
    assert_eq!(
        value,
        DynSoa::new([
            (i1_bytes, field_layouts[0]),
            (i2_plus_3_bytes, field_layouts[1]),
            (i3_bytes, field_layouts[2]),
        ]),
    );
    assert_eq!(vec.len(), 1);
    assert!(vec.capacity() >= 3);
    assert!(!vec.contains_by_refs(DynSoaRefs::new([i1_bytes, i2_plus_3_bytes, i3_bytes])));

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let value = vec.remove(0);
    assert_eq!(
        value,
        DynSoa::new([
            (i4_bytes, field_layouts[0]),
            (i5_plus_6_bytes, field_layouts[1]),
            (i6_bytes, field_layouts[2]),
        ]),
    );
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 3);
    assert!(!vec.contains_by_refs(DynSoaRefs::new([i4_bytes, i5_plus_6_bytes, i6_bytes])));

    // TODO: more tests
}
