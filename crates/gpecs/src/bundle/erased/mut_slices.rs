use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::MaybeUninit,
    ptr,
};

use gpecs_soa_erased::{CovariantFieldDescriptors, ErasedSoaMutSlices, ErasedSoaMutSlicesIter};

use crate::{
    archetype::erased::{ErasedArchetype, Iter, error::IncompatibleArchetypeError},
    bundle::{
        Bundle, BundleSlicesMut,
        erased::{ErasedBundleMutSlicePtrs, ErasedBundleSlices, ErasedBundleSlicesIter},
    },
    component::{
        erased::{ErasedComponentMutSlice, ErasedComponentSlice},
        registry::{
            ComponentId, ComponentRegistryView,
            traits::{ComponentIdFrom, FromComponentType},
        },
    },
    soa::{
        field::{FieldDescriptor, FieldDescriptors, FieldDescriptorsOutput},
        traits::SoaContext,
    },
};

type Inner<'data, 'a, Meta> =
    ErasedSoaMutSlices<'data, &'a ErasedArchetype<Meta>, *mut MaybeUninit<u8>>;

#[derive(Debug)]
pub struct ErasedBundleMutSlices<'data, 'a, Meta> {
    inner: Inner<'data, 'a, Meta>,
}

impl<'data, 'a, Meta> ErasedBundleMutSlices<'data, 'a, Meta> {
    #[inline]
    pub unsafe fn from_inner(inner: Inner<'data, 'a, Meta>) -> Self {
        Self { inner }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedBundleMutSlicePtrs<'a, Meta>) -> Self {
        let inner = ptrs.into_inner();
        let inner = unsafe { inner.deref_mut() };
        unsafe { Self::from_inner(inner) }
    }

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
    pub fn into_inner(self) -> Inner<'data, 'a, Meta> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedBundleMutSlicePtrs<'a, Meta> {
        let Self { inner } = self;

        let inner = inner.into_ptrs();
        unsafe { ErasedBundleMutSlicePtrs::from_inner(inner) }
    }
}

impl<'data, Meta> ErasedBundleMutSlices<'data, '_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleSlicesMut<'data, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let slices = self.into_ptrs().downcast::<B, T>(components)?;
        let slices = unsafe { B::CONTEXT.mut_slice_ptrs_to_mut_slices(slices) };
        Ok(slices)
    }

    #[inline]
    pub fn iter(&self) -> ErasedBundleSlicesIter<'_, '_, Meta> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleSlicesIter::from_inner(inner) }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> ErasedBundleMutSlicesIter<'_, '_, Meta> {
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

impl<'a, Meta> IntoIterator for &'a ErasedBundleMutSlices<'_, '_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Item = ErasedComponentSlice<'a>;
    type IntoIter = ErasedBundleSlicesIter<'a, 'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, Meta> IntoIterator for &'a mut ErasedBundleMutSlices<'_, '_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Item = ErasedComponentMutSlice<'a>;
    type IntoIter = ErasedBundleMutSlicesIter<'a, 'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'data, 'a, Meta> IntoIterator for ErasedBundleMutSlices<'data, 'a, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Item = ErasedComponentMutSlice<'data>;
    type IntoIter = ErasedBundleMutSlicesIter<'data, 'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ErasedBundleMutSlicesIter::from_inner(inner) }
    }
}

impl<'data, 'a, Meta> From<ErasedBundleMutSlices<'data, 'a, Meta>>
    for ErasedBundleSlices<'data, 'a, Meta>
{
    #[inline]
    fn from(slices: ErasedBundleMutSlices<'data, 'a, Meta>) -> Self {
        let inner = slices.into_inner();
        let inner = inner.into();
        unsafe { Self::from_inner(inner) }
    }
}

impl<'a, Meta> FieldDescriptors<'a> for ErasedBundleMutSlices<'_, '_, Meta>
where
    Meta: AsRef<FieldDescriptor> + 'a,
{
    type Output = &'a ErasedArchetype<Meta>;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        self.archetype()
    }
}

impl<Meta> CovariantFieldDescriptors for ErasedBundleMutSlices<'_, '_, Meta>
where
    Meta: AsRef<FieldDescriptor> + 'static,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        from
    }
}

type InnerIter<'data, 'a, Meta> =
    ErasedSoaMutSlicesIter<'data, Iter<'a, Meta>, *mut MaybeUninit<u8>>;

pub struct ErasedBundleMutSlicesIter<'data, 'a, Meta> {
    inner: InnerIter<'data, 'a, Meta>,
}

impl<'data, 'a, Meta> ErasedBundleMutSlicesIter<'data, 'a, Meta> {
    #[inline]
    pub(super) unsafe fn from_inner(inner: InnerIter<'data, 'a, Meta>) -> Self {
        Self { inner }
    }

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
    pub fn descriptors(&self) -> Iter<'a, Meta> {
        let Self { inner, .. } = self;
        inner.descriptors().clone()
    }
}

impl<Meta> Debug for ErasedBundleMutSlicesIter<'_, '_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = unsafe { ptr::from_ref(self).read() };
        f.debug_set().entries(entries).finish()
    }
}

impl<'data, Meta> Iterator for ErasedBundleMutSlicesIter<'data, '_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Item = ErasedComponentMutSlice<'data>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;

        let component_id = inner.descriptors().clone().next()?.into();
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

impl<Meta> ExactSizeIterator for ErasedBundleMutSlicesIter<'_, '_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<Meta> FusedIterator for ErasedBundleMutSlicesIter<'_, '_, Meta> where
    Meta: AsRef<FieldDescriptor>
{
}

impl<'a, Meta> FieldDescriptors<'a> for ErasedBundleMutSlicesIter<'_, '_, Meta>
where
    Meta: AsRef<FieldDescriptor> + 'a,
{
    type Output = Iter<'a, Meta>;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        self.descriptors()
    }
}

impl<Meta> CovariantFieldDescriptors for ErasedBundleMutSlicesIter<'_, '_, Meta>
where
    Meta: AsRef<FieldDescriptor> + 'static,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        from
    }
}
