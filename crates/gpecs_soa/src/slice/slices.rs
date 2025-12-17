use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    ops::Index,
};

use crate::{
    slice::{IndexHelper, Iter, RawIter, SoaSlicePtrs, SoaSlicePtrsIndex, SoaSlicesIndex},
    traits::{Ptrs, RawSoa, RawSoaContext, SlicePtrs, Soa},
};

pub struct SoaSlices<'c, 'a, T>
where
    T: RawSoa + ?Sized + 'a,
{
    ptrs: SoaSlicePtrs<'c, T>,
    phantom: PhantomData<&'a ()>,
}

impl<'c, T> SoaSlices<'c, '_, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    pub fn len(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn context(&self) -> &T::Context {
        let Self { ptrs, .. } = self;
        ptrs.context()
    }

    #[inline]
    pub fn as_ptrs(&self) -> Ptrs<'_, T> {
        let (_, ptrs) = self.as_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&T::Context, Ptrs<'_, T>) {
        let Self { ptrs, .. } = self;
        ptrs.as_ptrs_with_context()
    }

    #[inline]
    pub fn into_ptrs(self) -> Ptrs<'c, T> {
        let (_, ptrs) = self.into_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn into_ptrs_with_context(self) -> (&'c T::Context, Ptrs<'c, T>) {
        let Self { ptrs, .. } = self;
        ptrs.into_ptrs_with_context()
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> SlicePtrs<'_, T> {
        let (_, ptrs) = self.as_slice_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(&self) -> (&T::Context, SlicePtrs<'_, T>) {
        let Self { ptrs, .. } = self;
        ptrs.as_slice_ptrs_with_context()
    }

    #[inline]
    pub fn into_slice_ptrs_with_context(self) -> (&'c T::Context, SlicePtrs<'c, T>) {
        let Self { ptrs, .. } = self;
        ptrs.into_slice_ptrs_with_context()
    }

    #[inline]
    pub fn slice_ptrs(&self) -> SoaSlicePtrs<'_, T> {
        let Self { ptrs, .. } = self;
        ptrs.clone()
    }

    #[inline]
    pub fn into_slice_ptrs(self) -> SoaSlicePtrs<'c, T> {
        let Self { ptrs, .. } = self;
        ptrs
    }

    #[inline]
    pub fn slices(&self) -> SoaSlices<'_, '_, T> {
        let Self { ptrs, .. } = self;

        let len = ptrs.len();
        let (context, ptrs) = ptrs.as_ptrs_with_context();
        unsafe { SoaSlices::from_parts(context, ptrs, len) }
    }

    #[inline]
    pub fn into_parts(self) -> (&'c T::Context, Ptrs<'c, T>, usize) {
        let Self { ptrs, .. } = self;
        ptrs.into_parts()
    }

    #[inline]
    pub unsafe fn from_parts(context: &'c T::Context, ptrs: Ptrs<'c, T>, len: usize) -> Self {
        Self {
            ptrs: unsafe { SoaSlicePtrs::from_parts(context, ptrs, len) },
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn get_unchecked<I>(&self, index: I) -> I::Ptrs<'_>
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let (_, ptrs) = unsafe { self.get_unchecked_with_context(index) };
        ptrs
    }

    #[inline]
    pub unsafe fn get_unchecked_with_context<I>(&self, index: I) -> (&T::Context, I::Ptrs<'_>)
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let Self { ptrs, .. } = self;
        unsafe { ptrs.get_unchecked_with_context(index) }
    }

    #[inline]
    pub unsafe fn into_get_unchecked<I>(self, index: I) -> I::Ptrs<'c>
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let (_, ptrs) = unsafe { self.into_get_unchecked_with_context(index) };
        ptrs
    }

    #[inline]
    pub unsafe fn into_get_unchecked_with_context<I>(
        self,
        index: I,
    ) -> (&'c T::Context, I::Ptrs<'c>)
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let Self { ptrs, .. } = self;
        unsafe { ptrs.into_get_unchecked_with_context(index) }
    }

    #[inline]
    pub fn raw_iter(&self) -> RawIter<'_, T> {
        let (_, iter) = self.raw_iter_with_context();
        iter
    }

    #[inline]
    pub fn raw_iter_with_context(&self) -> (&T::Context, RawIter<'_, T>) {
        let Self { ptrs, .. } = self;
        ptrs.iter_with_context()
    }

    #[inline]
    pub fn into_raw_iter(self) -> RawIter<'c, T> {
        let (_, iter) = self.into_raw_iter_with_context();
        iter
    }

    #[inline]
    pub fn into_raw_iter_with_context(self) -> (&'c T::Context, RawIter<'c, T>) {
        let Self { ptrs, .. } = self;
        ptrs.into_iter_with_context()
    }
}

impl<'c, 'a, T> SoaSlices<'c, 'a, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    pub fn new(context: &'c T::Context, slices: T::Slices<'c, 'a>) -> Self {
        let slices = T::slices_as_slice_ptrs(context, slices);
        Self {
            ptrs: SoaSlicePtrs::new(context, slices),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn as_slices(&self) -> T::Slices<'_, '_> {
        let (_, slices) = self.as_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_slices_with_context(&self) -> (&T::Context, T::Slices<'_, '_>) {
        let Self { ptrs, .. } = self;

        let (context, slices) = ptrs.as_slice_ptrs_with_context();
        let slices = unsafe { T::slice_ptrs_to_slices(context, slices) };
        (context, slices)
    }

    #[inline]
    pub fn into_slices(self) -> T::Slices<'c, 'a> {
        let (_, slices) = self.into_slices_with_context();
        slices
    }

    #[inline]
    pub fn into_slices_with_context(self) -> (&'c T::Context, T::Slices<'c, 'a>) {
        let Self { ptrs, .. } = self;

        let (context, slices) = ptrs.into_slice_ptrs_with_context();
        let slices = unsafe { T::slice_ptrs_to_slices(context, slices) };
        (context, slices)
    }

    #[inline]
    pub fn get<I>(&self, index: I) -> Option<I::Refs<'_, '_>>
    where
        I: SoaSlicesIndex<T>,
    {
        let (_, refs) = self.get_with_context(index);
        refs
    }

    #[inline]
    pub fn get_with_context<I>(&self, index: I) -> (&T::Context, Option<I::Refs<'_, '_>>)
    where
        I: SoaSlicesIndex<T>,
    {
        let (context, slices) = self.as_slices_with_context();
        (context, index.get(context, slices))
    }

    #[inline]
    pub fn into_get<I>(self, index: I) -> Option<I::Refs<'c, 'a>>
    where
        I: SoaSlicesIndex<T>,
    {
        let (_, refs) = self.into_get_with_context(index);
        refs
    }

    #[inline]
    pub fn into_get_with_context<I>(self, index: I) -> (&'c T::Context, Option<I::Refs<'c, 'a>>)
    where
        I: SoaSlicesIndex<T>,
    {
        let (context, slices) = self.into_slices_with_context();
        (context, index.get(context, slices))
    }

    #[inline]
    #[track_caller]
    pub fn index<I>(&self, index: I) -> I::Refs<'_, '_>
    where
        I: SoaSlicesIndex<T>,
    {
        let (_, refs) = self.index_with_context(index);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn index_with_context<I>(&self, index: I) -> (&T::Context, I::Refs<'_, '_>)
    where
        I: SoaSlicesIndex<T>,
    {
        let (context, slices) = self.as_slices_with_context();
        (context, index.index(context, slices))
    }

    #[inline]
    #[track_caller]
    pub fn into_index<I>(self, index: I) -> I::Refs<'c, 'a>
    where
        I: SoaSlicesIndex<T>,
    {
        let (_, refs) = self.into_index_with_context(index);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn into_index_with_context<I>(self, index: I) -> (&'c T::Context, I::Refs<'c, 'a>)
    where
        I: SoaSlicesIndex<T>,
    {
        let (context, slices) = self.into_slices_with_context();
        (context, index.index(context, slices))
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, '_, T> {
        let (_, iter) = self.iter_with_context();
        iter
    }

    #[inline]
    pub fn iter_with_context(&self) -> (&T::Context, Iter<'_, '_, T>) {
        let (context, iter) = self.raw_iter_with_context();
        let iter = unsafe { iter.deref() };
        (context, iter)
    }

    #[inline]
    pub fn into_iter_with_context(self) -> (&'c T::Context, Iter<'c, 'a, T>) {
        let (context, iter) = self.into_raw_iter_with_context();
        let iter = unsafe { iter.deref() };
        (context, iter)
    }

    #[inline]
    pub fn contains<'me, V>(&'me self, value: V) -> bool
    where
        T::Refs<'me, 'me>: PartialEq<V>,
    {
        let mut iter = self.into_iter();
        iter.any(move |item| item.eq(&value))
    }
}

impl<'c, T> From<SoaSlices<'c, '_, T>> for SoaSlicePtrs<'c, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn from(slices: SoaSlices<'c, '_, T>) -> Self {
        slices.into_slice_ptrs()
    }
}

impl<'c, T> From<&'c T::Context> for SoaSlices<'c, '_, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn from(context: &'c T::Context) -> Self {
        let ptrs = context.ptrs_dangling();
        unsafe { Self::from_parts(context, ptrs, 0) }
    }
}

impl<T> Debug for SoaSlices<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("SoaSlices").field(&slices).finish()
    }
}

impl<T> AsRef<Self> for SoaSlices<'_, '_, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<T, U> AsRef<[U]> for SoaSlices<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Into<&'any [U]>,
{
    #[inline]
    fn as_ref(&self) -> &[U] {
        self.as_slices().into()
    }
}

impl<T> Eq for SoaSlices<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Eq,
{
}

impl<T> Ord for SoaSlices<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let this = self.as_slices();
        let other = other.as_slices();
        Ord::cmp(&this, &other)
    }
}

impl<T> Hash for SoaSlices<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Hash,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let slices = self.as_slices();
        slices.hash(state);
    }
}

impl<T> Clone for SoaSlices<'_, '_, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { ref ptrs, phantom } = *self;

        let ptrs = ptrs.clone();
        Self { ptrs, phantom }
    }
}

impl<T> Copy for SoaSlices<'_, '_, T>
where
    T: RawSoa + ?Sized,
    for<'any> Ptrs<'any, T>: Copy,
{
}

impl<T, U, I> Index<I> for SoaSlices<'_, '_, T>
where
    T: Soa + ?Sized,
    U: ?Sized,
    for<'c, 'any> I: IndexHelper<'c, 'any, T, Output = U>,
{
    type Output = U;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        SoaSlices::index(self, index)
    }
}

impl<'r, T> IntoIterator for &'r SoaSlices<'_, '_, T>
where
    T: Soa + ?Sized,
{
    type Item = T::Refs<'r, 'r>;
    type IntoIter = Iter<'r, 'r, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'c, 'a, T> IntoIterator for SoaSlices<'c, 'a, T>
where
    T: Soa + ?Sized,
{
    type Item = T::Refs<'c, 'a>;
    type IntoIter = Iter<'c, 'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let (_, iter) = self.into_iter_with_context();
        iter
    }
}
