use alloc::{borrow::ToOwned, boxed::Box};
use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ops::{Index, IndexMut},
    ptr::{self, NonNull},
};

use crate::{
    ptr::{is_zst, ptrs, slice_from_raw_parts, slice_from_raw_parts_mut, BufferData, SoaSlicePtr},
    traits::{SoaToOwned, SoaTrustedFields},
    vec::{IntoIter, SoaVec},
};

use super::{IndexHelper, IndexHelperMut, Iter, IterMut, SoaSliceIndex, SoaSlices, SoaSlicesMut};

#[repr(transparent)]
pub struct SoaSlice<T>
where
    T: SoaTrustedFields,
{
    buffer: [BufferData<T>],
}

impl<T> SoaSlice<T>
where
    T: SoaTrustedFields,
{
    #[inline]
    pub fn context(&self) -> &T::Context {
        unsafe { ptr::from_ref(self).context() }
    }

    #[inline]
    pub fn len(&self) -> usize {
        unsafe { ptr::from_ref(self).len() }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        if is_zst::<T>() {
            return usize::MAX;
        }
        unsafe { ptr::from_ref(self).capacity() }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const BufferData<T> {
        self.buffer.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut BufferData<T> {
        self.buffer.as_mut_ptr()
    }

    #[inline]
    pub fn as_ptrs(&self) -> T::Ptrs {
        let ptr = self.as_ptr().cast_mut();
        let context = self.context();
        let len = self.capacity();

        unsafe {
            let ptrs = ptrs::<T>(context, ptr, len).unwrap_unchecked();
            T::ptrs_cast_const(context, ptrs)
        }
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> T::MutPtrs {
        let ptr = self.as_mut_ptr();
        let context = self.context();
        let len = self.capacity();
        unsafe { ptrs::<T>(context, ptr, len).unwrap_unchecked() }
    }

    #[inline]
    pub fn as_slices(&self) -> T::Slices<'_> {
        let ptrs = self.as_ptrs();
        let len = self.len();
        let context = self.context();

        let slices = T::slices_from_raw_parts(context, ptrs, len);
        unsafe { T::slice_ptrs_to_slices(context, slices) }
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> T::SlicesMut<'_> {
        let ptrs = self.as_mut_ptrs();
        let len = self.len();
        let context = self.context();

        let slices = T::slices_from_raw_parts_mut(context, ptrs, len);
        unsafe { T::slice_ptrs_to_slices_mut(context, slices) }
    }

    #[inline]
    pub fn slices(&self) -> SoaSlices<'_, T> {
        let context = self.context();
        let slices = self.as_slices();
        SoaSlices::new(context, slices)
    }

    #[inline]
    pub fn slices_mut(&mut self) -> SoaSlicesMut<'_, T> {
        let ptrs = self.as_mut_ptrs();
        let context = self.context();
        let len = self.len();
        unsafe { SoaSlicesMut::from_parts(context, ptrs, len) }
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        self.slices().into_iter()
    }

    #[inline]
    pub fn iter_with_context(&self) -> (&T::Context, Iter<'_, T>) {
        self.slices().into_iter_with_context()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.slices_mut().into_iter()
    }

    #[inline]
    pub fn iter_mut_with_context(&mut self) -> (&T::Context, IterMut<'_, T>) {
        self.slices_mut().into_iter_mut_with_context()
    }

    #[inline]
    pub fn contains<'me>(&'me self, value: &T) -> bool
    where
        T::Refs<'me>: PartialEq<T>,
    {
        self.slices().contains(value)
    }

    #[inline]
    pub fn contains_by_refs<'me, 'r>(&'me self, refs: T::Refs<'r>) -> bool
    where
        T::Refs<'me>: PartialEq<T::Refs<'r>>,
    {
        self.slices().contains_by_refs(refs)
    }

    #[inline]
    pub fn into_vec(self: Box<Self>) -> SoaVec<T> {
        let len = self.len();
        let capacity = self.capacity();
        let ptr = Box::into_raw(self).cast();
        unsafe { SoaVec::from_raw_parts(ptr, len, capacity) }
    }

    #[inline]
    pub fn to_vec<'me>(&'me self) -> SoaVec<T>
    where
        T::Refs<'me>: SoaToOwned<'me, Owned = T>,
        T::Context: Clone,
    {
        self.slices().to_vec()
    }

    #[inline]
    #[track_caller]
    pub fn clone_from_slice<'src>(&mut self, src: &'src SoaSlice<T>)
    where
        T::Refs<'src>: SoaToOwned<'src, Owned = T>,
    {
        let src = src.slices();
        self.slices_mut().clone_from_slices(src);
    }

    #[inline]
    #[track_caller]
    pub fn copy_from_slice(&mut self, src: &SoaSlice<T>)
    where
        T::Fields: Copy,
    {
        let src = src.slices();
        self.slices_mut().copy_from_slices(src);
    }

    #[inline]
    pub fn get<I>(&self, index: I) -> Option<I::Refs<'_>>
    where
        I: SoaSliceIndex<T>,
    {
        self.slices().into_get(index)
    }

    #[inline]
    pub fn get_mut<I>(&mut self, index: I) -> Option<I::RefsMut<'_>>
    where
        I: SoaSliceIndex<T>,
    {
        self.slices_mut().into_get_mut(index)
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn get_unchecked<I>(&self, index: I) -> I::Ptrs
    where
        I: SoaSliceIndex<T>,
    {
        unsafe { self.slices().get_unchecked(index) }
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn get_unchecked_mut<I>(&mut self, index: I) -> I::MutPtrs
    where
        I: SoaSliceIndex<T>,
    {
        unsafe { self.slices_mut().get_unchecked_mut(index) }
    }

    #[inline]
    #[track_caller]
    pub fn index<I>(&self, index: I) -> I::Refs<'_>
    where
        I: SoaSliceIndex<T>,
    {
        self.slices().into_index(index)
    }

    #[inline]
    #[track_caller]
    pub fn index_mut<I>(&mut self, index: I) -> I::RefsMut<'_>
    where
        I: SoaSliceIndex<T>,
    {
        self.slices_mut().into_index_mut(index)
    }

    #[inline]
    #[track_caller]
    pub fn swap(&mut self, a: usize, b: usize) {
        self.slices_mut().swap(a, b);
    }

    #[inline]
    pub fn sort(&mut self)
    where
        for<'any> T::Refs<'any>: Ord,
    {
        self.slices_mut().sort();
    }

    #[inline]
    pub fn sort_by<F>(&mut self, compare: F)
    where
        for<'any> F: FnMut(T::Refs<'any>, T::Refs<'any>) -> cmp::Ordering,
    {
        self.slices_mut().sort_by(compare);
    }

    #[inline]
    pub fn sort_by_key<K, F>(&mut self, f: F)
    where
        F: FnMut(T::Refs<'_>) -> K,
        K: Ord,
    {
        self.slices_mut().sort_by_key(f);
    }

    #[inline]
    pub fn sort_by_cached_key<K, F>(&mut self, f: F)
    where
        F: FnMut(T::Refs<'_>) -> K,
        K: Ord,
    {
        self.slices_mut().sort_by_cached_key(f);
    }

    #[inline]
    pub fn sort_unstable(&mut self)
    where
        for<'any> T::Refs<'any>: Ord,
    {
        self.slices_mut().sort_unstable();
    }

    #[inline]
    pub fn sort_unstable_by<F>(&mut self, compare: F)
    where
        for<'any> F: FnMut(T::Refs<'any>, T::Refs<'any>) -> cmp::Ordering,
    {
        self.slices_mut().sort_unstable_by(compare);
    }

    #[inline]
    pub fn sort_unstable_by_key<K, F>(&mut self, f: F)
    where
        F: FnMut(T::Refs<'_>) -> K,
        K: Ord,
    {
        self.slices_mut().sort_unstable_by_key(f);
    }
}

impl<T> Debug for SoaSlice<T>
where
    T: SoaTrustedFields,
    for<'any> T::Slices<'any>: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("SoaSlice").field(&slices).finish()
    }
}

impl<T> Default for &SoaSlice<T>
where
    T: SoaTrustedFields,
{
    #[inline]
    fn default() -> Self {
        let data = NonNull::<BufferData<T>>::dangling().as_ptr().cast();
        unsafe { from_raw_parts(data, 0, 0) }
    }
}

impl<T> Default for &mut SoaSlice<T>
where
    T: SoaTrustedFields,
{
    #[inline]
    fn default() -> Self {
        let data = NonNull::<BufferData<T>>::dangling().as_ptr().cast();
        unsafe { from_raw_parts_mut(data, 0, 0) }
    }
}

impl<T> Default for Box<SoaSlice<T>>
where
    T: SoaTrustedFields,
{
    #[inline]
    fn default() -> Self {
        let data = NonNull::<BufferData<T>>::dangling().as_ptr().cast();
        unsafe { Box::from_raw(slice_from_raw_parts_mut(data, 0, 0)) }
    }
}

impl<T> AsRef<SoaSlice<T>> for SoaSlice<T>
where
    T: SoaTrustedFields,
{
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<T, U> AsRef<[U]> for SoaSlice<T>
where
    for<'any> T: SoaTrustedFields<Slices<'any> = &'any [U]> + 'any,
{
    fn as_ref(&self) -> &[U] {
        self.as_slices()
    }
}

impl<T> AsMut<SoaSlice<T>> for SoaSlice<T>
where
    T: SoaTrustedFields,
{
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<T, U> AsMut<[U]> for SoaSlice<T>
where
    for<'any> T: SoaTrustedFields<SlicesMut<'any> = &'any mut [U]> + 'any,
{
    fn as_mut(&mut self) -> &mut [U] {
        self.as_mut_slices()
    }
}

impl<T> PartialEq for SoaSlice<T>
where
    T: SoaTrustedFields,
    for<'any> T::Slices<'any>: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_slices() == other.as_slices()
    }
    #[inline]
    #[allow(clippy::partialeq_ne_impl)]
    fn ne(&self, other: &Self) -> bool {
        self.as_slices() != other.as_slices()
    }
}

impl<T> Eq for SoaSlice<T>
where
    T: SoaTrustedFields,
    for<'any> T::Slices<'any>: Eq,
{
}

impl<T> PartialOrd for SoaSlice<T>
where
    T: SoaTrustedFields,
    for<'any> T::Slices<'any>: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        PartialOrd::partial_cmp(&self.as_slices(), &other.as_slices())
    }
}

impl<T> Ord for SoaSlice<T>
where
    T: SoaTrustedFields,
    for<'any> T::Slices<'any>: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        Ord::cmp(&self.as_slices(), &other.as_slices())
    }
}

impl<T> Hash for SoaSlice<T>
where
    T: SoaTrustedFields,
    for<'any> T::Slices<'any>: Hash,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let capacity = self.capacity();
        state.write_usize(capacity);

        let slices = self.as_slices();
        slices.hash(state);
    }
}

impl<T> Drop for SoaSlice<T>
where
    T: SoaTrustedFields,
{
    #[inline]
    fn drop(&mut self) {
        if self.is_empty() {
            return;
        }

        let context = ptr::from_ref(self.context());
        let slices = self.as_mut_slices();
        unsafe {
            let context = &*context;
            let slices = T::mut_slice_refs_as_slice_ptrs(context, slices);
            T::slices_drop_in_place(&*context, slices);
        }
    }
}

impl<T> ToOwned for SoaSlice<T>
where
    T: SoaTrustedFields,
    T::Context: Clone,
    for<'any> T::Refs<'any>: SoaToOwned<'any, Owned = T> + 'any,
{
    type Owned = SoaVec<T>;

    #[inline]
    fn to_owned(&self) -> <Self as ToOwned>::Owned {
        self.to_vec()
    }

    #[inline]
    fn clone_into(&self, target: &mut <Self as ToOwned>::Owned) {
        // FIXME: decide if this impl will be better:
        // https://github.com/rust-lang/rust/blob/019fc4de2f3d49a2ef862d180599194d2be05193/library/alloc/src/slice.rs#L860

        target.clear();
        target.extend_from_slice(self);
    }
}

impl<T, U, I> Index<I> for SoaSlice<T>
where
    T: SoaTrustedFields,
    U: ?Sized,
    for<'any> I: IndexHelper<'any, T, Output = U>,
{
    type Output = U;

    fn index(&self, index: I) -> &Self::Output {
        SoaSlice::index(self, index)
    }
}

impl<T, U, I> IndexMut<I> for SoaSlice<T>
where
    T: SoaTrustedFields,
    U: ?Sized,
    for<'any> I: IndexHelperMut<'any, T, Output = U>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        SoaSlice::index_mut(self, index)
    }
}

impl<'a, T> IntoIterator for &'a SoaSlice<T>
where
    T: SoaTrustedFields,
{
    type Item = T::Refs<'a>;
    type IntoIter = Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a Box<SoaSlice<T>>
where
    T: SoaTrustedFields,
{
    type Item = T::Refs<'a>;
    type IntoIter = Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut SoaSlice<T>
where
    T: SoaTrustedFields,
{
    type Item = T::RefsMut<'a>;
    type IntoIter = IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, T> IntoIterator for &'a mut Box<SoaSlice<T>>
where
    T: SoaTrustedFields,
{
    type Item = T::RefsMut<'a>;
    type IntoIter = IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> IntoIterator for Box<SoaSlice<T>>
where
    T: SoaTrustedFields,
{
    type Item = T;
    type IntoIter = IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.into_vec().into_iter()
    }
}

unsafe impl<T> Send for SoaSlice<T>
where
    T: SoaTrustedFields,
    T::Fields: Send,
    T::Context: Send,
{
}

unsafe impl<T> Sync for SoaSlice<T>
where
    T: SoaTrustedFields,
    T::Fields: Sync,
    T::Context: Sync,
{
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn from_raw_parts<'slice, T>(
    data: *const BufferData<T>,
    len: usize,
    capacity: usize,
) -> &'slice SoaSlice<T>
where
    T: SoaTrustedFields,
{
    unsafe { &*slice_from_raw_parts(data, len, capacity) }
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn from_raw_parts_mut<'slice, T>(
    data: *mut BufferData<T>,
    len: usize,
    capacity: usize,
) -> &'slice mut SoaSlice<T>
where
    T: SoaTrustedFields,
{
    unsafe { &mut *slice_from_raw_parts_mut(data, len, capacity) }
}
