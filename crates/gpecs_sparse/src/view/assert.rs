use crate::{
    error::{
        DenseIndexMismatchError, DenseIndexOutOfBoundsError, EpochMismatchError, FromPartsError,
        OccupiedSparseItemExpectedError, SparseIndexMismatchError, SparseIndexOutOfBoundsError,
        TooLargeSparseIndexError, TooSmallSparseIndexError,
    },
    item::{SparseItem, SparseItemKind},
    key::Key,
    pair::{KeyValuePair, KeyValueRefs},
    soa::{slice::SoaSlices, traits::Soa},
};

pub fn check_parts<K, V>(
    dense: &SoaSlices<KeyValuePair<K, V>>,
    sparse: &[SparseItem<K>],
) -> Result<(), FromPartsError<K>>
where
    K: Key,
    V: Soa + ?Sized,
{
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
        let KeyValueRefs { key, .. } = dense
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

    for (dense_index, KeyValueRefs { key, .. }) in dense.iter().enumerate() {
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
