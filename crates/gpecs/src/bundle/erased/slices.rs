use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::MaybeUninit,
};

use gpecs_soa_erased::{
    CovariantFieldDescriptors, ErasedSoaSlices, ErasedSoaSlicesIter, ptr::slice::ConstSliceItemPtr,
};

use crate::{
    archetype::erased::ErasedArchetypeView,
    bundle::{
        Bundle, BundleSlices,
        erased::{
            ErasedBundleSlicePtrs,
            error::DowncastError,
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

pub struct ErasedBundleSlices<'a, D, P = *const MaybeUninit<u8>>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    inner: ErasedSoaSlices<'a, D, P>,
}

impl<'a, D, P> ErasedBundleSlices<'a, D, P>
where
    P: ConstSliceItemPtr,
{
    #[inline]
    pub unsafe fn from_inner(inner: ErasedSoaSlices<'a, D, P>) -> Self {
        Self { inner }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedBundleSlicePtrs<D, P>) -> Self {
        let inner = ptrs.into_inner();
        let inner = unsafe { inner.deref() };
        unsafe { Self::from_inner(inner) }
    }

    #[inline]
    pub fn into_inner(self) -> ErasedSoaSlices<'a, D, P> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedBundleSlicePtrs<D, P> {
        let Self { inner } = self;

        let inner = inner.into_ptrs();
        unsafe { ErasedBundleSlicePtrs::from_inner(inner) }
    }
}

impl<D, P> ErasedBundleSlices<'_, D, P>
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

impl<'a, D, P> ErasedBundleSlices<'_, D, P>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedBundleSlicesIter<'a, FieldDescriptorsIter<'a, D>, P> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleSlicesIter::from_inner(inner) }
    }
}

impl<'a, D, P> ErasedBundleSlices<'a, D, P>
where
    D: ErasedArchetypeKind,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleSlices<'a, B>, DowncastError<Self>>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs) };
        let slices = self
            .into_ptrs()
            .downcast::<B, T>(components)
            .map_err(|error| error.map_value(into_self))?;

        let slices = unsafe { B::CONTEXT.slice_ptrs_to_slices(slices) };
        Ok(slices)
    }
}

impl<D, P> ErasedBundleSlices<'_, D, P>
where
    D: ErasedArchetypeKind + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn archetype(&self) -> ErasedArchetypeView<'_, D::Meta> {
        self.field_descriptors()
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<ErasedComponentSlice<'_, P>> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter().nth(index)
    }
}

impl<D, P> Debug for ErasedBundleSlices<'_, D, P>
where
    D: Debug + ?Sized,
    P: ConstSliceItemPtr,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;
        f.debug_struct("ErasedBundleSlices")
            .field("inner", &inner)
            .finish()
    }
}

impl<D, P> Clone for ErasedBundleSlices<'_, D, P>
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

impl<D, P> Copy for ErasedBundleSlices<'_, D, P>
where
    D: Copy,
    P: ConstSliceItemPtr,
{
}

impl<'a, D, P> IntoIterator for &'a ErasedBundleSlices<'_, D, P>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Item = ErasedComponentSlice<'a, P>;
    type IntoIter = ErasedBundleSlicesIter<'a, FieldDescriptorsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D, P> IntoIterator for ErasedBundleSlices<'a, D, P>
where
    D: IntoErasedArchetypeIterator,
    P: ConstSliceItemPtr,
{
    type Item = ErasedComponentSlice<'a, P>;
    type IntoIter = ErasedBundleSlicesIter<'a, D::IntoIter, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ErasedBundleSlicesIter::from_inner(inner) }
    }
}

impl<'a, D, P> FieldDescriptors<'a> for ErasedBundleSlices<'_, D, P>
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

impl<D, P> CovariantFieldDescriptors for ErasedBundleSlices<'_, D, P>
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

pub struct ErasedBundleSlicesIter<'a, D, P = *const MaybeUninit<u8>>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    inner: ErasedSoaSlicesIter<'a, D, P>,
}

impl<'a, D, P> ErasedBundleSlicesIter<'a, D, P>
where
    P: ConstSliceItemPtr,
{
    #[inline]
    pub(super) unsafe fn from_inner(inner: ErasedSoaSlicesIter<'a, D, P>) -> Self {
        Self { inner }
    }
}

impl<D, P> ErasedBundleSlicesIter<'_, D, P>
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

impl<'a, D, P> ErasedBundleSlicesIter<'_, D, P>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedBundleSlicesIter<'a, FieldDescriptorsIter<'a, D>, P> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleSlicesIter::from_inner(inner) }
    }
}

impl<'a, D, P> IntoIterator for &'a ErasedBundleSlicesIter<'_, D, P>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Item = ErasedComponentSlice<'a, P>;
    type IntoIter = ErasedBundleSlicesIter<'a, FieldDescriptorsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P> Debug for ErasedBundleSlicesIter<'_, D, P>
where
    D: FieldDescriptorsOwned<Output: IntoErasedArchetypeIterator> + ?Sized,
    P: ConstSliceItemPtr<Item: Debug>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl<D, P> Clone for ErasedBundleSlicesIter<'_, D, P>
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

impl<'a, D, P> Iterator for ErasedBundleSlicesIter<'a, D, P>
where
    D: ErasedArchetypeIterator + ?Sized,
    P: ConstSliceItemPtr,
{
    type Item = ErasedComponentSlice<'a, P>;

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

impl<D, P> ExactSizeIterator for ErasedBundleSlicesIter<'_, D, P>
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

impl<D, P> FusedIterator for ErasedBundleSlicesIter<'_, D, P>
where
    D: ErasedArchetypeIterator + FusedIterator + ?Sized,
    P: ConstSliceItemPtr,
{
}

impl<'a, D, P> FieldDescriptors<'a> for ErasedBundleSlicesIter<'_, D, P>
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

impl<D, P> CovariantFieldDescriptors for ErasedBundleSlicesIter<'_, D, P>
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
