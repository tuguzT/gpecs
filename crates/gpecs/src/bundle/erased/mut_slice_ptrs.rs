use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::MaybeUninit,
};

use gpecs_soa_erased::{
    CovariantFieldDescriptors, ErasedSoaMutSlicePtrs, ErasedSoaMutSlicePtrsIter,
};

use crate::{
    archetype::{
        erased::{ErasedArchetype, ErasedArchetypeIter},
        error::IncompatibleArchetypeError,
    },
    bundle::{
        Bundle, BundleSliceMutPtrs,
        erased::{
            ErasedBundleMutPtrs, ErasedBundleMutSlices, ErasedBundleSlicePtrs,
            ErasedBundleSlicePtrsIter, ErasedBundleSlices,
        },
    },
    component::{
        erased::{
            ErasedComponentMutSlicePtr, ErasedComponentSlicePtr, WithErasedDrop,
            error::NotRegisteredError,
        },
        registry::{
            ComponentId, ComponentRegistry,
            traits::{ComponentIdFrom, FromComponentType},
        },
    },
    soa::{
        field::{FieldDescriptor, FieldDescriptors, FieldDescriptorsOutput},
        traits::RawSoaContext,
    },
};

type Inner<'a, Meta> = ErasedSoaMutSlicePtrs<&'a ErasedArchetype<Meta>, *mut MaybeUninit<u8>>;

#[derive(Debug)]
pub struct ErasedBundleMutSlicePtrs<'a, Meta> {
    inner: Inner<'a, Meta>,
}

impl<'a, Meta> ErasedBundleMutSlicePtrs<'a, Meta> {
    #[inline]
    pub unsafe fn from_inner(inner: Inner<'a, Meta>) -> Self {
        Self { inner }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedBundleMutPtrs<'a, Meta>, len: usize) -> Self {
        let inner = ptrs.into_inner();
        let inner = unsafe { Inner::from_ptrs(inner, len) };
        unsafe { Self::from_inner(inner) }
    }

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
    pub fn archetype(&self) -> &'a ErasedArchetype<Meta> {
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

    #[inline]
    pub fn into_inner(self) -> Inner<'a, Meta> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedBundleMutPtrs<'a, Meta> {
        let Self { inner } = self;

        let inner = inner.into_ptrs();
        unsafe { ErasedBundleMutPtrs::from_inner(inner) }
    }

    #[inline]
    pub fn cast_const(self) -> ErasedBundleSlicePtrs<'a, Meta> {
        let Self { inner } = self;

        let inner = inner.cast_const();
        unsafe { ErasedBundleSlicePtrs::from_inner(inner) }
    }

    #[inline]
    pub unsafe fn deref<'data>(self) -> ErasedBundleSlices<'data, 'a, Meta> {
        unsafe { self.cast_const().deref() }
    }

    #[inline]
    pub unsafe fn deref_mut<'data>(self) -> ErasedBundleMutSlices<'data, 'a, Meta> {
        unsafe { ErasedBundleMutSlices::from_ptrs(self) }
    }
}

impl<Meta> ErasedBundleMutSlicePtrs<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistry<impl Sized, T>,
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

    #[inline]
    pub fn iter(&self) -> ErasedBundleSlicePtrsIter<'_, Meta> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleSlicePtrsIter::from_inner(inner) }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> ErasedBundleMutSlicePtrsIter<'_, Meta> {
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
        self,
        registry: &ComponentRegistry<impl WithErasedDrop, impl ?Sized>,
    ) -> Result<(), NotRegisteredError> {
        self.iter()
            .map(ErasedComponentSlicePtr::component_id)
            .try_for_each(|id| {
                registry
                    .get_component_info(id)
                    .map(drop)
                    .ok_or(NotRegisteredError)
            })?;

        self.into_iter()
            .for_each(|ptr| unsafe { ptr.drop_in_place(registry) }.expect("should be registered"));
        Ok(())
    }
}

impl<Meta> Clone for ErasedBundleMutSlicePtrs<'_, Meta> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<Meta> Copy for ErasedBundleMutSlicePtrs<'_, Meta> {}

impl<'a, Meta> IntoIterator for &'a ErasedBundleMutSlicePtrs<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Item = ErasedComponentSlicePtr;
    type IntoIter = ErasedBundleSlicePtrsIter<'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, Meta> IntoIterator for &'a mut ErasedBundleMutSlicePtrs<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Item = ErasedComponentMutSlicePtr;
    type IntoIter = ErasedBundleMutSlicePtrsIter<'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, Meta> IntoIterator for ErasedBundleMutSlicePtrs<'a, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Item = ErasedComponentMutSlicePtr;
    type IntoIter = ErasedBundleMutSlicePtrsIter<'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ErasedBundleMutSlicePtrsIter::from_inner(inner) }
    }
}

impl<'me, 'a, Meta> FieldDescriptors<'me> for ErasedBundleMutSlicePtrs<'a, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Output = &'a ErasedArchetype<Meta>;

    #[inline]
    fn field_descriptors(&'me self) -> Self::Output {
        self.archetype()
    }
}

impl<Meta> CovariantFieldDescriptors for ErasedBundleMutSlicePtrs<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        from
    }
}

type InnerIter<'a, Meta> =
    ErasedSoaMutSlicePtrsIter<ErasedArchetypeIter<'a, Meta>, *mut MaybeUninit<u8>>;

pub struct ErasedBundleMutSlicePtrsIter<'a, Meta> {
    inner: InnerIter<'a, Meta>,
}

impl<'a, Meta> ErasedBundleMutSlicePtrsIter<'a, Meta> {
    #[inline]
    pub(super) unsafe fn from_inner(inner: InnerIter<'a, Meta>) -> Self {
        Self { inner }
    }

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
    pub fn descriptors(&self) -> ErasedArchetypeIter<'a, Meta> {
        let Self { inner, .. } = self;
        inner.descriptors().clone()
    }
}

impl<Meta> Debug for ErasedBundleMutSlicePtrsIter<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.clone();
        f.debug_set().entries(entries).finish()
    }
}

impl<Meta> Clone for ErasedBundleMutSlicePtrsIter<'_, Meta> {
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        Self { inner }
    }
}

impl<Meta> Iterator for ErasedBundleMutSlicePtrsIter<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Item = ErasedComponentMutSlicePtr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;

        let component_id = inner.descriptors().clone().next()?.into();
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

impl<Meta> ExactSizeIterator for ErasedBundleMutSlicePtrsIter<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<Meta> FusedIterator for ErasedBundleMutSlicePtrsIter<'_, Meta> where
    Meta: AsRef<FieldDescriptor>
{
}

impl<'me, 'a, Meta> FieldDescriptors<'me> for ErasedBundleMutSlicePtrsIter<'a, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Output = ErasedArchetypeIter<'a, Meta>;

    #[inline]
    fn field_descriptors(&'me self) -> Self::Output {
        self.descriptors()
    }
}

impl<Meta> CovariantFieldDescriptors for ErasedBundleMutSlicePtrsIter<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        from
    }
}
