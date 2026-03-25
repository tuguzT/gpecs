use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::MaybeUninit,
};

use gpecs_soa_erased::{
    CovariantFieldDescriptors, ErasedSoaPtrs, ErasedSoaPtrsIter, error::FromFieldsDescriptorsError,
    storage::AllocError,
};

use crate::{
    archetype::{
        erased::{ErasedArchetype, ErasedArchetypeIter},
        error::IncompatibleArchetypeError,
    },
    bundle::{
        Bundle, BundlePtrs,
        erased::{ErasedBorrowedBundle, ErasedBundleMutPtrs, ErasedBundleRefs, WithErasedDrop},
    },
    component::{
        erased::ErasedComponentPtr,
        registry::{ComponentId, ComponentRegistry},
    },
    soa::field::{FieldDescriptor, FieldDescriptors, FieldDescriptorsOutput},
};

type Inner<'a, Meta> = ErasedSoaPtrs<&'a ErasedArchetype<Meta>, *const MaybeUninit<u8>>;

#[derive(Debug)]
pub struct ErasedBundlePtrs<'a, Meta> {
    inner: Inner<'a, Meta>,
}

impl<'a, Meta> ErasedBundlePtrs<'a, Meta> {
    #[inline]
    pub unsafe fn from_inner(inner: Inner<'a, Meta>) -> Self {
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
    pub fn cast_mut(self) -> ErasedBundleMutPtrs<'a, Meta> {
        let Self { inner } = self;

        let inner = inner.cast_mut();
        unsafe { ErasedBundleMutPtrs::from_inner(inner) }
    }

    #[inline]
    pub unsafe fn deref<'data>(self) -> ErasedBundleRefs<'data, 'a, Meta> {
        unsafe { ErasedBundleRefs::from_ptrs(self) }
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { inner } = self;

        let inner = unsafe { inner.add(count) };
        unsafe { Self::from_inner(inner) }
    }
}

impl<Meta> ErasedBundlePtrs<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    pub fn downcast<B>(
        self,
        components: &ComponentRegistry,
    ) -> Result<BundlePtrs<B>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        self.archetype().check_compatibility_of::<B>(components)?;

        let ptrs = B::ptrs_from_erased(components, self)
            .expect("archetype compatibility should be already checked");
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
    pub fn iter(&self) -> ErasedBundlePtrsIter<'_, Meta> {
        let Self { inner } = self;

        let inner = inner.iter();
        ErasedBundlePtrsIter::from_inner(inner)
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<ErasedComponentPtr> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter().nth(index)
    }
}

impl<'a, Meta> ErasedBundlePtrs<'a, Meta>
where
    Meta: AsRef<FieldDescriptor> + WithErasedDrop + 'static,
{
    #[inline]
    pub unsafe fn read(
        &self,
    ) -> Result<ErasedBorrowedBundle<'a, Meta>, FromFieldsDescriptorsError<AllocError>> {
        let Self { inner } = self;

        let inner = unsafe { inner.read()? };
        let bundle = unsafe { ErasedBorrowedBundle::from_inner(inner) };
        Ok(bundle)
    }
}

impl<'a, Meta> ErasedBundlePtrs<'a, Meta>
where
    Meta: AsRef<FieldDescriptor> + 'static,
{
    #[inline]
    pub fn dangling(archetype: &'a ErasedArchetype<Meta>) -> Self {
        let inner = Inner::dangling(archetype)
            .expect("alignment of bytes should be sufficient for any component");
        unsafe { Self::from_inner(inner) }
    }
}

impl<Meta> Clone for ErasedBundlePtrs<'_, Meta> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<Meta> Copy for ErasedBundlePtrs<'_, Meta> {}

impl<'a, Meta> IntoIterator for &'a ErasedBundlePtrs<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Item = ErasedComponentPtr;
    type IntoIter = ErasedBundlePtrsIter<'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, Meta> IntoIterator for ErasedBundlePtrs<'a, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Item = ErasedComponentPtr;
    type IntoIter = ErasedBundlePtrsIter<'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        ErasedBundlePtrsIter::from_inner(inner)
    }
}

impl<'me, 'a, Meta> FieldDescriptors<'me> for ErasedBundlePtrs<'a, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Output = &'a ErasedArchetype<Meta>;

    #[inline]
    fn field_descriptors(&'me self) -> Self::Output {
        self.archetype()
    }
}

impl<Meta> CovariantFieldDescriptors for ErasedBundlePtrs<'_, Meta>
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

type InnerIter<'a, Meta> = ErasedSoaPtrsIter<ErasedArchetypeIter<'a, Meta>, *const MaybeUninit<u8>>;

pub struct ErasedBundlePtrsIter<'a, Meta> {
    inner: InnerIter<'a, Meta>,
}

impl<'a, Meta> ErasedBundlePtrsIter<'a, Meta> {
    #[inline]
    pub(super) fn from_inner(inner: InnerIter<'a, Meta>) -> Self {
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

impl<Meta> Debug for ErasedBundlePtrsIter<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.clone();
        f.debug_set().entries(entries).finish()
    }
}

impl<Meta> Clone for ErasedBundlePtrsIter<'_, Meta> {
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        Self { inner }
    }
}

impl<Meta> Iterator for ErasedBundlePtrsIter<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Item = ErasedComponentPtr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;

        let component_id = inner.descriptors().clone().next()?.into();
        let fields = inner.next()?;
        let item = unsafe { ErasedComponentPtr::from_parts(component_id, fields) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<Meta> ExactSizeIterator for ErasedBundlePtrsIter<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<Meta> FusedIterator for ErasedBundlePtrsIter<'_, Meta> where Meta: AsRef<FieldDescriptor> {}

impl<'me, 'a, Meta> FieldDescriptors<'me> for ErasedBundlePtrsIter<'a, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Output = ErasedArchetypeIter<'a, Meta>;

    #[inline]
    fn field_descriptors(&'me self) -> Self::Output {
        self.descriptors()
    }
}

impl<Meta> CovariantFieldDescriptors for ErasedBundlePtrsIter<'_, Meta>
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
