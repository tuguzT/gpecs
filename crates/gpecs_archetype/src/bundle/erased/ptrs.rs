use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use gpecs_component::{
    erased::ErasedComponentPtr,
    registry::{ComponentId, traits::WithComponentId},
};
use gpecs_soa_erased::{
    CovariantFieldLayouts, ErasedSoaPtrs, ErasedSoaPtrsIter,
    error::FromFieldsLayoutsError,
    ptr::slice::{CastMut, ConstSliceItemPtr},
    soa::field::{FieldLayouts, FieldLayoutsIter, FieldLayoutsOutput, FieldLayoutsOwned},
    storage::AlignedStorageFromLayout,
};

use crate::{
    bundle::erased::{
        ErasedBundleKind, ErasedBundleMutPtrs, ErasedBundleRefs,
        traits::{
            ErasedArchetypeIterator, ErasedArchetypeKind, ErasedBundleDrop,
            IntoErasedArchetypeIterator,
        },
    },
    erased::ErasedArchetypeView,
};

pub struct ErasedBundlePtrs<D, P>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    inner: ErasedSoaPtrs<D, P>,
}

impl<D, P> ErasedBundlePtrs<D, P>
where
    P: ConstSliceItemPtr,
{
    #[inline]
    pub unsafe fn from_inner(inner: ErasedSoaPtrs<D, P>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn into_inner(self) -> ErasedSoaPtrs<D, P> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedBundleMutPtrs<D, CastMut<P>> {
        let Self { inner } = self;

        let inner = inner.cast_mut();
        unsafe { ErasedBundleMutPtrs::from_inner(inner) }
    }

    #[inline]
    pub unsafe fn as_ref_unchecked<'a>(self) -> ErasedBundleRefs<'a, D, P> {
        unsafe { ErasedBundleRefs::from_ptrs(self) }
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { inner } = self;

        let inner = unsafe { inner.add(count) };
        unsafe { Self::from_inner(inner) }
    }
}

impl<D, P> ErasedBundlePtrs<D, P>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub unsafe fn as_inner(&self) -> &ErasedSoaPtrs<D, P> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn as_buffer(&self) -> *const [P::Item] {
        let Self { inner } = self;
        inner.as_buffer()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { inner } = self;
        inner.capacity()
    }

    #[inline]
    pub fn offset(&self) -> usize {
        let Self { inner } = self;
        inner.offset()
    }

    #[inline]
    pub fn layouts(&self) -> &D {
        let Self { inner } = self;
        inner.layouts()
    }
}

impl<'a, D, P> ErasedBundlePtrs<D, P>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    #[track_caller]
    pub unsafe fn offset_from<'n, N>(&'a self, origin: &'n ErasedBundlePtrs<N, P>) -> isize
    where
        N: FieldLayouts<'n, Output: IntoErasedArchetypeIterator> + ?Sized,
    {
        let Self { inner } = self;

        let origin = unsafe { origin.as_inner() };
        unsafe { inner.offset_from(origin) }
    }

    #[inline]
    pub fn iter(&'a self) -> ErasedBundlePtrsIter<FieldLayoutsIter<'a, D>, P> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundlePtrsIter::from_inner(inner) }
    }
}

impl<D, P> ErasedBundlePtrs<D, P>
where
    D: ErasedArchetypeKind + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn archetype(&self) -> ErasedArchetypeView<'_, D::Meta> {
        self.field_layouts()
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<ErasedComponentPtr<P>> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter().nth(index)
    }
}

type ReadResult<D, K, S, P> = Result<
    ErasedBundleKind<D, K, S, <P as ConstSliceItemPtr>::Ptrs>,
    FromFieldsLayoutsError<<S as AlignedStorageFromLayout>::Error>,
>;

impl<D, P> ErasedBundlePtrs<D, P>
where
    D: ErasedArchetypeKind + Clone,
    P: ConstSliceItemPtr<Item: Clone>,
{
    #[inline]
    pub unsafe fn read<K, S>(&self) -> ReadResult<D, K, S, P>
    where
        K: ErasedBundleDrop<D::Meta>,
        S: AlignedStorageFromLayout<Item = P::Item>,
    {
        let Self { inner } = self;

        let inner = unsafe { inner.read()? };
        let bundle = unsafe { ErasedBundleKind::from_inner(inner) };
        Ok(bundle)
    }
}

impl<D, P> Debug for ErasedBundlePtrs<D, P>
where
    D: Debug + ?Sized,
    P: ConstSliceItemPtr,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;
        f.debug_struct("ErasedBundlePtrs")
            .field("inner", &inner)
            .finish()
    }
}

impl<D, P> Clone for ErasedBundlePtrs<D, P>
where
    D: Clone,
    P: ConstSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        unsafe { Self::from_inner(inner) }
    }
}

impl<D, P> Copy for ErasedBundlePtrs<D, P>
where
    D: Copy,
    P: ConstSliceItemPtr,
{
}

impl<'a, D, P> IntoIterator for &'a ErasedBundlePtrs<D, P>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Item = ErasedComponentPtr<P>;
    type IntoIter = ErasedBundlePtrsIter<FieldLayoutsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P> IntoIterator for ErasedBundlePtrs<D, P>
where
    D: IntoErasedArchetypeIterator,
    P: ConstSliceItemPtr,
{
    type Item = ErasedComponentPtr<P>;
    type IntoIter = ErasedBundlePtrsIter<D::IntoIter, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ErasedBundlePtrsIter::from_inner(inner) }
    }
}

impl<'a, D, P> FieldLayouts<'a> for ErasedBundlePtrs<D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        let Self { inner } = self;
        inner.field_layouts()
    }
}

impl<D, P> CovariantFieldLayouts for ErasedBundlePtrs<D, P>
where
    D: CovariantFieldLayouts + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        D::upcast_field_layouts(from)
    }
}

pub struct ErasedBundlePtrsIter<D, P>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    inner: ErasedSoaPtrsIter<D, P>,
}

impl<D, P> ErasedBundlePtrsIter<D, P>
where
    P: ConstSliceItemPtr,
{
    #[inline]
    pub(super) unsafe fn from_inner(inner: ErasedSoaPtrsIter<D, P>) -> Self {
        Self { inner }
    }
}

impl<D, P> ErasedBundlePtrsIter<D, P>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> *const [P::Item] {
        let Self { inner } = self;
        inner.as_buffer()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { inner } = self;
        inner.capacity()
    }

    #[inline]
    pub fn offset(&self) -> usize {
        let Self { inner } = self;
        inner.offset()
    }

    #[inline]
    pub fn layouts(&self) -> &D {
        let Self { inner, .. } = self;
        inner.layouts()
    }
}

impl<'a, D, P> ErasedBundlePtrsIter<D, P>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedBundlePtrsIter<FieldLayoutsIter<'a, D>, P> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundlePtrsIter::from_inner(inner) }
    }
}

impl<'a, D, P> IntoIterator for &'a ErasedBundlePtrsIter<D, P>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Item = ErasedComponentPtr<P>;
    type IntoIter = ErasedBundlePtrsIter<FieldLayoutsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P> Debug for ErasedBundlePtrsIter<D, P>
where
    D: FieldLayoutsOwned<Output: IntoErasedArchetypeIterator> + ?Sized,
    P: ConstSliceItemPtr + Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl<D, P> Clone for ErasedBundlePtrsIter<D, P>
where
    D: Clone,
    P: ConstSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        Self { inner }
    }
}

impl<D, P> Iterator for ErasedBundlePtrsIter<D, P>
where
    D: ErasedArchetypeIterator + ?Sized,
    P: ConstSliceItemPtr,
{
    type Item = ErasedComponentPtr<P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;

        let component_id = inner.field_layouts().into_iter().next()?.component_id();
        let fields = inner.next()?;
        let item = unsafe { ErasedComponentPtr::from_parts(component_id, fields) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<D, P> ExactSizeIterator for ErasedBundlePtrsIter<D, P>
where
    D: ErasedArchetypeIterator + ExactSizeIterator + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<D, P> FusedIterator for ErasedBundlePtrsIter<D, P>
where
    D: ErasedArchetypeIterator + FusedIterator + ?Sized,
    P: ConstSliceItemPtr,
{
}

impl<'a, D, P> FieldLayouts<'a> for ErasedBundlePtrsIter<D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        self.layouts().field_layouts()
    }
}

impl<D, P> CovariantFieldLayouts for ErasedBundlePtrsIter<D, P>
where
    D: CovariantFieldLayouts + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        D::upcast_field_layouts(from)
    }
}
