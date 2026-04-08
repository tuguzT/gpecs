use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::MaybeUninit,
};

use gpecs_soa_erased::{
    CovariantFieldDescriptors, ErasedSoaMutSlicePtrs, ErasedSoaMutSlicePtrsIter,
};

use crate::{
    archetype::erased::{ErasedArchetypeView, Iter, error::IncompatibleArchetypeError},
    bundle::{
        Bundle, BundleSliceMutPtrs,
        erased::{
            ErasedBundleMutPtrs, ErasedBundleMutSlices, ErasedBundleSlicePtrs,
            ErasedBundleSlicePtrsIter, ErasedBundleSlices,
            traits::{ErasedArchetypeIterator, ErasedArchetypeKind, IntoErasedArchetypeIterator},
        },
    },
    component::{
        erased::{
            ErasedComponentMutSlicePtr, ErasedComponentSlicePtr, WithErasedDrop,
            error::NotRegisteredError,
        },
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

type Inner<D> = ErasedSoaMutSlicePtrs<D, *mut MaybeUninit<u8>>;

#[derive(Debug)]
pub struct ErasedBundleMutSlicePtrs<D>
where
    D: ?Sized,
{
    inner: Inner<D>,
}

impl<D> ErasedBundleMutSlicePtrs<D> {
    #[inline]
    pub unsafe fn from_inner(inner: Inner<D>) -> Self {
        Self { inner }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedBundleMutPtrs<D>, len: usize) -> Self {
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
    pub fn into_ptrs(self) -> ErasedBundleMutPtrs<D> {
        let Self { inner } = self;

        let inner = inner.into_ptrs();
        unsafe { ErasedBundleMutPtrs::from_inner(inner) }
    }

    #[inline]
    pub fn cast_const(self) -> ErasedBundleSlicePtrs<D> {
        let Self { inner } = self;

        let inner = inner.cast_const();
        unsafe { ErasedBundleSlicePtrs::from_inner(inner) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedBundleSlices<'a, D> {
        unsafe { self.cast_const().deref() }
    }

    #[inline]
    pub unsafe fn deref_mut<'a>(self) -> ErasedBundleMutSlices<'a, D> {
        unsafe { ErasedBundleMutSlices::from_ptrs(self) }
    }
}

impl<D> ErasedBundleMutSlicePtrs<D>
where
    D: ?Sized,
{
    #[inline]
    pub fn as_buffer(&self) -> *const [MaybeUninit<u8>] {
        let Self { inner } = self;
        inner.as_buffer()
    }

    #[inline]
    pub unsafe fn as_mut_buffer(&mut self) -> *mut [MaybeUninit<u8>] {
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

impl<D> ErasedBundleMutSlicePtrs<D>
where
    D: ErasedArchetypeKind,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleSliceMutPtrs<B>, IncompatibleArchetypeError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let len = self.len();
        let ptrs = self.into_ptrs().downcast::<B, T>(components)?;
        let slices = B::CONTEXT.mut_slice_ptrs_from_raw_parts(ptrs, len);
        Ok(slices)
    }
}

impl<D> ErasedBundleMutSlicePtrs<D>
where
    D: ErasedArchetypeKind + ?Sized,
{
    #[inline]
    pub fn archetype(&self) -> ErasedArchetypeView<'_, D::Meta> {
        self.field_descriptors()
    }

    #[inline]
    pub fn iter(&self) -> ErasedBundleSlicePtrsIter<Iter<'_, D::Meta>> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleSlicePtrsIter::from_inner(inner) }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> ErasedBundleMutSlicePtrsIter<Iter<'_, D::Meta>> {
        let Self { inner } = self;

        let inner = inner.iter_mut();
        unsafe { ErasedBundleMutSlicePtrsIter::from_inner(inner) }
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<ErasedComponentSlicePtr> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter().nth(index)
    }

    #[inline]
    pub fn get_mut(&mut self, component_id: ComponentId) -> Option<ErasedComponentMutSlicePtr> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter_mut().nth(index)
    }

    #[inline]
    pub unsafe fn drop_in_place(
        &mut self,
        components: &ComponentRegistryView<impl WithErasedDrop, impl ?Sized>,
    ) -> Result<(), NotRegisteredError> {
        self.iter()
            .map(ErasedComponentSlicePtr::component_id)
            .try_for_each(|id| {
                components
                    .get_component_info(id)
                    .map(drop)
                    .ok_or_else(NotRegisteredError::new)
            })?;

        self.iter_mut().for_each(|slice| {
            if let Err(error) = unsafe { slice.drop_in_place(components) } {
                unreachable!("{error}, but it was checked earlier to be registered")
            }
        });
        Ok(())
    }
}

impl<D> Clone for ErasedBundleMutSlicePtrs<D>
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

impl<D> Copy for ErasedBundleMutSlicePtrs<D> where D: Copy {}

impl<'a, D> IntoIterator for &'a ErasedBundleMutSlicePtrs<D>
where
    D: ErasedArchetypeKind + ?Sized,
{
    type Item = ErasedComponentSlicePtr;
    type IntoIter = ErasedBundleSlicePtrsIter<Iter<'a, D::Meta>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D> IntoIterator for &'a mut ErasedBundleMutSlicePtrs<D>
where
    D: ErasedArchetypeKind + ?Sized,
{
    type Item = ErasedComponentMutSlicePtr;
    type IntoIter = ErasedBundleMutSlicePtrsIter<Iter<'a, D::Meta>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<D> IntoIterator for ErasedBundleMutSlicePtrs<D>
where
    D: IntoErasedArchetypeIterator,
{
    type Item = ErasedComponentMutSlicePtr;
    type IntoIter = ErasedBundleMutSlicePtrsIter<D::IntoIter>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ErasedBundleMutSlicePtrsIter::from_inner(inner) }
    }
}

impl<'a, D> FieldDescriptors<'a> for ErasedBundleMutSlicePtrs<D>
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

impl<D> CovariantFieldDescriptors for ErasedBundleMutSlicePtrs<D>
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

type InnerIter<D> = ErasedSoaMutSlicePtrsIter<D, *mut MaybeUninit<u8>>;

pub struct ErasedBundleMutSlicePtrsIter<D>
where
    D: ?Sized,
{
    inner: InnerIter<D>,
}

impl<D> ErasedBundleMutSlicePtrsIter<D> {
    #[inline]
    pub(super) unsafe fn from_inner(inner: InnerIter<D>) -> Self {
        Self { inner }
    }
}

impl<D> ErasedBundleMutSlicePtrsIter<D>
where
    D: ?Sized,
{
    #[inline]
    pub fn as_buffer(&self) -> *const [MaybeUninit<u8>] {
        let Self { inner } = self;
        inner.as_buffer()
    }

    #[inline]
    pub unsafe fn as_mut_buffer(&mut self) -> *mut [MaybeUninit<u8>] {
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

impl<'a, D> ErasedBundleMutSlicePtrsIter<D>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedBundleMutSlicePtrsIter<FieldDescriptorsIter<'a, D>> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleMutSlicePtrsIter::from_inner(inner) }
    }
}

impl<'a, D> IntoIterator for &'a ErasedBundleMutSlicePtrsIter<D>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
{
    type Item = ErasedComponentMutSlicePtr;
    type IntoIter = ErasedBundleMutSlicePtrsIter<FieldDescriptorsIter<'a, D>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D> Debug for ErasedBundleMutSlicePtrsIter<D>
where
    D: FieldDescriptorsOwned<Output: IntoErasedArchetypeIterator> + ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl<D> Clone for ErasedBundleMutSlicePtrsIter<D>
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

impl<D> Iterator for ErasedBundleMutSlicePtrsIter<D>
where
    D: ErasedArchetypeIterator + ?Sized,
{
    type Item = ErasedComponentMutSlicePtr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;

        let component_id = inner.field_descriptors().into_iter().next()?.component_id();
        let fields = inner.next()?;
        let item = unsafe { ErasedComponentMutSlicePtr::from_parts(component_id, fields) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<D> ExactSizeIterator for ErasedBundleMutSlicePtrsIter<D>
where
    D: ErasedArchetypeIterator + ExactSizeIterator + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<D> FusedIterator for ErasedBundleMutSlicePtrsIter<D> where
    D: ErasedArchetypeIterator + FusedIterator + ?Sized
{
}

impl<'a, D> FieldDescriptors<'a> for ErasedBundleMutSlicePtrsIter<D>
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

impl<D> CovariantFieldDescriptors for ErasedBundleMutSlicePtrsIter<D>
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
