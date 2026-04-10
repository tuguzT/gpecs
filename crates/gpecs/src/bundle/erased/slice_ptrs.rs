use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::MaybeUninit,
};

use gpecs_soa_erased::{CovariantFieldDescriptors, ErasedSoaSlicePtrs, ErasedSoaSlicePtrsIter};

use crate::{
    archetype::erased::{ErasedArchetypeView, error::IncompatibleArchetypeError},
    bundle::{
        Bundle, BundleSlicePtrs,
        erased::{
            ErasedBundleMutSlicePtrs, ErasedBundlePtrs, ErasedBundleSlices,
            traits::{ErasedArchetypeIterator, ErasedArchetypeKind, IntoErasedArchetypeIterator},
        },
    },
    component::{
        erased::ErasedComponentSlicePtr,
        registry::{
            ComponentId, ComponentRegistryView,
            traits::{ComponentIdFrom, FromComponentType, WithComponentId},
        },
    },
    soa::{
        field::{
            FieldDescriptors, FieldDescriptorsIter, FieldDescriptorsOutput, FieldDescriptorsOwned,
        },
        traits::RawSoaContext,
    },
};

type Inner<D> = ErasedSoaSlicePtrs<D, *const MaybeUninit<u8>>;

#[derive(Debug)]
pub struct ErasedBundleSlicePtrs<D>
where
    D: ?Sized,
{
    inner: Inner<D>,
}

impl<D> ErasedBundleSlicePtrs<D> {
    #[inline]
    pub unsafe fn from_inner(inner: Inner<D>) -> Self {
        Self { inner }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedBundlePtrs<D>, len: usize) -> Self {
        let inner = ptrs.into_inner();
        let inner = unsafe { Inner::from_ptrs(inner, len) };
        unsafe { Self::from_inner(inner) }
    }

    #[inline]
    pub fn into_inner(self) -> Inner<D> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedBundlePtrs<D> {
        let Self { inner } = self;

        let inner = inner.into_ptrs();
        unsafe { ErasedBundlePtrs::from_inner(inner) }
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedBundleMutSlicePtrs<D> {
        let Self { inner } = self;

        let inner = inner.cast_mut();
        unsafe { ErasedBundleMutSlicePtrs::from_inner(inner) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedBundleSlices<'a, D> {
        unsafe { ErasedBundleSlices::from_ptrs(self) }
    }
}

impl<D> ErasedBundleSlicePtrs<D>
where
    D: ?Sized,
{
    #[inline]
    pub fn as_buffer(&self) -> *const [MaybeUninit<u8>] {
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

impl<'a, D> ErasedBundleSlicePtrs<D>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedBundleSlicePtrsIter<FieldDescriptorsIter<'a, D>> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleSlicePtrsIter::from_inner(inner) }
    }
}

impl<D> ErasedBundleSlicePtrs<D>
where
    D: ErasedArchetypeKind,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleSlicePtrs<B>, IncompatibleArchetypeError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let len = self.len();
        let ptrs = self.into_ptrs().downcast::<B, T>(components)?;
        let slices = B::CONTEXT.slice_ptrs_from_raw_parts(ptrs, len);
        Ok(slices)
    }
}

impl<D> ErasedBundleSlicePtrs<D>
where
    D: ErasedArchetypeKind + ?Sized,
{
    #[inline]
    pub fn archetype(&self) -> ErasedArchetypeView<'_, D::Meta> {
        self.field_descriptors()
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<ErasedComponentSlicePtr> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter().nth(index)
    }
}

impl<D> Clone for ErasedBundleSlicePtrs<D>
where
    D: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        unsafe { Self::from_inner(inner) }
    }
}

impl<D> Copy for ErasedBundleSlicePtrs<D> where D: Copy {}

impl<'a, D> IntoIterator for &'a ErasedBundleSlicePtrs<D>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
{
    type Item = ErasedComponentSlicePtr;
    type IntoIter = ErasedBundleSlicePtrsIter<FieldDescriptorsIter<'a, D>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D> IntoIterator for ErasedBundleSlicePtrs<D>
where
    D: IntoErasedArchetypeIterator,
{
    type Item = ErasedComponentSlicePtr;
    type IntoIter = ErasedBundleSlicePtrsIter<D::IntoIter>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ErasedBundleSlicePtrsIter::from_inner(inner) }
    }
}

impl<'a, D> FieldDescriptors<'a> for ErasedBundleSlicePtrs<D>
where
    D: FieldDescriptors<'a> + ?Sized,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        let Self { inner } = self;
        inner.field_descriptors()
    }
}

impl<D> CovariantFieldDescriptors for ErasedBundleSlicePtrs<D>
where
    D: CovariantFieldDescriptors + ?Sized,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        D::upcast_field_descriptors(from)
    }
}

type InnerIter<D> = ErasedSoaSlicePtrsIter<D, *const MaybeUninit<u8>>;

pub struct ErasedBundleSlicePtrsIter<D>
where
    D: ?Sized,
{
    inner: InnerIter<D>,
}

impl<D> ErasedBundleSlicePtrsIter<D> {
    #[inline]
    pub(super) unsafe fn from_inner(inner: InnerIter<D>) -> Self {
        Self { inner }
    }
}

impl<D> ErasedBundleSlicePtrsIter<D>
where
    D: ?Sized,
{
    #[inline]
    pub fn as_buffer(&self) -> *const [MaybeUninit<u8>] {
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

impl<'a, D> ErasedBundleSlicePtrsIter<D>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedBundleSlicePtrsIter<FieldDescriptorsIter<'a, D>> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleSlicePtrsIter::from_inner(inner) }
    }
}

impl<'a, D> IntoIterator for &'a ErasedBundleSlicePtrsIter<D>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
{
    type Item = ErasedComponentSlicePtr;
    type IntoIter = ErasedBundleSlicePtrsIter<FieldDescriptorsIter<'a, D>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D> Debug for ErasedBundleSlicePtrsIter<D>
where
    D: FieldDescriptorsOwned<Output: IntoErasedArchetypeIterator> + ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl<D> Clone for ErasedBundleSlicePtrsIter<D>
where
    D: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        Self { inner }
    }
}

impl<D> Iterator for ErasedBundleSlicePtrsIter<D>
where
    D: ErasedArchetypeIterator + ?Sized,
{
    type Item = ErasedComponentSlicePtr;

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

impl<D> ExactSizeIterator for ErasedBundleSlicePtrsIter<D>
where
    D: ErasedArchetypeIterator + ExactSizeIterator + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<D> FusedIterator for ErasedBundleSlicePtrsIter<D> where
    D: ErasedArchetypeIterator + FusedIterator + ?Sized
{
}

impl<'a, D> FieldDescriptors<'a> for ErasedBundleSlicePtrsIter<D>
where
    D: FieldDescriptors<'a> + ?Sized,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        let Self { inner } = self;
        inner.field_descriptors()
    }
}

impl<D> CovariantFieldDescriptors for ErasedBundleSlicePtrsIter<D>
where
    D: CovariantFieldDescriptors + ?Sized,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        D::upcast_field_descriptors(from)
    }
}
