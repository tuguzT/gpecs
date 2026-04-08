use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::MaybeUninit,
};

use gpecs_soa_erased::{CovariantFieldDescriptors, ErasedSoaMutSlices, ErasedSoaMutSlicesIter};

use crate::{
    archetype::erased::{ErasedArchetypeView, Iter, error::IncompatibleArchetypeError},
    bundle::{
        Bundle, BundleSlicesMut,
        erased::{
            ErasedBundleMutSlicePtrs, ErasedBundleSlices, ErasedBundleSlicesIter,
            traits::{ErasedArchetypeIterator, ErasedArchetypeKind, IntoErasedArchetypeIterator},
        },
    },
    component::{
        erased::{ErasedComponentMutSlice, ErasedComponentSlice},
        registry::{
            ComponentId, ComponentRegistryView,
            traits::{ComponentIdFrom, FromComponentType},
        },
    },
    soa::{
        field::{
            FieldDescriptors, FieldDescriptorsIter, FieldDescriptorsOutput, FieldDescriptorsOwned,
        },
        traits::SoaContext,
    },
};

type Inner<'a, D> = ErasedSoaMutSlices<'a, D, *mut MaybeUninit<u8>>;

#[derive(Debug)]
pub struct ErasedBundleMutSlices<'a, D>
where
    D: ?Sized,
{
    inner: Inner<'a, D>,
}

impl<'a, D> ErasedBundleMutSlices<'a, D> {
    #[inline]
    pub unsafe fn from_inner(inner: Inner<'a, D>) -> Self {
        Self { inner }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedBundleMutSlicePtrs<D>) -> Self {
        let inner = ptrs.into_inner();
        let inner = unsafe { inner.deref_mut() };
        unsafe { Self::from_inner(inner) }
    }

    #[inline]
    pub fn into_inner(self) -> Inner<'a, D> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedBundleMutSlicePtrs<D> {
        let Self { inner } = self;

        let inner = inner.into_ptrs();
        unsafe { ErasedBundleMutSlicePtrs::from_inner(inner) }
    }
}

impl<D> ErasedBundleMutSlices<'_, D>
where
    D: ?Sized,
{
    #[inline]
    pub fn as_buffer(&self) -> &[MaybeUninit<u8>] {
        let Self { inner } = self;
        inner.as_buffer()
    }

    #[inline]
    pub unsafe fn as_mut_buffer(&mut self) -> &mut [MaybeUninit<u8>] {
        let Self { inner } = self;
        inner.as_mut_buffer()
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

impl<'a, D> ErasedBundleMutSlices<'a, D>
where
    D: ErasedArchetypeKind,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleSlicesMut<'a, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let slices = self.into_ptrs().downcast::<B, T>(components)?;
        let slices = unsafe { B::CONTEXT.mut_slice_ptrs_to_mut_slices(slices) };
        Ok(slices)
    }
}

impl<D> ErasedBundleMutSlices<'_, D>
where
    D: ErasedArchetypeKind + ?Sized,
{
    #[inline]
    pub fn archetype(&self) -> ErasedArchetypeView<'_, D::Meta> {
        self.field_descriptors()
    }

    #[inline]
    pub fn iter(&self) -> ErasedBundleSlicesIter<'_, Iter<'_, D::Meta>> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleSlicesIter::from_inner(inner) }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> ErasedBundleMutSlicesIter<'_, Iter<'_, D::Meta>> {
        let Self { inner } = self;

        let inner = inner.iter_mut();
        unsafe { ErasedBundleMutSlicesIter::from_inner(inner) }
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<ErasedComponentSlice<'_>> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter().nth(index)
    }

    #[inline]
    pub fn get_mut(&mut self, component_id: ComponentId) -> Option<ErasedComponentMutSlice<'_>> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter_mut().nth(index)
    }
}

impl<'a, D> IntoIterator for &'a ErasedBundleMutSlices<'_, D>
where
    D: ErasedArchetypeKind + ?Sized,
{
    type Item = ErasedComponentSlice<'a>;
    type IntoIter = ErasedBundleSlicesIter<'a, Iter<'a, D::Meta>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D> IntoIterator for &'a mut ErasedBundleMutSlices<'_, D>
where
    D: ErasedArchetypeKind + ?Sized,
{
    type Item = ErasedComponentMutSlice<'a>;
    type IntoIter = ErasedBundleMutSlicesIter<'a, Iter<'a, D::Meta>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, D> IntoIterator for ErasedBundleMutSlices<'a, D>
where
    D: IntoErasedArchetypeIterator,
{
    type Item = ErasedComponentMutSlice<'a>;
    type IntoIter = ErasedBundleMutSlicesIter<'a, D::IntoIter>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ErasedBundleMutSlicesIter::from_inner(inner) }
    }
}

impl<'a, D> From<ErasedBundleMutSlices<'a, D>> for ErasedBundleSlices<'a, D> {
    #[inline]
    fn from(slices: ErasedBundleMutSlices<'a, D>) -> Self {
        let inner = slices.into_inner();
        let inner = inner.into();
        unsafe { Self::from_inner(inner) }
    }
}

impl<'a, D> FieldDescriptors<'a> for ErasedBundleMutSlices<'_, D>
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

impl<D> CovariantFieldDescriptors for ErasedBundleMutSlices<'_, D>
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

type InnerIter<'a, D> = ErasedSoaMutSlicesIter<'a, D, *mut MaybeUninit<u8>>;

pub struct ErasedBundleMutSlicesIter<'a, D>
where
    D: ?Sized,
{
    inner: InnerIter<'a, D>,
}

impl<'a, D> ErasedBundleMutSlicesIter<'a, D> {
    #[inline]
    pub(super) unsafe fn from_inner(inner: InnerIter<'a, D>) -> Self {
        Self { inner }
    }
}

impl<D> ErasedBundleMutSlicesIter<'_, D>
where
    D: ?Sized,
{
    #[inline]
    pub fn as_buffer(&self) -> &[MaybeUninit<u8>] {
        let Self { inner } = self;
        inner.as_buffer()
    }

    #[inline]
    pub unsafe fn as_mut_buffer(&mut self) -> &mut [MaybeUninit<u8>] {
        let Self { inner } = self;
        inner.as_mut_buffer()
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

impl<'a, D> ErasedBundleMutSlicesIter<'_, D>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedBundleMutSlicesIter<'a, FieldDescriptorsIter<'a, D>> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleMutSlicesIter::from_inner(inner) }
    }
}

impl<'a, D> IntoIterator for &'a ErasedBundleMutSlicesIter<'_, D>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
{
    type Item = ErasedComponentMutSlice<'a>;
    type IntoIter = ErasedBundleMutSlicesIter<'a, FieldDescriptorsIter<'a, D>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D> Debug for ErasedBundleMutSlicesIter<'_, D>
where
    D: FieldDescriptorsOwned<Output: IntoErasedArchetypeIterator> + ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl<'a, D> Iterator for ErasedBundleMutSlicesIter<'a, D>
where
    D: ErasedArchetypeIterator + ?Sized,
{
    type Item = ErasedComponentMutSlice<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;

        let component_id = inner.field_descriptors().into_iter().next()?.into();
        let fields = inner.next()?;
        let item = unsafe { ErasedComponentMutSlice::from_parts(component_id, fields) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<D> ExactSizeIterator for ErasedBundleMutSlicesIter<'_, D>
where
    D: ErasedArchetypeIterator + ExactSizeIterator + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<D> FusedIterator for ErasedBundleMutSlicesIter<'_, D> where
    D: ErasedArchetypeIterator + FusedIterator + ?Sized
{
}

impl<'a, D> FieldDescriptors<'a> for ErasedBundleMutSlicesIter<'_, D>
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

impl<D> CovariantFieldDescriptors for ErasedBundleMutSlicesIter<'_, D>
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
