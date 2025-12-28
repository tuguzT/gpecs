#![cfg(feature = "alloc")]

use std::iter;

use gpecs_soa_erased::{
    erased::{BoxedErasedSoa, ErasedSoa, ErasedSoaContext},
    soa::{field::FieldDescriptor, vec::SoaVec},
};

#[test]
fn new() {
    type Soa = (u8, u64, u16, ());
    type Vec = SoaVec<BoxedErasedSoa>;

    let context = Default::default();
    let erased_context = ErasedSoaContext::of::<Soa>(&context);

    let descriptors = [
        FieldDescriptor::of::<u8>(),
        FieldDescriptor::of::<()>(),
        FieldDescriptor::of::<u16>(),
        FieldDescriptor::of::<u64>(),
    ];
    assert!(
        erased_context
            .field_descriptors()
            .iter()
            .map(FieldDescriptor::layout)
            .eq(descriptors.iter().map(FieldDescriptor::layout))
    );

    let vec = Vec::with_context(erased_context);
    assert!(vec.is_empty());

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    assert_eq!(
        unsafe { vec.as_slices().try_into::<Soa>(&context) }.unwrap(),
        ([].as_slice(), [].as_slice(), [].as_slice(), [].as_slice()),
    );

    let slices = vec.slices();
    assert!(slices.is_empty());

    assert_eq!(
        unsafe { slices.as_slices().try_into::<Soa>(&context) }.unwrap(),
        ([].as_slice(), [].as_slice(), [].as_slice(), [].as_slice()),
    );

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let into_iter = vec.into_iter();
    assert!(into_iter.is_empty());

    assert_eq!(
        unsafe { into_iter.as_slices().try_into::<Soa>(&context) }.unwrap(),
        ([].as_slice(), [].as_slice(), [].as_slice(), [].as_slice()),
    );
}

#[test]
fn new_zst() {
    type Soa = ();
    type Vec = SoaVec<BoxedErasedSoa>;

    let context = ();
    let erased_context = ErasedSoaContext::of::<Soa>(&context);

    let descriptors = [FieldDescriptor::of::<()>()];
    assert!(
        erased_context
            .field_descriptors()
            .iter()
            .map(FieldDescriptor::layout)
            .eq(descriptors.iter().map(FieldDescriptor::layout))
    );

    let vec = Vec::with_context(erased_context);
    assert!(vec.is_empty());

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    assert_eq!(
        unsafe { vec.as_slices().try_into::<Soa>(&context) }.unwrap(),
        [].as_slice(),
    );

    let slices = vec.slices();
    assert!(slices.is_empty());

    assert_eq!(
        unsafe { slices.as_slices().try_into::<Soa>(&context) }.unwrap(),
        [].as_slice(),
    );

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let into_iter = vec.into_iter();
    assert!(into_iter.is_empty());

    assert_eq!(
        unsafe { into_iter.as_slices().try_into::<Soa>(&context) }.unwrap(),
        [].as_slice(),
    );
}

#[test]
fn with_capacity() {
    type Soa = (u8, u64, u16, ());
    type Vec = SoaVec<BoxedErasedSoa>;

    let context = Default::default();
    let erased_context = ErasedSoaContext::of::<Soa>(&context);

    let descriptors = [
        FieldDescriptor::of::<u8>(),
        FieldDescriptor::of::<()>(),
        FieldDescriptor::of::<u16>(),
        FieldDescriptor::of::<u64>(),
    ];
    assert!(
        erased_context
            .field_descriptors()
            .iter()
            .map(FieldDescriptor::layout)
            .eq(descriptors.iter().map(FieldDescriptor::layout))
    );

    let vec = Vec::with_context_and_capacity(erased_context, 10);
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 10);

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    assert_eq!(
        unsafe { vec.as_slices().try_into::<Soa>(&context) }.unwrap(),
        ([].as_slice(), [].as_slice(), [].as_slice(), [].as_slice()),
    );

    let slices = vec.slices();
    assert!(slices.is_empty());

    assert_eq!(
        unsafe { slices.as_slices().try_into::<Soa>(&context) }.unwrap(),
        ([].as_slice(), [].as_slice(), [].as_slice(), [].as_slice()),
    );

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let into_iter = vec.into_iter();
    assert!(into_iter.is_empty());

    assert_eq!(
        unsafe { into_iter.as_slices().try_into::<Soa>(&context) }.unwrap(),
        ([].as_slice(), [].as_slice(), [].as_slice(), [].as_slice()),
    );
}

#[test]
fn with_capacity_zst() {
    type Soa = ();
    type Vec = SoaVec<BoxedErasedSoa>;

    let context = ();
    let erased_context = ErasedSoaContext::of::<Soa>(&context);

    let descriptors = [FieldDescriptor::of::<()>()];
    assert!(
        erased_context
            .field_descriptors()
            .iter()
            .map(FieldDescriptor::layout)
            .eq(descriptors.iter().map(FieldDescriptor::layout))
    );

    let vec = Vec::with_context_and_capacity(erased_context, 10);
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 10);

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    assert_eq!(
        unsafe { vec.as_slices().try_into::<Soa>(&context) }.unwrap(),
        [].as_slice(),
    );

    let slices = vec.slices();
    assert!(slices.is_empty());

    assert_eq!(
        unsafe { slices.as_slices().try_into::<Soa>(&context) }.unwrap(),
        [].as_slice(),
    );

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let into_iter = vec.into_iter();
    assert!(into_iter.is_empty());

    assert_eq!(
        unsafe { into_iter.as_slices().try_into::<Soa>(&context) }.unwrap(),
        [].as_slice(),
    );
}

#[test]
fn one_item() {
    type Soa = (u8, u64, u16, ());
    type Vec = SoaVec<BoxedErasedSoa>;

    let context = Default::default();
    let erased_context = ErasedSoaContext::of::<Soa>(&context);

    let descriptors = [
        FieldDescriptor::of::<u8>(),
        FieldDescriptor::of::<()>(),
        FieldDescriptor::of::<u16>(),
        FieldDescriptor::of::<u64>(),
    ];
    assert!(
        erased_context
            .field_descriptors()
            .iter()
            .map(FieldDescriptor::layout)
            .eq(descriptors.iter().map(FieldDescriptor::layout))
    );

    let mut vec = Vec::with_context(erased_context);

    let u8 = 1;
    let u64 = 2;
    let u16 = 3;
    vec.push(ErasedSoa::try_from(&context, (u8, u64, u16, ())).unwrap());
    assert_eq!(vec.len(), 1);
    assert!(vec.capacity() >= 1);
    assert_eq!(
        vec.slices()
            .into_get(0)
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some((&u8, &u64, &u16, &())),
    );

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    assert_eq!(
        unsafe { vec.as_slices().try_into::<Soa>(&context) }.unwrap(),
        (
            [u8].as_slice(),
            [u64].as_slice(),
            [u16].as_slice(),
            [()].as_slice(),
        ),
    );

    let slices = vec.slices();
    assert_eq!(slices.len(), 1);
    assert_eq!(
        unsafe { slices.as_slices().try_into::<Soa>(&context) }.unwrap(),
        (
            [u8].as_slice(),
            [u64].as_slice(),
            [u16].as_slice(),
            [()].as_slice(),
        ),
    );
    assert_eq!(
        vec.slices()
            .into_get(0)
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some((&u8, &u64, &u16, &())),
    );

    assert_eq!(
        unsafe { slices.as_slices().try_into::<Soa>(&context) }.unwrap(),
        (
            [u8].as_slice(),
            [u64].as_slice(),
            [u16].as_slice(),
            [()].as_slice(),
        ),
    );

    let mut iter = vec.slices().into_iter();
    assert_eq!(iter.len(), 1);
    assert_eq!(
        iter.next_back()
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some((&u8, &u64, &u16, &())),
    );
    assert!(iter.next().is_none());

    let value = vec.pop().expect("vector should not be empty");
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 1);
    assert!(vec.slices().into_get(0).is_none());

    let value = unsafe { value.try_into::<Soa>(&context) }.unwrap();
    assert_eq!(value, (u8, u64, u16, ()));

    assert_eq!(
        unsafe { vec.as_slices().try_into::<Soa>(&context) }.unwrap(),
        ([].as_slice(), [].as_slice(), [].as_slice(), [].as_slice()),
    );

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let into_iter = vec.into_iter();
    assert!(into_iter.is_empty());
}

#[test]
fn one_item_zst() {
    type Soa = ();
    type Vec = SoaVec<BoxedErasedSoa>;

    let context = ();
    let erased_context = ErasedSoaContext::of::<Soa>(&context);

    let descriptors = [FieldDescriptor::of::<()>()];
    assert!(
        erased_context
            .field_descriptors()
            .iter()
            .map(FieldDescriptor::layout)
            .eq(descriptors.iter().map(FieldDescriptor::layout))
    );

    let mut vec = Vec::with_context(erased_context);

    vec.push(ErasedSoa::try_from(&context, ()).unwrap());
    assert_eq!(vec.len(), 1);
    assert!(vec.capacity() >= 1);
    assert_eq!(
        vec.slices()
            .into_get(0)
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some(&()),
    );

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    assert_eq!(
        unsafe { vec.as_slices().try_into::<Soa>(&context) }.unwrap(),
        [()].as_slice(),
    );

    let slices = vec.slices();
    assert_eq!(slices.len(), 1);
    assert_eq!(
        unsafe { slices.as_slices().try_into::<Soa>(&context) }.unwrap(),
        [()].as_slice(),
    );
    assert_eq!(
        vec.slices()
            .into_get(0)
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some(&()),
    );

    assert_eq!(
        unsafe { slices.as_slices().try_into::<Soa>(&context) }.unwrap(),
        [()].as_slice(),
    );

    let mut iter = vec.slices().into_iter();
    assert_eq!(iter.len(), 1);
    assert_eq!(
        iter.next_back()
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some(&()),
    );
    assert!(iter.next().is_none());

    let value = vec.pop().expect("vector should not be empty");
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 1);
    assert!(vec.slices().into_get(0).is_none());

    let value = unsafe { value.try_into::<Soa>(&context) }.unwrap();
    assert_eq!(value, ());

    assert_eq!(
        unsafe { vec.as_slices().try_into::<Soa>(&context) }.unwrap(),
        [].as_slice(),
    );

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let into_iter = vec.into_iter();
    assert!(into_iter.is_empty());
}

#[test]
fn three_items() {
    type Soa = (u8, String, u64, ());
    type Vec = SoaVec<BoxedErasedSoa>;

    let context = Default::default();
    let erased_context = ErasedSoaContext::of::<Soa>(&context);

    let descriptors = [
        FieldDescriptor::of::<u8>(),
        FieldDescriptor::of::<()>(),
        FieldDescriptor::of::<String>(),
        FieldDescriptor::of::<u64>(),
    ];
    assert!(
        erased_context
            .field_descriptors()
            .iter()
            .map(FieldDescriptor::layout)
            .eq(descriptors.iter().map(FieldDescriptor::layout))
    );

    let mut vec = Vec::with_context(erased_context);

    let iter = iter::repeat_with(|| {
        ErasedSoa::try_from::<Soa>(&context, (0, "0".to_owned(), 0, ())).unwrap()
    })
    .take(3);
    vec.extend(iter);

    assert_eq!(vec.len(), 3);
    assert!(vec.capacity() >= 3);

    assert_eq!(
        vec.slices()
            .into_get(0)
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some((&0, &"0".to_owned(), &0, &())),
    );
    assert_eq!(
        vec.slices()
            .into_get(1)
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some((&0, &"0".to_owned(), &0, &())),
    );
    assert_eq!(
        vec.slices()
            .into_get(2)
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some((&0, &"0".to_owned(), &0, &())),
    );

    assert_eq!(
        unsafe { vec.as_slices().try_into::<Soa>(&context) }.unwrap(),
        (
            [0; 3].as_slice(),
            ["0".to_owned(), "0".to_owned(), "0".to_owned()].as_slice(),
            [0; 3].as_slice(),
            [(); 3].as_slice(),
        ),
    );

    // use `drain` instead of `truncate` to drop all the contents,
    // erased vec does not do it automatically
    for erased in vec.drain(..) {
        let (t, u, v, w) = unsafe { erased.try_into::<Soa>(&context) }.unwrap();
        assert_eq!((t, u, v, w), (0, "0".to_owned(), 0, ()));
    }

    vec.insert(
        0,
        ErasedSoa::try_from::<Soa>(&context, (1, "2".to_owned(), 3, ())).unwrap(),
    );
    vec.insert(
        0,
        ErasedSoa::try_from::<Soa>(&context, (4, "5".to_owned(), 6, ())).unwrap(),
    );
    vec.insert(
        1,
        ErasedSoa::try_from::<Soa>(&context, (7, "8".to_owned(), 9, ())).unwrap(),
    );

    assert_eq!(vec.len(), 3);
    assert!(vec.capacity() >= 3);

    assert_eq!(
        unsafe { vec.as_slices().try_into::<Soa>(&context) }.unwrap(),
        (
            [4, 7, 1].as_slice(),
            ["5".to_owned(), "8".to_owned(), "2".to_owned()].as_slice(),
            [6, 9, 3].as_slice(),
            [(), (), ()].as_slice(),
        ),
    );

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let slices = vec.slices();
    assert_eq!(slices.len(), 3);

    assert_eq!(
        unsafe { slices.as_slices().try_into::<Soa>(&context) }.unwrap(),
        (
            [4, 7, 1].as_slice(),
            ["5".to_owned(), "8".to_owned(), "2".to_owned()].as_slice(),
            [6, 9, 3].as_slice(),
            [(), (), ()].as_slice(),
        ),
    );

    assert_eq!(
        slices
            .get(0)
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some((&4, &"5".to_owned(), &6, &())),
    );
    assert_eq!(
        slices
            .get(1)
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some((&7, &"8".to_owned(), &9, &())),
    );
    assert_eq!(
        slices
            .get(2)
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some((&1, &"2".to_owned(), &3, &())),
    );

    for refs in &mut vec {
        let (t, _, _, _) = unsafe { refs.try_into::<Soa>(&context) }.unwrap();
        *t += 1;
    }

    let mut iter = vec.slices().into_iter();
    assert_eq!(iter.len(), 3);

    assert_eq!(
        iter.next()
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some((&5, &"5".to_owned(), &6, &())),
    );
    assert_eq!(iter.len(), 2);

    assert_eq!(
        iter.next_back()
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some((&2, &"2".to_owned(), &3, &())),
    );
    assert_eq!(iter.len(), 1);

    assert_eq!(
        iter.next()
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some((&8, &"8".to_owned(), &9, &())),
    );
    assert_eq!(iter.len(), 0);

    assert!(iter.next_back().is_none());

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    vec.push(ErasedSoa::try_from::<Soa>(&context, (8, "8".to_owned(), 9, ())).unwrap());
    vec.push(ErasedSoa::try_from::<Soa>(&context, (2, "2".to_owned(), 3, ())).unwrap());
    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);

    assert_eq!(
        vec.slices()
            .into_get(0)
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some((&5, &"5".to_owned(), &6, &())),
    );
    assert_eq!(
        vec.slices()
            .into_get(1)
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some((&8, &"8".to_owned(), &9, &())),
    );
    assert_eq!(
        vec.slices()
            .into_get(2)
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some((&2, &"2".to_owned(), &3, &())),
    );
    assert_eq!(
        vec.slices()
            .into_get(3)
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some((&8, &"8".to_owned(), &9, &())),
    );
    assert_eq!(
        vec.slices()
            .into_get(4)
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some((&2, &"2".to_owned(), &3, &())),
    );

    {
        let mut drain = vec.drain(2..4);
        assert_eq!(drain.len(), 2);

        let value = drain
            .next_back()
            .expect("drain iterator should not be empty");
        let value = unsafe { value.try_into::<Soa>(&context) }.unwrap();
        assert_eq!(value, (8, "8".to_owned(), 9, ()));
        assert_eq!(drain.len(), 1);

        let value = drain.next().expect("drain iterator should not be empty");
        let value = unsafe { value.try_into::<Soa>(&context) }.unwrap();
        assert_eq!(value, (2, "2".to_owned(), 3, ()));
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
    let value = unsafe { value.try_into::<Soa>(&context) }.unwrap();
    assert_eq!(value, (8, "8".to_owned(), 9, ()));

    assert_eq!(vec.len(), 2);
    assert!(vec.capacity() >= 3);

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let value = vec.pop().expect("vector should not be empty");
    let value = unsafe { value.try_into::<Soa>(&context) }.unwrap();
    assert_eq!(value, (2, "2".to_owned(), 3, ()));

    assert_eq!(vec.len(), 1);
    assert!(vec.capacity() >= 3);

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let value = vec.remove(0);
    let value = unsafe { value.try_into::<Soa>(&context) }.unwrap();
    assert_eq!(value, (5, "5".to_owned(), 6, ()));

    assert!(vec.is_empty());
    assert!(vec.capacity() >= 3);

    let iter = iter::repeat_with(|| {
        ErasedSoa::try_from::<Soa>(&context, (0, "0".to_owned(), 0, ())).unwrap()
    })
    .take(3);
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

    // use `drain` instead of `truncate` to drop needed contents,
    // erased vec does not do it automatically
    for erased in vec.drain(1..) {
        let (t, u, v, w) = unsafe { erased.try_into::<Soa>(&context) }.unwrap();
        assert_eq!((t, u, v, w), (0, "0".to_owned(), 0, ()));
    }
    assert_eq!(vec.len(), 1);
    assert!(vec.capacity() >= 3);

    // use `drain` instead of `clear` to drop all the contents,
    // erased vec does not do it automatically
    for erased in vec.drain(..) {
        let (t, u, v, w) = unsafe { erased.try_into::<Soa>(&context) }.unwrap();
        assert_eq!((t, u, v, w), (0, "0".to_owned(), 0, ()));
    }
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 3);

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    vec.push(ErasedSoa::try_from::<Soa>(&context, (1, "2".to_owned(), 3, ())).unwrap());
    for _ in 0..10 {
        vec.push(ErasedSoa::try_from::<Soa>(&context, (4, "5".to_owned(), 6, ())).unwrap());
        vec.push(ErasedSoa::try_from::<Soa>(&context, (7, "8".to_owned(), 9, ())).unwrap());
    }

    // use this code instead of `retain_mut` to drop needed contents,
    // erased vec does not do it automatically
    for index in (0..vec.len()).rev() {
        let refs = vec.mut_slices().into_index_mut(index);
        let (x, _, _, _) = unsafe { refs.try_into::<Soa>(&context) }.unwrap();
        if *x <= 3 {
            *x += 1;
        } else {
            let erased = vec.remove(index);
            let _ = unsafe { erased.try_into::<Soa>(&context) };
        }
    }

    assert_eq!(vec.len(), 1);
    assert!(vec.capacity() >= (1 + 2 * 10));

    assert_eq!(
        unsafe { vec.as_slices().try_into::<Soa>(&context) }.unwrap(),
        (
            [2].as_slice(),
            ["2".to_owned()].as_slice(),
            [3].as_slice(),
            [()].as_slice(),
        ),
    );

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let mut into_iter = vec.into_iter();
    assert_eq!(into_iter.len(), 1);

    let value = into_iter.next_back().expect("iterator should not be empty");
    let value = unsafe { value.try_into::<Soa>(&context) }.unwrap();
    assert_eq!(value, (2, "2".to_owned(), 3, ()));

    assert!(into_iter.next().is_none());
    assert!(into_iter.next_back().is_none());
}

#[test]
fn three_items_zst() {
    type Soa = ();
    type Vec = SoaVec<BoxedErasedSoa>;

    let context = ();
    let erased_context = ErasedSoaContext::of::<Soa>(&context);

    let descriptors = [FieldDescriptor::of::<()>()];
    assert!(
        erased_context
            .field_descriptors()
            .iter()
            .map(FieldDescriptor::layout)
            .eq(descriptors.iter().map(FieldDescriptor::layout))
    );

    let mut vec = Vec::with_context(erased_context);

    let iter = iter::repeat_with(|| ErasedSoa::try_from::<Soa>(&context, ()).unwrap()).take(3);
    vec.extend(iter);

    assert_eq!(vec.len(), 3);
    assert!(vec.capacity() >= 3);

    assert_eq!(
        vec.slices()
            .into_get(0)
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some(&()),
    );
    assert_eq!(
        vec.slices()
            .into_get(1)
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some(&()),
    );
    assert_eq!(
        vec.slices()
            .into_get(2)
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some(&()),
    );

    assert_eq!(
        unsafe { vec.as_slices().try_into::<Soa>(&context) }.unwrap(),
        [(); 3].as_slice(),
    );

    // use `drain` instead of `truncate` to drop all the contents,
    // erased vec does not do it automatically
    for erased in vec.drain(..) {
        let () = unsafe { erased.try_into::<Soa>(&context) }.unwrap();
    }

    vec.insert(0, ErasedSoa::try_from::<Soa>(&context, ()).unwrap());
    vec.insert(0, ErasedSoa::try_from::<Soa>(&context, ()).unwrap());
    vec.insert(1, ErasedSoa::try_from::<Soa>(&context, ()).unwrap());

    assert_eq!(vec.len(), 3);
    assert!(vec.capacity() >= 3);

    assert_eq!(
        unsafe { vec.as_slices().try_into::<Soa>(&context) }.unwrap(),
        [(); 3].as_slice(),
    );

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let slices = vec.slices();
    assert_eq!(slices.len(), 3);

    assert_eq!(
        unsafe { slices.as_slices().try_into::<Soa>(&context) }.unwrap(),
        [(); 3].as_slice(),
    );

    assert_eq!(
        slices
            .get(0)
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some(&()),
    );
    assert_eq!(
        slices
            .get(1)
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some(&()),
    );
    assert_eq!(
        slices
            .get(2)
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some(&()),
    );

    for refs in &mut vec {
        let () = unsafe { refs.try_into::<Soa>(&context) }.unwrap();
    }

    let mut iter = vec.slices().into_iter();
    assert_eq!(iter.len(), 3);

    assert_eq!(
        iter.next()
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some(&()),
    );
    assert_eq!(iter.len(), 2);

    assert_eq!(
        iter.next_back()
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some(&()),
    );
    assert_eq!(iter.len(), 1);

    assert_eq!(
        iter.next()
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some(&()),
    );
    assert_eq!(iter.len(), 0);

    assert!(iter.next_back().is_none());

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    vec.push(ErasedSoa::try_from::<Soa>(&context, ()).unwrap());
    vec.push(ErasedSoa::try_from::<Soa>(&context, ()).unwrap());
    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);

    assert_eq!(
        vec.slices()
            .into_get(0)
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some(&()),
    );
    assert_eq!(
        vec.slices()
            .into_get(1)
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some(&()),
    );
    assert_eq!(
        vec.slices()
            .into_get(2)
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some(&()),
    );
    assert_eq!(
        vec.slices()
            .into_get(3)
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some(&()),
    );
    assert_eq!(
        vec.slices()
            .into_get(4)
            .map(|refs| unsafe { refs.try_into::<Soa>(&context) }.unwrap()),
        Some(&()),
    );

    {
        let mut drain = vec.drain(2..4);
        assert_eq!(drain.len(), 2);

        let value = drain
            .next_back()
            .expect("drain iterator should not be empty");
        let value = unsafe { value.try_into::<Soa>(&context) }.unwrap();
        assert_eq!(value, ());
        assert_eq!(drain.len(), 1);

        let value = drain.next().expect("drain iterator should not be empty");
        let value = unsafe { value.try_into::<Soa>(&context) }.unwrap();
        assert_eq!(value, ());
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
    let value = unsafe { value.try_into::<Soa>(&context) }.unwrap();
    assert_eq!(value, ());

    assert_eq!(vec.len(), 2);
    assert!(vec.capacity() >= 3);

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let value = vec.pop().expect("vector should not be empty");
    let value = unsafe { value.try_into::<Soa>(&context) }.unwrap();
    assert_eq!(value, ());

    assert_eq!(vec.len(), 1);
    assert!(vec.capacity() >= 3);

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let value = vec.remove(0);
    let value = unsafe { value.try_into::<Soa>(&context) }.unwrap();
    assert_eq!(value, ());

    assert!(vec.is_empty());
    assert!(vec.capacity() >= 3);

    let iter = iter::repeat_with(|| ErasedSoa::try_from::<Soa>(&context, ()).unwrap()).take(3);
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

    // use `drain` instead of `truncate` to drop needed contents,
    // erased vec does not do it automatically
    for erased in vec.drain(1..) {
        let () = unsafe { erased.try_into::<Soa>(&context) }.unwrap();
    }
    assert_eq!(vec.len(), 1);
    assert!(vec.capacity() >= 3);

    // use `drain` instead of `clear` to drop all the contents,
    // erased vec does not do it automatically
    for erased in vec.drain(..) {
        let () = unsafe { erased.try_into::<Soa>(&context) }.unwrap();
    }
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 3);

    let mut vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    for _ in 0..3 {
        vec.push(ErasedSoa::try_from::<Soa>(&context, ()).unwrap());
    }

    assert_eq!(vec.len(), 3);
    assert!(vec.capacity() >= 3);

    assert_eq!(
        unsafe { vec.as_slices().try_into::<Soa>(&context) }.unwrap(),
        [(); 3].as_slice(),
    );

    let vec = {
        let (ptr, len, capacity) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    };

    let mut into_iter = vec.into_iter();
    assert_eq!(into_iter.len(), 3);

    let value = into_iter.next_back().expect("iterator should not be empty");
    let value = unsafe { value.try_into::<Soa>(&context) }.unwrap();
    assert_eq!(value, ());

    assert_eq!(into_iter.len(), 2);
}
