use core::{mem::forget, ops::Not};

use gpecs_sparse::prelude::*;

type Key = EpochKey;

#[test]
fn empty() {
    let sparse_set = SparseSet::<(i32,)>::new();
    assert!(sparse_set.is_empty());
}

#[test]
fn with_capacity() {
    let sparse_set = SparseSet::<(i32,)>::with_capacity(10, 10);
    assert!(sparse_set.is_empty());
    assert!(sparse_set.capacity() >= 10);
    assert!(sparse_set.sparse_capacity() >= 10);
}

#[test]
fn empty_parts() {
    let sparse_set = SparseSet::<(i32,)>::new();

    let (dense, sparse) = sparse_set.into_parts();
    assert_eq!(dense.len(), 0);
    assert_eq!(sparse.len(), 0);

    let sparse_set = SparseSet::from_parts(dense, sparse)
        .expect("creation of sparse set from empty parts should not fail");
    assert_eq!(sparse_set.len(), 0);
}

#[test]
fn empty_keys() {
    let sparse_set = SparseSet::<(i32,)>::new();

    let keys = sparse_set.keys();
    assert_eq!(keys.len(), 0);
    assert_eq!(keys.as_slice(), &[]);
}

#[test]
fn empty_into_keys() {
    let sparse_set = SparseSet::<(i32,)>::new();

    let keys = sparse_set.into_keys();
    assert_eq!(keys.len(), 0);
    assert_eq!(keys.as_slice(), &[]);
}

#[test]
fn empty_values() {
    let sparse_set = SparseSet::<(i32,)>::new();

    let values = sparse_set.values();
    assert_eq!(values.len(), 0);
    assert_eq!(values.as_slice(), ([].as_slice(),));
}

#[test]
fn empty_values_mut() {
    let mut sparse_set = SparseSet::<(i32,)>::new();
    let values_mut = sparse_set.values_mut();

    assert_eq!(values_mut.len(), 0);
    assert_eq!(values_mut.into_slice(), ([].as_mut_slice(),));
}

#[test]
fn empty_into_values() {
    let sparse_set = SparseSet::<(i32,)>::new();

    let values = sparse_set.into_values();
    assert_eq!(values.len(), 0);
    assert_eq!(values.as_slice(), ([].as_slice(),));
}

#[test]
fn empty_iter() {
    let sparse_set = SparseSet::<(i32,)>::new();

    let iter = sparse_set.iter();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.as_keys_slice(), &[]);
    assert_eq!(iter.as_values_slice(), ([].as_slice(),));
}

#[test]
fn empty_iter_mut() {
    let mut sparse_set = SparseSet::<(i32,)>::new();
    let iter_mut = sparse_set.iter_mut();

    assert_eq!(iter_mut.len(), 0);
    assert_eq!(iter_mut.as_keys_slice(), &[]);
    assert_eq!(iter_mut.into_values_slice(), ([].as_mut_slice(),));
}

#[test]
fn empty_into_iter() {
    let sparse_set = SparseSet::<(i32,)>::new();
    let into_iter = sparse_set.into_iter();

    assert_eq!(into_iter.len(), 0);
    assert_eq!(into_iter.as_keys_slice(), &[]);
    assert_eq!(into_iter.as_values_slice(), ([].as_slice(),));
}

#[test]
fn empty_insert_one() {
    let mut sparse_set = SparseSet::new();
    let previous = sparse_set
        .insert(0, (42,))
        .expect("key-to-usize conversions should not fail");
    assert_eq!(previous, None);

    assert_eq!(sparse_set.len(), 1);
    assert_eq!(sparse_set.get(0), Some((&42,)));
    assert!(sparse_set.contains_key(0));
}

#[test]
fn with_capacity_insert_one() {
    let mut sparse_set = SparseSet::with_capacity(10, 10);
    let previous = sparse_set
        .insert(0, (42,))
        .expect("key-to-usize conversions should not fail");
    assert_eq!(previous, None);

    assert_eq!(sparse_set.len(), 1);
    assert_eq!(sparse_set.get(0), Some((&42,)));
    assert!(sparse_set.contains_key(0));
}

#[test]
fn empty_insert_one_mutate() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(0, (42,))
        .expect("key-to-usize conversions should not fail");
    *sparse_set.index_mut(0).0 = 43;

    assert_eq!(sparse_set.len(), 1);
    assert_eq!(sparse_set.get(0), Some((&43,)));
    assert!(sparse_set.contains_key(0));
}

#[test]
fn with_capacity_insert_one_mutate() {
    let mut sparse_set = SparseSet::with_capacity(10, 10);
    sparse_set
        .insert(0, (42,))
        .expect("key-to-usize conversions should not fail");
    *sparse_set.index_mut(0).0 = 43;

    assert_eq!(sparse_set.len(), 1);
    assert_eq!(sparse_set.get(0), Some((&43,)));
    assert!(sparse_set.contains_key(0));
}

#[test]
fn empty_insert_far() {
    let mut sparse_set = SparseSet::new();

    let (key, value) = (3, (42,));
    sparse_set
        .insert(key, value)
        .expect("key-to-usize conversions should not fail");

    assert_eq!(sparse_set.len(), 1);
    assert_eq!(sparse_set.get(key), Some((&value.0,)));
    assert!(sparse_set.contains_key(key));

    let (key, value) = (6, (69,));
    sparse_set
        .insert(key, value)
        .expect("key-to-usize conversions should not fail");

    assert_eq!(sparse_set.len(), 2);
    assert_eq!(sparse_set.get(key), Some((&value.0,)));
    assert!(sparse_set.contains_key(key));
}

#[test]
fn empty_insert_far_remove() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(3, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (69,))
        .expect("key-to-usize conversions should not fail");

    let key = 3;
    let value = sparse_set.remove(key).unwrap();

    assert_eq!(value, (42,));
    assert_eq!(sparse_set.len(), 1);
    assert_eq!(sparse_set.get(key), None);
    assert!(sparse_set.contains_key(key).not());

    let key = 1;
    let value = sparse_set.remove(key).unwrap();

    assert_eq!(value, (69,));
    assert_eq!(sparse_set.len(), 0);
    assert_eq!(sparse_set.get(key), None);
    assert!(sparse_set.contains_key(key).not());
}

#[test]
fn empty_push() {
    let mut sparse_set = SparseSet::new();

    let key = sparse_set
        .push((42,))
        .expect("key-to-usize conversions should not fail");
    assert_eq!(key, 0);
    assert_eq!(sparse_set.len(), 1);
    assert_eq!(sparse_set.get(key), Some((&42,)));
    assert!(sparse_set.contains_key(key));
}

#[test]
fn empty_pop() {
    let mut sparse_set = SparseSet::<(i32,)>::new();

    let popped = sparse_set.pop();
    assert_eq!(popped, None);
    assert_eq!(sparse_set.len(), 0);
}

#[test]
fn one_item_insert_remove_one() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(0, (42,))
        .expect("key-to-usize conversions should not fail");

    let removed = sparse_set.remove(0);
    assert_eq!(removed, Some((42,)));

    assert_eq!(sparse_set.len(), 0);
    assert_eq!(sparse_set.get(0), None);
    assert!(sparse_set.contains_key(0).not());
}

#[test]
fn one_item_insert_remove_one_epoch() {
    let mut sparse_set = EpochSparseSet::new();

    let key = Key::new(0, 1);
    sparse_set
        .insert(key, (42,))
        .expect("key-to-usize conversions should not fail");

    let removed = sparse_set.remove(key);
    assert_eq!(removed, Some((42,)));

    assert_eq!(sparse_set.len(), 0);
    assert_eq!(sparse_set.get(key), None);
    assert!(sparse_set.contains_key(key).not());

    assert_eq!(
        sparse_set.get_epoch(*key.sparse_index()),
        Some(key.epoch().next()),
    );
    let key = Key::new(0, key.epoch().next());
    assert_eq!(sparse_set.get(key), None);
    assert!(sparse_set.contains_key(key).not());
}

#[test]
fn one_item_insert_swap_remove_one() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(0, (42,))
        .expect("key-to-usize conversions should not fail");

    let removed = sparse_set.swap_remove(0);
    assert_eq!(removed, Some((42,)));

    assert_eq!(sparse_set.len(), 0);
    assert_eq!(sparse_set.get(0), None);
    assert!(sparse_set.contains_key(0).not());
}

#[test]
fn one_item_insert_swap_remove_one_epoch() {
    let mut sparse_set = EpochSparseSet::new();

    let key = Key::new(0, 1);
    sparse_set
        .insert(key, (42,))
        .expect("key-to-usize conversions should not fail");

    let removed = sparse_set.swap_remove(key);
    assert_eq!(removed, Some((42,)));

    assert_eq!(sparse_set.len(), 0);
    assert_eq!(sparse_set.get(key), None);
    assert!(sparse_set.contains_key(key).not());

    assert_eq!(
        sparse_set.get_epoch(*key.sparse_index()),
        Some(key.epoch().next()),
    );
    let key = Key::new(0, key.epoch().next());
    assert_eq!(sparse_set.get(key), None);
    assert!(sparse_set.contains_key(key).not());
}

#[test]
fn one_item_push_remove_one() {
    let mut sparse_set = SparseSet::new();
    let key = sparse_set
        .push((42,))
        .expect("key-to-usize conversions should not fail");

    let removed = sparse_set.remove(key);
    assert_eq!(removed, Some((42,)));

    assert_eq!(sparse_set.len(), 0);
    assert_eq!(sparse_set.get(key), None);
    assert!(sparse_set.contains_key(key).not());
}

#[test]
fn one_item_push_remove_one_epoch() {
    let mut sparse_set = EpochSparseSet::<Key, _>::new();
    let key = sparse_set
        .push((42,))
        .expect("key-to-usize conversions should not fail");

    let removed = sparse_set.remove(key);
    assert_eq!(removed, Some((42,)));

    assert_eq!(sparse_set.len(), 0);
    assert_eq!(sparse_set.get(key), None);
    assert!(sparse_set.contains_key(key).not());

    assert_eq!(
        sparse_set.get_epoch(*key.sparse_index()),
        Some(key.epoch().next()),
    );
    let key = Key::new(0, key.epoch().next());
    assert_eq!(sparse_set.get(key), None);
    assert!(sparse_set.contains_key(key).not());
}

#[test]
fn one_item_push_swap_remove_one() {
    let mut sparse_set = SparseSet::new();
    let key = sparse_set
        .push((42,))
        .expect("key-to-usize conversions should not fail");

    let removed = sparse_set.swap_remove(key);
    assert_eq!(removed, Some((42,)));

    assert_eq!(sparse_set.len(), 0);
    assert_eq!(sparse_set.get(key), None);
    assert!(sparse_set.contains_key(key).not());
}

#[test]
fn one_item_push_swap_remove_one_epoch() {
    let mut sparse_set = EpochSparseSet::<Key, _>::new();
    let key = sparse_set
        .push((42,))
        .expect("key-to-usize conversions should not fail");

    let removed = sparse_set.swap_remove(key);
    assert_eq!(removed, Some((42,)));

    assert_eq!(sparse_set.len(), 0);
    assert_eq!(sparse_set.get(key), None);
    assert!(sparse_set.contains_key(key).not());

    assert_eq!(
        sparse_set.get_epoch(*key.sparse_index()),
        Some(key.epoch().next()),
    );
    let key = Key::new(0, key.epoch().next());
    assert_eq!(sparse_set.get(key), None);
    assert!(sparse_set.contains_key(key).not());
}

#[test]
#[should_panic]
fn one_item_swap() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(0, (42,))
        .expect("key-to-usize conversions should not fail");

    sparse_set.swap(0, 0);
    assert_eq!(sparse_set.len(), 1);
    assert_eq!(sparse_set.as_slices(), ([42].as_slice(),));
    assert_eq!(sparse_set.get(0), Some((&42,)));
    assert!(sparse_set.contains_key(0));

    sparse_set.swap(0, 1);
    assert_eq!(sparse_set.len(), 1);
    assert_eq!(sparse_set.as_slices(), ([42].as_slice(),));
    assert_eq!(sparse_set.get(0), Some((&42,)));
    assert!(sparse_set.contains_key(0));
}

#[test]
#[should_panic]
fn one_item_swap_keys() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(0, (42,))
        .expect("key-to-usize conversions should not fail");

    sparse_set.swap_keys(0, 0);
    assert_eq!(sparse_set.len(), 1);
    assert_eq!(sparse_set.as_slices(), ([42].as_slice(),));
    assert_eq!(sparse_set.get(0), Some((&42,)));
    assert!(sparse_set.contains_key(0));

    sparse_set.swap_keys(0, 1);
    assert_eq!(sparse_set.len(), 1);
    assert_eq!(sparse_set.as_slices(), ([42].as_slice(),));
    assert_eq!(sparse_set.get(0), Some((&42,)));
    assert!(sparse_set.contains_key(0));
}

#[test]
fn one_item_parts() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(2, (42,))
        .expect("key-to-usize conversions should not fail");

    let (dense, sparse) = sparse_set.into_parts();
    let (keys, values) = dense.as_slices().into();
    assert_eq!(keys, &[2]);
    assert_eq!(values, ([42].as_slice(),));
    assert_eq!(
        sparse,
        &[
            SparseItem::vacant(0, ()),
            SparseItem::vacant(0, ()),
            SparseItem::occupied(0, ()),
        ]
    );

    let sparse_set = SparseSet::from_parts(dense, sparse)
        .expect("creation of sparse set from valid parts should not fail");
    assert_eq!(sparse_set.len(), 1);
    assert_eq!(sparse_set.as_slices(), ([42].as_slice(),));
    assert_eq!(sparse_set.as_keys_slice(), &[2]);
    assert_eq!(sparse_set.get(2), Some((&42,)));
}

#[test]
fn one_item_keys() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(0, (42,))
        .expect("key-to-usize conversions should not fail");

    let keys = sparse_set.keys();
    assert_eq!(keys.len(), 1);
    assert_eq!(keys.as_slice(), &[0]);
}

#[test]
fn one_item_into_keys() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(0, (42,))
        .expect("key-to-usize conversions should not fail");

    let keys = sparse_set.into_keys();
    assert_eq!(keys.len(), 1);
    assert_eq!(keys.as_slice(), &[0]);
}

#[test]
fn one_item_values() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(0, (42,))
        .expect("key-to-usize conversions should not fail");

    let values = sparse_set.values();
    assert_eq!(values.len(), 1);
    assert_eq!(values.as_slice(), ([42].as_slice(),));
}

#[test]
fn one_item_values_mut() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(0, (42,))
        .expect("key-to-usize conversions should not fail");

    let values_mut = sparse_set.values_mut();
    assert_eq!(values_mut.len(), 1);
    assert_eq!(values_mut.into_slice(), ([42].as_mut_slice(),));
}

#[test]
fn one_item_into_values() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(0, (42,))
        .expect("key-to-usize conversions should not fail");

    let values = sparse_set.into_values();
    assert_eq!(values.len(), 1);
    assert_eq!(values.as_slice(), ([42].as_slice(),));
}

#[test]
fn one_item_iter() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(0, (42,))
        .expect("key-to-usize conversions should not fail");

    let iter = sparse_set.iter();
    assert_eq!(iter.len(), 1);
    assert_eq!(iter.as_keys_slice(), &[0]);
    assert_eq!(iter.as_values_slice(), ([42].as_slice(),));
}

#[test]
fn one_item_iter_mut() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(0, (42,))
        .expect("key-to-usize conversions should not fail");

    let iter_mut = sparse_set.iter_mut();
    assert_eq!(iter_mut.len(), 1);
    assert_eq!(iter_mut.as_keys_slice(), &[0]);
    assert_eq!(iter_mut.into_values_slice(), ([42].as_mut_slice(),));
}

#[test]
fn one_item_into_iter() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(0, (42,))
        .expect("key-to-usize conversions should not fail");

    let into_iter = sparse_set.into_iter();
    assert_eq!(into_iter.len(), 1);
    assert_eq!(into_iter.as_keys_slice(), &[0]);
    assert_eq!(into_iter.as_values_slice(), ([42].as_slice(),));
}

#[test]
fn two_items_insert_first() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(0, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (69,))
        .expect("key-to-usize conversions should not fail");

    assert_eq!(sparse_set.len(), 2);
    assert_eq!(sparse_set.get(0), Some((&42,)));
    assert_eq!(sparse_set.get(1), Some((&69,)));

    let previous = sparse_set
        .insert(0, (34,))
        .expect("key-to-usize conversions should not fail");
    assert_eq!(previous, Some((42,)));

    assert_eq!(sparse_set.len(), 2);
    assert_eq!(sparse_set.get(0), Some((&34,)));
    assert_eq!(sparse_set.get(1), Some((&69,)));
    assert!(sparse_set.contains_key(0));
    assert!(sparse_set.contains_key(1));
}

#[test]
fn two_items_insert_first_epoch() {
    let mut sparse_set = EpochSparseSet::new();

    let first_key = Key::new(0, 3);
    sparse_set
        .insert(first_key, (42,))
        .expect("key-to-usize conversions should not fail");

    let second_key = Key::new(1, 0);
    sparse_set
        .insert(second_key, (69,))
        .expect("key-to-usize conversions should not fail");

    assert_eq!(sparse_set.len(), 2);
    assert_eq!(sparse_set.get(first_key), Some((&42,)));
    assert_eq!(sparse_set.get(second_key), Some((&69,)));

    let first_key = Key::new(*first_key.sparse_index(), first_key.epoch().next());
    let previous = sparse_set
        .insert(first_key, (34,))
        .expect("key-to-usize conversions should not fail");
    assert_eq!(previous, Some((42,)));

    assert_eq!(sparse_set.len(), 2);
    assert_eq!(sparse_set.get(first_key), Some((&34,)));
    assert_eq!(sparse_set.get(second_key), Some((&69,)));
    assert!(sparse_set.contains_key(first_key));
    assert!(sparse_set.contains_key(second_key));
}

#[test]
fn two_items_insert_second() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(0, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (69,))
        .expect("key-to-usize conversions should not fail");

    assert_eq!(sparse_set.len(), 2);
    assert_eq!(sparse_set.get(0), Some((&42,)));
    assert_eq!(sparse_set.get(1), Some((&69,)));

    let previous = sparse_set
        .insert(1, (34,))
        .expect("key-to-usize conversions should not fail");
    assert_eq!(previous, Some((69,)));

    assert_eq!(sparse_set.len(), 2);
    assert_eq!(sparse_set.get(0), Some((&42,)));
    assert_eq!(sparse_set.get(1), Some((&34,)));
    assert!(sparse_set.contains_key(0));
    assert!(sparse_set.contains_key(1));
}

#[test]
fn two_items_remove_first() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(0, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (69,))
        .expect("key-to-usize conversions should not fail");

    assert_eq!(sparse_set.len(), 2);
    assert_eq!(sparse_set.get(0), Some((&42,)));
    assert_eq!(sparse_set.get(1), Some((&69,)));

    let removed = sparse_set.remove(0);
    assert_eq!(removed, Some((42,)));

    assert_eq!(sparse_set.len(), 1);
    assert_eq!(sparse_set.get(0), None);
    assert_eq!(sparse_set.get(1), Some((&69,)));
    assert!(sparse_set.contains_key(0).not());
    assert!(sparse_set.contains_key(1));
}

#[test]
fn two_items_swap_remove_first() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(0, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (69,))
        .expect("key-to-usize conversions should not fail");

    assert_eq!(sparse_set.len(), 2);
    assert_eq!(sparse_set.get(0), Some((&42,)));
    assert_eq!(sparse_set.get(1), Some((&69,)));

    let removed = sparse_set.swap_remove(0);
    assert_eq!(removed, Some((42,)));

    assert_eq!(sparse_set.len(), 1);
    assert_eq!(sparse_set.get(0), None);
    assert_eq!(sparse_set.get(1), Some((&69,)));
    assert!(sparse_set.contains_key(0).not());
    assert!(sparse_set.contains_key(1));
}

#[test]
fn two_items_remove_second() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(0, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (69,))
        .expect("key-to-usize conversions should not fail");

    assert_eq!(sparse_set.len(), 2);
    assert_eq!(sparse_set.get(0), Some((&42,)));
    assert_eq!(sparse_set.get(1), Some((&69,)));

    let removed = sparse_set.remove(1);
    assert_eq!(removed, Some((69,)));

    assert_eq!(sparse_set.len(), 1);
    assert_eq!(sparse_set.get(0), Some((&42,)));
    assert_eq!(sparse_set.get(1), None);
    assert!(sparse_set.contains_key(0));
    assert!(sparse_set.contains_key(1).not());
}

#[test]
fn two_items_swap_remove_second() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(0, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (69,))
        .expect("key-to-usize conversions should not fail");

    assert_eq!(sparse_set.len(), 2);
    assert_eq!(sparse_set.get(0), Some((&42,)));
    assert_eq!(sparse_set.get(1), Some((&69,)));

    let removed = sparse_set.swap_remove(1);
    assert_eq!(removed, Some((69,)));

    assert_eq!(sparse_set.len(), 1);
    assert_eq!(sparse_set.get(0), Some((&42,)));
    assert_eq!(sparse_set.get(1), None);
    assert!(sparse_set.contains_key(0));
    assert!(sparse_set.contains_key(1).not());
}

#[test]
fn two_items_remove_one_insert_one() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(0, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (69,))
        .expect("key-to-usize conversions should not fail");

    let removed = sparse_set.remove(0);
    assert_eq!(removed, Some((42,)));
    assert_eq!(sparse_set.get(0), None);

    sparse_set
        .insert(0, (34,))
        .expect("key-to-usize conversions should not fail");
    assert_eq!(sparse_set.get(0), Some((&34,)));
    assert_eq!(sparse_set.get(1), Some((&69,)));
    assert!(sparse_set.contains_key(0));
    assert!(sparse_set.contains_key(1));
}

#[test]
fn two_items_swap_remove_one_insert_one() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(0, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (69,))
        .expect("key-to-usize conversions should not fail");

    let removed = sparse_set.swap_remove(0);
    assert_eq!(removed, Some((42,)));
    assert_eq!(sparse_set.get(0), None);

    sparse_set
        .insert(0, (34,))
        .expect("key-to-usize conversions should not fail");
    assert_eq!(sparse_set.get(0), Some((&34,)));
    assert_eq!(sparse_set.get(1), Some((&69,)));
    assert!(sparse_set.contains_key(0));
    assert!(sparse_set.contains_key(1));
}

#[test]
fn two_items_remove_one_push_one() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(0, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (69,))
        .expect("key-to-usize conversions should not fail");

    let removed = sparse_set.remove(0);
    assert_eq!(removed, Some((42,)));
    assert_eq!(sparse_set.get(0), None);

    let key = sparse_set
        .push((34,))
        .expect("key-to-usize conversions should not fail");
    assert_eq!(key, 0);

    assert_eq!(sparse_set.get(0), Some((&34,)));
    assert_eq!(sparse_set.get(1), Some((&69,)));
    assert!(sparse_set.contains_key(0));
    assert!(sparse_set.contains_key(1));
}

#[test]
fn two_items_swap_remove_one_push_one() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(0, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (69,))
        .expect("key-to-usize conversions should not fail");

    let removed = sparse_set.swap_remove(0);
    assert_eq!(removed, Some((42,)));
    assert_eq!(sparse_set.get(0), None);

    let key = sparse_set
        .push((34,))
        .expect("key-to-usize conversions should not fail");
    assert_eq!(key, 0);

    assert_eq!(sparse_set.get(0), Some((&34,)));
    assert_eq!(sparse_set.get(1), Some((&69,)));
    assert!(sparse_set.contains_key(0));
    assert!(sparse_set.contains_key(1));
}

#[test]
fn two_items_swap() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(0, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (69,))
        .expect("key-to-usize conversions should not fail");

    sparse_set.swap(0, 0);
    assert_eq!(sparse_set.len(), 2);
    assert_eq!(sparse_set.as_slices(), ([42, 69].as_slice(),));
    assert_eq!(sparse_set.get(0), Some((&42,)));
    assert_eq!(sparse_set.get(1), Some((&69,)));

    sparse_set.swap(0, 1);
    assert_eq!(sparse_set.len(), 2);
    assert_eq!(sparse_set.as_slices(), ([69, 42].as_slice(),));
    assert_eq!(sparse_set.get(0), Some((&69,)));
    assert_eq!(sparse_set.get(1), Some((&42,)));

    sparse_set.swap(1, 1);
    assert_eq!(sparse_set.len(), 2);
    assert_eq!(sparse_set.as_slices(), ([69, 42].as_slice(),));
    assert_eq!(sparse_set.get(0), Some((&69,)));
    assert_eq!(sparse_set.get(1), Some((&42,)));
}

#[test]
fn two_items_swap_keys() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(0, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (69,))
        .expect("key-to-usize conversions should not fail");

    sparse_set.swap_keys(0, 0);
    assert_eq!(sparse_set.len(), 2);
    assert_eq!(sparse_set.as_slices(), ([42, 69].as_slice(),));
    assert_eq!(sparse_set.get(0), Some((&42,)));
    assert_eq!(sparse_set.get(1), Some((&69,)));

    sparse_set.swap_keys(0, 1);
    assert_eq!(sparse_set.len(), 2);
    assert_eq!(sparse_set.as_slices(), ([42, 69].as_slice(),));
    assert_eq!(sparse_set.get(0), Some((&69,)));
    assert_eq!(sparse_set.get(1), Some((&42,)));

    sparse_set.swap_keys(1, 1);
    assert_eq!(sparse_set.len(), 2);
    assert_eq!(sparse_set.as_slices(), ([42, 69].as_slice(),));
    assert_eq!(sparse_set.get(0), Some((&69,)));
    assert_eq!(sparse_set.get(1), Some((&42,)));
}

#[test]
fn two_items_insert_pop() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(5, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(2, (69,))
        .expect("key-to-usize conversions should not fail");

    let popped = sparse_set.pop();
    assert_eq!(popped, Some((2, (69,)).into()));
    assert_eq!(sparse_set.len(), 1);
    assert_eq!(sparse_set.get(5), Some((&42,)));
    assert_eq!(sparse_set.get(2), None);
}

#[test]
fn two_items_push_pop() {
    let mut sparse_set = SparseSet::new();
    let first_key = sparse_set
        .push((42,))
        .expect("key-to-usize conversions should not fail");
    let second_key = sparse_set
        .push((69,))
        .expect("key-to-usize conversions should not fail");

    let popped = sparse_set.pop();
    assert_eq!(popped, Some((second_key, (69,)).into()));
    assert_eq!(sparse_set.len(), 1);
    assert_eq!(sparse_set.get(first_key), Some((&42,)));
    assert_eq!(sparse_set.get(second_key), None);
}

#[test]
fn two_items_insert_pop_epoch() {
    let mut sparse_set = EpochSparseSet::new();

    let first_key = Key::new(5, 1);
    sparse_set
        .insert(first_key, (42,))
        .expect("key-to-usize conversions should not fail");

    let second_key = Key::new(2, 0);
    sparse_set
        .insert(second_key, (69,))
        .expect("key-to-usize conversions should not fail");

    let popped = sparse_set.pop();
    assert_eq!(popped, Some((second_key, (69,)).into()));
    assert_eq!(sparse_set.len(), 1);
    assert_eq!(sparse_set.get(first_key), Some((&42,)));
    assert_eq!(sparse_set.get(second_key), None);

    assert_eq!(
        sparse_set.get_epoch(*second_key.sparse_index()),
        Some(second_key.epoch().next()),
    );
}

#[test]
fn two_items_push_pop_epoch() {
    let mut sparse_set = EpochSparseSet::<Key, _>::new();
    let first_key = sparse_set
        .push((42,))
        .expect("key-to-usize conversions should not fail");
    let second_key = sparse_set
        .push((69,))
        .expect("key-to-usize conversions should not fail");

    let popped = sparse_set.pop();
    assert_eq!(popped, Some((second_key, (69,)).into()));
    assert_eq!(sparse_set.len(), 1);
    assert_eq!(sparse_set.get(first_key), Some((&42,)));
    assert_eq!(sparse_set.get(second_key), None);

    assert_eq!(
        sparse_set.get_epoch(*second_key.sparse_index()),
        Some(second_key.epoch().next()),
    );
}

#[test]
fn two_items_invalidate_epoch() {
    let mut sparse_set = EpochSparseSet::new();

    let first_key = Key::new(5, 1);
    sparse_set
        .insert(first_key, (42,))
        .expect("key-to-usize conversions should not fail");

    let second_key = Key::new(2, 0);
    sparse_set
        .insert(second_key, (69,))
        .expect("key-to-usize conversions should not fail");

    let new_first_key = sparse_set
        .invalidate_epoch(first_key)
        .expect("first key should be present");
    assert_eq!(new_first_key.sparse_index(), first_key.sparse_index());
    assert_eq!(new_first_key.epoch(), &first_key.epoch().next());
    assert_eq!(new_first_key, Key::new(5, 2));
    assert_eq!(sparse_set.get(first_key), None);
    assert_eq!(sparse_set.get(new_first_key), Some((&42,)));

    let new_second_key = sparse_set
        .invalidate_epoch(second_key)
        .expect("second key should be present");
    assert_eq!(new_second_key.sparse_index(), second_key.sparse_index());
    assert_eq!(new_second_key.epoch(), &second_key.epoch().next());
    assert_eq!(new_second_key, Key::new(2, 1));
    assert_eq!(sparse_set.get(second_key), None);
    assert_eq!(sparse_set.get(new_second_key), Some((&69,)));
}

#[test]
fn three_items_insert_remove_middle() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(2, (34,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(5, (69,))
        .expect("key-to-usize conversions should not fail");

    let removed = sparse_set.remove(2);
    assert_eq!(removed, Some((34,)));

    assert_eq!(sparse_set.len(), 2);
    assert_eq!(sparse_set.get(2), None);
    assert_eq!(sparse_set.get(1), Some((&42,)));
    assert_eq!(sparse_set.get(5), Some((&69,)));
    assert!(sparse_set.contains_key(2).not());
    assert!(sparse_set.contains_key(1));
    assert!(sparse_set.contains_key(5));
}

#[test]
fn three_items_push_remove_middle() {
    let mut sparse_set = SparseSet::new();
    let first_key = sparse_set
        .push((34,))
        .expect("key-to-usize conversions should not fail");
    let middle_key = sparse_set
        .push((42,))
        .expect("key-to-usize conversions should not fail");
    let last_key = sparse_set
        .push((69,))
        .expect("key-to-usize conversions should not fail");

    let removed = sparse_set.remove(middle_key);
    assert_eq!(removed, Some((42,)));

    assert_eq!(sparse_set.len(), 2);
    assert_eq!(sparse_set.get(first_key), Some((&34,)));
    assert_eq!(sparse_set.get(middle_key), None);
    assert_eq!(sparse_set.get(last_key), Some((&69,)));
    assert!(sparse_set.contains_key(first_key));
    assert!(sparse_set.contains_key(middle_key).not());
    assert!(sparse_set.contains_key(last_key));
}

#[test]
fn three_items_swap_remove_middle() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(0, (34,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(2, (69,))
        .expect("key-to-usize conversions should not fail");

    let removed = sparse_set.swap_remove(1);
    assert_eq!(removed, Some((42,)));

    assert_eq!(sparse_set.len(), 2);
    assert_eq!(sparse_set.get(0), Some((&34,)));
    assert_eq!(sparse_set.get(1), None);
    assert_eq!(sparse_set.get(2), Some((&69,)));
    assert!(sparse_set.contains_key(0));
    assert!(sparse_set.contains_key(1).not());
    assert!(sparse_set.contains_key(2));
}

#[test]
fn three_items_parts() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(2, (34,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(5, (69,))
        .expect("key-to-usize conversions should not fail");

    let (mut dense, sparse) = sparse_set.into_parts();
    let (keys, values) = dense.as_slices().into();
    assert_eq!(keys, &[2, 1, 5]);
    assert_eq!(values, ([34, 42, 69].as_slice(),));
    assert_eq!(
        sparse,
        &[
            SparseItem::vacant(0, ()),
            SparseItem::occupied(1, ()),
            SparseItem::occupied(0, ()),
            SparseItem::vacant(0, ()),
            SparseItem::vacant(0, ()),
            SparseItem::occupied(2, ()),
        ]
    );

    dense.swap_remove(0);
    let sparse_set = SparseSet::from_parts(dense, sparse)
        .expect("creation of sparse set from valid parts should not fail");
    assert_eq!(sparse_set.len(), 2);
    assert_eq!(sparse_set.as_slices(), ([69, 42].as_slice(),));
    assert_eq!(sparse_set.as_keys_slice(), &[5, 1]);
    assert_eq!(sparse_set.get(5), Some((&69,)));
}

#[test]
fn three_items_keys() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(2, (34,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(5, (69,))
        .expect("key-to-usize conversions should not fail");

    let keys = sparse_set.keys();
    assert_eq!(keys.len(), 3);
    assert_eq!(keys.as_slice(), &[2, 1, 5]);
}

#[test]
fn three_items_into_keys() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(2, (34,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(5, (69,))
        .expect("key-to-usize conversions should not fail");

    let keys = sparse_set.into_keys();
    assert_eq!(keys.len(), 3);
    assert_eq!(keys.as_slice(), &[2, 1, 5]);
}

#[test]
fn three_items_values() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(2, (34,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(5, (69,))
        .expect("key-to-usize conversions should not fail");

    let values = sparse_set.values();
    assert_eq!(values.len(), 3);
    assert_eq!(values.as_slice(), ([34, 42, 69].as_slice(),));
}

#[test]
fn three_items_values_mut() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(2, (34,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(5, (69,))
        .expect("key-to-usize conversions should not fail");

    let values_mut = sparse_set.values_mut();
    assert_eq!(values_mut.len(), 3);
    assert_eq!(values_mut.into_slice(), ([34, 42, 69].as_mut_slice(),));
}

#[test]
fn three_items_into_values() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(2, (34,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(5, (69,))
        .expect("key-to-usize conversions should not fail");

    let values = sparse_set.into_values();
    assert_eq!(values.len(), 3);
    assert_eq!(values.as_slice(), ([34, 42, 69].as_slice(),));
}

#[test]
fn three_items_iter() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(2, (34,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(5, (69,))
        .expect("key-to-usize conversions should not fail");

    let iter = sparse_set.iter();
    assert_eq!(iter.len(), 3);
    assert_eq!(iter.as_keys_slice(), &[2, 1, 5]);
    assert_eq!(iter.as_values_slice(), ([34, 42, 69].as_slice(),));
}

#[test]
fn three_items_iter_mut() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(2, (34,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(5, (69,))
        .expect("key-to-usize conversions should not fail");

    let iter_mut = sparse_set.iter_mut();
    assert_eq!(iter_mut.len(), 3);
    assert_eq!(iter_mut.as_keys_slice(), &[2, 1, 5]);
    assert_eq!(iter_mut.into_values_slice(), ([34, 42, 69].as_mut_slice(),));
}

#[test]
fn three_items_into_iter() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(2, (34,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(5, (69,))
        .expect("key-to-usize conversions should not fail");

    let into_iter = sparse_set.into_iter();
    assert_eq!(into_iter.len(), 3);
    assert_eq!(into_iter.as_keys_slice(), &[2, 1, 5]);
    assert_eq!(into_iter.as_values_slice(), ([34, 42, 69].as_slice(),));
}

#[test]
fn five_items_remove_insert() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(4, (34,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(2, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (69,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(6, (228,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(0, (666,))
        .expect("key-to-usize conversions should not fail");

    let key = 1;
    let value = sparse_set.remove(key).unwrap();
    assert_eq!(value, (69,));

    let key = 6;
    let value = sparse_set.remove(key).unwrap();
    assert_eq!(value, (228,));

    let key = 4;
    let value = sparse_set.remove(key).unwrap();
    assert_eq!(value, (34,));

    let key = 0;
    let value = sparse_set.remove(key).unwrap();
    assert_eq!(value, (666,));

    let key = 3;
    let value = (0,);
    let previous = sparse_set
        .insert(key, value)
        .expect("key-to-usize conversions should not fail");

    assert_eq!(previous, None);
    assert_eq!(sparse_set.get(key), Some((&value.0,)));
    assert!(sparse_set.contains_key(key));

    let key = 2;
    let value = (1,);
    let previous = sparse_set
        .insert(key, value)
        .expect("key-to-usize conversions should not fail");

    assert_eq!(previous, Some((42,)));
    assert_eq!(sparse_set.get(key), Some((&value.0,)));
    assert!(sparse_set.contains_key(key));

    let key = 4;
    let value = (10,);
    let previous = sparse_set
        .insert(key, value)
        .expect("key-to-usize conversions should not fail");

    assert_eq!(previous, None);
    assert_eq!(sparse_set.get(key), Some((&value.0,)));
    assert!(sparse_set.contains_key(key));
}

#[test]
fn five_items_swap_remove_insert() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(4, (34,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(2, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (69,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(6, (228,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(0, (666,))
        .expect("key-to-usize conversions should not fail");

    let key = 1;
    let value = sparse_set.swap_remove(key).unwrap();
    assert_eq!(value, (69,));

    let key = 6;
    let value = sparse_set.swap_remove(key).unwrap();
    assert_eq!(value, (228,));

    let key = 4;
    let value = sparse_set.swap_remove(key).unwrap();
    assert_eq!(value, (34,));

    let key = 0;
    let value = sparse_set.swap_remove(key).unwrap();
    assert_eq!(value, (666,));

    let key = 3;
    let value = (0,);
    let previous = sparse_set
        .insert(key, value)
        .expect("key-to-usize conversions should not fail");

    assert_eq!(previous, None);
    assert_eq!(sparse_set.get(key), Some((&value.0,)));
    assert!(sparse_set.contains_key(key));

    let key = 2;
    let value = (1,);
    let previous = sparse_set
        .insert(key, value)
        .expect("key-to-usize conversions should not fail");

    assert_eq!(previous, Some((42,)));
    assert_eq!(sparse_set.get(key), Some((&value.0,)));
    assert!(sparse_set.contains_key(key));

    let key = 4;
    let value = (10,);
    let previous = sparse_set
        .insert(key, value)
        .expect("key-to-usize conversions should not fail");

    assert_eq!(previous, None);
    assert_eq!(sparse_set.get(key), Some((&value.0,)));
    assert!(sparse_set.contains_key(key));
}

#[test]
fn five_items_remove_push() {
    let mut sparse_set = SparseSet::new();
    let _key0 = sparse_set
        .push((34,))
        .expect("key-to-usize conversions should not fail");
    let key1 = sparse_set
        .push((42,))
        .expect("key-to-usize conversions should not fail");
    let key2 = sparse_set
        .push((69,))
        .expect("key-to-usize conversions should not fail");
    let key3 = sparse_set
        .push((228,))
        .expect("key-to-usize conversions should not fail");
    let key4 = sparse_set
        .push((666,))
        .expect("key-to-usize conversions should not fail");

    let value = sparse_set.remove(key1).unwrap();
    assert_eq!(value, (42,));

    let value = sparse_set.remove(key3).unwrap();
    assert_eq!(value, (228,));

    let value = sparse_set.remove(key4).unwrap();
    assert_eq!(value, (666,));

    let value = sparse_set.remove(key2).unwrap();
    assert_eq!(value, (69,));

    let value = (0,);
    let key = sparse_set
        .push(value)
        .expect("key-to-usize conversions should not fail");
    assert_eq!(sparse_set.get(key), Some((&value.0,)));
    assert!(sparse_set.contains_key(key));

    let value = (1,);
    let key = sparse_set
        .push(value)
        .expect("key-to-usize conversions should not fail");
    assert_eq!(sparse_set.get(key), Some((&value.0,)));
    assert!(sparse_set.contains_key(key));

    let value = (10,);
    let key = sparse_set
        .push(value)
        .expect("key-to-usize conversions should not fail");
    assert_eq!(sparse_set.get(key), Some((&value.0,)));
    assert!(sparse_set.contains_key(key));
}

#[test]
fn five_items_swap_remove_push() {
    let mut sparse_set = SparseSet::new();
    let _key0 = sparse_set
        .push((34,))
        .expect("key-to-usize conversions should not fail");
    let key1 = sparse_set
        .push((42,))
        .expect("key-to-usize conversions should not fail");
    let key2 = sparse_set
        .push((69,))
        .expect("key-to-usize conversions should not fail");
    let key3 = sparse_set
        .push((228,))
        .expect("key-to-usize conversions should not fail");
    let key4 = sparse_set
        .push((666,))
        .expect("key-to-usize conversions should not fail");

    let value = sparse_set.swap_remove(key1).unwrap();
    assert_eq!(value, (42,));

    let value = sparse_set.swap_remove(key3).unwrap();
    assert_eq!(value, (228,));

    let value = sparse_set.swap_remove(key4).unwrap();
    assert_eq!(value, (666,));

    let value = sparse_set.swap_remove(key2).unwrap();
    assert_eq!(value, (69,));

    let value = (0,);
    let key = sparse_set
        .push(value)
        .expect("key-to-usize conversions should not fail");
    assert_eq!(sparse_set.get(key), Some((&value.0,)));
    assert!(sparse_set.contains_key(key));

    let value = (1,);
    let key = sparse_set
        .push(value)
        .expect("key-to-usize conversions should not fail");
    assert_eq!(sparse_set.get(key), Some((&value.0,)));
    assert!(sparse_set.contains_key(key));

    let value = (10,);
    let key = sparse_set
        .push(value)
        .expect("key-to-usize conversions should not fail");
    assert_eq!(sparse_set.get(key), Some((&value.0,)));
    assert!(sparse_set.contains_key(key));
}

#[test]
fn five_items_retain() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(8, (34,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(4, (69,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(3, (228,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(6, (666,))
        .expect("key-to-usize conversions should not fail");

    sparse_set.retain(|key, _| key % 2 == 0);
    assert_eq!(sparse_set.len(), 3);
    assert_eq!(sparse_set.keys().as_slice(), &[8, 4, 6]);
    assert_eq!(sparse_set.values().as_slice(), ([34, 69, 666].as_slice(),));

    assert_eq!(sparse_set.get(8), Some((&34,)));
    assert_eq!(sparse_set.get(1), None);
    assert_eq!(sparse_set.get(4), Some((&69,)));
    assert_eq!(sparse_set.get(3), None);
    assert_eq!(sparse_set.get(6), Some((&666,)));

    sparse_set.retain(|_, (value,)| *value % 2 == 1);
    assert_eq!(sparse_set.len(), 1);
    assert_eq!(sparse_set.keys().as_slice(), &[4]);
    assert_eq!(sparse_set.values().as_slice(), ([69].as_slice(),));

    assert_eq!(sparse_set.get(8), None);
    assert_eq!(sparse_set.get(1), None);
    assert_eq!(sparse_set.get(4), Some((&69,)));
    assert_eq!(sparse_set.get(3), None);
    assert_eq!(sparse_set.get(6), None);
}

#[test]
fn five_items_drain() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(8, (34,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(4, (69,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(3, (228,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(6, (666,))
        .expect("key-to-usize conversions should not fail");

    let drain = sparse_set.drain();
    assert_eq!(drain.as_keys_slice(), &[8, 1, 4, 3, 6]);
    assert_eq!(
        drain.as_values_slice(),
        ([34, 42, 69, 228, 666].as_slice(),),
    );

    forget(drain);
    assert_eq!(sparse_set.len(), 0);
    assert_ne!(sparse_set.sparse_len(), 0);
    assert_eq!(sparse_set.keys().as_slice(), &[]);
    assert_eq!(sparse_set.values().as_slice(), ([].as_slice(),));
}

#[test]
fn five_items_insert_truncate() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(8, (34,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(4, (69,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(3, (228,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(6, (666,))
        .expect("key-to-usize conversions should not fail");

    sparse_set.truncate(usize::MAX, 5);
    assert_eq!(sparse_set.sparse_len(), 5);
    assert_eq!(sparse_set.keys().as_slice(), &[1, 4, 3]);
    assert_eq!(sparse_set.values().as_slice(), ([42, 69, 228].as_slice(),));

    assert_eq!(sparse_set.get(1), Some((&42,)));
    assert_eq!(sparse_set.get(4), Some((&69,)));
    assert_eq!(sparse_set.get(3), Some((&228,)));

    sparse_set.truncate(1, usize::MAX);
    assert_eq!(sparse_set.len(), 1);
    assert_eq!(sparse_set.keys().as_slice(), &[1]);
    assert_eq!(sparse_set.values().as_slice(), ([42].as_slice(),));

    assert_eq!(sparse_set.get(1), Some((&42,)));
}

#[test]
fn five_items_push_truncate() {
    let mut sparse_set = SparseSet::new();
    let key0 = sparse_set
        .push((34,))
        .expect("key-to-usize conversions should not fail");
    let key1 = sparse_set
        .push((42,))
        .expect("key-to-usize conversions should not fail");
    let key2 = sparse_set
        .push((69,))
        .expect("key-to-usize conversions should not fail");
    let key3 = sparse_set
        .push((228,))
        .expect("key-to-usize conversions should not fail");
    let key4 = sparse_set
        .push((666,))
        .expect("key-to-usize conversions should not fail");

    sparse_set.truncate(usize::MAX, 3);
    assert_eq!(sparse_set.sparse_len(), 3);
    assert_eq!(sparse_set.as_keys_slice(), &[key0, key1, key2]);
    assert_eq!(sparse_set.as_slices(), ([34, 42, 69].as_slice(),));

    assert_eq!(sparse_set.get(key0), Some((&34,)));
    assert_eq!(sparse_set.get(key1), Some((&42,)));
    assert_eq!(sparse_set.get(key2), Some((&69,)));
    assert_eq!(sparse_set.get(key3), None);
    assert_eq!(sparse_set.get(key4), None);

    sparse_set.truncate(1, usize::MAX);
    assert_eq!(sparse_set.len(), 1);
    assert_eq!(sparse_set.as_keys_slice(), &[key0]);
    assert_eq!(sparse_set.as_slices(), ([34].as_slice(),));

    assert_eq!(sparse_set.get(key0), Some((&34,)));
}

#[test]
fn five_items_sort() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(8, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (228,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(4, (69,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(3, (666,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(6, (34,))
        .expect("key-to-usize conversions should not fail");

    sparse_set.sort();
    assert_eq!(sparse_set.keys().as_slice(), &[6, 8, 4, 1, 3]);
    assert_eq!(
        sparse_set.values().as_slice(),
        ([34, 42, 69, 228, 666].as_slice(),),
    );

    assert_eq!(sparse_set.get(8), Some((&42,)));
    assert_eq!(sparse_set.get(1), Some((&228,)));
    assert_eq!(sparse_set.get(4), Some((&69,)));
    assert_eq!(sparse_set.get(3), Some((&666,)));
    assert_eq!(sparse_set.get(6), Some((&34,)));
}

#[test]
fn five_items_sort_keys() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(8, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (228,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(4, (69,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(3, (666,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(6, (34,))
        .expect("key-to-usize conversions should not fail");

    sparse_set.sort_keys();
    assert_eq!(sparse_set.keys().as_slice(), &[1, 3, 4, 6, 8]);
    assert_eq!(
        sparse_set.values().as_slice(),
        ([228, 666, 69, 34, 42].as_slice(),),
    );

    assert_eq!(sparse_set.get(8), Some((&42,)));
    assert_eq!(sparse_set.get(1), Some((&228,)));
    assert_eq!(sparse_set.get(4), Some((&69,)));
    assert_eq!(sparse_set.get(3), Some((&666,)));
    assert_eq!(sparse_set.get(6), Some((&34,)));
}

#[test]
fn five_items_sort_by() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(8, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (228,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(4, (69,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(3, (666,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(6, (34,))
        .expect("key-to-usize conversions should not fail");

    sparse_set.sort_by(|(_, (a,)), (_, (b,))| Ord::cmp(b, a));
    assert_eq!(sparse_set.keys().as_slice(), &[3, 1, 4, 8, 6]);
    assert_eq!(
        sparse_set.values().as_slice(),
        ([666, 228, 69, 42, 34].as_slice(),),
    );

    assert_eq!(sparse_set.get(8), Some((&42,)));
    assert_eq!(sparse_set.get(1), Some((&228,)));
    assert_eq!(sparse_set.get(4), Some((&69,)));
    assert_eq!(sparse_set.get(3), Some((&666,)));
    assert_eq!(sparse_set.get(6), Some((&34,)));
}

#[test]
fn five_items_entry() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(8, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (228,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(4, (69,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(3, (666,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(6, (34,))
        .expect("key-to-usize conversions should not fail");

    let entry = sparse_set
        .entry(0)
        .expect("key-to-usize conversions should not fail");
    assert_eq!(entry.key(), 0);
    assert_eq!(entry.get(), None);

    let entry = entry.and_modify(|(value,)| *value += 1);
    assert_eq!(entry.key(), 0);
    assert_eq!(entry.get(), None);

    let entry = entry
        .replace_key(1)
        .expect("key-to-usize conversions should not fail");
    assert_eq!(entry.key(), 1);
    assert_eq!(entry.get(), Some((&228,)));

    let value = entry.and_modify(|(value,)| *value += 1).or_insert((47,));
    assert_eq!(value, (&mut 229,));
}

#[test]
fn from_keys_values_iter() {
    let keys = [3, 10, 5, 10, 1, usize::MAX];
    let values = [(34,), (42,), (69,), (228,), (666,)];

    let sparse_set: SparseSet<(_,)> = keys.into_iter().zip(values).collect();
    assert_eq!(sparse_set.len(), 4);
    assert_eq!(sparse_set.keys().as_slice(), &[3, 10, 5, 1]);
    assert_eq!(
        sparse_set.values().as_slice(),
        ([34, 228, 69, 666].as_slice(),)
    );

    assert_eq!(sparse_set.get(3), Some((&34,)));
    assert_eq!(sparse_set.get(10), Some((&228,)));
    assert_eq!(sparse_set.get(5), Some((&69,)));
    assert_eq!(sparse_set.get(1), Some((&666,)));
}

#[test]
#[should_panic(expected = "capacity overflow")]
fn from_keys_values_iter_too_large_key() {
    let keys = [3, 10, 5, 10, 1, usize::MAX];
    let values = [(34,), (42,), (69,), (228,), (666,), (999,)];

    let sparse_set: SparseSet<(_,)> = keys.into_iter().zip(values).collect();
    assert_eq!(sparse_set.len(), 4);
    assert_eq!(sparse_set.keys().as_slice(), &[3, 10, 5, 1, usize::MAX]);
    assert_eq!(
        sparse_set.values().as_slice(),
        ([34, 228, 69, 666, 999].as_slice(),),
    );

    assert_eq!(sparse_set.get(3), Some((&34,)));
    assert_eq!(sparse_set.get(10), Some((&228,)));
    assert_eq!(sparse_set.get(5), Some((&69,)));
    assert_eq!(sparse_set.get(1), Some((&666,)));
    assert_eq!(sparse_set.get(usize::MAX), Some((&999,)));
}

#[test]
fn from_values_iter() {
    let values = [(34,), (42,), (69,), (228,), (666,)];
    let sparse_set: SparseSet<_> = values.into_iter().collect();

    assert_eq!(sparse_set.len(), 5);
    assert_eq!(sparse_set.keys().as_slice(), &[0, 1, 2, 3, 4]);
    assert_eq!(
        sparse_set.values().as_slice(),
        ([34, 42, 69, 228, 666].as_slice(),),
    );

    assert_eq!(sparse_set.get(0), Some((&34,)));
    assert_eq!(sparse_set.get(1), Some((&42,)));
    assert_eq!(sparse_set.get(2), Some((&69,)));
    assert_eq!(sparse_set.get(3), Some((&228,)));
    assert_eq!(sparse_set.get(4), Some((&666,)));
}

#[test]
fn extend_keys_values() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(2, (34,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(5, (69,))
        .expect("key-to-usize conversions should not fail");

    let keys = [3, 0, 2, 8];
    let values = [(228,), (666,), (42,), (69,)];
    sparse_set.extend(keys.into_iter().zip(values));

    assert_eq!(sparse_set.keys().as_slice(), &[2, 1, 5, 3, 0, 8]);
    assert_eq!(
        sparse_set.values().as_slice(),
        ([42, 42, 69, 228, 666, 69].as_slice(),),
    );

    assert_eq!(sparse_set.get(2), Some((&42,)));
    assert_eq!(sparse_set.get(1), Some((&42,)));
    assert_eq!(sparse_set.get(5), Some((&69,)));
    assert_eq!(sparse_set.get(3), Some((&228,)));
    assert_eq!(sparse_set.get(0), Some((&666,)));
    assert_eq!(sparse_set.get(8), Some((&69,)));
}

#[test]
fn extend_values() {
    let mut sparse_set = SparseSet::new();
    sparse_set
        .insert(2, (34,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(1, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_set
        .insert(4, (69,))
        .expect("key-to-usize conversions should not fail");

    let values = [(228,), (666,), (201,)];
    sparse_set.extend(values);

    assert_eq!(sparse_set.keys().as_slice(), &[2, 1, 4, 0, 3, 5]);
    assert_eq!(
        sparse_set.values().as_slice(),
        ([34, 42, 69, 228, 666, 201].as_slice(),),
    );

    assert_eq!(sparse_set.get(2), Some((&34,)));
    assert_eq!(sparse_set.get(1), Some((&42,)));
    assert_eq!(sparse_set.get(4), Some((&69,)));
    assert_eq!(sparse_set.get(0), Some((&228,)));
    assert_eq!(sparse_set.get(3), Some((&666,)));
    assert_eq!(sparse_set.get(5), Some((&201,)));
}

#[test]
fn from_arena() {
    let mut sparse_arena = SparseArena::new();
    sparse_arena
        .insert(2, (34,))
        .expect("key-to-usize conversions should not fail");
    sparse_arena
        .insert(1, (42,))
        .expect("key-to-usize conversions should not fail");
    sparse_arena
        .insert(5, (69,))
        .expect("key-to-usize conversions should not fail");

    let sparse_set = SparseSet::from(sparse_arena);
    assert_eq!(sparse_set.len(), 3);
    assert_eq!(sparse_set.keys().as_slice(), &[2, 1, 5]);
    assert_eq!(sparse_set.values().as_slice(), ([34, 42, 69].as_slice(),));

    assert_eq!(sparse_set.get(2), Some((&34,)));
    assert_eq!(sparse_set.get(1), Some((&42,)));
    assert_eq!(sparse_set.get(5), Some((&69,)));
}
