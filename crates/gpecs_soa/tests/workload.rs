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
    type Soa = (u8, u64, u16);
    type Vec = SoaVec<DynSoa<Soa>>;

    let context = ();
    let dyn_context = DynSoaContext::of::<Soa>(&context);

    let optimized_layout = [
        Layout::new::<u8>(),
        Layout::new::<u16>(),
        Layout::new::<u64>(),
    ];
    assert_eq!(dyn_context.layouts(), optimized_layout);

    let vec = Vec::with_context(dyn_context);
    assert!(vec.is_empty());

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    assert_eq!(
        format!("{vec:?}"),
        format!(
            "SoaVec(DynSoaSlices {{ len: 0, slices: [({l0:?}, []), ({l1:?}, []), ({l2:?}, [])] }})",
            l0 = optimized_layout[0],
            l1 = optimized_layout[1],
            l2 = optimized_layout[2],
        ),
    );

    let slice = vec.as_slice();
    assert!(slice.is_empty());

    assert_eq!(
        format!("{slice:?}"),
        format!(
            "SoaSlice(DynSoaSlices {{ len: 0, slices: [({l0:?}, []), ({l1:?}, []), ({l2:?}, [])] }})",
            l0 = optimized_layout[0],
            l1 = optimized_layout[1],
            l2 = optimized_layout[2],
        ),
    );

    assert_eq!(vec, slice);
    assert!(slice.to_owned().is_empty());

    let (dyn_context, vecs) = vec.into_vecs();
    // assert_eq!(vecs, (vec![], vec![], vec![]));

    let vec = Vec::from_vecs(dyn_context, vecs);
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
        format!(
            "IntoIter(DynSoaSlices {{ len: 0, slices: [({l0:?}, []), ({l1:?}, []), ({l2:?}, [])] }})",
            l0 = optimized_layout[0],
            l1 = optimized_layout[1],
            l2 = optimized_layout[2],
        ),
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
    type Soa = (u8, u64, u16);
    type Vec = SoaVec<DynSoa<Soa>>;

    let context = ();
    let dyn_context = DynSoaContext::of::<Soa>(&context);

    let optimized_layout = [
        Layout::new::<u8>(),
        Layout::new::<u16>(),
        Layout::new::<u64>(),
    ];
    assert_eq!(dyn_context.layouts(), optimized_layout);

    let vec = Vec::with_context_and_capacity(dyn_context, 10);
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 10);

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    assert_eq!(
        format!("{vec:?}"),
        format!(
            "SoaVec(DynSoaSlices {{ len: 0, slices: [({l0:?}, []), ({l1:?}, []), ({l2:?}, [])] }})",
            l0 = optimized_layout[0],
            l1 = optimized_layout[1],
            l2 = optimized_layout[2],
        ),
    );

    let slice = vec.as_slice();
    assert!(slice.is_empty());
    assert!(slice.capacity() >= 10);

    assert_eq!(
        format!("{slice:?}"),
        format!(
            "SoaSlice(DynSoaSlices {{ len: 0, slices: [({l0:?}, []), ({l1:?}, []), ({l2:?}, [])] }})",
            l0 = optimized_layout[0],
            l1 = optimized_layout[1],
            l2 = optimized_layout[2],
        ),
    );

    assert_eq!(vec, slice);
    assert!(slice.to_owned().is_empty());

    let (dyn_context, vecs) = vec.into_vecs();
    // assert_eq!(vecs, (vec![], vec![], vec![]));

    let vec = Vec::from_vecs(dyn_context, vecs);
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
        format!(
            "IntoIter(DynSoaSlices {{ len: 0, slices: [({l0:?}, []), ({l1:?}, []), ({l2:?}, [])] }})",
            l0 = optimized_layout[0],
            l1 = optimized_layout[1],
            l2 = optimized_layout[2],
        ),
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
    type Soa = (u8, u64, u16);
    type Vec = SoaVec<DynSoa<Soa>>;

    let context = ();
    let dyn_context = DynSoaContext::of::<Soa>(&context);

    let optimized_layout = [
        Layout::new::<u8>(),
        Layout::new::<u16>(),
        Layout::new::<u64>(),
    ];
    assert_eq!(dyn_context.layouts(), optimized_layout);

    let u8 = 1u8;
    let u64 = 2u64;
    let u16 = 3u16;

    let u8_bytes = u8.to_ne_bytes();
    let u64_bytes = u64.to_ne_bytes();
    let u16_bytes = u16.to_ne_bytes();

    let u8_bytes = u8_bytes.as_slice();
    let u64_bytes = u64_bytes.as_slice();
    let u16_bytes = u16_bytes.as_slice();

    let fields = [u8_bytes, u16_bytes, u64_bytes];
    let value = DynSoa::from(&context, (u8, u64, u16));
    assert_eq!(
        value.as_refs(&dyn_context).as_ref(),
        [
            (optimized_layout[0], u8_bytes),
            (optimized_layout[1], u16_bytes),
            (optimized_layout[2], u64_bytes),
        ],
    );

    let mut vec = Vec::with_context(dyn_context);

    vec.push(value);
    assert_eq!(vec.len(), 1);
    assert!(vec.capacity() >= 1);

    let refs = DynSoaRefs::new(vec.context(), fields);
    assert_eq!(vec.get(0), Some(refs.clone()));
    assert!(vec.contains_by_refs(refs.clone()));

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    assert_eq!(
        format!("{vec:?}"),
        format!(
            "SoaVec(DynSoaSlices {{ len: 1, slices: [({l0:?}, {u8_bytes:?}), ({l1:?}, {u16_bytes:?}), ({l2:?}, {u64_bytes:?})] }})",
            l0 = optimized_layout[0],
            l1 = optimized_layout[1],
            l2 = optimized_layout[2],
        ),
    );

    let slice = vec.as_slice();
    assert_eq!(slice.len(), 1);
    assert!(slice.capacity() >= 1);
    assert_eq!(
        slice.as_slices(),
        DynSoaSlices::new(slice.context(), 1, fields),
    );
    assert_eq!(slice.get(0), Some(refs.clone()));

    assert_eq!(
        format!("{slice:?}"),
        format!(
            "SoaSlice(DynSoaSlices {{ len: 1, slices: [({l0:?}, {u8_bytes:?}), ({l1:?}, {u16_bytes:?}), ({l2:?}, {u64_bytes:?})] }})",
            l0 = optimized_layout[0],
            l1 = optimized_layout[1],
            l2 = optimized_layout[2],
        ),
    );

    assert_eq!(vec, slice);
    assert!(slice.to_owned().is_empty().not());

    let (dyn_context, vecs) = vec.into_vecs();
    // assert_eq!(vecs, (vec![ZST1], vec![ZST2(())], vec![ZST3 { empty: () }]));

    let mut vec = Vec::from_vecs(dyn_context, vecs);
    assert_eq!(vec.len(), 1);
    assert!(vec.capacity() >= 1);
    assert_eq!(vec.as_slices(), DynSoaSlices::new(vec.context(), 1, fields));

    let mut iter = vec.iter();
    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next_back(), Some(refs.clone()));
    assert_eq!(iter.next(), None);

    let value = vec.pop().expect("vector should not be empty");
    let dyn_context = vec.context();
    assert_eq!(
        value.as_refs(dyn_context).as_ref(),
        DynSoa::new(dyn_context, fields)
            .as_refs(dyn_context)
            .as_ref(),
    );
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 1);
    assert_eq!(vec.get(0), None);

    let value = unsafe { value.into::<Soa>(&context) };
    assert_eq!(value, (u8, u64, u16));

    assert_eq!(
        format!("{vec:?}"),
        format!(
            "SoaVec(DynSoaSlices {{ len: 0, slices: [({l0:?}, []), ({l1:?}, []), ({l2:?}, [])] }})",
            l0 = optimized_layout[0],
            l1 = optimized_layout[1],
            l2 = optimized_layout[2],
        ),
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
    type Soa = (u8, u64, u16);
    type Vec = SoaVec<DynSoa<Soa>>;

    let context = ();
    let dyn_context = DynSoaContext::of::<Soa>(&context);

    let optimized_layout = [
        Layout::new::<u8>(),
        Layout::new::<u16>(),
        Layout::new::<u64>(),
    ];
    assert_eq!(optimized_layout, dyn_context.layouts());

    let mut vec = Vec::with_context(dyn_context);

    let i0_u8 = 0u8;
    let i0_u64 = 0u64;
    let i0_u16 = 0u16;
    let iter = iter::repeat_with(|| DynSoa::from::<Soa>(&context, (i0_u8, i0_u64, i0_u16))).take(3);
    vec.extend(iter);

    assert_eq!(vec.len(), 3);
    assert!(vec.capacity() >= 3);

    let refs = (&i0_u8, &i0_u64, &i0_u16);
    let dyn_refs = DynSoaRefs::from::<Soa>(&context, refs);
    assert!(vec.contains_by_refs(dyn_refs));

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
        DynSoaSlices::new(
            vec.context(),
            3,
            [i0_u8s_bytes, i0_u16s_bytes, i0_u64s_bytes],
        ),
    );
    assert_eq!(
        format!("{vec:?}"),
        format!(
            "SoaVec(DynSoaSlices {{ len: 3, slices: [({l0:?}, {i0_u8s_bytes:?}), ({l1:?}, {i0_u16s_bytes:?}), ({l2:?}, {i0_u64s_bytes:?})] }})",
            l0 = optimized_layout[0],
            l1 = optimized_layout[1],
            l2 = optimized_layout[2],
        ),
    );

    vec.truncate(0);

    let i1 = 1u8;
    let i2 = 2u64;
    let i3 = 3u16;
    vec.insert(0, DynSoa::from::<Soa>(&context, (i1, i2, i3)));

    let i4 = 4u8;
    let i5 = 5u64;
    let i6 = 6u16;
    vec.insert(0, DynSoa::from::<Soa>(&context, (i4, i5, i6)));

    let i7 = 7u8;
    let i8 = 8u64;
    let i9 = 9u16;
    vec.insert(1, DynSoa::from::<Soa>(&context, (i7, i8, i9)));

    assert_eq!(vec.len(), 3);
    assert!(vec.capacity() >= 3);

    let i471 = [i4, i7, i1];
    let i582 = [i5, i8, i2];
    let i693 = [i6, i9, i3];

    let i471_slice = i471.as_slice();
    let i582_slice = i582.as_slice();
    let i693_slice = i693.as_slice();

    let i471_bytes: &[u8] = unsafe {
        let data = ptr::from_ref(&i471).cast();
        let len = size_of_val(&i471);
        slice::from_raw_parts(data, len)
    };
    let i582_bytes: &[u8] = unsafe {
        let data = ptr::from_ref(&i582).cast();
        let len = size_of_val(&i582);
        slice::from_raw_parts(data, len)
    };
    let i693_bytes: &[u8] = unsafe {
        let data = ptr::from_ref(&i693).cast();
        let len = size_of_val(&i693);
        slice::from_raw_parts(data, len)
    };

    assert_eq!(
        vec.as_slices(),
        DynSoaSlices::from::<Soa>(&context, (i471_slice, i582_slice, i693_slice)),
    );
    assert_eq!(
        format!("{vec:?}"),
        format!(
            "SoaVec(DynSoaSlices {{ len: 3, slices: [({l0:?}, {i471_bytes:?}), ({l1:?}, {i693_bytes:?}), ({l2:?}, {i582_bytes:?})] }})",
            l0 = optimized_layout[0],
            l1 = optimized_layout[1],
            l2 = optimized_layout[2],
        ),
    );

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let slice = vec.as_slice();
    assert_eq!(slice.len(), 3);
    assert!(slice.capacity() >= 3);

    let slices = DynSoaSlices::from::<Soa>(&context, (i471_slice, i582_slice, i693_slice));
    assert_eq!(slice.as_slices(), slices.clone());

    assert_eq!(
        slice.get(0),
        Some(DynSoaRefs::from::<Soa>(&context, (&i4, &i5, &i6))),
    );
    assert_eq!(
        slice.get(1),
        Some(DynSoaRefs::from::<Soa>(&context, (&i7, &i8, &i9))),
    );
    assert_eq!(
        slice.get(2),
        Some(DynSoaRefs::from::<Soa>(&context, (&i1, &i2, &i3))),
    );
    assert_eq!(slice.get(0..), Some(slices.clone()));

    assert_eq!(
        format!("{slice:?}"),
        format!(
            "SoaSlice(DynSoaSlices {{ len: 3, slices: [({l0:?}, {i471_bytes:?}), ({l1:?}, {i693_bytes:?}), ({l2:?}, {i582_bytes:?})] }})",
            l0 = optimized_layout[0],
            l1 = optimized_layout[1],
            l2 = optimized_layout[2],
        ),
    );

    assert_eq!(vec, slice);
    assert!(slice.to_owned().is_empty().not());

    let (dyn_context, vecs) = vec.into_vecs();
    // assert_eq!(vecs, ..);

    let mut vec = Vec::from_vecs(dyn_context, vecs);
    assert_eq!(vec.len(), 3);
    assert!(vec.capacity() >= 3);
    assert_eq!(vec.as_slices(), slices.clone());

    for refs in &mut vec {
        let (_, u64, u16) = unsafe { refs.into::<Soa>(&context) };
        *u64 += u64::from(*u16);
    }

    let mut iter = vec.iter();
    assert_eq!(iter.len(), 3);

    let i5_plus_6 = 11_u64;
    assert_eq!(
        iter.next(),
        Some(DynSoaRefs::from::<Soa>(&context, (&i4, &i5_plus_6, &i6))),
    );
    assert_eq!(iter.len(), 2);

    let i2_plus_3 = 5_u64;
    assert_eq!(
        iter.next_back(),
        Some(DynSoaRefs::from::<Soa>(&context, (&i1, &i2_plus_3, &i3))),
    );
    assert_eq!(iter.len(), 1);

    let i8_plus_9 = 17_u64;
    assert_eq!(
        iter.next(),
        Some(DynSoaRefs::from::<Soa>(&context, (&i7, &i8_plus_9, &i9))),
    );
    assert_eq!(iter.len(), 0);

    assert_eq!(iter.next_back(), None);

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    vec.push(DynSoa::from::<Soa>(&context, (i7, i8_plus_9, i9)));
    vec.push(DynSoa::from::<Soa>(&context, (i1, i2_plus_3, i3)));
    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);

    assert_eq!(
        vec.get(0),
        Some(DynSoaRefs::from::<Soa>(&context, (&i4, &i5_plus_6, &i6))),
    );
    assert_eq!(
        vec.get(1),
        Some(DynSoaRefs::from::<Soa>(&context, (&i7, &i8_plus_9, &i9))),
    );
    assert_eq!(
        vec.get(2),
        Some(DynSoaRefs::from::<Soa>(&context, (&i1, &i2_plus_3, &i3))),
    );
    assert_eq!(
        vec.get(3),
        Some(DynSoaRefs::from::<Soa>(&context, (&i7, &i8_plus_9, &i9))),
    );
    assert_eq!(
        vec.get(4),
        Some(DynSoaRefs::from::<Soa>(&context, (&i1, &i2_plus_3, &i3))),
    );

    {
        let mut drain = vec.drain(2..4);
        assert_eq!(drain.len(), 2);

        let value = drain
            .next_back()
            .expect("drain iterator should not be empty");
        let value = unsafe { value.into::<Soa>(&context) };
        assert_eq!(value, (i7, i8_plus_9, i9));
        assert_eq!(drain.len(), 1);

        let value = drain.next().expect("drain iterator should not be empty");
        let value = unsafe { value.into::<Soa>(&context) };
        assert_eq!(value, (i1, i2_plus_3, i3));
        assert_eq!(drain.len(), 0);

        assert!(drain.next().is_none());
        assert!(drain.next_back().is_none());
    }

    assert_eq!(vec.len(), 3);
    assert!(vec.capacity() >= 5);

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let value = vec.swap_remove(1);
    let value = unsafe { value.into::<Soa>(&context) };
    assert_eq!(value, (i7, i8_plus_9, i9));

    assert_eq!(vec.len(), 2);
    assert!(vec.capacity() >= 3);

    let refs = DynSoaRefs::from::<Soa>(&context, (&i7, &i8_plus_9, &i9));
    assert!(!vec.contains_by_refs(refs));

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let value = vec.pop().expect("vector should not be empty");
    let value = unsafe { value.into::<Soa>(&context) };
    assert_eq!(value, (i1, i2_plus_3, i3));

    assert_eq!(vec.len(), 1);
    assert!(vec.capacity() >= 3);

    let refs = DynSoaRefs::from::<Soa>(&context, (&i1, &i2_plus_3, &i3));
    assert!(!vec.contains_by_refs(refs));

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let value = vec.remove(0);
    let value = unsafe { value.into::<Soa>(&context) };
    assert_eq!(value, (i4, i5_plus_6, i6));

    assert!(vec.is_empty());
    assert!(vec.capacity() >= 3);

    let refs = DynSoaRefs::from::<Soa>(&context, (&i4, &i5_plus_6, &i6));
    assert!(!vec.contains_by_refs(refs));

    let iter = iter::repeat_with(|| DynSoa::from::<Soa>(&context, (i0_u8, i0_u64, i0_u16))).take(3);
    vec.extend(iter);

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

    vec.push(DynSoa::from::<Soa>(&context, (i1, i2, i3)));
    for _ in 0..10 {
        vec.push(DynSoa::from::<Soa>(&context, (i4, i5, i6)));
        vec.push(DynSoa::from::<Soa>(&context, (i7, i8, i9)));
    }
    vec.retain_mut(|refs| {
        let (x, _, _) = unsafe { refs.into::<Soa>(&context) };
        if *x <= 3 {
            *x += 1;
            true
        } else {
            false
        }
    });

    assert_eq!(vec.len(), 1);
    assert!(vec.capacity() >= (1 + 2 * 10));

    let i2_u8 = 2_u8;
    assert_eq!(
        vec.as_slices(),
        DynSoaSlices::from::<Soa>(&context, (&[i2_u8], &[i2], &[i3])),
    );

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let boxed_slice = vec.into_boxed_slice();
    assert_eq!(boxed_slice.len(), 1);
    assert!(boxed_slice.capacity() >= 1);
    assert_eq!(
        boxed_slice.get(0),
        Some(DynSoaRefs::from::<Soa>(&context, (&i2_u8, &i2, &i3))),
    );

    let vec = boxed_slice.into_vec();
    assert_eq!(vec.len(), 1);
    assert!(vec.capacity() >= 1);
    assert_eq!(
        vec.get(0),
        Some(DynSoaRefs::from::<Soa>(&context, (&i2_u8, &i2, &i3))),
    );

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let boxed_slice = vec.into_boxed_slice();
    assert_eq!(boxed_slice.len(), 1);
    assert!(boxed_slice.capacity() >= 1);
    assert_eq!(
        boxed_slice.get(0),
        Some(DynSoaRefs::from::<Soa>(&context, (&i2_u8, &i2, &i3))),
    );

    let mut into_iter = boxed_slice.into_iter();
    assert_eq!(into_iter.len(), 1);

    let value = into_iter.next_back().expect("iterator should not be empty");
    let value = unsafe { value.into::<Soa>(&context) };
    assert_eq!(value, (i2_u8, i2, i3));

    assert!(into_iter.next().is_none());
    assert!(into_iter.next_back().is_none());
}
