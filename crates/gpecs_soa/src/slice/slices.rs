use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use crate::{
    ptr::is_zst,
    traits::{Soa, SoaToOwned},
    wrappers::{MutPtrs, Ptrs},
};

use super::{IndexHelper, IndexHelperMut, Iter, IterMut, SoaSliceIndex, slice_index_usize_fail};

pub struct SoaSlices<'c, 'a, T>
where
    T: Soa + 'a,
{
    context: &'c T::Context,
    ptrs: Ptrs<'c, T>,
    len: usize,
    phantom: PhantomData<&'a ()>,
}

impl<'c, 'a, T> SoaSlices<'c, 'a, T>
where
    T: Soa,
{
    #[inline]
    pub fn new(context: &'c T::Context, slices: T::Slices<'c, 'a>) -> Self {
        let slices = T::slices_as_slice_ptrs(context, slices);
        Self {
            context,
            len: T::slice_ptrs_len(context, &slices),
            ptrs: Ptrs::new(T::slice_ptrs_as_ptrs(context, slices)),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn context(&self) -> &T::Context {
        let Self { context, .. } = self;
        context
    }

    #[inline]
    pub fn as_slices(&self) -> T::Slices<'_, '_> {
        let (_, slices) = self.as_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_slices_with_context(&self) -> (&T::Context, T::Slices<'_, '_>) {
        let Self {
            context,
            ref ptrs,
            len,
            ..
        } = *self;

        let slices = T::slices_from_raw_parts(context, ptrs.as_inner().clone(), len);
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
        let Self {
            context, ptrs, len, ..
        } = self;

        let slices = T::slices_from_raw_parts(context, ptrs.into_inner(), len);
        let slices = unsafe { T::slice_ptrs_to_slices(context, slices) };
        (context, slices)
    }

    #[inline]
    pub fn as_ptrs(&self) -> T::Ptrs<'_> {
        let Self { ptrs, .. } = self;
        ptrs.as_inner().clone()
    }

    #[inline]
    pub fn into_parts(self) -> (&'c T::Context, T::Ptrs<'c>, usize) {
        let Self {
            context, ptrs, len, ..
        } = self;
        (context, ptrs.into_inner(), len)
    }

    #[inline]
    pub unsafe fn from_parts(context: &'c T::Context, ptrs: T::Ptrs<'c>, len: usize) -> Self {
        let ptrs = Ptrs::new(ptrs);
        Self {
            context,
            ptrs,
            len,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn get<I>(&self, index: I) -> Option<I::Refs<'_, '_>>
    where
        I: SoaSliceIndex<T>,
    {
        let (_, refs) = self.get_with_context(index);
        refs
    }

    #[inline]
    pub fn get_with_context<I>(&self, index: I) -> (&T::Context, Option<I::Refs<'_, '_>>)
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.as_slices_with_context();
        (context, index.get(context, slices))
    }

    #[inline]
    pub fn into_get<I>(self, index: I) -> Option<I::Refs<'c, 'a>>
    where
        I: SoaSliceIndex<T>,
    {
        let (_, refs) = self.into_get_with_context(index);
        refs
    }

    #[inline]
    pub fn into_get_with_context<I>(self, index: I) -> (&'c T::Context, Option<I::Refs<'c, 'a>>)
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.into_slices_with_context();
        (context, index.get(context, slices))
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn get_unchecked<I>(&self, index: I) -> I::Ptrs<'_>
    where
        I: SoaSliceIndex<T>,
    {
        let (_, ptrs) = unsafe { self.get_unchecked_with_context(index) };
        ptrs
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn get_unchecked_with_context<I>(&self, index: I) -> (&T::Context, I::Ptrs<'_>)
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.as_slices_with_context();
        let slices = T::slices_as_slice_ptrs(context, slices);
        let ptrs = unsafe { index.get_unchecked(context, slices) };
        (context, ptrs)
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn into_get_unchecked<I>(self, index: I) -> I::Ptrs<'c>
    where
        I: SoaSliceIndex<T>,
    {
        let (_, ptrs) = unsafe { self.into_get_unchecked_with_context(index) };
        ptrs
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn into_get_unchecked_with_context<I>(
        self,
        index: I,
    ) -> (&'c T::Context, I::Ptrs<'c>)
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.into_slices_with_context();
        let slices = T::slices_as_slice_ptrs(context, slices);
        let ptrs = unsafe { index.get_unchecked(context, slices) };
        (context, ptrs)
    }

    #[inline]
    #[track_caller]
    pub fn index<I>(&self, index: I) -> I::Refs<'_, '_>
    where
        I: SoaSliceIndex<T>,
    {
        let (_, refs) = self.index_with_context(index);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn index_with_context<I>(&self, index: I) -> (&T::Context, I::Refs<'_, '_>)
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.as_slices_with_context();
        (context, index.index(context, slices))
    }

    #[inline]
    #[track_caller]
    pub fn into_index<I>(self, index: I) -> I::Refs<'c, 'a>
    where
        I: SoaSliceIndex<T>,
    {
        let (_, refs) = self.into_index_with_context(index);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn into_index_with_context<I>(self, index: I) -> (&'c T::Context, I::Refs<'c, 'a>)
    where
        I: SoaSliceIndex<T>,
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
        let context = self.context();
        (context, Iter::new(self.clone()))
    }

    #[inline]
    pub fn into_iter_with_context(self) -> (&'c T::Context, Iter<'c, 'a, T>) {
        let Self { context, .. } = self;
        (context, Iter::new(self))
    }

    #[inline]
    pub fn contains(&self, value: &T) -> bool
    where
        T::Refs<'c, 'a>: PartialEq<T>,
    {
        let mut iter = self.clone().into_iter();
        iter.any(|item| item == *value)
    }

    #[inline]
    pub fn contains_by_refs<'cr, 'r>(&self, refs: T::Refs<'cr, 'r>) -> bool
    where
        T::Refs<'c, 'a>: PartialEq<T::Refs<'cr, 'r>>,
    {
        let mut iter = self.clone().into_iter();
        iter.any(|item| item == refs)
    }
}

impl<'c, 'a, T> From<&'c T::Context> for SoaSlices<'c, 'a, T>
where
    T: Soa,
{
    fn from(context: &'c T::Context) -> Self {
        let ptrs = T::ptrs_dangling(context);
        let ptrs = T::ptrs_cast_const(context, ptrs);
        unsafe { Self::from_parts(context, ptrs, 0) }
    }
}

impl<T> Debug for SoaSlices<'_, '_, T>
where
    T: Soa,
    for<'c, 'any> T::Slices<'c, 'any>: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("SoaSlices").field(&slices).finish()
    }
}

impl<'c, 'a, T> AsRef<SoaSlices<'c, 'a, T>> for SoaSlices<'c, 'a, T>
where
    T: Soa,
{
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<T, U> AsRef<[U]> for SoaSlices<'_, '_, T>
where
    for<'c, 'any> T: Soa<Slices<'c, 'any> = &'any [U]> + 'any,
{
    fn as_ref(&self) -> &[U] {
        self.as_slices()
    }
}

impl<T> PartialEq for SoaSlices<'_, '_, T>
where
    T: Soa,
    for<'c, 'any> T::Slices<'c, 'any>: PartialEq,
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

impl<T> Eq for SoaSlices<'_, '_, T>
where
    T: Soa,
    for<'c, 'any> T::Slices<'c, 'any>: Eq,
{
}

impl<T> PartialOrd for SoaSlices<'_, '_, T>
where
    T: Soa,
    for<'c, 'any> T::Slices<'c, 'any>: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        PartialOrd::partial_cmp(&self.as_slices(), &other.as_slices())
    }
}

impl<T> Ord for SoaSlices<'_, '_, T>
where
    T: Soa,
    for<'c, 'any> T::Slices<'c, 'any>: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        Ord::cmp(&self.as_slices(), &other.as_slices())
    }
}

impl<T> Hash for SoaSlices<'_, '_, T>
where
    T: Soa,
    for<'c, 'any> T::Slices<'c, 'any>: Hash,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let len = self.len();
        state.write_usize(len);

        let slices = self.as_slices();
        slices.hash(state);
    }
}

impl<T> Clone for SoaSlices<'_, '_, T>
where
    T: Soa,
{
    fn clone(&self) -> Self {
        let Self {
            context,
            ref ptrs,
            len,
            phantom,
        } = *self;
        Self {
            context,
            ptrs: ptrs.clone(),
            len,
            phantom,
        }
    }
}

impl<T> Copy for SoaSlices<'_, '_, T>
where
    T: Soa,
    for<'any> T::Ptrs<'any>: Copy,
{
}

impl<T, U, I> Index<I> for SoaSlices<'_, '_, T>
where
    T: Soa,
    U: ?Sized,
    for<'c, 'any> I: IndexHelper<'c, 'any, T, Output = U>,
{
    type Output = U;

    fn index(&self, index: I) -> &Self::Output {
        SoaSlices::index(self, index)
    }
}

impl<'r, T> IntoIterator for &'r SoaSlices<'_, '_, T>
where
    T: Soa,
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
    T: Soa,
{
    type Item = T::Refs<'c, 'a>;
    type IntoIter = Iter<'c, 'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

unsafe impl<T> Send for SoaSlices<'_, '_, T>
where
    T: Soa,
    T::Fields: Send,
    T::Context: Send,
{
}

unsafe impl<T> Sync for SoaSlices<'_, '_, T>
where
    T: Soa,
    T::Fields: Sync,
    T::Context: Sync,
{
}

pub struct SoaSlicesMut<'c, 'a, T>
where
    T: Soa + 'a,
{
    context: &'c T::Context,
    ptrs: MutPtrs<'c, T>,
    len: usize,
    phantom: PhantomData<&'a ()>,
}

impl<'c, 'a, T> SoaSlicesMut<'c, 'a, T>
where
    T: Soa,
{
    #[inline]
    pub fn new(context: &'c T::Context, slices: T::SlicesMut<'c, 'a>) -> Self {
        let slices = T::slices_mut_as_slice_ptrs(context, slices);
        Self {
            context,
            len: T::slice_mut_ptrs_len(context, &slices),
            ptrs: MutPtrs::new(T::slice_mut_ptrs_as_ptrs(context, slices)),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn context(&self) -> &T::Context {
        let Self { context, .. } = self;
        context
    }

    #[inline]
    pub fn as_slices(&self) -> T::Slices<'_, '_> {
        let (_, slices) = self.as_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_slices_with_context(&self) -> (&T::Context, T::Slices<'_, '_>) {
        let Self {
            context,
            ref ptrs,
            len,
            ..
        } = *self;

        let slices = T::slices_from_raw_parts_mut(context, ptrs.as_inner().clone(), len);
        let slices = T::slice_ptrs_cast_const(context, slices);
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
        let Self {
            context,
            ref ptrs,
            len,
            ..
        } = *self;

        let slices = T::slices_from_raw_parts_mut(context, ptrs.as_inner().clone(), len);
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
        let Self {
            context, ptrs, len, ..
        } = self;

        let slices = T::slices_from_raw_parts_mut(context, ptrs.into_inner(), len);
        let slices = unsafe { T::slice_mut_ptrs_to_slices(context, slices) };
        (context, slices)
    }

    #[inline]
    pub fn as_ptrs(&self) -> T::Ptrs<'_> {
        let Self {
            context, ref ptrs, ..
        } = *self;
        T::ptrs_cast_const(context, ptrs.as_inner().clone())
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> T::MutPtrs<'_> {
        let Self { ptrs, .. } = self;
        ptrs.as_inner().clone()
    }

    #[inline]
    pub(crate) unsafe fn as_parts(&self) -> (&'c T::Context, T::MutPtrs<'c>, usize) {
        let Self {
            context,
            ref ptrs,
            len,
            ..
        } = *self;
        (context, ptrs.as_inner().clone(), len)
    }

    #[inline]
    pub fn into_parts(self) -> (&'c T::Context, T::MutPtrs<'c>, usize) {
        let Self {
            context, ptrs, len, ..
        } = self;
        (context, ptrs.into_inner(), len)
    }

    #[inline]
    pub unsafe fn from_parts(context: &'c T::Context, ptrs: T::MutPtrs<'c>, len: usize) -> Self {
        let ptrs = MutPtrs::new(ptrs);
        Self {
            context,
            ptrs,
            len,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn get<I>(&self, index: I) -> Option<I::Refs<'_, '_>>
    where
        I: SoaSliceIndex<T>,
    {
        let (_, refs) = self.get_with_context(index);
        refs
    }

    #[inline]
    pub fn get_with_context<I>(&self, index: I) -> (&T::Context, Option<I::Refs<'_, '_>>)
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.as_slices_with_context();
        (context, index.get(context, slices))
    }

    #[inline]
    pub fn into_get<I>(self, index: I) -> Option<I::Refs<'c, 'a>>
    where
        I: SoaSliceIndex<T>,
    {
        let (_, refs) = self.into_get_with_context(index);
        refs
    }

    #[inline]
    pub fn into_get_with_context<I>(self, index: I) -> (&'c T::Context, Option<I::Refs<'c, 'a>>)
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.into_slices_with_context();
        let slices = T::slices_mut_as_slices(context, slices);
        (context, index.get(context, slices))
    }

    #[inline]
    pub fn get_mut<I>(&mut self, index: I) -> Option<I::RefsMut<'_, '_>>
    where
        I: SoaSliceIndex<T>,
    {
        let (_, refs_mut) = self.get_mut_with_context(index);
        refs_mut
    }

    #[inline]
    pub fn get_mut_with_context<I>(&mut self, index: I) -> (&T::Context, Option<I::RefsMut<'_, '_>>)
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.as_mut_slices_with_context();
        (context, index.get_mut(context, slices))
    }

    #[inline]
    pub fn into_get_mut<I>(self, index: I) -> Option<I::RefsMut<'c, 'a>>
    where
        I: SoaSliceIndex<T>,
    {
        let (_, refs_mut) = self.into_get_mut_with_context(index);
        refs_mut
    }

    #[inline]
    pub fn into_get_mut_with_context<I>(
        self,
        index: I,
    ) -> (&'c T::Context, Option<I::RefsMut<'c, 'a>>)
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.into_slices_with_context();
        (context, index.get_mut(context, slices))
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn get_unchecked<I>(&self, index: I) -> I::Ptrs<'_>
    where
        I: SoaSliceIndex<T>,
    {
        let (_, ptrs) = unsafe { self.get_unchecked_with_context(index) };
        ptrs
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn get_unchecked_with_context<I>(&self, index: I) -> (&T::Context, I::Ptrs<'_>)
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.as_slices_with_context();
        let slices = T::slices_as_slice_ptrs(context, slices);
        let ptrs = unsafe { index.get_unchecked(context, slices) };
        (context, ptrs)
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn into_get_unchecked<I>(self, index: I) -> I::Ptrs<'c>
    where
        I: SoaSliceIndex<T>,
    {
        let (_, ptrs) = unsafe { self.into_get_unchecked_with_context(index) };
        ptrs
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn into_get_unchecked_with_context<I>(
        self,
        index: I,
    ) -> (&'c T::Context, I::Ptrs<'c>)
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.into_slices_with_context();
        let slices = T::slices_mut_as_slices(context, slices);
        let slices = T::slices_as_slice_ptrs(context, slices);
        let ptrs = unsafe { index.get_unchecked(context, slices) };
        (context, ptrs)
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn get_unchecked_mut<I>(&mut self, index: I) -> I::MutPtrs<'_>
    where
        I: SoaSliceIndex<T>,
    {
        let (_, ptrs) = unsafe { self.get_unchecked_mut_with_context(index) };
        ptrs
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn get_unchecked_mut_with_context<I>(
        &mut self,
        index: I,
    ) -> (&T::Context, I::MutPtrs<'_>)
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.as_mut_slices_with_context();
        let slices = T::slices_mut_as_slice_ptrs(context, slices);
        let ptrs = unsafe { index.get_unchecked_mut(context, slices) };
        (context, ptrs)
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn into_get_unchecked_mut<I>(self, index: I) -> I::MutPtrs<'c>
    where
        I: SoaSliceIndex<T>,
    {
        let (_, ptrs) = unsafe { self.into_get_unchecked_mut_with_context(index) };
        ptrs
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn into_get_unchecked_mut_with_context<I>(
        self,
        index: I,
    ) -> (&'c T::Context, I::MutPtrs<'c>)
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.into_slices_with_context();
        let slices = T::slices_mut_as_slice_ptrs(context, slices);
        let ptrs = unsafe { index.get_unchecked_mut(context, slices) };
        (context, ptrs)
    }

    #[inline]
    #[track_caller]
    pub fn index<I>(&self, index: I) -> I::Refs<'_, '_>
    where
        I: SoaSliceIndex<T>,
    {
        let (_, refs) = self.index_with_context(index);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn index_with_context<I>(&self, index: I) -> (&T::Context, I::Refs<'_, '_>)
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.as_slices_with_context();
        (context, index.index(context, slices))
    }

    #[inline]
    #[track_caller]
    pub fn into_index<I>(self, index: I) -> I::Refs<'c, 'a>
    where
        I: SoaSliceIndex<T>,
    {
        let (_, refs) = self.into_index_with_context(index);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn into_index_with_context<I>(self, index: I) -> (&'c T::Context, I::Refs<'c, 'a>)
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.into_slices_with_context();
        let slices = T::slices_mut_as_slices(context, slices);
        (context, index.index(context, slices))
    }

    #[inline]
    #[track_caller]
    pub fn index_mut<I>(&mut self, index: I) -> I::RefsMut<'_, '_>
    where
        I: SoaSliceIndex<T>,
    {
        let (_, refs_mut) = self.index_mut_with_context(index);
        refs_mut
    }

    #[inline]
    #[track_caller]
    pub fn index_mut_with_context<I>(&mut self, index: I) -> (&T::Context, I::RefsMut<'_, '_>)
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.as_mut_slices_with_context();
        (context, index.index_mut(context, slices))
    }

    #[inline]
    #[track_caller]
    pub fn into_index_mut<I>(self, index: I) -> I::RefsMut<'c, 'a>
    where
        I: SoaSliceIndex<T>,
    {
        let (_, refs_mut) = self.into_index_mut_with_context(index);
        refs_mut
    }

    #[inline]
    #[track_caller]
    pub fn into_index_mut_with_context<I>(self, index: I) -> (&'c T::Context, I::RefsMut<'c, 'a>)
    where
        I: SoaSliceIndex<T>,
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
        let (context, ptrs, len) = unsafe { self.as_parts() };
        let ptrs = T::ptrs_cast_const(context, ptrs);
        let slices = unsafe { SoaSlices::from_parts(context, ptrs, len) };
        (context, Iter::new(slices))
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, '_, T> {
        let (_, iter) = self.iter_mut_with_context();
        iter
    }

    #[inline]
    pub fn iter_mut_with_context(&mut self) -> (&T::Context, IterMut<'_, '_, T>) {
        let (context, ptrs, len) = unsafe { self.as_parts() };
        let slices = unsafe { Self::from_parts(context, ptrs.clone(), len) };
        (context, IterMut::new(slices))
    }

    #[inline]
    pub fn into_iter_with_context(self) -> (&'c T::Context, Iter<'c, 'a, T>) {
        let Self { context, .. } = self;
        (context, Iter::new(self.into()))
    }

    #[inline]
    pub fn into_iter_mut_with_context(self) -> (&'c T::Context, IterMut<'c, 'a, T>) {
        let Self { context, .. } = self;
        (context, IterMut::new(self))
    }

    #[inline]
    pub fn contains(&self, value: &T) -> bool
    where
        T::Refs<'c, 'a>: PartialEq<T>,
    {
        let (context, ptrs, len) = unsafe { self.as_parts() };
        let ptrs = T::ptrs_cast_const(context, ptrs.clone());
        let slices = unsafe { SoaSlices::from_parts(context, ptrs, len) };
        slices.contains(value)
    }

    #[inline]
    pub fn contains_by_refs<'cr, 'r>(&self, refs: T::Refs<'cr, 'r>) -> bool
    where
        T::Refs<'c, 'a>: PartialEq<T::Refs<'cr, 'r>>,
    {
        let (context, ptrs, len) = unsafe { self.as_parts() };
        let ptrs = T::ptrs_cast_const(context, ptrs.clone());
        let slices = unsafe { SoaSlices::<T>::from_parts(context, ptrs, len) };
        slices.contains_by_refs(refs)
    }

    #[inline]
    #[track_caller]
    pub fn clone_from_slices(&mut self, src: SoaSlices<T>)
    where
        for<'ca, 'any> T::Refs<'ca, 'any>: SoaToOwned<'ca, 'any, Owned = T>,
    {
        let len = self.len();
        if len != src.len() {
            len_mismatch_fail(len, src.len());
        }

        for index in 0..len {
            unsafe {
                let (context, dst) = self.get_unchecked_mut_with_context(index);
                let src = T::ptrs_to_refs(context, src.get_unchecked(index));
                T::ptrs_drop_in_place(context, dst.clone());
                src.clone_into_ptrs(context, dst);
            }
        }
    }

    #[inline]
    #[track_caller]
    pub fn copy_from_slices(&mut self, src: SoaSlices<T>)
    where
        T::Fields: Copy,
    {
        let len = self.len();
        if len != src.len() {
            len_mismatch_fail(len, src.len());
        }

        // SAFETY: `self` is valid for `self.len()` elements by definition, and `src` was
        // checked to have the same length. The slices cannot overlap because
        // mutable references are exclusive.
        unsafe {
            let dst = self.ptrs.as_inner().clone();
            let context = self.context;
            T::ptrs_copy_nonoverlapping(context, src.as_ptrs(), dst, len);
        }
    }

    #[inline]
    #[track_caller]
    pub fn swap(&mut self, a: usize, b: usize) {
        let len = self.len();
        if a >= len {
            slice_index_usize_fail(len, a);
        }
        if b >= len {
            slice_index_usize_fail(len, b);
        }

        // call `get_unchecked_mut` directly on slice pointers to avoid creating multiple mutable references
        let (context, slices) = self.as_mut_slices_with_context();
        let slices = T::slices_mut_as_slice_ptrs(context, slices);
        unsafe {
            let a = SoaSliceIndex::<T>::get_unchecked_mut(a, context, slices.clone());
            let b = SoaSliceIndex::<T>::get_unchecked_mut(b, context, slices);
            T::ptrs_swap(context, a, b)
        }
    }

    #[inline]
    pub fn sort_unstable_with_permutation(&mut self, permutation: &mut [usize])
    where
        for<'ca, 'any> T::Refs<'ca, 'any>: Ord,
    {
        self.sort_unstable_with_permutation_by(permutation, |a, b| Ord::cmp(&a, &b))
    }

    #[inline]
    pub fn sort_unstable_with_permutation_by<F>(
        &mut self,
        permutation: &mut [usize],
        mut compare: F,
    ) where
        for<'ca, 'any> F: FnMut(T::Refs<'ca, 'any>, T::Refs<'ca, 'any>) -> cmp::Ordering,
    {
        self.sort_impl(permutation, |me, permutation| {
            let (context, ptrs, _) = unsafe { me.as_parts() };
            permutation.sort_unstable_by(|&a, &b| {
                let a = unsafe {
                    let ptrs = T::ptrs_add_mut(context, ptrs.clone(), a);
                    let ptrs = T::ptrs_cast_const(context, ptrs);
                    T::ptrs_to_refs(context, ptrs)
                };
                let b = unsafe {
                    let ptrs = T::ptrs_add_mut(context, ptrs.clone(), b);
                    let ptrs = T::ptrs_cast_const(context, ptrs);
                    T::ptrs_to_refs(context, ptrs)
                };
                compare(a, b)
            })
        })
    }

    #[inline]
    pub fn sort_unstable_with_permutation_by_key<K, F>(
        &mut self,
        permutation: &mut [usize],
        mut f: F,
    ) where
        F: FnMut(T::Refs<'_, '_>) -> K,
        K: Ord,
    {
        self.sort_impl(permutation, |me, permutation| {
            let (context, ptrs, _) = unsafe { me.as_parts() };
            permutation.sort_unstable_by_key(|&index| unsafe {
                let ptrs = T::ptrs_add_mut(context, ptrs.clone(), index);
                let ptrs = T::ptrs_cast_const(context, ptrs);
                let refs = T::ptrs_to_refs(context, ptrs);
                f(refs)
            })
        })
    }

    pub(crate) fn sort_impl<F>(&mut self, permutation: &mut [usize], f: F)
    where
        F: FnOnce(&mut Self, &mut [usize]),
    {
        #[inline(never)]
        #[cold]
        #[track_caller]
        fn permutation_len_fail(permutation_len: usize, len: usize) -> ! {
            panic!("permutation must be at least {len} long, but its length is {permutation_len}")
        }

        let len = self.len();
        let context = self.context();
        if is_zst::<T>(context) || len < 2 {
            return;
        }
        if permutation.len() < len {
            permutation_len_fail(permutation.len(), len);
        }

        f(self, permutation);

        for src in 0..len {
            let dst = permutation[src];
            if src == dst {
                continue;
            }
            self.swap(src, dst);
            permutation.swap(src, dst);
        }
    }
}

#[inline(never)]
#[cold]
#[track_caller]
fn len_mismatch_fail(dst_len: usize, src_len: usize) -> ! {
    panic!("source slice length ({src_len}) does not match destination slice length ({dst_len})")
}

impl<'c, 'a, T> From<SoaSlicesMut<'c, 'a, T>> for SoaSlices<'c, 'a, T>
where
    T: Soa,
{
    fn from(slices: SoaSlicesMut<'c, 'a, T>) -> Self {
        let (context, ptrs, len) = slices.into_parts();

        let ptrs = T::ptrs_cast_const(context, ptrs);
        unsafe { Self::from_parts(context, ptrs, len) }
    }
}

impl<'c, 'a, T> From<&'c T::Context> for SoaSlicesMut<'c, 'a, T>
where
    T: Soa,
{
    fn from(context: &'c T::Context) -> Self {
        let ptrs = T::ptrs_dangling(context);
        unsafe { Self::from_parts(context, ptrs, 0) }
    }
}

impl<T> Debug for SoaSlicesMut<'_, '_, T>
where
    T: Soa,
    for<'c, 'any> T::Slices<'c, 'any>: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("SoaSlicesMut").field(&slices).finish()
    }
}

impl<'c, 'a, T> AsRef<SoaSlicesMut<'c, 'a, T>> for SoaSlicesMut<'c, 'a, T>
where
    T: Soa,
{
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<T, U> AsRef<[U]> for SoaSlicesMut<'_, '_, T>
where
    for<'c, 'any> T: Soa<Slices<'c, 'any> = &'any [U]> + 'any,
{
    fn as_ref(&self) -> &[U] {
        self.as_slices()
    }
}

impl<'c, 'a, T> AsMut<SoaSlicesMut<'c, 'a, T>> for SoaSlicesMut<'c, 'a, T>
where
    T: Soa,
{
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<T, U> AsMut<[U]> for SoaSlicesMut<'_, '_, T>
where
    for<'c, 'any> T: Soa<SlicesMut<'c, 'any> = &'any mut [U]> + 'any,
{
    fn as_mut(&mut self) -> &mut [U] {
        self.as_mut_slices()
    }
}

impl<T> PartialEq for SoaSlicesMut<'_, '_, T>
where
    T: Soa,
    for<'c, 'any> T::Slices<'c, 'any>: PartialEq,
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

impl<T> Eq for SoaSlicesMut<'_, '_, T>
where
    T: Soa,
    for<'c, 'any> T::Slices<'c, 'any>: Eq,
{
}

impl<T> PartialOrd for SoaSlicesMut<'_, '_, T>
where
    T: Soa,
    for<'c, 'any> T::Slices<'c, 'any>: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        PartialOrd::partial_cmp(&self.as_slices(), &other.as_slices())
    }
}

impl<T> Ord for SoaSlicesMut<'_, '_, T>
where
    T: Soa,
    for<'c, 'any> T::Slices<'c, 'any>: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        Ord::cmp(&self.as_slices(), &other.as_slices())
    }
}

impl<T> Hash for SoaSlicesMut<'_, '_, T>
where
    T: Soa,
    for<'c, 'any> T::Slices<'c, 'any>: Hash,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let len = self.len();
        state.write_usize(len);

        let slices = self.as_slices();
        slices.hash(state);
    }
}

impl<T, U, I> Index<I> for SoaSlicesMut<'_, '_, T>
where
    T: Soa,
    U: ?Sized,
    for<'c, 'any> I: IndexHelper<'c, 'any, T, Output = U>,
{
    type Output = U;

    fn index(&self, index: I) -> &Self::Output {
        SoaSlicesMut::index(self, index)
    }
}

impl<T, U, I> IndexMut<I> for SoaSlicesMut<'_, '_, T>
where
    T: Soa,
    U: ?Sized,
    for<'c, 'any> I: IndexHelperMut<'c, 'any, T, Output = U>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        SoaSlicesMut::index_mut(self, index)
    }
}

impl<'r, T> IntoIterator for &'r SoaSlicesMut<'_, '_, T>
where
    T: Soa,
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
    T: Soa,
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
    T: Soa,
{
    type Item = T::RefsMut<'c, 'a>;
    type IntoIter = IterMut<'c, 'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IterMut::new(self)
    }
}

unsafe impl<T> Send for SoaSlicesMut<'_, '_, T>
where
    T: Soa,
    T::Fields: Send,
    T::Context: Send,
{
}

unsafe impl<T> Sync for SoaSlicesMut<'_, '_, T>
where
    T: Soa,
    T::Fields: Sync,
    T::Context: Sync,
{
}
