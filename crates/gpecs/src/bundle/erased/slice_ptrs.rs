use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::MaybeUninit,
};

use gpecs_soa_erased::{CovariantFieldDescriptors, ErasedSoaSlicePtrs, ErasedSoaSlicePtrsIter};

use crate::{
    archetype::{
        erased::{ErasedArchetype, ErasedArchetypeIter},
        error::IncompatibleArchetypeError,
    },
    bundle::{
        Bundle, BundleSlicePtrs,
        erased::{ErasedBundleMutSlicePtrs, ErasedBundlePtrs, ErasedBundleSlices},
    },
    component::{
        erased::ErasedComponentSlicePtr,
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

type Inner<'a, Meta> = ErasedSoaSlicePtrs<&'a ErasedArchetype<Meta>, *const MaybeUninit<u8>>;

#[derive(Debug)]
pub struct ErasedBundleSlicePtrs<'a, Meta> {
    inner: Inner<'a, Meta>,
}

impl<'a, Meta> ErasedBundleSlicePtrs<'a, Meta> {
    #[inline]
    pub unsafe fn from_inner(inner: Inner<'a, Meta>) -> Self {
        Self { inner }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedBundlePtrs<'a, Meta>, len: usize) -> Self {
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
    pub fn into_ptrs(self) -> ErasedBundlePtrs<'a, Meta> {
        let Self { inner } = self;

        let inner = inner.into_ptrs();
        unsafe { ErasedBundlePtrs::from_inner(inner) }
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedBundleMutSlicePtrs<'a, Meta> {
        let Self { inner } = self;

        let inner = inner.cast_mut();
        unsafe { ErasedBundleMutSlicePtrs::from_inner(inner) }
    }

    #[inline]
    pub unsafe fn deref<'data>(self) -> ErasedBundleSlices<'data, 'a, Meta> {
        unsafe { ErasedBundleSlices::from_ptrs(self) }
    }
}

impl<Meta> ErasedBundleSlicePtrs<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistry<impl Sized, T>,
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

    #[inline]
    pub fn iter(&self) -> ErasedBundleSlicePtrsIter<'_, Meta> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleSlicePtrsIter::from_inner(inner) }
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<ErasedComponentSlicePtr> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter().nth(index)
    }
}

impl<Meta> Clone for ErasedBundleSlicePtrs<'_, Meta> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<Meta> Copy for ErasedBundleSlicePtrs<'_, Meta> {}

impl<'a, Meta> IntoIterator for &'a ErasedBundleSlicePtrs<'_, Meta>
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

impl<'a, Meta> IntoIterator for ErasedBundleSlicePtrs<'a, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Item = ErasedComponentSlicePtr;
    type IntoIter = ErasedBundleSlicePtrsIter<'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ErasedBundleSlicePtrsIter::from_inner(inner) }
    }
}

impl<'me, 'a, Meta> FieldDescriptors<'me> for ErasedBundleSlicePtrs<'a, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Output = &'a ErasedArchetype<Meta>;

    #[inline]
    fn field_descriptors(&'me self) -> Self::Output {
        self.archetype()
    }
}

impl<Meta> CovariantFieldDescriptors for ErasedBundleSlicePtrs<'_, Meta>
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
    ErasedSoaSlicePtrsIter<ErasedArchetypeIter<'a, Meta>, *const MaybeUninit<u8>>;

pub struct ErasedBundleSlicePtrsIter<'a, Meta> {
    inner: InnerIter<'a, Meta>,
}

impl<'a, Meta> ErasedBundleSlicePtrsIter<'a, Meta> {
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

impl<Meta> Debug for ErasedBundleSlicePtrsIter<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.clone();
        f.debug_set().entries(entries).finish()
    }
}

impl<Meta> Clone for ErasedBundleSlicePtrsIter<'_, Meta> {
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        Self { inner }
    }
}

impl<Meta> Iterator for ErasedBundleSlicePtrsIter<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Item = ErasedComponentSlicePtr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;

        let component_id = inner.descriptors().clone().next()?.into();
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

impl<Meta> ExactSizeIterator for ErasedBundleSlicePtrsIter<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<Meta> FusedIterator for ErasedBundleSlicePtrsIter<'_, Meta> where Meta: AsRef<FieldDescriptor> {}

impl<'me, 'a, Meta> FieldDescriptors<'me> for ErasedBundleSlicePtrsIter<'a, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Output = ErasedArchetypeIter<'a, Meta>;

    #[inline]
    fn field_descriptors(&'me self) -> Self::Output {
        self.descriptors()
    }
}

impl<Meta> CovariantFieldDescriptors for ErasedBundleSlicePtrsIter<'_, Meta>
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
