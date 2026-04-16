use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr,
};

use crate::{
    algo::sparse_get_unchecked,
    item::{DenseItem, DensePtrs, DenseSlicePtrs, SparseItem},
    iter::{RawIter, RawKeys, RawValues},
    key::Key,
    soa::{
        identity::Identity,
        slice::SoaSlicePtrs,
        traits::{Ptrs, RawSoa, SlicePtrs},
    },
    view::{EpochSparseView, EpochSparseViewMutPtr},
};

pub struct EpochSparseViewPtr<'ctx, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
    V::Context: 'ctx,
{
    dense: SoaSlicePtrs<'ctx, DenseItem<K, V>>,
    sparse: *const [SparseItem<K>],
}

impl<'ctx, K, V> EpochSparseViewPtr<'ctx, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
{
    #[inline]
    pub unsafe fn from_parts(
        dense: SoaSlicePtrs<'ctx, DenseItem<K, V>>,
        sparse: *const [SparseItem<K>],
    ) -> Self {
        Self { dense, sparse }
    }

    #[inline]
    pub fn into_parts(self) -> (SoaSlicePtrs<'ctx, DenseItem<K, V>>, *const [SparseItem<K>]) {
        let Self { dense, sparse } = self;
        (dense, sparse)
    }

    #[inline]
    pub fn cast_mut(self) -> EpochSparseViewMutPtr<'ctx, K, V> {
        let Self { dense, sparse } = self;

        let dense = dense.cast_mut();
        let sparse = sparse.cast_mut();
        unsafe { EpochSparseViewMutPtr::from_parts(dense, sparse) }
    }

    #[inline]
    pub unsafe fn as_ref_unchecked<'a>(self) -> EpochSparseView<'ctx, 'a, K, V> {
        let Self { dense, sparse } = self;

        let dense = unsafe { dense.as_ref_unchecked() };
        let sparse = unsafe { sparse.as_ref_unchecked() };
        unsafe { EpochSparseView::from_parts(dense, sparse) }
    }

    #[inline]
    pub fn context(&self) -> &V::Context {
        let Self { dense, .. } = self;
        dense.context()
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { dense, .. } = self;
        dense.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn sparse_len(&self) -> usize {
        let Self { sparse, .. } = self;
        sparse.len()
    }

    #[inline]
    pub fn sparse_is_empty(&self) -> bool {
        self.sparse_len() == 0
    }

    #[inline]
    pub fn as_ptrs(&self) -> (DensePtrs<'_, K, V>, *const SparseItem<K>) {
        let (_, dense, sparse) = self.as_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&V::Context, DensePtrs<'_, K, V>, *const SparseItem<K>) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.as_ptrs_with_context();
        let sparse = sparse.cast();
        (context, dense, sparse)
    }

    #[inline]
    pub fn as_key_ptr(&self) -> *const K {
        let (_, key) = self.as_key_ptr_with_context();
        key
    }

    #[inline]
    pub fn as_key_ptr_with_context(&self) -> (&V::Context, *const K) {
        let (context, dense) = self.as_dense_ptrs_with_context();
        let (key, _) = dense.into_parts();
        (context, key)
    }

    #[inline]
    pub fn as_value_ptrs(&self) -> Ptrs<'_, V> {
        let (_, value) = self.as_value_ptrs_with_context();
        value
    }

    #[inline]
    pub fn as_value_ptrs_with_context(&self) -> (&V::Context, Ptrs<'_, V>) {
        let (context, dense) = self.as_dense_ptrs_with_context();
        let (_, value) = dense.into_parts();
        (context, value)
    }

    #[inline]
    pub fn as_dense_ptrs(&self) -> DensePtrs<'_, K, V> {
        let (_, dense) = self.as_dense_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn as_dense_ptrs_with_context(&self) -> (&V::Context, DensePtrs<'_, K, V>) {
        let (context, dense, _) = self.as_ptrs_with_context();
        (context, dense)
    }

    #[inline]
    pub fn as_sparse_ptr(&self) -> *const SparseItem<K> {
        let (_, sparse) = self.as_sparse_ptr_with_context();
        sparse
    }

    #[inline]
    pub fn as_sparse_ptr_with_context(&self) -> (&V::Context, *const SparseItem<K>) {
        let (context, _, sparse) = self.as_ptrs_with_context();
        (context, sparse)
    }

    #[inline]
    pub fn into_ptrs(self) -> (DensePtrs<'ctx, K, V>, *const SparseItem<K>) {
        let (_, dense, sparse) = self.into_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn into_ptrs_with_context(
        self,
    ) -> (
        &'ctx V::Context,
        DensePtrs<'ctx, K, V>,
        *const SparseItem<K>,
    ) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_ptrs_with_context();
        let sparse = sparse.cast();
        (context, dense, sparse)
    }

    #[inline]
    pub fn into_key_ptr(self) -> *const K {
        let (_, key) = self.into_key_ptr_with_context();
        key
    }

    #[inline]
    pub fn into_key_ptr_with_context(self) -> (&'ctx V::Context, *const K) {
        let (context, dense) = self.into_dense_ptrs_with_context();
        let (key, _) = dense.into_parts();
        (context, key)
    }

    #[inline]
    pub fn into_value_ptrs(self) -> Ptrs<'ctx, V> {
        let (_, value) = self.into_value_ptrs_with_context();
        value
    }

    #[inline]
    pub fn into_value_ptrs_with_context(self) -> (&'ctx V::Context, Ptrs<'ctx, V>) {
        let (context, dense) = self.into_dense_ptrs_with_context();
        let (_, value) = dense.into_parts();
        (context, value)
    }

    #[inline]
    pub fn into_dense_ptrs(self) -> DensePtrs<'ctx, K, V> {
        let (_, dense) = self.into_dense_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn into_dense_ptrs_with_context(self) -> (&'ctx V::Context, DensePtrs<'ctx, K, V>) {
        let (context, dense, _) = self.into_ptrs_with_context();
        (context, dense)
    }

    #[inline]
    pub fn into_sparse_ptr(self) -> *const SparseItem<K> {
        let (_, sparse) = self.into_sparse_ptr_with_context();
        sparse
    }

    #[inline]
    pub fn into_sparse_ptr_with_context(self) -> (&'ctx V::Context, *const SparseItem<K>) {
        let (context, _, sparse) = self.into_ptrs_with_context();
        (context, sparse)
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> (DenseSlicePtrs<'_, K, V>, *const [SparseItem<K>]) {
        let (_, dense, sparse) = self.as_slice_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(
        &self,
    ) -> (
        &V::Context,
        DenseSlicePtrs<'_, K, V>,
        *const [SparseItem<K>],
    ) {
        let Self { ref dense, sparse } = *self;
        let (context, dense) = dense.as_slice_ptrs_with_context();
        (context, dense, sparse)
    }

    #[inline]
    pub fn as_key_slice_ptr(&self) -> *const [K] {
        let (_, key) = self.as_key_slice_ptr_with_context();
        key
    }

    #[inline]
    pub fn as_key_slice_ptr_with_context(&self) -> (&V::Context, *const [K]) {
        let (context, dense) = self.as_dense_slice_ptrs_with_context();
        let (key, _) = dense.into_parts();
        (context, key)
    }

    #[inline]
    pub fn as_value_slice_ptrs(&self) -> SlicePtrs<'_, V> {
        let (_, value) = self.as_value_slice_ptrs_with_context();
        value
    }

    #[inline]
    pub fn as_value_slice_ptrs_with_context(&self) -> (&V::Context, SlicePtrs<'_, V>) {
        let (context, dense) = self.as_dense_slice_ptrs_with_context();
        let (_, value) = dense.into_parts();
        (context, value)
    }

    #[inline]
    pub fn as_dense_slice_ptrs(&self) -> DenseSlicePtrs<'_, K, V> {
        let (_, dense) = self.as_dense_slice_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn as_dense_slice_ptrs_with_context(&self) -> (&V::Context, DenseSlicePtrs<'_, K, V>) {
        let (context, dense, _) = self.as_slice_ptrs_with_context();
        (context, dense)
    }

    #[inline]
    pub fn as_sparse_slice_ptr(&self) -> *const [SparseItem<K>] {
        let (_, sparse) = self.as_sparse_slice_ptr_with_context();
        sparse
    }

    #[inline]
    pub fn as_sparse_slice_ptr_with_context(&self) -> (&V::Context, *const [SparseItem<K>]) {
        let (context, _, sparse) = self.as_slice_ptrs_with_context();
        (context, sparse)
    }

    #[inline]
    pub fn into_slice_ptrs(self) -> (DenseSlicePtrs<'ctx, K, V>, *const [SparseItem<K>]) {
        let (_, dense, sparse) = self.into_slice_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn into_slice_ptrs_with_context(
        self,
    ) -> (
        &'ctx V::Context,
        DenseSlicePtrs<'ctx, K, V>,
        *const [SparseItem<K>],
    ) {
        let Self { dense, sparse } = self;
        let (context, dense) = dense.into_slice_ptrs_with_context();
        (context, dense, sparse)
    }

    #[inline]
    pub fn into_key_slice_ptr(self) -> *const [K] {
        let (_, key) = self.into_key_slice_ptr_with_context();
        key
    }

    #[inline]
    pub fn into_key_slice_ptr_with_context(self) -> (&'ctx V::Context, *const [K]) {
        let (context, dense) = self.into_dense_slice_ptrs_with_context();
        let (key, _) = dense.into_parts();
        (context, key)
    }

    #[inline]
    pub fn into_value_slice_ptrs(self) -> SlicePtrs<'ctx, V> {
        let (_, value) = self.into_value_slice_ptrs_with_context();
        value
    }

    #[inline]
    pub fn into_value_slice_ptrs_with_context(self) -> (&'ctx V::Context, SlicePtrs<'ctx, V>) {
        let (context, dense) = self.into_dense_slice_ptrs_with_context();
        let (_, value) = dense.into_parts();
        (context, value)
    }

    #[inline]
    pub fn into_dense_slice_ptrs(self) -> DenseSlicePtrs<'ctx, K, V> {
        let (_, dense) = self.into_dense_slice_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn into_dense_slice_ptrs_with_context(
        self,
    ) -> (&'ctx V::Context, DenseSlicePtrs<'ctx, K, V>) {
        let (context, dense, _) = self.into_slice_ptrs_with_context();
        (context, dense)
    }

    #[inline]
    pub fn into_sparse_slice_ptr(self) -> *const [SparseItem<K>] {
        let (_, sparse) = self.into_sparse_slice_ptr_with_context();
        sparse
    }

    #[inline]
    pub fn into_sparse_slice_ptr_with_context(self) -> (&'ctx V::Context, *const [SparseItem<K>]) {
        let (context, _, sparse) = self.into_slice_ptrs_with_context();
        (context, sparse)
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, key: K) -> Ptrs<'_, V> {
        let (_, ptrs) = unsafe { self.get_unchecked_with_context(key) };
        ptrs
    }

    #[inline]
    pub unsafe fn get_unchecked_with_context(&self, key: K) -> (&V::Context, Ptrs<'_, V>) {
        let Self { ref dense, sparse } = *self;

        let (context, dense) = dense.iter_with_context();
        let dense = dense.map(From::from);
        let (_, value) = unsafe { sparse_get_unchecked(dense, sparse, key.sparse_index()) };
        (context, value)
    }

    #[inline]
    pub unsafe fn into_get_unchecked(self, key: K) -> Ptrs<'ctx, V> {
        let (_, ptrs) = unsafe { self.into_get_unchecked_with_context(key) };
        ptrs
    }

    #[inline]
    pub unsafe fn into_get_unchecked_with_context(
        self,
        key: K,
    ) -> (&'ctx V::Context, Ptrs<'ctx, V>) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_iter_with_context();
        let dense = dense.map(From::from);
        let (_, value) = unsafe { sparse_get_unchecked(dense, sparse, key.sparse_index()) };
        (context, value)
    }

    #[inline]
    pub unsafe fn get_with_key_unchecked(
        &self,
        sparse_index: K::SparseIndex,
    ) -> (*const K, Ptrs<'_, V>) {
        let (_, key, value) = unsafe { self.get_with_key_unchecked_with_context(sparse_index) };
        (key, value)
    }

    #[inline]
    pub unsafe fn get_with_key_unchecked_with_context(
        &self,
        sparse_index: K::SparseIndex,
    ) -> (&V::Context, *const K, Ptrs<'_, V>) {
        let Self { ref dense, sparse } = *self;

        let (context, dense) = dense.iter_with_context();
        let dense = dense.map(From::from);
        let (key, value) = unsafe { sparse_get_unchecked(dense, sparse, sparse_index) };
        (context, key, value)
    }

    #[inline]
    pub unsafe fn into_get_with_key_unchecked(
        self,
        sparse_index: K::SparseIndex,
    ) -> (*const K, Ptrs<'ctx, V>) {
        let (_, key, value) =
            unsafe { self.into_get_with_key_unchecked_with_context(sparse_index) };
        (key, value)
    }

    #[inline]
    pub unsafe fn into_get_with_key_unchecked_with_context(
        self,
        sparse_index: K::SparseIndex,
    ) -> (&'ctx V::Context, *const K, Ptrs<'ctx, V>) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_iter_with_context();
        let dense = dense.map(From::from);
        let (key, value) = unsafe { sparse_get_unchecked(dense, sparse, sparse_index) };
        (context, key, value)
    }

    #[inline]
    pub fn keys(&self) -> RawKeys<'_, K, V> {
        let (_, iter) = self.keys_with_context();
        iter
    }

    #[inline]
    pub fn keys_with_context(&self) -> (&V::Context, RawKeys<'_, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.iter_with_context();
        let iter = RawKeys::from_inner(inner);
        (context, iter)
    }

    #[inline]
    pub fn into_keys(self) -> RawKeys<'ctx, K, V> {
        let (_, iter) = self.into_keys_with_context();
        iter
    }

    #[inline]
    pub fn into_keys_with_context(self) -> (&'ctx V::Context, RawKeys<'ctx, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.into_iter_with_context();
        let iter = RawKeys::from_inner(inner);
        (context, iter)
    }

    #[inline]
    pub fn values(&self) -> RawValues<'_, K, V> {
        let (_, iter) = self.values_with_context();
        iter
    }

    #[inline]
    pub fn values_with_context(&self) -> (&V::Context, RawValues<'_, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.iter_with_context();
        let iter = RawValues::from_inner(inner);
        (context, iter)
    }

    #[inline]
    pub fn into_values(self) -> RawValues<'ctx, K, V> {
        let (_, iter) = self.into_values_with_context();
        iter
    }

    #[inline]
    pub fn into_values_with_context(self) -> (&'ctx V::Context, RawValues<'ctx, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.into_iter_with_context();
        let iter = RawValues::from_inner(inner);
        (context, iter)
    }

    #[inline]
    pub fn iter(&self) -> RawIter<'_, K, V> {
        let (_, iter) = self.iter_with_context();
        iter
    }

    #[inline]
    pub fn iter_with_context(&self) -> (&V::Context, RawIter<'_, K, V>) {
        let Self { dense, .. } = self;

        let (context, iter) = dense.iter_with_context();
        let iter = RawIter::from_inner(iter);
        (context, iter)
    }

    #[inline]
    pub fn into_iter_with_context(self) -> (&'ctx V::Context, RawIter<'ctx, K, V>) {
        let Self { dense, .. } = self;

        let (context, iter) = dense.into_iter_with_context();
        let iter = RawIter::from_inner(iter);
        (context, iter)
    }
}

impl<'ctx, K, V> From<&'ctx V::Context> for EpochSparseViewPtr<'ctx, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
{
    #[inline]
    fn from(context: &'ctx V::Context) -> Self {
        let context = Identity::from_inner_ref(context);
        let dense = SoaSlicePtrs::from(context);
        let sparse = ptr::from_ref(Default::default());
        unsafe { Self::from_parts(dense, sparse) }
    }
}

impl<'ctx, K, V> Default for EpochSparseViewPtr<'ctx, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
    &'ctx V::Context: Default,
{
    fn default() -> Self {
        let context: &V::Context = Default::default();
        Self::from(context)
    }
}

impl<K, V> Debug for EpochSparseViewPtr<'_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
    for<'ctx> SlicePtrs<'ctx, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { dense, sparse } = self;
        f.debug_struct("EpochSparseViewPtr")
            .field("dense", dense)
            .field("sparse", sparse)
            .finish()
    }
}

impl<K, V> Clone for EpochSparseViewPtr<'_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { ref dense, sparse } = *self;
        let dense = dense.clone();
        Self { dense, sparse }
    }
}

impl<K, V> Copy for EpochSparseViewPtr<'_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
    for<'ctx> Ptrs<'ctx, V>: Copy,
{
}

impl<K, V> PartialEq for EpochSparseViewPtr<'_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
    V::Context: PartialEq,
    for<'ctx> Ptrs<'ctx, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { dense, sparse } = self;
        *dense == other.dense && ptr::eq(*sparse, other.sparse)
    }
}

impl<K, V> Eq for EpochSparseViewPtr<'_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
    V::Context: Eq,
    for<'ctx> Ptrs<'ctx, V>: Eq,
{
}

impl<K, V> PartialOrd for EpochSparseViewPtr<'_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
    V::Context: PartialOrd,
    for<'ctx> Ptrs<'ctx, V>: PartialOrd,
{
    #[expect(ambiguous_wide_pointer_comparisons)]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { dense, sparse } = self;

        match dense.partial_cmp(&other.dense) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        sparse.partial_cmp(&other.sparse)
    }
}

impl<K, V> Ord for EpochSparseViewPtr<'_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
    V::Context: Ord,
    for<'ctx> Ptrs<'ctx, V>: Ord,
{
    #[expect(ambiguous_wide_pointer_comparisons)]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { dense, sparse } = self;

        match dense.cmp(&other.dense) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        sparse.cmp(&other.sparse)
    }
}

impl<K, V> Hash for EpochSparseViewPtr<'_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
    V::Context: Hash,
    for<'ctx> Ptrs<'ctx, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { dense, sparse } = self;
        dense.hash(state);
        sparse.hash(state);
    }
}

impl<'a, K, V> IntoIterator for &'a EpochSparseViewPtr<'_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
{
    type Item = (*const K, Ptrs<'a, V>);
    type IntoIter = RawIter<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'ctx, K, V> IntoIterator for EpochSparseViewPtr<'ctx, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
{
    type Item = (*const K, Ptrs<'ctx, V>);
    type IntoIter = RawIter<'ctx, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let (_, iter) = self.into_iter_with_context();
        iter
    }
}
