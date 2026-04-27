use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ptr::NonNull,
};

use gpecs_component::{
    erased::ErasedComponentNonNullPtr,
    registry::{
        ComponentId, ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType, WithComponentId},
    },
};
use gpecs_soa_erased::{
    CovariantFieldLayouts, ErasedSoaNonNullPtrs, ErasedSoaNonNullPtrsIter,
    ptr::slice::{NonNullAsPtr, NonNullSliceItemPtr},
    soa::{
        field::{FieldLayouts, FieldLayoutsIter, FieldLayoutsOutput, FieldLayoutsOwned},
        traits::RawSoaContext,
    },
};

use crate::{
    bundle::{
        Bundle, BundleNonNullPtrs,
        erased::{
            ErasedBundleMutPtrs,
            error::DowncastError,
            traits::{ErasedArchetypeIterator, ErasedArchetypeKind, IntoErasedArchetypeIterator},
        },
    },
    erased::ErasedArchetypeView,
};

pub struct ErasedBundleNonNullPtrs<D, P>
where
    D: ?Sized,
    P: NonNullSliceItemPtr,
{
    inner: ErasedSoaNonNullPtrs<D, P>,
}

impl<D, P> ErasedBundleNonNullPtrs<D, P>
where
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub fn new(ptrs: ErasedBundleMutPtrs<D, NonNullAsPtr<P>>) -> Option<Self> {
        let ptrs = ptrs.into_inner();
        let inner = ErasedSoaNonNullPtrs::new(ptrs)?;

        let me = unsafe { Self::from_inner(inner) };
        Some(me)
    }

    #[inline]
    pub unsafe fn new_unchecked(ptrs: ErasedBundleMutPtrs<D, NonNullAsPtr<P>>) -> Self {
        let ptrs = ptrs.into_inner();
        let inner = unsafe { ErasedSoaNonNullPtrs::new_unchecked(ptrs) };
        unsafe { Self::from_inner(inner) }
    }

    #[inline]
    pub unsafe fn from_inner(inner: ErasedSoaNonNullPtrs<D, P>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn into_inner(self) -> ErasedSoaNonNullPtrs<D, P> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { inner } = self;

        let inner = unsafe { inner.add(count) };
        unsafe { Self::from_inner(inner) }
    }
}

impl<D, P> ErasedBundleNonNullPtrs<D, P>
where
    D: ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub unsafe fn as_inner(&self) -> &ErasedSoaNonNullPtrs<D, P> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub unsafe fn as_mut_inner(&mut self) -> &mut ErasedSoaNonNullPtrs<D, P> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn as_buffer(&self) -> NonNull<[P::Item]> {
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

impl<'a, D, P> ErasedBundleNonNullPtrs<D, P>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    #[track_caller]
    pub unsafe fn offset_from<'n, N>(&'a self, origin: &'n ErasedBundleNonNullPtrs<N, P>) -> isize
    where
        N: FieldLayouts<'n, Output: IntoErasedArchetypeIterator> + ?Sized,
    {
        let Self { inner } = self;

        let origin = unsafe { origin.as_inner() };
        unsafe { inner.offset_from(origin) }
    }

    #[inline]
    pub fn iter(&'a self) -> ErasedBundleNonNullPtrsIter<FieldLayoutsIter<'a, D>, P> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleNonNullPtrsIter::from_inner(inner) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn swap<'n, N>(&'a mut self, with: &'n mut ErasedBundleNonNullPtrs<N, P>)
    where
        N: FieldLayouts<'n, Output: IntoErasedArchetypeIterator> + ?Sized,
    {
        let Self { inner } = self;

        let with = unsafe { with.as_mut_inner() };
        unsafe { inner.swap(with) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_forward<'n, N>(
        &'a mut self,
        src: &'n ErasedBundleNonNullPtrs<N, P>,
        count: usize,
    ) where
        N: FieldLayouts<'n, Output: IntoErasedArchetypeIterator> + ?Sized,
    {
        let Self { inner } = self;

        let src = unsafe { src.as_inner() };
        unsafe { inner.copy_from_forward(src, count) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_backward<'n, N>(
        &'a mut self,
        src: &'n ErasedBundleNonNullPtrs<N, P>,
        count: usize,
    ) where
        N: FieldLayouts<'n, Output: IntoErasedArchetypeIterator> + ?Sized,
    {
        let Self { inner } = self;

        let src = unsafe { src.as_inner() };
        unsafe { inner.copy_from_backward(src, count) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_nonoverlapping<'n, N>(
        &'a mut self,
        src: &'n ErasedBundleNonNullPtrs<N, P>,
        count: usize,
    ) where
        N: FieldLayouts<'n, Output: IntoErasedArchetypeIterator> + ?Sized,
    {
        let Self { inner } = self;

        let src = unsafe { src.as_inner() };
        unsafe { inner.copy_from_nonoverlapping(src, count) }
    }
}

impl<D, P> ErasedBundleNonNullPtrs<D, P>
where
    D: ErasedArchetypeKind,
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleNonNullPtrs<B>, DowncastError<Self>>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let into_self = |ptrs| unsafe { Self::new_unchecked(ptrs) };
        let ptrs = ErasedBundleMutPtrs::from(self)
            .downcast::<B, T>(components)
            .map_err(|error| error.map_value(into_self))?;

        let ptrs = unsafe { B::CONTEXT.ptrs_to_nonnull(ptrs) };
        Ok(ptrs)
    }
}

impl<D, P> ErasedBundleNonNullPtrs<D, P>
where
    D: ErasedArchetypeKind + ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub fn archetype(&self) -> ErasedArchetypeView<'_, D::Meta> {
        self.field_layouts()
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<ErasedComponentNonNullPtr<P>> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter().nth(index)
    }
}

impl<D, P> Debug for ErasedBundleNonNullPtrs<D, P>
where
    D: Debug + ?Sized,
    P: NonNullSliceItemPtr,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;
        f.debug_struct("ErasedBundleNonNullPtrs")
            .field("inner", &inner)
            .finish()
    }
}

impl<D, P> Clone for ErasedBundleNonNullPtrs<D, P>
where
    D: Clone,
    P: NonNullSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        unsafe { Self::from_inner(inner) }
    }
}

impl<D, P> Copy for ErasedBundleNonNullPtrs<D, P>
where
    D: Copy,
    P: NonNullSliceItemPtr,
{
}

impl<'a, D, P> IntoIterator for &'a ErasedBundleNonNullPtrs<D, P>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: NonNullSliceItemPtr,
{
    type Item = ErasedComponentNonNullPtr<P>;
    type IntoIter = ErasedBundleNonNullPtrsIter<FieldLayoutsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P> IntoIterator for ErasedBundleNonNullPtrs<D, P>
where
    D: IntoErasedArchetypeIterator,
    P: NonNullSliceItemPtr,
{
    type Item = ErasedComponentNonNullPtr<P>;
    type IntoIter = ErasedBundleNonNullPtrsIter<D::IntoIter, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ErasedBundleNonNullPtrsIter::from_inner(inner) }
    }
}

impl<D, P> From<ErasedBundleNonNullPtrs<D, P>> for ErasedBundleMutPtrs<D, NonNullAsPtr<P>>
where
    P: NonNullSliceItemPtr,
{
    #[inline]
    fn from(ptrs: ErasedBundleNonNullPtrs<D, P>) -> Self {
        let inner = ptrs.into_inner();
        let inner = inner.into();
        unsafe { ErasedBundleMutPtrs::from_inner(inner) }
    }
}

impl<'a, D, P> FieldLayouts<'a> for ErasedBundleNonNullPtrs<D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: NonNullSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        let Self { inner } = self;
        inner.field_layouts()
    }
}

impl<D, P> CovariantFieldLayouts for ErasedBundleNonNullPtrs<D, P>
where
    D: CovariantFieldLayouts + ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        D::upcast_field_layouts(from)
    }
}

pub struct ErasedBundleNonNullPtrsIter<D, P>
where
    D: ?Sized,
    P: NonNullSliceItemPtr,
{
    inner: ErasedSoaNonNullPtrsIter<D, P>,
}

impl<D, P> ErasedBundleNonNullPtrsIter<D, P>
where
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub(super) unsafe fn from_inner(inner: ErasedSoaNonNullPtrsIter<D, P>) -> Self {
        Self { inner }
    }
}

impl<D, P> ErasedBundleNonNullPtrsIter<D, P>
where
    D: ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> NonNull<[P::Item]> {
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

impl<'a, D, P> ErasedBundleNonNullPtrsIter<D, P>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedBundleNonNullPtrsIter<FieldLayoutsIter<'a, D>, P> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleNonNullPtrsIter::from_inner(inner) }
    }
}

impl<'a, D, P> IntoIterator for &'a ErasedBundleNonNullPtrsIter<D, P>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: NonNullSliceItemPtr,
{
    type Item = ErasedComponentNonNullPtr<P>;
    type IntoIter = ErasedBundleNonNullPtrsIter<FieldLayoutsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P> Debug for ErasedBundleNonNullPtrsIter<D, P>
where
    D: FieldLayoutsOwned<Output: IntoErasedArchetypeIterator> + ?Sized,
    P: NonNullSliceItemPtr + Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl<D, P> Clone for ErasedBundleNonNullPtrsIter<D, P>
where
    D: Clone,
    P: NonNullSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        Self { inner }
    }
}

impl<D, P> Iterator for ErasedBundleNonNullPtrsIter<D, P>
where
    D: ErasedArchetypeIterator + ?Sized,
    P: NonNullSliceItemPtr,
{
    type Item = ErasedComponentNonNullPtr<P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;

        let component_id = inner.field_layouts().into_iter().next()?.component_id();
        let fields = inner.next()?;
        let item = unsafe { ErasedComponentNonNullPtr::from_parts(component_id, fields) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<D, P> ExactSizeIterator for ErasedBundleNonNullPtrsIter<D, P>
where
    D: ErasedArchetypeIterator + ExactSizeIterator + ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<D, P> FusedIterator for ErasedBundleNonNullPtrsIter<D, P>
where
    D: ErasedArchetypeIterator + FusedIterator + ?Sized,
    P: NonNullSliceItemPtr,
{
}

impl<'a, D, P> FieldLayouts<'a> for ErasedBundleNonNullPtrsIter<D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: NonNullSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        let Self { inner } = self;
        inner.field_layouts()
    }
}

impl<D, P> CovariantFieldLayouts for ErasedBundleNonNullPtrsIter<D, P>
where
    D: CovariantFieldLayouts + ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        D::upcast_field_layouts(from)
    }
}
