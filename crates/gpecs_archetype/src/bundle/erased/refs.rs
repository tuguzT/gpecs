use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use gpecs_component::{
    erased::ErasedComponentRef,
    registry::{ComponentId, traits::WithComponentId},
};
use gpecs_soa_erased::{
    CovariantFieldLayouts, ErasedSoaRefs, ErasedSoaRefsIter,
    ptr::slice::ConstSliceItemPtr,
    soa::field::{FieldLayouts, FieldLayoutsIter, FieldLayoutsOutput, FieldLayoutsOwned},
};

use crate::{
    bundle::erased::{
        ErasedBundlePtrs,
        traits::{ErasedArchetypeIterator, ErasedArchetypeKind, IntoErasedArchetypeIterator},
    },
    erased::ErasedArchetypeView,
};

pub struct ErasedBundleRefs<'a, D, P>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    inner: ErasedSoaRefs<'a, D, P>,
}

impl<'a, D, P> ErasedBundleRefs<'a, D, P>
where
    P: ConstSliceItemPtr,
{
    #[inline]
    pub unsafe fn from_inner(inner: ErasedSoaRefs<'a, D, P>) -> Self {
        Self { inner }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedBundlePtrs<D, P>) -> Self {
        let inner = ptrs.into_inner();
        let inner = unsafe { inner.as_ref_unchecked() };
        unsafe { Self::from_inner(inner) }
    }

    #[inline]
    pub fn into_inner(self) -> ErasedSoaRefs<'a, D, P> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedBundlePtrs<D, P> {
        let Self { inner } = self;

        let inner = inner.into_ptrs();
        unsafe { ErasedBundlePtrs::from_inner(inner) }
    }
}

impl<D, P> ErasedBundleRefs<'_, D, P>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> &[P::Item] {
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

impl<'a, D, P> ErasedBundleRefs<'_, D, P>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedBundleRefsIter<'a, FieldLayoutsIter<'a, D>, P> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleRefsIter::from_inner(inner) }
    }
}

impl<D, P> ErasedBundleRefs<'_, D, P>
where
    D: ErasedArchetypeKind + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn archetype(&self) -> ErasedArchetypeView<'_, D::Meta> {
        self.field_layouts()
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<ErasedComponentRef<'_, P>> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter().nth(index)
    }
}

impl<D, P> Debug for ErasedBundleRefs<'_, D, P>
where
    D: Debug + ?Sized,
    P: ConstSliceItemPtr,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;
        f.debug_struct("ErasedBundleRefs")
            .field("inner", &inner)
            .finish()
    }
}

impl<D, P> Clone for ErasedBundleRefs<'_, D, P>
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

impl<D, P> Copy for ErasedBundleRefs<'_, D, P>
where
    D: Copy,
    P: ConstSliceItemPtr,
{
}

impl<'a, D, P> IntoIterator for &'a ErasedBundleRefs<'_, D, P>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Item = ErasedComponentRef<'a, P>;
    type IntoIter = ErasedBundleRefsIter<'a, FieldLayoutsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D, P> IntoIterator for ErasedBundleRefs<'a, D, P>
where
    D: IntoErasedArchetypeIterator,
    P: ConstSliceItemPtr,
{
    type Item = ErasedComponentRef<'a, P>;
    type IntoIter = ErasedBundleRefsIter<'a, D::IntoIter, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ErasedBundleRefsIter::from_inner(inner) }
    }
}

impl<'a, D, P> FieldLayouts<'a> for ErasedBundleRefs<'_, D, P>
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

impl<D, P> CovariantFieldLayouts for ErasedBundleRefs<'_, D, P>
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

pub struct ErasedBundleRefsIter<'a, D, P>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    inner: ErasedSoaRefsIter<'a, D, P>,
}

impl<'a, D, P> ErasedBundleRefsIter<'a, D, P>
where
    P: ConstSliceItemPtr,
{
    #[inline]
    pub(super) unsafe fn from_inner(inner: ErasedSoaRefsIter<'a, D, P>) -> Self {
        Self { inner }
    }
}

impl<D, P> ErasedBundleRefsIter<'_, D, P>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> &[P::Item] {
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

impl<'a, D, P> ErasedBundleRefsIter<'_, D, P>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedBundleRefsIter<'a, FieldLayoutsIter<'a, D>, P> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleRefsIter::from_inner(inner) }
    }
}

impl<'a, D, P> IntoIterator for &'a ErasedBundleRefsIter<'_, D, P>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Item = ErasedComponentRef<'a, P>;
    type IntoIter = ErasedBundleRefsIter<'a, FieldLayoutsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P> Debug for ErasedBundleRefsIter<'_, D, P>
where
    D: FieldLayoutsOwned<Output: IntoErasedArchetypeIterator> + ?Sized,
    P: ConstSliceItemPtr<Item: Debug>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl<D, P> Clone for ErasedBundleRefsIter<'_, D, P>
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

impl<'a, D, P> Iterator for ErasedBundleRefsIter<'a, D, P>
where
    D: ErasedArchetypeIterator + ?Sized,
    P: ConstSliceItemPtr,
{
    type Item = ErasedComponentRef<'a, P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;

        let component_id = inner.field_layouts().into_iter().next()?.component_id();
        let fields = inner.next()?;
        let item = unsafe { ErasedComponentRef::from_parts(component_id, fields) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<D, P> ExactSizeIterator for ErasedBundleRefsIter<'_, D, P>
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

impl<D, P> FusedIterator for ErasedBundleRefsIter<'_, D, P>
where
    D: ErasedArchetypeIterator + FusedIterator + ?Sized,
    P: ConstSliceItemPtr,
{
}

impl<'a, D, P> FieldLayouts<'a> for ErasedBundleRefsIter<'_, D, P>
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

impl<D, P> CovariantFieldLayouts for ErasedBundleRefsIter<'_, D, P>
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
