use core::{borrow::Borrow, fmt::Debug};

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
    item::{DefaultSparseItem, KeyValuePair},
    key::Key,
    soa::{slice::SoaSlices, traits::RawSoa},
};

#[inline]
unsafe fn sparse_item_unchecked<K>(
    sparse: *const [DefaultSparseItem<K>],
    sparse_index: K::SparseIndex,
) -> *const DefaultSparseItem<K>
where
    K: Key,
{
    let sparse_index = unsafe { sparse_index.try_into().unwrap_unchecked() };
    unsafe { sparse.cast::<DefaultSparseItem<K>>().add(sparse_index) }
}

#[inline]
fn sparse_item<K>(
    sparse: &[DefaultSparseItem<K>],
    sparse_index: usize,
) -> Option<&DefaultSparseItem<K>>
where
    K: Key,
{
    sparse.get(sparse_index)
}

#[inline]
fn sparse_item_mut<K>(
    sparse: &mut [DefaultSparseItem<K>],
    sparse_index: usize,
) -> Option<&mut DefaultSparseItem<K>>
where
    K: Key,
{
    sparse.get_mut(sparse_index)
}

#[inline]
fn filter_sparse_item<K>(sparse_item: &DefaultSparseItem<K>, epoch: K::Epoch) -> bool
where
    K: Key,
{
    sparse_item.epoch == epoch
}

#[inline]
pub fn sparse_item_by_epoch<K>(
    sparse: &[DefaultSparseItem<K>],
    sparse_index: usize,
    epoch: K::Epoch,
) -> Option<&DefaultSparseItem<K>>
where
    K: Key,
{
    sparse_item(sparse, sparse_index).filter(|item| filter_sparse_item(item, epoch))
}

#[inline]
pub fn sparse_item_mut_by_epoch<K>(
    sparse: &mut [DefaultSparseItem<K>],
    sparse_index: usize,
    epoch: K::Epoch,
) -> Option<&mut DefaultSparseItem<K>>
where
    K: Key,
{
    sparse_item_mut(sparse, sparse_index).filter(|item| filter_sparse_item(item, epoch))
}

#[inline]
pub fn sparse_item_by_index<K>(
    sparse: &[DefaultSparseItem<K>],
    sparse_index: K::SparseIndex,
) -> Option<&DefaultSparseItem<K>>
where
    K: Key,
{
    let sparse_index = sparse_index.try_into().ok()?;
    sparse_item(sparse, sparse_index)
}

#[inline]
pub fn sparse_item_mut_by_index<K>(
    sparse: &mut [DefaultSparseItem<K>],
    sparse_index: K::SparseIndex,
) -> Option<&mut DefaultSparseItem<K>>
where
    K: Key,
{
    let sparse_index = sparse_index.try_into().ok()?;
    sparse_item_mut(sparse, sparse_index)
}

#[inline]
pub fn sparse_item_by_key<K>(
    sparse: &[DefaultSparseItem<K>],
    key: K,
) -> Option<&DefaultSparseItem<K>>
where
    K: Key,
{
    let sparse_index = key.sparse_index().try_into().ok()?;
    sparse_item_by_epoch(sparse, sparse_index, key.epoch())
}

#[inline]
pub fn sparse_item_mut_by_key<K>(
    sparse: &mut [DefaultSparseItem<K>],
    key: K,
) -> Option<&mut DefaultSparseItem<K>>
where
    K: Key,
{
    let sparse_index = key.sparse_index().try_into().ok()?;
    sparse_item_mut_by_epoch(sparse, sparse_index, key.epoch())
}

pub unsafe fn sparse_get_unchecked<K, T>(
    dense: impl IntoIterator<Item = T>,
    sparse: *const [DefaultSparseItem<K>],
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

pub fn sparse_get<K, T>(
    dense: impl IntoIterator<Item = (impl Borrow<K>, T)>,
    sparse: &[DefaultSparseItem<K>],
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
    sparse: &[DefaultSparseItem<K>],
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

pub fn sparse_get_with_key<K, T>(
    dense: impl IntoIterator<Item = (impl Borrow<K>, T)>,
    sparse: &[DefaultSparseItem<K>],
    sparse_index: K::SparseIndex,
) -> Option<(K, T)>
where
    K: Key,
{
    let sparse_item = sparse_item_by_index(sparse, sparse_index)?;
    let dense_index = unwrap_into_usize(sparse_item.into_dense_index()?);

    let (dense_key, value) = unwrap_dense(dense, dense_index);
    let &dense_key = dense_key.borrow();
    assert_compatible_key(K::new(sparse_index, sparse_item.epoch), dense_key);

    Some((dense_key, value))
}

pub fn sparse_get_epoch<K>(
    dense_keys: &[K],
    sparse: &[DefaultSparseItem<K>],
    sparse_index: K::SparseIndex,
) -> Option<K::Epoch>
where
    K: Key,
{
    let sparse_item = sparse_item_by_index(sparse, sparse_index)?;
    let epoch = sparse_item.epoch;

    if let Some(dense_index) = sparse_item.dense_index() {
        let dense_index = unwrap_into_usize(*dense_index);
        let &dense_key = unwrap_dense(dense_keys, dense_index);
        assert_compatible_key(K::new(sparse_index, epoch), dense_key);
    }

    Some(epoch)
}

pub fn sparse_contains_key<K>(dense_keys: &[K], sparse: &[DefaultSparseItem<K>], key: K) -> bool
where
    K: Key,
{
    let Some(sparse_item) = sparse_item_by_key(sparse, key) else {
        return false;
    };
    let Some(dense_index) = sparse_item.into_dense_index() else {
        return false;
    };
    let dense_index = unwrap_into_usize(dense_index);

    assert_dense_index_bounds(dense_index, dense_keys.len());
    true
}

#[inline]
pub fn dense_keys<'a, K, V>(dense: SoaSlices<'_, 'a, KeyValuePair<K, V>>) -> &'a [K]
where
    V: RawSoa + ?Sized,
{
    let (_, keys) = dense_keys_with_context(dense);
    keys
}

#[inline]
pub fn dense_keys_with_context<'ctx, 'a, K, V>(
    dense: SoaSlices<'ctx, 'a, KeyValuePair<K, V>>,
) -> (&'ctx V::Context, &'a [K])
where
    V: RawSoa + ?Sized,
{
    let (context, slices) = dense.into_slice_ptrs_with_context();

    let (keys, _) = slices.into_parts();
    let keys = unsafe { keys.as_ref_unchecked() };

    (context, keys)
}

pub fn check_parts<'a, K, V>(
    dense: SoaSlices<'_, 'a, KeyValuePair<K, V>>,
    sparse: &[DefaultSparseItem<K>],
) -> Result<(), FromPartsError<K>>
where
    K: Key + 'a,
    V: RawSoa + ?Sized,
{
    let dense = dense_keys(dense);

    for (sparse_index, sparse_item) in sparse.iter().enumerate() {
        let sparse_index = sparse_index
            .try_into()
            .map_err(TooSmallSparseIndexError::new)?;
        let Some(dense_index) = sparse_item.dense_index().copied() else {
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
        let expected_epoch = sparse_item.epoch;
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
        let Some(dense_index_from_item) = sparse_item.into_dense_index() else {
            let error = OccupiedSparseItemExpectedError::new(sparse_index);
            return Err(error.into());
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
