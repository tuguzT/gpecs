use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::MaybeUninit,
};

use gpecs_component::{
    erased::ErasedComponentSlicePtr,
    registry::{ComponentId, traits::WithComponentId},
};
use gpecs_soa_erased::{
    CovariantFieldDescriptors, ErasedSoaSlicePtrs, ErasedSoaSlicePtrsIter,
    ptr::slice::{CastMut, ConstSliceItemPtr},
    soa::field::{
        FieldDescriptors, FieldDescriptorsIter, FieldDescriptorsOutput, FieldDescriptorsOwned,
    },
};

use crate::{
    bundle::erased::{
        ErasedBundleMutSlicePtrs, ErasedBundlePtrs, ErasedBundleSlices,
        traits::{ErasedArchetypeIterator, ErasedArchetypeKind, IntoErasedArchetypeIterator},
    },
    erased::ErasedArchetypeView,
};

pub struct ErasedBundleSlicePtrs<D, P = *const MaybeUninit<u8>>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    inner: ErasedSoaSlicePtrs<D, P>,
}

impl<D, P> ErasedBundleSlicePtrs<D, P>
where
    P: ConstSliceItemPtr,
{
    #[inline]
    pub unsafe fn from_inner(inner: ErasedSoaSlicePtrs<D, P>) -> Self {
        Self { inner }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedBundlePtrs<D, P>, len: usize) -> Self {
        let inner = ptrs.into_inner();
        let inner = unsafe { ErasedSoaSlicePtrs::from_ptrs(inner, len) };
        unsafe { Self::from_inner(inner) }
    }

    #[inline]
    pub fn into_inner(self) -> ErasedSoaSlicePtrs<D, P> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedBundlePtrs<D, P> {
        let Self { inner } = self;

        let inner = inner.into_ptrs();
        unsafe { ErasedBundlePtrs::from_inner(inner) }
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedBundleMutSlicePtrs<D, CastMut<P>> {
        let Self { inner } = self;

        let inner = inner.cast_mut();
        unsafe { ErasedBundleMutSlicePtrs::from_inner(inner) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedBundleSlices<'a, D, P> {
        unsafe { ErasedBundleSlices::from_ptrs(self) }
    }
}

impl<D, P> ErasedBundleSlicePtrs<D, P>
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
    pub fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn descriptors(&self) -> &D {
        let Self { inner } = self;
        inner.descriptors()
    }
}

impl<'a, D, P> ErasedBundleSlicePtrs<D, P>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedBundleSlicePtrsIter<FieldDescriptorsIter<'a, D>, P> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleSlicePtrsIter::from_inner(inner) }
    }
}

impl<D, P> ErasedBundleSlicePtrs<D, P>
where
    D: ErasedArchetypeKind + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn archetype(&self) -> ErasedArchetypeView<'_, D::Meta> {
        self.field_descriptors()
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<ErasedComponentSlicePtr<P>> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter().nth(index)
    }
}

impl<D, P> Debug for ErasedBundleSlicePtrs<D, P>
where
    D: Debug + ?Sized,
    P: ConstSliceItemPtr,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;
        f.debug_struct("ErasedBundleSlicePtrs")
            .field("inner", &inner)
            .finish()
    }
}

impl<D, P> Clone for ErasedBundleSlicePtrs<D, P>
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

impl<D, P> Copy for ErasedBundleSlicePtrs<D, P>
where
    D: Copy,
    P: ConstSliceItemPtr,
{
}

impl<'a, D, P> IntoIterator for &'a ErasedBundleSlicePtrs<D, P>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Item = ErasedComponentSlicePtr<P>;
    type IntoIter = ErasedBundleSlicePtrsIter<FieldDescriptorsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P> IntoIterator for ErasedBundleSlicePtrs<D, P>
where
    D: IntoErasedArchetypeIterator,
    P: ConstSliceItemPtr,
{
    type Item = ErasedComponentSlicePtr<P>;
    type IntoIter = ErasedBundleSlicePtrsIter<D::IntoIter, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ErasedBundleSlicePtrsIter::from_inner(inner) }
    }
}

impl<'a, D, P> FieldDescriptors<'a> for ErasedBundleSlicePtrs<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        let Self { inner } = self;
        inner.field_descriptors()
    }
}

impl<D, P> CovariantFieldDescriptors for ErasedBundleSlicePtrs<D, P>
where
    D: CovariantFieldDescriptors + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        D::upcast_field_descriptors(from)
    }
}

pub struct ErasedBundleSlicePtrsIter<D, P>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    inner: ErasedSoaSlicePtrsIter<D, P>,
}

impl<D, P> ErasedBundleSlicePtrsIter<D, P>
where
    P: ConstSliceItemPtr,
{
    #[inline]
    pub(super) unsafe fn from_inner(inner: ErasedSoaSlicePtrsIter<D, P>) -> Self {
        Self { inner }
    }
}

impl<D, P> ErasedBundleSlicePtrsIter<D, P>
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
    pub fn slice_len(&self) -> usize {
        let Self { inner } = self;
        inner.slice_len()
    }

    #[inline]
    pub fn descriptors(&self) -> &D {
        let Self { inner, .. } = self;
        inner.descriptors()
    }
}

impl<'a, D, P> ErasedBundleSlicePtrsIter<D, P>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedBundleSlicePtrsIter<FieldDescriptorsIter<'a, D>, P> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleSlicePtrsIter::from_inner(inner) }
    }
}

impl<'a, D, P> IntoIterator for &'a ErasedBundleSlicePtrsIter<D, P>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Item = ErasedComponentSlicePtr<P>;
    type IntoIter = ErasedBundleSlicePtrsIter<FieldDescriptorsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P> Debug for ErasedBundleSlicePtrsIter<D, P>
where
    D: FieldDescriptorsOwned<Output: IntoErasedArchetypeIterator> + ?Sized,
    P: ConstSliceItemPtr + Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl<D, P> Clone for ErasedBundleSlicePtrsIter<D, P>
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

impl<D, P> Iterator for ErasedBundleSlicePtrsIter<D, P>
where
    D: ErasedArchetypeIterator + ?Sized,
    P: ConstSliceItemPtr,
{
    type Item = ErasedComponentSlicePtr<P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;

        let component_id = inner.field_descriptors().into_iter().next()?.component_id();
        let fields = inner.next()?;
        let item = unsafe { ErasedComponentSlicePtr::from_parts(component_id, fields) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<D, P> ExactSizeIterator for ErasedBundleSlicePtrsIter<D, P>
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

impl<D, P> FusedIterator for ErasedBundleSlicePtrsIter<D, P>
where
    D: ErasedArchetypeIterator + FusedIterator + ?Sized,
    P: ConstSliceItemPtr,
{
}

impl<'a, D, P> FieldDescriptors<'a> for ErasedBundleSlicePtrsIter<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        let Self { inner } = self;
        inner.field_descriptors()
    }
}

impl<D, P> CovariantFieldDescriptors for ErasedBundleSlicePtrsIter<D, P>
where
    D: CovariantFieldDescriptors + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        D::upcast_field_descriptors(from)
    }
}
