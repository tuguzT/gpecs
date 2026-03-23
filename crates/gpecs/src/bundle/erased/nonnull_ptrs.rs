use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::MaybeUninit,
    ptr::NonNull,
};

use gpecs_soa_erased::{CovariantFieldDescriptors, ErasedSoaNonNullPtrs, ErasedSoaNonNullPtrsIter};

use crate::{
    archetype::{
        erased::{ErasedArchetype, ErasedArchetypeIter},
        error::IncompatibleArchetypeError,
    },
    bundle::{Bundle, BundleNonNullPtrs, erased::ErasedBundleMutPtrs},
    component::{
        erased::ErasedComponentNonNullPtr,
        registry::{ComponentId, ComponentRegistry},
    },
    soa::{
        field::{FieldDescriptor, FieldDescriptors, FieldDescriptorsOutput},
        traits::RawSoaContext,
    },
};

type Inner<'a, Meta> = ErasedSoaNonNullPtrs<&'a ErasedArchetype<Meta>, NonNull<MaybeUninit<u8>>>;

#[derive(Debug)]
pub struct ErasedBundleNonNullPtrs<'a, Meta> {
    inner: Inner<'a, Meta>,
}

impl<'a, Meta> ErasedBundleNonNullPtrs<'a, Meta> {
    #[inline]
    pub fn new(ptrs: ErasedBundleMutPtrs<'a, Meta>) -> Option<Self> {
        let ptrs = ptrs.into_inner();
        let inner = Inner::new(ptrs)?;

        let me = unsafe { Self::from_inner(inner) };
        Some(me)
    }

    #[inline]
    pub unsafe fn new_unchecked(ptrs: ErasedBundleMutPtrs<'a, Meta>) -> Self {
        let ptrs = ptrs.into_inner();
        let inner = unsafe { Inner::new_unchecked(ptrs) };
        unsafe { Self::from_inner(inner) }
    }

    #[inline]
    pub unsafe fn from_inner(inner: Inner<'a, Meta>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn as_buffer(&self) -> NonNull<[MaybeUninit<u8>]> {
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
    pub fn into_inner(self) -> Inner<'a, Meta> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { inner } = self;

        let inner = unsafe { inner.add(count) };
        unsafe { Self::from_inner(inner) }
    }
}

impl<Meta> ErasedBundleNonNullPtrs<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    pub fn downcast<B>(
        self,
        components: &ComponentRegistry,
    ) -> Result<BundleNonNullPtrs<B>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        let ptrs = ErasedBundleMutPtrs::from(self).downcast::<B>(components)?;
        let ptrs = unsafe { B::CONTEXT.ptrs_to_nonnull(ptrs) };
        Ok(ptrs)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn offset_from(&self, origin: &Self) -> isize {
        let Self { inner } = self;

        let origin = &origin.into_inner();
        unsafe { inner.offset_from(origin) }
    }

    #[inline]
    pub fn iter(&self) -> ErasedBundleNonNullPtrsIter<'_, Meta> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleNonNullPtrsIter::from_inner(inner) }
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<ErasedComponentNonNullPtr> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter().nth(index)
    }
}

impl<'a, Meta> ErasedBundleNonNullPtrs<'a, Meta>
where
    Meta: AsRef<FieldDescriptor> + 'static,
{
    #[inline]
    pub fn dangling(archetype: &'a ErasedArchetype<Meta>) -> Self {
        let inner = Inner::dangling(archetype)
            .expect("alignment of bytes should be sufficient for any component");
        unsafe { Self::from_inner(inner) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn swap(&mut self, with: &mut Self) {
        let Self { inner } = self;

        let with = &mut with.into_inner();
        unsafe { inner.swap(with) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_forward(&mut self, src: &Self, count: usize) {
        let Self { inner } = self;

        let src = &src.into_inner();
        unsafe { inner.copy_from_forward(src, count) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_backward(&mut self, src: &Self, count: usize) {
        let Self { inner } = self;

        let src = &src.into_inner();
        unsafe { inner.copy_from_backward(src, count) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_nonoverlapping(&mut self, src: &Self, count: usize) {
        let Self { inner } = self;

        let src = &src.into_inner();
        unsafe { inner.copy_from_nonoverlapping(src, count) }
    }
}

impl<Meta> Clone for ErasedBundleNonNullPtrs<'_, Meta> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<Meta> Copy for ErasedBundleNonNullPtrs<'_, Meta> {}

impl<'a, Meta> IntoIterator for &'a ErasedBundleNonNullPtrs<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Item = ErasedComponentNonNullPtr;
    type IntoIter = ErasedBundleNonNullPtrsIter<'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, Meta> IntoIterator for ErasedBundleNonNullPtrs<'a, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Item = ErasedComponentNonNullPtr;
    type IntoIter = ErasedBundleNonNullPtrsIter<'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ErasedBundleNonNullPtrsIter::from_inner(inner) }
    }
}

impl<'a, Meta> From<ErasedBundleNonNullPtrs<'a, Meta>> for ErasedBundleMutPtrs<'a, Meta> {
    #[inline]
    fn from(ptrs: ErasedBundleNonNullPtrs<'a, Meta>) -> Self {
        let inner = ptrs.into_inner();
        let inner = inner.into();
        unsafe { ErasedBundleMutPtrs::from_inner(inner) }
    }
}

impl<'me, 'a, Meta> FieldDescriptors<'me> for ErasedBundleNonNullPtrs<'a, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Output = &'a ErasedArchetype<Meta>;

    #[inline]
    fn field_descriptors(&'me self) -> Self::Output {
        self.archetype()
    }
}

impl<Meta> CovariantFieldDescriptors for ErasedBundleNonNullPtrs<'_, Meta>
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
    ErasedSoaNonNullPtrsIter<ErasedArchetypeIter<'a, Meta>, NonNull<MaybeUninit<u8>>>;

pub struct ErasedBundleNonNullPtrsIter<'a, Meta> {
    inner: InnerIter<'a, Meta>,
}

impl<'a, Meta> ErasedBundleNonNullPtrsIter<'a, Meta> {
    #[inline]
    pub(super) unsafe fn from_inner(inner: InnerIter<'a, Meta>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn as_buffer(&self) -> NonNull<[MaybeUninit<u8>]> {
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

impl<Meta> Debug for ErasedBundleNonNullPtrsIter<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.clone();
        f.debug_set().entries(entries).finish()
    }
}

impl<Meta> Clone for ErasedBundleNonNullPtrsIter<'_, Meta> {
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        Self { inner }
    }
}

impl<Meta> Iterator for ErasedBundleNonNullPtrsIter<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Item = ErasedComponentNonNullPtr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;

        let component_id = inner.descriptors().clone().next()?.into();
        let fields = inner.next()?;
        let item = unsafe { ErasedComponentNonNullPtr::from_parts(component_id, fields) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<Meta> ExactSizeIterator for ErasedBundleNonNullPtrsIter<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<Meta> FusedIterator for ErasedBundleNonNullPtrsIter<'_, Meta> where Meta: AsRef<FieldDescriptor>
{}

impl<'me, 'a, Meta> FieldDescriptors<'me> for ErasedBundleNonNullPtrsIter<'a, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Output = ErasedArchetypeIter<'a, Meta>;

    #[inline]
    fn field_descriptors(&'me self) -> Self::Output {
        self.descriptors()
    }
}

impl<Meta> CovariantFieldDescriptors for ErasedBundleNonNullPtrsIter<'_, Meta>
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
