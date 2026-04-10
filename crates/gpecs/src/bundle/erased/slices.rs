use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::MaybeUninit,
};

use gpecs_soa_erased::{CovariantFieldDescriptors, ErasedSoaSlices, ErasedSoaSlicesIter};

use crate::{
    archetype::erased::{ErasedArchetypeView, error::IncompatibleArchetypeError},
    bundle::{
        Bundle, BundleSlices,
        erased::{
            ErasedBundleSlicePtrs,
            traits::{ErasedArchetypeIterator, ErasedArchetypeKind, IntoErasedArchetypeIterator},
        },
    },
    component::{
        erased::ErasedComponentSlice,
        registry::{
            ComponentId, ComponentRegistryView,
            traits::{ComponentIdFrom, FromComponentType, WithComponentId},
        },
    },
    soa::{
        field::{
            FieldDescriptors, FieldDescriptorsIter, FieldDescriptorsOutput, FieldDescriptorsOwned,
        },
        traits::SoaContext,
    },
};

type Inner<'a, D> = ErasedSoaSlices<'a, D, *const MaybeUninit<u8>>;

#[derive(Debug)]
pub struct ErasedBundleSlices<'a, D>
where
    D: ?Sized,
{
    inner: Inner<'a, D>,
}

impl<'a, D> ErasedBundleSlices<'a, D> {
    #[inline]
    pub unsafe fn from_inner(inner: Inner<'a, D>) -> Self {
        Self { inner }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedBundleSlicePtrs<D>) -> Self {
        let inner = ptrs.into_inner();
        let inner = unsafe { inner.deref() };
        unsafe { Self::from_inner(inner) }
    }

    #[inline]
    pub fn into_inner(self) -> Inner<'a, D> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedBundleSlicePtrs<D> {
        let Self { inner } = self;

        let inner = inner.into_ptrs();
        unsafe { ErasedBundleSlicePtrs::from_inner(inner) }
    }
}

impl<D> ErasedBundleSlices<'_, D>
where
    D: ?Sized,
{
    #[inline]
    pub fn as_buffer(&self) -> &[MaybeUninit<u8>] {
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
    pub fn descriptors(&self) -> &D {
        let Self { inner } = self;
        inner.descriptors()
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
}

impl<'a, D> ErasedBundleSlices<'_, D>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedBundleSlicesIter<'a, FieldDescriptorsIter<'a, D>> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleSlicesIter::from_inner(inner) }
    }
}

impl<'a, D> ErasedBundleSlices<'a, D>
where
    D: ErasedArchetypeKind,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleSlices<'a, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let slices = self.into_ptrs().downcast::<B, T>(components)?;
        let slices = unsafe { B::CONTEXT.slice_ptrs_to_slices(slices) };
        Ok(slices)
    }
}

impl<D> ErasedBundleSlices<'_, D>
where
    D: ErasedArchetypeKind + ?Sized,
{
    #[inline]
    pub fn archetype(&self) -> ErasedArchetypeView<'_, D::Meta> {
        self.field_descriptors()
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<ErasedComponentSlice<'_>> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter().nth(index)
    }
}

impl<D> Clone for ErasedBundleSlices<'_, D>
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

impl<D> Copy for ErasedBundleSlices<'_, D> where D: Copy {}

impl<'a, D> IntoIterator for &'a ErasedBundleSlices<'_, D>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
{
    type Item = ErasedComponentSlice<'a>;
    type IntoIter = ErasedBundleSlicesIter<'a, FieldDescriptorsIter<'a, D>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D> IntoIterator for ErasedBundleSlices<'a, D>
where
    D: IntoErasedArchetypeIterator,
{
    type Item = ErasedComponentSlice<'a>;
    type IntoIter = ErasedBundleSlicesIter<'a, D::IntoIter>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ErasedBundleSlicesIter::from_inner(inner) }
    }
}

impl<'a, D> FieldDescriptors<'a> for ErasedBundleSlices<'_, D>
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

impl<D> CovariantFieldDescriptors for ErasedBundleSlices<'_, D>
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

type InnerIter<'a, D> = ErasedSoaSlicesIter<'a, D, *const MaybeUninit<u8>>;

pub struct ErasedBundleSlicesIter<'a, D>
where
    D: ?Sized,
{
    inner: InnerIter<'a, D>,
}

impl<'a, D> ErasedBundleSlicesIter<'a, D> {
    #[inline]
    pub(super) unsafe fn from_inner(inner: InnerIter<'a, D>) -> Self {
        Self { inner }
    }
}

impl<D> ErasedBundleSlicesIter<'_, D>
where
    D: ?Sized,
{
    #[inline]
    pub fn as_buffer(&self) -> &[MaybeUninit<u8>] {
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

impl<'a, D> ErasedBundleSlicesIter<'_, D>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedBundleSlicesIter<'a, FieldDescriptorsIter<'a, D>> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleSlicesIter::from_inner(inner) }
    }
}

impl<'a, D> IntoIterator for &'a ErasedBundleSlicesIter<'_, D>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
{
    type Item = ErasedComponentSlice<'a>;
    type IntoIter = ErasedBundleSlicesIter<'a, FieldDescriptorsIter<'a, D>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D> Debug for ErasedBundleSlicesIter<'_, D>
where
    D: FieldDescriptorsOwned<Output: IntoErasedArchetypeIterator> + ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl<D> Clone for ErasedBundleSlicesIter<'_, D>
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

impl<'a, D> Iterator for ErasedBundleSlicesIter<'a, D>
where
    D: ErasedArchetypeIterator + ?Sized,
{
    type Item = ErasedComponentSlice<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;

        let component_id = inner.field_descriptors().into_iter().next()?.component_id();
        let fields = inner.next()?;
        let item = unsafe { ErasedComponentSlice::from_parts(component_id, fields) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<D> ExactSizeIterator for ErasedBundleSlicesIter<'_, D>
where
    D: ErasedArchetypeIterator + ExactSizeIterator + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<D> FusedIterator for ErasedBundleSlicesIter<'_, D> where
    D: ErasedArchetypeIterator + FusedIterator + ?Sized
{
}

impl<'a, D> FieldDescriptors<'a> for ErasedBundleSlicesIter<'_, D>
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

impl<D> CovariantFieldDescriptors for ErasedBundleSlicesIter<'_, D>
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
