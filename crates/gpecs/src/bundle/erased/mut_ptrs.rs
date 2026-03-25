use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::MaybeUninit,
};

use gpecs_soa_erased::{CovariantFieldDescriptors, ErasedSoaMutPtrs, ErasedSoaMutPtrsIter};

use crate::{
    archetype::{
        erased::{ErasedArchetype, ErasedArchetypeIter},
        error::IncompatibleArchetypeError,
    },
    bundle::{
        Bundle, BundleMutPtrs,
        erased::{
            ErasedArchetypeKind, ErasedBundleKind, ErasedBundlePtrs, ErasedBundlePtrsIter,
            ErasedBundleRefs, mut_refs::ErasedBundleMutRefs,
        },
    },
    component::{
        erased::{ErasedComponentMutPtr, ErasedComponentPtr, ErasedDrop},
        error::NotRegisteredError,
        registry::{ComponentId, ComponentRegistry},
    },
    soa::field::{FieldDescriptor, FieldDescriptors, FieldDescriptorsOutput},
};

type Inner<'a, Meta> = ErasedSoaMutPtrs<&'a ErasedArchetype<Meta>, *mut MaybeUninit<u8>>;

#[derive(Debug)]
pub struct ErasedBundleMutPtrs<'a, Meta> {
    inner: Inner<'a, Meta>,
}

impl<'a, Meta> ErasedBundleMutPtrs<'a, Meta> {
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
    pub fn into_inner(self) -> Inner<'a, Meta> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn cast_const(self) -> ErasedBundlePtrs<'a, Meta> {
        let Self { inner } = self;

        let inner = inner.cast_const();
        unsafe { ErasedBundlePtrs::from_inner(inner) }
    }

    #[inline]
    pub unsafe fn deref<'data>(self) -> ErasedBundleRefs<'data, 'a, Meta> {
        unsafe { self.cast_const().deref() }
    }

    #[inline]
    pub unsafe fn deref_mut<'data>(self) -> ErasedBundleMutRefs<'data, 'a, Meta> {
        unsafe { ErasedBundleMutRefs::from_ptrs(self) }
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { inner } = self;

        let inner = unsafe { inner.add(count) };
        unsafe { Self::from_inner(inner) }
    }
}

impl<Meta> ErasedBundleMutPtrs<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    pub fn downcast<B>(
        self,
        components: &ComponentRegistry,
    ) -> Result<BundleMutPtrs<B>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        self.archetype().check_compatibility_of::<B>(components)?;

        let ptrs = B::mut_ptrs_from_erased(components, self)
            .expect("archetype compatibility should be already checked");
        Ok(ptrs)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn offset_from(&self, origin: &ErasedBundlePtrs<'_, Meta>) -> isize {
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
    pub fn iter_mut(&mut self) -> ErasedBundleMutPtrsIter<'_, Meta> {
        let Self { inner } = self;

        let inner = inner.iter_mut();
        unsafe { ErasedBundleMutPtrsIter::from_inner(inner) }
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<ErasedComponentPtr> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter().nth(index)
    }

    #[inline]
    pub fn get_mut(&mut self, component_id: ComponentId) -> Option<ErasedComponentMutPtr> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter_mut().nth(index)
    }

    #[inline]
    pub unsafe fn drop_in_place(
        self,
        registry: &ComponentRegistry,
    ) -> Result<(), NotRegisteredError> {
        self.iter()
            .map(ErasedComponentPtr::component_id)
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

impl<'a, Meta> ErasedBundleMutPtrs<'a, Meta>
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
    pub unsafe fn swap(&mut self, with: &mut ErasedBundleMutPtrs<'_, Meta>) {
        let Self { inner } = self;

        let with = &mut with.into_inner();
        unsafe { inner.swap(with) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_forward(&mut self, src: &ErasedBundlePtrs<'_, Meta>, count: usize) {
        let Self { inner } = self;

        let src = &src.into_inner();
        unsafe { inner.copy_from_forward(src, count) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_backward(&mut self, src: &ErasedBundlePtrs<'_, Meta>, count: usize) {
        let Self { inner } = self;

        let src = &src.into_inner();
        unsafe { inner.copy_from_backward(src, count) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_nonoverlapping(
        &mut self,
        src: &ErasedBundlePtrs<'_, Meta>,
        count: usize,
    ) {
        let Self { inner } = self;

        let src = &src.into_inner();
        unsafe { inner.copy_from_nonoverlapping(src, count) }
    }
}

impl<Meta> ErasedBundleMutPtrs<'_, Meta>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<ErasedDrop>> + 'static,
{
    #[inline]
    pub unsafe fn write<T>(&mut self, value: ErasedBundleKind<T>)
    where
        T: ErasedArchetypeKind<Meta = Meta>,
    {
        let Self { inner } = self;

        let value = value.into_inner();
        unsafe { inner.write(value) }
    }
}

impl<Meta> Clone for ErasedBundleMutPtrs<'_, Meta> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<Meta> Copy for ErasedBundleMutPtrs<'_, Meta> {}

impl<'a, Meta> IntoIterator for &'a ErasedBundleMutPtrs<'_, Meta>
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

impl<'a, Meta> IntoIterator for &'a mut ErasedBundleMutPtrs<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Item = ErasedComponentMutPtr;
    type IntoIter = ErasedBundleMutPtrsIter<'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, Meta> IntoIterator for ErasedBundleMutPtrs<'a, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Item = ErasedComponentMutPtr;
    type IntoIter = ErasedBundleMutPtrsIter<'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ErasedBundleMutPtrsIter::from_inner(inner) }
    }
}

impl<'me, 'a, Meta> FieldDescriptors<'me> for ErasedBundleMutPtrs<'a, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Output = &'a ErasedArchetype<Meta>;

    #[inline]
    fn field_descriptors(&'me self) -> Self::Output {
        self.archetype()
    }
}

impl<Meta> CovariantFieldDescriptors for ErasedBundleMutPtrs<'_, Meta>
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
    ErasedSoaMutPtrsIter<ErasedArchetypeIter<'a, Meta>, *mut MaybeUninit<u8>>;

pub struct ErasedBundleMutPtrsIter<'a, Meta> {
    inner: InnerIter<'a, Meta>,
}

impl<'a, Meta> ErasedBundleMutPtrsIter<'a, Meta> {
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

impl<Meta> Debug for ErasedBundleMutPtrsIter<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.clone();
        f.debug_set().entries(entries).finish()
    }
}

impl<Meta> Clone for ErasedBundleMutPtrsIter<'_, Meta> {
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        Self { inner }
    }
}

impl<Meta> Iterator for ErasedBundleMutPtrsIter<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Item = ErasedComponentMutPtr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;

        let component_id = inner.descriptors().clone().next()?.into();
        let fields = inner.next()?;
        let item = unsafe { ErasedComponentMutPtr::from_parts(component_id, fields) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<Meta> ExactSizeIterator for ErasedBundleMutPtrsIter<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<Meta> FusedIterator for ErasedBundleMutPtrsIter<'_, Meta> where Meta: AsRef<FieldDescriptor> {}

impl<'me, 'a, Meta> FieldDescriptors<'me> for ErasedBundleMutPtrsIter<'a, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Output = ErasedArchetypeIter<'a, Meta>;

    #[inline]
    fn field_descriptors(&'me self) -> Self::Output {
        self.descriptors()
    }
}

impl<Meta> CovariantFieldDescriptors for ErasedBundleMutPtrsIter<'_, Meta>
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
