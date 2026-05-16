use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr,
};

use crate::{
    algo::sparse_get_unchecked,
    item::{self, KeyValuePair, KeyValuePtrs, KeyValueSlicePtrs, SparseItem},
    iter::{RawIter, RawKeys, RawValues},
    key::Key,
    soa::{
        identity::Identity,
        slice::SoaSlicePtrs,
        traits::{Ptrs, RawSoa, SlicePtrs},
    },
    view::{EpochSparseView, EpochSparseViewMutPtr},
};

pub type SparseViewPtr<'ctx, T, S = item::DefaultSparseItem<usize>> =
    EpochSparseViewPtr<'ctx, usize, T, S>;

pub struct EpochSparseViewPtr<'ctx, K, V, S = item::DefaultSparseItem<K>>
where
    K: Key,
    V: RawSoa<Context: 'ctx> + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    dense: SoaSlicePtrs<'ctx, KeyValuePair<K, V>>,
    sparse: *const [S],
}

impl<'ctx, K, V, S> EpochSparseViewPtr<'ctx, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    pub unsafe fn from_parts(
        dense: SoaSlicePtrs<'ctx, KeyValuePair<K, V>>,
        sparse: *const [S],
    ) -> Self {
        Self { dense, sparse }
    }

    #[inline]
    pub fn into_parts(self) -> (SoaSlicePtrs<'ctx, KeyValuePair<K, V>>, *const [S]) {
        let Self { dense, sparse } = self;
        (dense, sparse)
    }

    #[inline]
    pub fn cast_mut(self) -> EpochSparseViewMutPtr<'ctx, K, V, S> {
        let Self { dense, sparse } = self;

        let dense = dense.cast_mut();
        let sparse = sparse.cast_mut();
        unsafe { EpochSparseViewMutPtr::from_parts(dense, sparse) }
    }

    #[inline]
    pub unsafe fn as_ref_unchecked<'a>(self) -> EpochSparseView<'ctx, 'a, K, V, S> {
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
    pub fn as_ptrs(&self) -> (KeyValuePtrs<'_, K, V>, *const S) {
        let (_, dense, sparse) = self.as_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&V::Context, KeyValuePtrs<'_, K, V>, *const S) {
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
    pub fn as_dense_ptrs(&self) -> KeyValuePtrs<'_, K, V> {
        let (_, dense) = self.as_dense_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn as_dense_ptrs_with_context(&self) -> (&V::Context, KeyValuePtrs<'_, K, V>) {
        let (context, dense, _) = self.as_ptrs_with_context();
        (context, dense)
    }

    #[inline]
    pub fn as_sparse_ptr(&self) -> *const S {
        let (_, sparse) = self.as_sparse_ptr_with_context();
        sparse
    }

    #[inline]
    pub fn as_sparse_ptr_with_context(&self) -> (&V::Context, *const S) {
        let (context, _, sparse) = self.as_ptrs_with_context();
        (context, sparse)
    }

    #[inline]
    pub fn into_ptrs(self) -> (KeyValuePtrs<'ctx, K, V>, *const S) {
        let (_, dense, sparse) = self.into_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn into_ptrs_with_context(self) -> (&'ctx V::Context, KeyValuePtrs<'ctx, K, V>, *const S) {
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
    pub fn into_dense_ptrs(self) -> KeyValuePtrs<'ctx, K, V> {
        let (_, dense) = self.into_dense_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn into_dense_ptrs_with_context(self) -> (&'ctx V::Context, KeyValuePtrs<'ctx, K, V>) {
        let (context, dense, _) = self.into_ptrs_with_context();
        (context, dense)
    }

    #[inline]
    pub fn into_sparse_ptr(self) -> *const S {
        let (_, sparse) = self.into_sparse_ptr_with_context();
        sparse
    }

    #[inline]
    pub fn into_sparse_ptr_with_context(self) -> (&'ctx V::Context, *const S) {
        let (context, _, sparse) = self.into_ptrs_with_context();
        (context, sparse)
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> (KeyValueSlicePtrs<'_, K, V>, *const [S]) {
        let (_, dense, sparse) = self.as_slice_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(
        &self,
    ) -> (&V::Context, KeyValueSlicePtrs<'_, K, V>, *const [S]) {
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
    pub fn as_dense_slice_ptrs(&self) -> KeyValueSlicePtrs<'_, K, V> {
        let (_, dense) = self.as_dense_slice_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn as_dense_slice_ptrs_with_context(&self) -> (&V::Context, KeyValueSlicePtrs<'_, K, V>) {
        let (context, dense, _) = self.as_slice_ptrs_with_context();
        (context, dense)
    }

    #[inline]
    pub fn as_sparse_slice_ptr(&self) -> *const [S] {
        let (_, sparse) = self.as_sparse_slice_ptr_with_context();
        sparse
    }

    #[inline]
    pub fn as_sparse_slice_ptr_with_context(&self) -> (&V::Context, *const [S]) {
        let (context, _, sparse) = self.as_slice_ptrs_with_context();
        (context, sparse)
    }

    #[inline]
    pub fn into_slice_ptrs(self) -> (KeyValueSlicePtrs<'ctx, K, V>, *const [S]) {
        let (_, dense, sparse) = self.into_slice_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn into_slice_ptrs_with_context(
        self,
    ) -> (&'ctx V::Context, KeyValueSlicePtrs<'ctx, K, V>, *const [S]) {
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
    pub fn into_dense_slice_ptrs(self) -> KeyValueSlicePtrs<'ctx, K, V> {
        let (_, dense) = self.into_dense_slice_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn into_dense_slice_ptrs_with_context(
        self,
    ) -> (&'ctx V::Context, KeyValueSlicePtrs<'ctx, K, V>) {
        let (context, dense, _) = self.into_slice_ptrs_with_context();
        (context, dense)
    }

    #[inline]
    pub fn into_sparse_slice_ptr(self) -> *const [S] {
        let (_, sparse) = self.into_sparse_slice_ptr_with_context();
        sparse
    }

    #[inline]
    pub fn into_sparse_slice_ptr_with_context(self) -> (&'ctx V::Context, *const [S]) {
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
        let sparse_index = key.sparse_index();
        let (_, value) = unsafe { sparse_get_unchecked::<K, _, _>(dense, sparse, sparse_index) };
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
        let sparse_index = key.sparse_index();
        let (_, value) = unsafe { sparse_get_unchecked::<K, _, _>(dense, sparse, sparse_index) };
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
        let (key, value) = unsafe { sparse_get_unchecked::<K, _, _>(dense, sparse, sparse_index) };
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
        let (key, value) = unsafe { sparse_get_unchecked::<K, _, _>(dense, sparse, sparse_index) };
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

        let (context, slices) = dense.as_slice_ptrs_with_context();
        let (keys, _) = slices.into();
        let iter = RawKeys::new(context.as_inner(), keys);
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

        let (context, slices) = dense.into_slice_ptrs_with_context();
        let (keys, _) = slices.into();
        let iter = RawKeys::new(context.as_inner(), keys);
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

impl<'ctx, K, V, S> From<&'ctx V::Context> for EpochSparseViewPtr<'ctx, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    fn from(context: &'ctx V::Context) -> Self {
        let context = Identity::from_inner_ref(context);
        let dense = SoaSlicePtrs::from(context);
        let sparse = ptr::from_ref(Default::default());
        unsafe { Self::from_parts(dense, sparse) }
    }
}

impl<'ctx, K, V, S> Default for EpochSparseViewPtr<'ctx, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    &'ctx V::Context: Default,
{
    #[inline]
    fn default() -> Self {
        let context: &V::Context = Default::default();
        Self::from(context)
    }
}

impl<K, V, S> Debug for EpochSparseViewPtr<'_, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
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

impl<K, V, S> Clone for EpochSparseViewPtr<'_, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { ref dense, sparse } = *self;

        let dense = dense.clone();
        Self { dense, sparse }
    }
}

impl<K, V, S> Copy for EpochSparseViewPtr<'_, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    for<'ctx> Ptrs<'ctx, V>: Copy,
{
}

impl<K, V, S> PartialEq for EpochSparseViewPtr<'_, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    V::Context: PartialEq,
    for<'ctx> Ptrs<'ctx, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { dense, sparse } = self;

        let other = (&other.dense, &other.sparse);
        (dense, sparse) == other
    }
}

impl<K, V, S> Eq for EpochSparseViewPtr<'_, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    V::Context: Eq,
    for<'ctx> Ptrs<'ctx, V>: Eq,
{
}

impl<K, V, S> PartialOrd for EpochSparseViewPtr<'_, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    V::Context: PartialOrd,
    for<'ctx> Ptrs<'ctx, V>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { dense, sparse } = self;

        let other = (&other.dense, &other.sparse);
        (dense, sparse).partial_cmp(&other)
    }
}

impl<K, V, S> Ord for EpochSparseViewPtr<'_, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    V::Context: Ord,
    for<'ctx> Ptrs<'ctx, V>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { dense, sparse } = self;

        let other = (&other.dense, &other.sparse);
        (dense, sparse).cmp(&other)
    }
}

impl<K, V, S> Hash for EpochSparseViewPtr<'_, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    V::Context: Hash,
    for<'ctx> Ptrs<'ctx, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { dense, sparse } = self;
        (dense, sparse).hash(state);
    }
}

impl<'a, K, V, S> IntoIterator for &'a EpochSparseViewPtr<'_, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    type Item = (*const K, Ptrs<'a, V>);
    type IntoIter = RawIter<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'ctx, K, V, S> IntoIterator for EpochSparseViewPtr<'ctx, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    type Item = (*const K, Ptrs<'ctx, V>);
    type IntoIter = RawIter<'ctx, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let (_, iter) = self.into_iter_with_context();
        iter
    }
}
