use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    ops::Index,
};

use crate::{
    slice::{IndexHelper, Iter, RawIter, SoaSlicePtrs, SoaSlicePtrsIndex, SoaSlicesIndex},
    traits::{Ptrs, RawSoa, RawSoaContext, Refs, SlicePtrs, Slices, Soa, SoaContext},
};

pub struct SoaSlices<'ctx, 'a, T>
where
    T: RawSoa + ?Sized + 'a,
{
    ptrs: SoaSlicePtrs<'ctx, T>,
    phantom: PhantomData<fn(&'a ()) -> &'a ()>,
}

impl<'ctx, T> SoaSlices<'ctx, '_, T>
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
    pub fn into_ptrs(self) -> Ptrs<'ctx, T> {
        let (_, ptrs) = self.into_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn into_ptrs_with_context(self) -> (&'ctx T::Context, Ptrs<'ctx, T>) {
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
    pub fn into_slice_ptrs_with_context(self) -> (&'ctx T::Context, SlicePtrs<'ctx, T>) {
        let Self { ptrs, .. } = self;
        ptrs.into_slice_ptrs_with_context()
    }

    #[inline]
    pub fn slice_ptrs(&self) -> SoaSlicePtrs<'_, T> {
        let Self { ptrs, .. } = self;
        ptrs.clone()
    }

    #[inline]
    pub fn into_slice_ptrs(self) -> SoaSlicePtrs<'ctx, T> {
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
    pub fn into_parts(self) -> (&'ctx T::Context, Ptrs<'ctx, T>, usize) {
        let Self { ptrs, .. } = self;
        ptrs.into_parts()
    }

    #[inline]
    pub unsafe fn from_parts(context: &'ctx T::Context, ptrs: Ptrs<'ctx, T>, len: usize) -> Self {
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
    pub unsafe fn into_get_unchecked<I>(self, index: I) -> I::Ptrs<'ctx>
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
    ) -> (&'ctx T::Context, I::Ptrs<'ctx>)
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
    pub fn into_raw_iter(self) -> RawIter<'ctx, T> {
        let (_, iter) = self.into_raw_iter_with_context();
        iter
    }

    #[inline]
    pub fn into_raw_iter_with_context(self) -> (&'ctx T::Context, RawIter<'ctx, T>) {
        let Self { ptrs, .. } = self;
        ptrs.into_iter_with_context()
    }
}

impl<'ctx, 'a, T> SoaSlices<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
{
    #[inline]
    pub fn new(context: &'ctx T::Context, slices: Slices<'ctx, 'a, T>) -> Self {
        let slices = context.slices_as_slice_ptrs(slices);
        Self {
            ptrs: SoaSlicePtrs::new(context, slices),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn into_slices(self) -> Slices<'ctx, 'a, T> {
        let (_, slices) = self.into_slices_with_context();
        slices
    }

    #[inline]
    pub fn into_slices_with_context(self) -> (&'ctx T::Context, Slices<'ctx, 'a, T>) {
        let Self { ptrs, .. } = self;

        let (context, slices) = ptrs.into_slice_ptrs_with_context();
        let slices = unsafe { context.slice_ptrs_to_slices(slices) };
        (context, slices)
    }

    #[inline]
    pub fn into_get<I>(self, index: I) -> Option<I::Refs<'ctx>>
    where
        I: SoaSlicesIndex<'a, T>,
    {
        let (_, refs) = self.into_get_with_context(index);
        refs
    }

    #[inline]
    pub fn into_get_with_context<I>(self, index: I) -> (&'ctx T::Context, Option<I::Refs<'ctx>>)
    where
        I: SoaSlicesIndex<'a, T>,
    {
        let (context, slices) = self.into_slices_with_context();
        (context, index.get(context, slices))
    }

    #[inline]
    #[track_caller]
    pub fn into_index<I>(self, index: I) -> I::Refs<'ctx>
    where
        I: SoaSlicesIndex<'a, T>,
    {
        let (_, refs) = self.into_index_with_context(index);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn into_index_with_context<I>(self, index: I) -> (&'ctx T::Context, I::Refs<'ctx>)
    where
        I: SoaSlicesIndex<'a, T>,
    {
        let (context, slices) = self.into_slices_with_context();
        (context, index.index(context, slices))
    }

    #[inline]
    pub fn into_iter_with_context(self) -> (&'ctx T::Context, Iter<'ctx, 'a, T>) {
        let (context, iter) = self.into_raw_iter_with_context();
        let iter = unsafe { iter.deref() };
        (context, iter)
    }
}

impl<'a, T> SoaSlices<'_, '_, T>
where
    T: Soa<'a> + ?Sized,
{
    #[inline]
    pub fn as_slices(&'a self) -> Slices<'a, 'a, T> {
        let (_, slices) = self.as_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_slices_with_context(&'a self) -> (&'a T::Context, Slices<'a, 'a, T>) {
        let Self { ptrs, .. } = self;

        let (context, slices) = ptrs.as_slice_ptrs_with_context();
        let slices = unsafe { context.slice_ptrs_to_slices(slices) };
        (context, slices)
    }

    #[inline]
    pub fn get<I>(&'a self, index: I) -> Option<I::Refs<'a>>
    where
        I: SoaSlicesIndex<'a, T>,
    {
        let (_, refs) = self.get_with_context(index);
        refs
    }

    #[inline]
    pub fn get_with_context<I>(&'a self, index: I) -> (&'a T::Context, Option<I::Refs<'a>>)
    where
        I: SoaSlicesIndex<'a, T>,
    {
        let (context, slices) = self.as_slices_with_context();
        (context, index.get(context, slices))
    }

    #[inline]
    #[track_caller]
    pub fn index<I>(&'a self, index: I) -> I::Refs<'a>
    where
        I: SoaSlicesIndex<'a, T>,
    {
        let (_, refs) = self.index_with_context(index);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn index_with_context<I>(&'a self, index: I) -> (&'a T::Context, I::Refs<'a>)
    where
        I: SoaSlicesIndex<'a, T>,
    {
        let (context, slices) = self.as_slices_with_context();
        (context, index.index(context, slices))
    }

    #[inline]
    pub fn iter(&'a self) -> Iter<'a, 'a, T> {
        let (_, iter) = self.iter_with_context();
        iter
    }

    #[inline]
    pub fn iter_with_context(&'a self) -> (&'a T::Context, Iter<'a, 'a, T>) {
        let (context, iter) = self.raw_iter_with_context();
        let iter = unsafe { iter.deref() };
        (context, iter)
    }

    #[inline]
    pub fn contains<V>(&'a self, value: V) -> bool
    where
        Refs<'a, 'a, T>: PartialEq<V>,
    {
        let mut iter = self.into_iter();
        iter.any(move |item| item.eq(&value))
    }
}

impl<'ctx, T> From<SoaSlices<'ctx, '_, T>> for SoaSlicePtrs<'ctx, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn from(slices: SoaSlices<'ctx, '_, T>) -> Self {
        slices.into_slice_ptrs()
    }
}

impl<'ctx, T> From<&'ctx T::Context> for SoaSlices<'ctx, '_, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn from(context: &'ctx T::Context) -> Self {
        let ptrs = context.ptrs_dangling();
        unsafe { Self::from_parts(context, ptrs, 0) }
    }
}

impl<T> Debug for SoaSlices<'_, '_, T>
where
    T: ?Sized,
    for<'a> T: Soa<'a>,
    for<'ctx, 'a> Slices<'ctx, 'a, T>: Debug,
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
    T: ?Sized,
    for<'a> T: Soa<'a>,
    for<'ctx, 'a> Slices<'ctx, 'a, T>: Into<&'a [U]>,
{
    #[inline]
    fn as_ref(&self) -> &[U] {
        self.as_slices().into()
    }
}

impl<T> Eq for SoaSlices<'_, '_, T>
where
    T: ?Sized,
    for<'a> T: Soa<'a>,
    for<'ctx, 'a> Slices<'ctx, 'a, T>: Eq,
{
}

impl<T> Ord for SoaSlices<'_, '_, T>
where
    T: ?Sized,
    for<'a> T: Soa<'a>,
    for<'ctx, 'a> Slices<'ctx, 'a, T>: Ord,
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
    T: ?Sized,
    for<'a> T: Soa<'a>,
    for<'ctx, 'a> Slices<'ctx, 'a, T>: Hash,
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
    for<'a> Ptrs<'a, T>: Copy,
{
}

impl<T, U, I> Index<I> for SoaSlices<'_, '_, T>
where
    T: ?Sized,
    U: ?Sized,
    for<'a> T: Soa<'a>,
    for<'ctx, 'a> I: IndexHelper<'ctx, 'a, T, Output = U>,
{
    type Output = U;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        SoaSlices::index(self, index)
    }
}

impl<'a, T> IntoIterator for &'a SoaSlices<'_, '_, T>
where
    T: Soa<'a> + ?Sized,
{
    type Item = Refs<'a, 'a, T>;
    type IntoIter = Iter<'a, 'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'ctx, 'a, T> IntoIterator for SoaSlices<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
{
    type Item = Refs<'ctx, 'a, T>;
    type IntoIter = Iter<'ctx, 'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let (_, iter) = self.into_iter_with_context();
        iter
    }
}
