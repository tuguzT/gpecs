use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use crate::{
    layout::is_zst,
    slice::{
        assert::slice_index_usize_fail,
        index::{IndexHelper, IndexHelperMut, SoaSlicePtrsIndex, SoaSlicesIndex},
        iter::{Iter, IterMut},
    },
    traits::{
        MutPtrs, Ptrs, RawSoa, RawSoaContext, SliceMutPtrs, SlicePtrs, Soa, SoaToOwned, SoaWrite,
    },
    wrapper::{MutPtrs as MutPtrsWrapper, Ptrs as PtrsWrapper},
};

pub struct SoaSlicePtrs<'c, T>
where
    T: RawSoa + ?Sized,
{
    ptrs: PtrsWrapper<'c, T>,
    context: &'c T::Context,
    len: usize,
}

impl<'c, T> SoaSlicePtrs<'c, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    pub fn new(context: &'c T::Context, slices: SlicePtrs<'c, T>) -> Self {
        let len = context.slice_ptrs_len(&slices);
        let ptrs = context.slice_ptrs_as_ptrs(slices);
        unsafe { Self::from_parts(context, ptrs, len) }
    }

    #[inline]
    pub unsafe fn from_parts(context: &'c T::Context, ptrs: Ptrs<'c, T>, len: usize) -> Self {
        let ptrs = PtrsWrapper::new(ptrs);
        Self { ptrs, context, len }
    }

    #[inline]
    pub fn into_parts(self) -> (&'c T::Context, Ptrs<'c, T>, usize) {
        let Self { context, ptrs, len } = self;
        (context, ptrs.into_inner(), len)
    }

    #[inline]
    pub fn cast_mut(self) -> SoaSliceMutPtrs<'c, T> {
        let (context, ptrs, len) = self.into_parts();
        let ptrs = context.ptrs_cast_mut(ptrs);
        unsafe { SoaSliceMutPtrs::from_parts(context, ptrs, len) }
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { len, .. } = *self;
        len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn context(&self) -> &T::Context {
        let Self { context, .. } = *self;
        context
    }

    #[inline]
    pub fn as_ptrs(&self) -> Ptrs<'_, T> {
        let (_, ptrs) = self.as_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&T::Context, Ptrs<'_, T>) {
        let Self {
            context, ref ptrs, ..
        } = *self;

        let ptrs = ptrs.clone().into_inner();
        (context, ptrs)
    }

    #[inline]
    pub fn into_ptrs(self) -> Ptrs<'c, T> {
        let (_, ptrs) = self.into_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn into_ptrs_with_context(self) -> (&'c T::Context, Ptrs<'c, T>) {
        let Self { context, ptrs, .. } = self;

        let ptrs = ptrs.into_inner();
        (context, ptrs)
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> SlicePtrs<'_, T> {
        let (_, slices) = self.as_slice_ptrs_with_context();
        slices
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(&self) -> (&T::Context, SlicePtrs<'_, T>) {
        let Self {
            context,
            len,
            ref ptrs,
        } = *self;

        let ptrs = ptrs.clone().into_inner();
        let slices = context.slice_ptrs_from_raw_parts(ptrs, len);
        (context, slices)
    }

    #[inline]
    pub fn into_slice_ptrs(self) -> SlicePtrs<'c, T> {
        let (_, slices) = self.into_slice_ptrs_with_context();
        slices
    }

    #[inline]
    pub fn into_slice_ptrs_with_context(self) -> (&'c T::Context, SlicePtrs<'c, T>) {
        let Self { context, len, ptrs } = self;

        let ptrs = ptrs.into_inner();
        let slices = context.slice_ptrs_from_raw_parts(ptrs, len);
        (context, slices)
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
        let (context, slices) = self.as_slice_ptrs_with_context();
        let ptrs = unsafe { index.get_unchecked(context, slices) };
        (context, ptrs)
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
        let (context, slices) = self.into_slice_ptrs_with_context();
        let ptrs = unsafe { index.get_unchecked(context, slices) };
        (context, ptrs)
    }
}

impl<'c, T> From<&'c T::Context> for SoaSlicePtrs<'c, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn from(context: &'c T::Context) -> Self {
        let ptrs = context.ptrs_dangling();
        unsafe { Self::from_parts(context, ptrs, 0) }
    }
}

impl<T> Debug for SoaSlicePtrs<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: Debug,
    for<'any> Ptrs<'any, T>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            context,
            ref ptrs,
            ref len,
        } = *self;

        f.debug_struct("SoaSlicePtrs")
            .field("len", len)
            .field("context", context)
            .field("ptrs", ptrs)
            .finish()
    }
}

impl<T> Clone for SoaSlicePtrs<'_, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self {
            ref ptrs,
            context,
            len,
        } = *self;

        let ptrs = ptrs.clone();
        Self { ptrs, context, len }
    }
}

impl<T> Copy for SoaSlicePtrs<'_, T>
where
    T: RawSoa + ?Sized,
    for<'any> Ptrs<'any, T>: Copy,
{
}

impl<T> PartialEq for SoaSlicePtrs<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: PartialEq,
    for<'any> Ptrs<'any, T>: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let Self { ptrs, context, len } = self;
        *ptrs == other.ptrs && *context == other.context && *len == other.len
    }
}

impl<T> Eq for SoaSlicePtrs<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: Eq,
    for<'any> Ptrs<'any, T>: Eq,
{
}

impl<T> PartialOrd for SoaSlicePtrs<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: PartialOrd,
    for<'any> Ptrs<'any, T>: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { ptrs, context, len } = self;

        match len.partial_cmp(&other.len) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match context.partial_cmp(&other.context) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        ptrs.partial_cmp(&other.ptrs)
    }
}

impl<T> Ord for SoaSlicePtrs<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: Ord,
    for<'any> Ptrs<'any, T>: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { ptrs, context, len } = self;

        match len.cmp(&other.len) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match context.cmp(&other.context) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        ptrs.cmp(&other.ptrs)
    }
}

impl<T> Hash for SoaSlicePtrs<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: Hash,
    for<'any> Ptrs<'any, T>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { ptrs, context, len } = self;

        len.hash(state);
        context.hash(state);
        ptrs.hash(state);
    }
}

unsafe impl<T> Send for SoaSlicePtrs<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: Send,
    T::Fields: Send,
{
}

unsafe impl<T> Sync for SoaSlicePtrs<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: Sync,
    T::Fields: Sync,
{
}

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
        let Self { ptrs, .. } = self;
        ptrs.as_ptrs()
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
    pub fn slices(&self) -> SoaSlices<'_, '_, T> {
        let Self { ptrs, .. } = self;

        let len = ptrs.len();
        let (context, ptrs) = ptrs.as_ptrs_with_context();
        unsafe { SoaSlices::from_parts(context, ptrs, len) }
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
        let (context, slices) = self.as_slices_with_context();
        (context, Iter::new(context, slices))
    }

    #[inline]
    pub fn into_iter_with_context(self) -> (&'c T::Context, Iter<'c, 'a, T>) {
        let (context, slices) = self.into_slices_with_context();
        (context, Iter::new(context, slices))
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
        let SoaSlices { ptrs, .. } = slices;
        ptrs
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
        Self {
            ptrs: ptrs.clone(),
            phantom,
        }
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

pub struct SoaSliceMutPtrs<'c, T>
where
    T: RawSoa + ?Sized,
{
    ptrs: MutPtrsWrapper<'c, T>,
    context: &'c T::Context,
    len: usize,
}

impl<'c, T> SoaSliceMutPtrs<'c, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    pub fn new(context: &'c T::Context, slices: SliceMutPtrs<'c, T>) -> Self {
        let len = context.slice_mut_ptrs_len(&slices);
        let ptrs = context.slice_mut_ptrs_as_ptrs(slices);
        unsafe { Self::from_parts(context, ptrs, len) }
    }

    #[inline]
    pub unsafe fn from_parts(context: &'c T::Context, ptrs: MutPtrs<'c, T>, len: usize) -> Self {
        let ptrs = MutPtrsWrapper::new(ptrs);
        Self { ptrs, context, len }
    }

    #[inline]
    pub fn into_parts(self) -> (&'c T::Context, MutPtrs<'c, T>, usize) {
        let Self { context, ptrs, len } = self;
        (context, ptrs.into_inner(), len)
    }

    #[inline]
    pub fn cast_const(self) -> SoaSlicePtrs<'c, T> {
        let (context, ptrs, len) = self.into_parts();
        let ptrs = context.ptrs_cast_const(ptrs);
        unsafe { SoaSlicePtrs::from_parts(context, ptrs, len) }
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { len, .. } = *self;
        len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn context(&self) -> &T::Context {
        let Self { context, .. } = *self;
        context
    }

    #[inline]
    pub fn as_ptrs(&self) -> Ptrs<'_, T> {
        let (_, ptrs) = self.as_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&T::Context, Ptrs<'_, T>) {
        let Self {
            context, ref ptrs, ..
        } = *self;

        let ptrs = ptrs.clone().into_inner();
        let ptrs = context.ptrs_cast_const(ptrs);
        (context, ptrs)
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> MutPtrs<'_, T> {
        let (_, ptrs) = self.as_mut_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_mut_ptrs_with_context(&mut self) -> (&T::Context, MutPtrs<'_, T>) {
        let Self {
            context, ref ptrs, ..
        } = *self;

        let ptrs = ptrs.clone().into_inner();
        (context, ptrs)
    }

    #[inline]
    pub fn into_ptrs(self) -> Ptrs<'c, T> {
        let (_, ptrs) = self.into_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn into_ptrs_with_context(self) -> (&'c T::Context, Ptrs<'c, T>) {
        let Self { context, ptrs, .. } = self;

        let ptrs = ptrs.into_inner();
        let ptrs = context.ptrs_cast_const(ptrs);
        (context, ptrs)
    }

    #[inline]
    pub fn into_mut_ptrs(self) -> MutPtrs<'c, T> {
        let (_, ptrs) = self.into_mut_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn into_mut_ptrs_with_context(self) -> (&'c T::Context, MutPtrs<'c, T>) {
        let Self { context, ptrs, .. } = self;

        let ptrs = ptrs.into_inner();
        (context, ptrs)
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> SlicePtrs<'_, T> {
        let (_, slices) = self.as_slice_ptrs_with_context();
        slices
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(&self) -> (&T::Context, SlicePtrs<'_, T>) {
        let Self {
            context,
            len,
            ref ptrs,
        } = *self;

        let ptrs = ptrs.clone().into_inner();
        let ptrs = context.ptrs_cast_const(ptrs);
        let slices = context.slice_ptrs_from_raw_parts(ptrs, len);
        (context, slices)
    }

    #[inline]
    pub fn as_slice_mut_ptrs(&mut self) -> SliceMutPtrs<'_, T> {
        let (_, slices) = self.as_slice_mut_ptrs_with_context();
        slices
    }

    #[inline]
    pub fn as_slice_mut_ptrs_with_context(&mut self) -> (&T::Context, SliceMutPtrs<'_, T>) {
        let Self {
            context,
            len,
            ref ptrs,
        } = *self;

        let ptrs = ptrs.clone().into_inner();
        let slices = context.slice_mut_ptrs_from_raw_parts(ptrs, len);
        (context, slices)
    }

    #[inline]
    pub fn into_slice_ptrs(self) -> SlicePtrs<'c, T> {
        let (_, slices) = self.into_slice_ptrs_with_context();
        slices
    }

    #[inline]
    pub fn into_slice_ptrs_with_context(self) -> (&'c T::Context, SlicePtrs<'c, T>) {
        let Self { context, len, ptrs } = self;

        let ptrs = ptrs.into_inner();
        let ptrs = context.ptrs_cast_const(ptrs);
        let slices = context.slice_ptrs_from_raw_parts(ptrs, len);
        (context, slices)
    }

    #[inline]
    pub fn into_slice_mut_ptrs(self) -> SliceMutPtrs<'c, T> {
        let (_, slices) = self.into_slice_mut_ptrs_with_context();
        slices
    }

    #[inline]
    pub fn into_slice_mut_ptrs_with_context(self) -> (&'c T::Context, SliceMutPtrs<'c, T>) {
        let Self { context, len, ptrs } = self;

        let ptrs = ptrs.into_inner();
        let slices = context.slice_mut_ptrs_from_raw_parts(ptrs, len);
        (context, slices)
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
        let (context, slices) = self.as_slice_ptrs_with_context();
        let ptrs = unsafe { index.get_unchecked(context, slices) };
        (context, ptrs)
    }

    #[inline]
    pub unsafe fn get_unchecked_mut<I>(&mut self, index: I) -> I::MutPtrs<'_>
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let (_, ptrs) = unsafe { self.get_unchecked_mut_with_context(index) };
        ptrs
    }

    #[inline]
    pub unsafe fn get_unchecked_mut_with_context<I>(
        &mut self,
        index: I,
    ) -> (&T::Context, I::MutPtrs<'_>)
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let (context, slices) = self.as_slice_mut_ptrs_with_context();
        let ptrs = unsafe { index.get_unchecked_mut(context, slices) };
        (context, ptrs)
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
        let (context, slices) = self.into_slice_ptrs_with_context();
        let ptrs = unsafe { index.get_unchecked(context, slices) };
        (context, ptrs)
    }

    #[inline]
    pub unsafe fn into_get_unchecked_mut<I>(self, index: I) -> I::MutPtrs<'c>
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let (_, ptrs) = unsafe { self.into_get_unchecked_mut_with_context(index) };
        ptrs
    }

    #[inline]
    pub unsafe fn into_get_unchecked_mut_with_context<I>(
        self,
        index: I,
    ) -> (&'c T::Context, I::MutPtrs<'c>)
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let (context, slices) = self.into_slice_mut_ptrs_with_context();
        let ptrs = unsafe { index.get_unchecked_mut(context, slices) };
        (context, ptrs)
    }
}

impl<'c, T> From<&'c T::Context> for SoaSliceMutPtrs<'c, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn from(context: &'c T::Context) -> Self {
        let ptrs = context.ptrs_dangling_mut();
        unsafe { Self::from_parts(context, ptrs, 0) }
    }
}

impl<T> Debug for SoaSliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: Debug,
    for<'any> MutPtrs<'any, T>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            context,
            ref ptrs,
            ref len,
        } = *self;

        f.debug_struct("SoaSliceMutPtrs")
            .field("len", len)
            .field("context", context)
            .field("ptrs", ptrs)
            .finish()
    }
}

impl<T> Clone for SoaSliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self {
            ref ptrs,
            context,
            len,
        } = *self;

        let ptrs = ptrs.clone();
        Self { ptrs, context, len }
    }
}

impl<T> Copy for SoaSliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    for<'any> MutPtrs<'any, T>: Copy,
{
}

impl<T> PartialEq for SoaSliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: PartialEq,
    for<'any> MutPtrs<'any, T>: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let Self { ptrs, context, len } = self;
        *ptrs == other.ptrs && *context == other.context && *len == other.len
    }
}

impl<T> Eq for SoaSliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: Eq,
    for<'any> MutPtrs<'any, T>: Eq,
{
}

impl<T> PartialOrd for SoaSliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: PartialOrd,
    for<'any> MutPtrs<'any, T>: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { ptrs, context, len } = self;

        match len.partial_cmp(&other.len) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match context.partial_cmp(&other.context) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        ptrs.partial_cmp(&other.ptrs)
    }
}

impl<T> Ord for SoaSliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: Ord,
    for<'any> MutPtrs<'any, T>: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { ptrs, context, len } = self;

        match len.cmp(&other.len) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match context.cmp(&other.context) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        ptrs.cmp(&other.ptrs)
    }
}

impl<T> Hash for SoaSliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: Hash,
    for<'any> MutPtrs<'any, T>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { ptrs, context, len } = self;

        len.hash(state);
        context.hash(state);
        ptrs.hash(state);
    }
}

unsafe impl<T> Send for SoaSliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: Send,
    T::Fields: Send,
{
}

unsafe impl<T> Sync for SoaSliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: Sync,
    T::Fields: Sync,
{
}

pub struct SoaSlicesMut<'c, 'a, T>
where
    T: RawSoa + ?Sized + 'a,
{
    ptrs: SoaSliceMutPtrs<'c, T>,
    phantom: PhantomData<&'a ()>,
}

impl<'c, T> SoaSlicesMut<'c, '_, T>
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
        let Self { ptrs, .. } = self;
        ptrs.as_ptrs()
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> MutPtrs<'_, T> {
        let Self { ptrs, .. } = self;
        ptrs.as_mut_ptrs()
    }

    #[inline]
    pub fn slices(&self) -> SoaSlices<'_, '_, T> {
        let Self { ptrs, .. } = self;

        let len = ptrs.len();
        let (context, ptrs) = ptrs.as_ptrs_with_context();
        unsafe { SoaSlices::from_parts(context, ptrs, len) }
    }

    #[inline]
    pub fn slices_mut(&mut self) -> SoaSlicesMut<'_, '_, T> {
        let Self { ptrs, .. } = self;

        let len = ptrs.len();
        let (context, ptrs) = ptrs.as_mut_ptrs_with_context();
        unsafe { SoaSlicesMut::from_parts(context, ptrs, len) }
    }

    #[inline]
    pub fn into_parts(self) -> (&'c T::Context, MutPtrs<'c, T>, usize) {
        let Self { ptrs, .. } = self;
        ptrs.into_parts()
    }

    #[inline]
    pub unsafe fn from_parts(context: &'c T::Context, ptrs: MutPtrs<'c, T>, len: usize) -> Self {
        Self {
            ptrs: unsafe { SoaSliceMutPtrs::from_parts(context, ptrs, len) },
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
    pub unsafe fn get_unchecked_mut<I>(&mut self, index: I) -> I::MutPtrs<'_>
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let (_, ptrs) = unsafe { self.get_unchecked_mut_with_context(index) };
        ptrs
    }

    #[inline]
    pub unsafe fn get_unchecked_mut_with_context<I>(
        &mut self,
        index: I,
    ) -> (&T::Context, I::MutPtrs<'_>)
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let Self { ptrs, .. } = self;
        unsafe { ptrs.get_unchecked_mut_with_context(index) }
    }

    #[inline]
    pub unsafe fn into_get_unchecked_mut<I>(self, index: I) -> I::MutPtrs<'c>
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let (_, ptrs) = unsafe { self.into_get_unchecked_mut_with_context(index) };
        ptrs
    }

    #[inline]
    pub unsafe fn into_get_unchecked_mut_with_context<I>(
        self,
        index: I,
    ) -> (&'c T::Context, I::MutPtrs<'c>)
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let Self { ptrs, .. } = self;
        unsafe { ptrs.into_get_unchecked_mut_with_context(index) }
    }

    #[inline]
    #[track_caller]
    pub fn copy_from_slices(&mut self, src: &SoaSlices<T>)
    where
        T::Fields: Copy,
    {
        let Self { ptrs, .. } = self;

        let len = ptrs.len();
        if len != src.len() {
            len_mismatch_fail(len, src.len());
        }

        // SAFETY: `self` is valid for `self.len()` elements by definition, and `src` was
        // checked to have the same length. The slices cannot overlap because
        // mutable references are exclusive.
        let (context, dst) = ptrs.as_mut_ptrs_with_context();
        unsafe { context.ptrs_copy_nonoverlapping(src.as_ptrs(), dst, len) }
    }

    #[inline]
    #[track_caller]
    pub fn swap(&mut self, a: usize, b: usize) {
        let Self { ptrs, .. } = self;

        let len = ptrs.len();
        if a >= len {
            slice_index_usize_fail(len, a);
        }
        if b >= len {
            slice_index_usize_fail(len, b);
        }

        // call `get_unchecked_mut` directly on slice pointers to avoid creating multiple mutable references
        let (context, slices) = ptrs.as_slice_mut_ptrs_with_context();
        unsafe {
            let a = SoaSlicePtrsIndex::<T>::get_unchecked_mut(a, context, slices.clone());
            let b = SoaSlicePtrsIndex::<T>::get_unchecked_mut(b, context, slices);
            context.ptrs_swap(a, b);
        }
    }
}

impl<'c, 'a, T> SoaSlicesMut<'c, 'a, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    pub fn new(context: &'c T::Context, slices: T::SlicesMut<'c, 'a>) -> Self {
        let slices = T::slices_mut_as_slice_ptrs(context, slices);
        Self {
            ptrs: SoaSliceMutPtrs::new(context, slices),
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
    pub fn as_mut_slices(&mut self) -> T::SlicesMut<'_, '_> {
        let (_, slices) = self.as_mut_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_mut_slices_with_context(&mut self) -> (&T::Context, T::SlicesMut<'_, '_>) {
        let Self { ptrs, .. } = self;

        let (context, slices) = ptrs.as_slice_mut_ptrs_with_context();
        let slices = unsafe { T::slice_mut_ptrs_to_slices(context, slices) };
        (context, slices)
    }

    #[inline]
    pub fn into_slices(self) -> T::SlicesMut<'c, 'a> {
        let (_, slices) = self.into_slices_with_context();
        slices
    }

    #[inline]
    pub fn into_slices_with_context(self) -> (&'c T::Context, T::SlicesMut<'c, 'a>) {
        let Self { ptrs, .. } = self;

        let (context, slices) = ptrs.into_slice_mut_ptrs_with_context();
        let slices = unsafe { T::slice_mut_ptrs_to_slices(context, slices) };
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
        let slices = T::slices_mut_as_slices(context, slices);
        (context, index.get(context, slices))
    }

    #[inline]
    pub fn get_mut<I>(&mut self, index: I) -> Option<I::RefsMut<'_, '_>>
    where
        I: SoaSlicesIndex<T>,
    {
        let (_, refs) = self.get_mut_with_context(index);
        refs
    }

    #[inline]
    pub fn get_mut_with_context<I>(&mut self, index: I) -> (&T::Context, Option<I::RefsMut<'_, '_>>)
    where
        I: SoaSlicesIndex<T>,
    {
        let (context, slices) = self.as_mut_slices_with_context();
        (context, index.get_mut(context, slices))
    }

    #[inline]
    pub fn into_get_mut<I>(self, index: I) -> Option<I::RefsMut<'c, 'a>>
    where
        I: SoaSlicesIndex<T>,
    {
        let (_, refs) = self.into_get_mut_with_context(index);
        refs
    }

    #[inline]
    pub fn into_get_mut_with_context<I>(
        self,
        index: I,
    ) -> (&'c T::Context, Option<I::RefsMut<'c, 'a>>)
    where
        I: SoaSlicesIndex<T>,
    {
        let (context, slices) = self.into_slices_with_context();
        (context, index.get_mut(context, slices))
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
        let slices = T::slices_mut_as_slices(context, slices);
        (context, index.index(context, slices))
    }

    #[inline]
    #[track_caller]
    pub fn index_mut<I>(&mut self, index: I) -> I::RefsMut<'_, '_>
    where
        I: SoaSlicesIndex<T>,
    {
        let (_, refs) = self.index_mut_with_context(index);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn index_mut_with_context<I>(&mut self, index: I) -> (&T::Context, I::RefsMut<'_, '_>)
    where
        I: SoaSlicesIndex<T>,
    {
        let (context, slices) = self.as_mut_slices_with_context();
        (context, index.index_mut(context, slices))
    }

    #[inline]
    #[track_caller]
    pub fn into_index_mut<I>(self, index: I) -> I::RefsMut<'c, 'a>
    where
        I: SoaSlicesIndex<T>,
    {
        let (_, refs) = self.into_index_mut_with_context(index);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn into_index_mut_with_context<I>(self, index: I) -> (&'c T::Context, I::RefsMut<'c, 'a>)
    where
        I: SoaSlicesIndex<T>,
    {
        let (context, slices) = self.into_slices_with_context();
        (context, index.index_mut(context, slices))
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, '_, T> {
        let (_, iter) = self.iter_with_context();
        iter
    }

    #[inline]
    pub fn iter_with_context(&self) -> (&T::Context, Iter<'_, '_, T>) {
        let (context, slices) = self.as_slices_with_context();
        (context, Iter::new(context, slices))
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, '_, T> {
        let (_, iter) = self.iter_mut_with_context();
        iter
    }

    #[inline]
    pub fn iter_mut_with_context(&mut self) -> (&T::Context, IterMut<'_, '_, T>) {
        let (context, slices) = self.as_mut_slices_with_context();
        (context, IterMut::new(context, slices))
    }

    #[inline]
    pub fn into_iter_with_context(self) -> (&'c T::Context, IterMut<'c, 'a, T>) {
        let (context, slices) = self.into_slices_with_context();
        (context, IterMut::new(context, slices))
    }

    #[inline]
    pub fn contains<'me, V>(&'me self, value: V) -> bool
    where
        T::Refs<'me, 'me>: PartialEq<V>,
    {
        let mut iter = self.into_iter();
        iter.any(move |item| item.eq(&value))
    }

    #[inline]
    pub fn sort_unstable_with_permutation<P>(&mut self, permutation: P)
    where
        P: AsMut<[usize]>,
        for<'ca, 'any> T::Refs<'ca, 'any>: Ord,
    {
        self.sort_unstable_with_permutation_by(permutation, |a, b| Ord::cmp(&a, &b));
    }

    #[inline]
    pub fn sort_unstable_with_permutation_by<P, F>(&mut self, permutation: P, mut compare: F)
    where
        P: AsMut<[usize]>,
        for<'ca, 'any> F: FnMut(T::Refs<'ca, 'any>, T::Refs<'ca, 'any>) -> cmp::Ordering,
    {
        self.sort_impl(permutation, |me, permutation| {
            let (context, ptrs, _) = me.slices().into_parts();
            permutation.sort_unstable_by(|&a, &b| {
                let a = unsafe {
                    let ptrs = context.ptrs_add(ptrs.clone(), a);
                    T::ptrs_to_refs(context, ptrs)
                };
                let b = unsafe {
                    let ptrs = context.ptrs_add(ptrs.clone(), b);
                    T::ptrs_to_refs(context, ptrs)
                };
                compare(a, b)
            });
        });
    }

    #[inline]
    pub fn sort_unstable_with_permutation_by_key<P, K, F>(&mut self, permutation: P, mut f: F)
    where
        P: AsMut<[usize]>,
        F: FnMut(T::Refs<'_, '_>) -> K,
        K: Ord,
    {
        self.sort_impl(permutation, |me, permutation| {
            let (context, ptrs, _) = me.slices().into_parts();
            permutation.sort_unstable_by_key(|&index| unsafe {
                let ptrs = context.ptrs_add(ptrs.clone(), index);
                let refs = T::ptrs_to_refs(context, ptrs);
                f(refs)
            });
        });
    }

    pub(crate) fn sort_impl<P, F>(&mut self, mut permutation: P, f: F)
    where
        P: AsMut<[usize]>,
        F: FnOnce(&mut Self, &mut [usize]),
    {
        #[inline(never)]
        #[cold]
        #[track_caller]
        fn permutation_len_fail(permutation_len: usize, len: usize) -> ! {
            panic!("permutation must be at least {len} long, but its length is {permutation_len}")
        }

        let len = self.len();
        let permutation = permutation.as_mut();
        if permutation.len() < len {
            permutation_len_fail(permutation.len(), len);
        }

        let context = self.context();
        if is_zst::<T>(context) || len < 2 {
            return;
        }

        f(self, permutation);

        // were taken from `sort_by_cached_key()` method of slice primitive
        for src in 0..len {
            let mut dst = permutation[src];
            while dst < src {
                dst = permutation[dst];
            }
            permutation[src] = dst;
            self.swap(src, dst);
        }
    }
}

impl<T> SoaSlicesMut<'_, '_, T>
where
    T: Soa + SoaWrite,
{
    #[inline]
    #[track_caller]
    pub fn clone_from_slices(&mut self, src: &SoaSlices<T>)
    where
        for<'c, 'a> T::Refs<'c, 'a>: SoaToOwned<'c, 'a, Owned = T>,
    {
        let len = self.len();
        if len != src.len() {
            len_mismatch_fail(len, src.len());
        }

        for index in 0..len {
            let (context, dst) = unsafe { self.get_unchecked_mut_with_context(index) };
            unsafe { context.ptrs_drop_in_place(dst.clone()) }

            let src = unsafe { T::ptrs_to_refs(context, src.get_unchecked(index)) };
            unsafe { T::write(context, dst, src.to_owned(context)) }
        }
    }
}

#[inline(never)]
#[cold]
#[track_caller]
fn len_mismatch_fail(dst_len: usize, src_len: usize) -> ! {
    panic!("source slice length ({src_len}) does not match destination slice length ({dst_len})")
}

impl<'c, T> From<SoaSlicesMut<'c, '_, T>> for SoaSlicePtrs<'c, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    fn from(slices: SoaSlicesMut<'c, '_, T>) -> Self {
        let SoaSlicesMut { ptrs, .. } = slices;
        ptrs.cast_const()
    }
}

impl<'c, T> From<SoaSlicesMut<'c, '_, T>> for SoaSliceMutPtrs<'c, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    fn from(slices: SoaSlicesMut<'c, '_, T>) -> Self {
        let SoaSlicesMut { ptrs, .. } = slices;
        ptrs
    }
}

impl<'c, 'a, T> From<SoaSlicesMut<'c, 'a, T>> for SoaSlices<'c, 'a, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    fn from(slices: SoaSlicesMut<'c, 'a, T>) -> Self {
        let (context, ptrs, len) = slices.into_parts();

        let ptrs = context.ptrs_cast_const(ptrs);
        unsafe { Self::from_parts(context, ptrs, len) }
    }
}

impl<'c, T> From<&'c T::Context> for SoaSlicesMut<'c, '_, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    fn from(context: &'c T::Context) -> Self {
        let ptrs = context.ptrs_dangling_mut();
        unsafe { Self::from_parts(context, ptrs, 0) }
    }
}

impl<T> Debug for SoaSlicesMut<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("SoaSlicesMut").field(&slices).finish()
    }
}

impl<T> AsRef<Self> for SoaSlicesMut<'_, '_, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<T, U> AsRef<[U]> for SoaSlicesMut<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Into<&'any [U]>,
{
    #[inline]
    fn as_ref(&self) -> &[U] {
        self.as_slices().into()
    }
}

impl<T> AsMut<Self> for SoaSlicesMut<'_, '_, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<T, U> AsMut<[U]> for SoaSlicesMut<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::SlicesMut<'c, 'any>: Into<&'any mut [U]>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [U] {
        self.as_mut_slices().into()
    }
}

impl<T> Eq for SoaSlicesMut<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Eq,
{
}

impl<T> Ord for SoaSlicesMut<'_, '_, T>
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

impl<T> Hash for SoaSlicesMut<'_, '_, T>
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

impl<T, U, I> Index<I> for SoaSlicesMut<'_, '_, T>
where
    T: Soa + ?Sized,
    U: ?Sized,
    for<'c, 'any> I: IndexHelper<'c, 'any, T, Output = U>,
{
    type Output = U;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        SoaSlicesMut::index(self, index)
    }
}

impl<T, U, I> IndexMut<I> for SoaSlicesMut<'_, '_, T>
where
    T: Soa + ?Sized,
    U: ?Sized,
    for<'c, 'any> I: IndexHelperMut<'c, 'any, T, Output = U>,
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        SoaSlicesMut::index_mut(self, index)
    }
}

impl<'r, T> IntoIterator for &'r SoaSlicesMut<'_, '_, T>
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

impl<'r, T> IntoIterator for &'r mut SoaSlicesMut<'_, '_, T>
where
    T: Soa + ?Sized,
{
    type Item = T::RefsMut<'r, 'r>;
    type IntoIter = IterMut<'r, 'r, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'c, 'a, T> IntoIterator for SoaSlicesMut<'c, 'a, T>
where
    T: Soa + ?Sized,
{
    type Item = T::RefsMut<'c, 'a>;
    type IntoIter = IterMut<'c, 'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let (_, iter) = self.into_iter_with_context();
        iter
    }
}
