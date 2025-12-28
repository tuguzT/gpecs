use core::{borrow::Borrow, fmt::Debug, ops, slice};

use crate::{
    assert::{
        assert_compatible_key, assert_dense_index_bounds, assert_equal_key, unwrap_dense,
        unwrap_into_usize,
    },
    error::{
        DenseIndexMismatchError, DenseIndexOutOfBoundsError, EpochMismatchError, FromPartsError,
        OccupiedSparseItemExpectedError, SparseIndexMismatchError, SparseIndexOutOfBoundsError,
        TooLargeSparseIndexError, TooSmallSparseIndexError,
    },
    item::{DenseItem, SparseItem, SparseItemKind},
    key::Key,
    soa::{slice::SoaSlices, traits::RawSoa},
};

// https://stackoverflow.com/a/73428605/14928295
fn get_range<T>(
    iterator: impl IntoIterator<Item = T>,
    range: impl ops::RangeBounds<usize>,
) -> impl Iterator<Item = T> {
    let start_bound = match range.start_bound() {
        ops::Bound::Included(&num) => num,
        ops::Bound::Excluded(&num) => num + 1,
        ops::Bound::Unbounded => 0,
    };

    let mut end_bound = match range.end_bound() {
        ops::Bound::Included(&num) => Some(num + 1),
        ops::Bound::Excluded(&num) => Some(num),
        ops::Bound::Unbounded => None,
    };

    iterator
        .into_iter()
        .take_while(move |_| {
            if let Some(num) = &mut end_bound {
                if *num == 0 {
                    false
                } else {
                    *num -= 1;
                    true
                }
            } else {
                true
            }
        })
        .skip(start_bound)
}

pub fn get_pair<T>(iter: impl IntoIterator<Item = T>, a: usize, b: usize) -> Option<(T, T)> {
    let (first, second) = (usize::min(a, b), usize::max(a, b));

    let mut iter = get_range(iter, first..=second);
    let first = iter.next()?;
    let second = iter.last()?;

    let pair = if a < b {
        (first, second)
    } else {
        (second, first)
    };
    Some(pair)
}

#[inline]
unsafe fn sparse_item_unchecked<K>(
    sparse: *const [SparseItem<K>],
    sparse_index: K::SparseIndex,
) -> *const SparseItem<K>
where
    K: Key,
{
    let sparse_index = unsafe { sparse_index.try_into().unwrap_unchecked() };
    unsafe { sparse.cast::<SparseItem<K>>().add(sparse_index) }
}

pub unsafe fn sparse_get_unchecked<K, T>(
    dense: impl IntoIterator<Item = T>,
    sparse: *const [SparseItem<K>],
    sparse_index: K::SparseIndex,
) -> T
where
    K: Key,
{
    let sparse_item = unsafe { sparse_item_unchecked(sparse, sparse_index).read() };
    let dense_index = unsafe { sparse_item.into_dense_index().unwrap_unchecked() };
    let dense_index = unsafe { dense_index.try_into().unwrap_unchecked() };
    unsafe { dense.into_iter().nth(dense_index).unwrap_unchecked() }
}

#[inline]
fn sparse_item_by_key<K>(sparse: &[SparseItem<K>], key: K) -> Option<&SparseItem<K>>
where
    K: Key,
{
    let sparse_index = key.sparse_index().try_into().ok()?;
    let sparse_item = sparse
        .get(sparse_index)
        .take_if(|item| item.epoch == key.epoch())?;
    Some(sparse_item)
}

pub fn sparse_get<K, T>(
    dense: impl IntoIterator<Item = (impl Borrow<K>, T)>,
    sparse: &[SparseItem<K>],
    key: K,
) -> Option<T>
where
    K: Key,
{
    let sparse_item = sparse_item_by_key(sparse, key)?;
    let dense_index = unwrap_into_usize(sparse_item.into_dense_index()?);

    let (dense_key, value) = unwrap_dense(dense, dense_index);
    let &dense_key = dense_key.borrow();
    assert_equal_key(key, dense_key);

    Some(value)
}

#[cold]
#[track_caller]
#[inline(never)]
fn sparse_index_failed<K>(key: &K) -> !
where
    K: Debug,
{
    panic!("key {key:?} not found")
}

#[inline]
#[track_caller]
pub fn sparse_index<K, T>(
    dense: impl IntoIterator<Item = (impl Borrow<K>, T)>,
    sparse: &[SparseItem<K>],
    key: K,
) -> T
where
    K: Key + Debug,
{
    match sparse_get(dense, sparse, key) {
        Some(value) => value,
        None => sparse_index_failed(&key),
    }
}

#[inline]
fn sparse_item<K>(sparse: &[SparseItem<K>], sparse_index: K::SparseIndex) -> Option<&SparseItem<K>>
where
    K: Key,
{
    let sparse_index = sparse_index.try_into().ok()?;
    let sparse_item = sparse.get(sparse_index)?;
    Some(sparse_item)
}

pub fn sparse_get_with_key<K, T>(
    dense: impl IntoIterator<Item = (impl Borrow<K>, T)>,
    sparse: &[SparseItem<K>],
    sparse_index: K::SparseIndex,
) -> Option<(K, T)>
where
    K: Key,
{
    let sparse_item = sparse_item(sparse, sparse_index)?;
    let dense_index = unwrap_into_usize(sparse_item.into_dense_index()?);

    let (dense_key, value) = unwrap_dense(dense, dense_index);
    let &dense_key = dense_key.borrow();
    assert_compatible_key(K::new(sparse_index, sparse_item.epoch), dense_key);

    Some((dense_key, value))
}

pub fn sparse_get_epoch<K>(
    dense_keys: &[K],
    sparse: &[SparseItem<K>],
    sparse_index: K::SparseIndex,
) -> Option<K::Epoch>
where
    K: Key,
{
    let sparse_item = sparse_item(sparse, sparse_index)?;
    let epoch = sparse_item.epoch;

    if let Some(dense_index) = sparse_item.dense_index() {
        let dense_index = unwrap_into_usize(*dense_index);
        let &dense_key = unwrap_dense(dense_keys, dense_index);
        assert_compatible_key(K::new(sparse_index, epoch), dense_key);
    }

    Some(epoch)
}

pub fn sparse_contains_key<K>(dense_keys: &[K], sparse: &[SparseItem<K>], key: K) -> bool
where
    K: Key,
{
    let Some(sparse_item) = sparse_item_by_key(sparse, key) else {
        return false;
    };
    let SparseItemKind::Occupied { dense_index } = sparse_item.kind else {
        return false;
    };
    let dense_index = unwrap_into_usize(dense_index);

    assert_dense_index_bounds(dense_index, dense_keys.len());
    true
}

#[inline]
pub fn dense_keys<'a, K, V>(dense: SoaSlices<'_, 'a, DenseItem<K, V>>) -> &'a [K]
where
    V: RawSoa + ?Sized,
{
    let (_, keys) = dense_keys_with_context(dense);
    keys
}

#[inline]
pub fn dense_keys_with_context<'ctx, 'a, K, V>(
    dense: SoaSlices<'ctx, 'a, DenseItem<K, V>>,
) -> (&'ctx V::Context, &'a [K])
where
    V: RawSoa + ?Sized,
{
    let (context, slices) = dense.into_slice_ptrs_with_context();

    let (keys, _) = slices.into_parts();
    let keys = unsafe { slice::from_raw_parts(keys.cast(), keys.len()) };

    (context, keys)
}

pub fn check_parts<K, V>(
    dense: SoaSlices<DenseItem<K, V>>,
    sparse: &[SparseItem<K>],
) -> Result<(), FromPartsError<K>>
where
    K: Key,
    V: RawSoa + ?Sized,
{
    let dense = dense_keys(dense);

    for (sparse_index, &SparseItem { kind, epoch }) in sparse.iter().enumerate() {
        let sparse_index = sparse_index
            .try_into()
            .map_err(TooSmallSparseIndexError::new)?;
        let Some(dense_index) = kind.dense_index().copied() else {
            continue;
        };

        let dense_index = dense_index
            .try_into()
            .map_err(TooLargeSparseIndexError::new)?;
        let key = dense
            .get(dense_index)
            .ok_or_else(|| DenseIndexOutOfBoundsError::new(dense_index, dense.len()))?;

        let sparse_index_from_key = key.sparse_index();
        if sparse_index_from_key != sparse_index {
            let error = SparseIndexMismatchError::new(sparse_index_from_key, sparse_index);
            return Err(error.into());
        }

        let epoch_from_key = key.epoch();
        let expected_epoch = epoch;
        if epoch_from_key != expected_epoch {
            let error = EpochMismatchError::new(epoch_from_key, expected_epoch);
            return Err(error.into());
        }
    }

    for (dense_index, key) in dense.iter().enumerate() {
        let sparse_index = key
            .sparse_index()
            .try_into()
            .map_err(TooLargeSparseIndexError::new)?;
        let sparse_item = sparse
            .get(sparse_index)
            .ok_or_else(|| SparseIndexOutOfBoundsError::new(sparse_index, sparse.len()))?;

        let dense_index = dense_index
            .try_into()
            .map_err(TooSmallSparseIndexError::new)?;
        let dense_index_from_item = match *sparse_item.kind() {
            SparseItemKind::Occupied { dense_index } => dense_index,
            SparseItemKind::Vacant { next_vacant } => {
                let error = OccupiedSparseItemExpectedError::new(next_vacant);
                return Err(error.into());
            }
        };
        if dense_index_from_item != dense_index {
            let error = DenseIndexMismatchError::new(dense_index_from_item, dense_index);
            return Err(error.into());
        }

        let epoch_from_item = sparse_item.epoch;
        let expected_epoch = key.epoch();
        if epoch_from_item != expected_epoch {
            let error = EpochMismatchError::new(epoch_from_item, expected_epoch);
            return Err(error.into());
        }
    }

    Ok(())
}
