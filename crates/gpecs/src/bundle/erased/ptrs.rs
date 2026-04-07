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
    archetype::erased::{
        ErasedArchetypeView, ErasedArchetypeViewExt, Iter, error::IncompatibleArchetypeError,
    },
    bundle::{
        Bundle, BundlePtrs,
        erased::{ErasedArchetypeKind, ErasedBundleKind, ErasedBundleMutPtrs, ErasedBundleRefs},
    },
    component::{
        erased::{ErasedComponentPtr, WithErasedDrop},
        registry::{
            ComponentId, ComponentRegistryView,
            traits::{ComponentIdFrom, FromComponentType},
        },
    },
    soa::field::{
        FieldDescriptor, FieldDescriptors, FieldDescriptorsItem, FieldDescriptorsIter,
        FieldDescriptorsOutput, FieldDescriptorsOwned,
    },
};

type Inner<D> = ErasedSoaPtrs<D, *const MaybeUninit<u8>>;

#[derive(Debug)]
pub struct ErasedBundlePtrs<D>
where
    D: ?Sized,
{
    inner: Inner<D>,
}

impl<D> ErasedBundlePtrs<D> {
    #[inline]
    pub unsafe fn from_inner(inner: Inner<D>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn into_inner(self) -> Inner<D> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedBundleMutPtrs<D> {
        let Self { inner } = self;

        let inner = inner.cast_mut();
        unsafe { ErasedBundleMutPtrs::from_inner(inner) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedBundleRefs<'a, D> {
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

impl<D> ErasedBundlePtrs<D>
where
    D: ?Sized,
{
    #[inline]
    pub unsafe fn as_inner(&self) -> &Inner<D> {
        let Self { inner } = self;
        inner
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
    pub fn descriptors(&self) -> &D {
        let Self { inner } = self;
        inner.descriptors()
    }
}

impl<D> ErasedBundlePtrs<D>
where
    D: ErasedArchetypeKind,
{
    #[inline]
    pub fn dangling(archetype: D) -> Self {
        let inner = Inner::dangling(archetype)
            .expect("alignment of bytes should be sufficient for any component");
        unsafe { Self::from_inner(inner) }
    }

    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundlePtrs<B>, IncompatibleArchetypeError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        self.archetype()
            .check_compatibility_of::<B, T>(components)?;

        let ptrs = B::ptrs_from_erased(components, self.iter())
            .expect("archetype compatibility should be already checked");
        Ok(ptrs)
    }
}

impl<D> ErasedBundlePtrs<D>
where
    D: ErasedArchetypeKind + ?Sized,
{
    #[inline]
    pub fn archetype(&self) -> ErasedArchetypeView<'_, D::Meta> {
        self.field_descriptors()
    }

    #[inline]
    #[track_caller]
    pub unsafe fn offset_from<N>(&self, origin: &ErasedBundlePtrs<N>) -> isize
    where
        N: ErasedArchetypeKind + ?Sized,
    {
        let Self { inner } = self;

        let origin = unsafe { origin.as_inner() };
        unsafe { inner.offset_from(origin) }
    }

    #[inline]
    pub fn iter(&self) -> ErasedBundlePtrsIter<Iter<'_, D::Meta>> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundlePtrsIter::from_inner(inner) }
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<ErasedComponentPtr> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter().nth(index)
    }
}

impl<D> ErasedBundlePtrs<D>
where
    D: ErasedArchetypeKind<Meta: WithErasedDrop> + Clone,
{
    #[inline]
    pub unsafe fn read(
        &self,
    ) -> Result<ErasedBundleKind<D>, FromFieldsDescriptorsError<AllocError>> {
        let Self { inner } = self;

        let inner = unsafe { inner.read()? };
        let bundle = unsafe { ErasedBundleKind::from_inner(inner) };
        Ok(bundle)
    }
}

impl<D> Clone for ErasedBundlePtrs<D>
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

impl<D> Copy for ErasedBundlePtrs<D> where D: Copy {}

impl<'a, D> IntoIterator for &'a ErasedBundlePtrs<D>
where
    D: ErasedArchetypeKind + ?Sized,
{
    type Item = ErasedComponentPtr;
    type IntoIter = ErasedBundlePtrsIter<Iter<'a, D::Meta>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D> IntoIterator for ErasedBundlePtrs<D>
where
    D: IntoIterator<Item: AsRef<FieldDescriptor>, IntoIter: FieldDescriptorsOwned>,
    for<'a> FieldDescriptorsItem<'a, D::IntoIter>: Into<ComponentId>,
{
    type Item = ErasedComponentPtr;
    type IntoIter = ErasedBundlePtrsIter<D::IntoIter>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ErasedBundlePtrsIter::from_inner(inner) }
    }
}

impl<'a, D> FieldDescriptors<'a> for ErasedBundlePtrs<D>
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

impl<D> CovariantFieldDescriptors for ErasedBundlePtrs<D>
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

type InnerIter<D> = ErasedSoaPtrsIter<D, *const MaybeUninit<u8>>;

pub struct ErasedBundlePtrsIter<D>
where
    D: ?Sized,
{
    inner: InnerIter<D>,
}

impl<D> ErasedBundlePtrsIter<D> {
    #[inline]
    pub(super) unsafe fn from_inner(inner: InnerIter<D>) -> Self {
        Self { inner }
    }
}

impl<D> ErasedBundlePtrsIter<D>
where
    D: ?Sized,
{
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
    pub fn descriptors(&self) -> &D {
        let Self { inner, .. } = self;
        inner.descriptors()
    }
}

impl<'a, D> ErasedBundlePtrsIter<D>
where
    D: FieldDescriptors<'a> + ?Sized,
    FieldDescriptorsIter<'a, D>: FieldDescriptorsOwned,
    for<'b> FieldDescriptorsItem<'b, FieldDescriptorsIter<'a, D>>: Into<ComponentId>,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedBundlePtrsIter<FieldDescriptorsIter<'a, D>> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundlePtrsIter::from_inner(inner) }
    }
}

impl<'a, D> IntoIterator for &'a ErasedBundlePtrsIter<D>
where
    D: FieldDescriptors<'a> + ?Sized,
    FieldDescriptorsIter<'a, D>: FieldDescriptorsOwned,
    for<'b> FieldDescriptorsItem<'b, FieldDescriptorsIter<'a, D>>: Into<ComponentId>,
{
    type Item = ErasedComponentPtr;
    type IntoIter = ErasedBundlePtrsIter<FieldDescriptorsIter<'a, D>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D> Debug for ErasedBundlePtrsIter<D>
where
    D: FieldDescriptorsOwned + ?Sized,
    for<'a> FieldDescriptorsIter<'a, D>: FieldDescriptorsOwned,
    for<'a, 'b> FieldDescriptorsItem<'b, FieldDescriptorsIter<'a, D>>: Into<ComponentId>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl<D> Clone for ErasedBundlePtrsIter<D>
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

impl<D> Iterator for ErasedBundlePtrsIter<D>
where
    D: Iterator<Item: AsRef<FieldDescriptor>> + FieldDescriptorsOwned + ?Sized,
    for<'a> FieldDescriptorsItem<'a, D>: Into<ComponentId>,
{
    type Item = ErasedComponentPtr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;

        let component_id = inner.field_descriptors().into_iter().next()?.into();
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

impl<D> ExactSizeIterator for ErasedBundlePtrsIter<D>
where
    D: ExactSizeIterator<Item: AsRef<FieldDescriptor>> + FieldDescriptorsOwned + ?Sized,
    for<'a> FieldDescriptorsItem<'a, D>: Into<ComponentId>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<D> FusedIterator for ErasedBundlePtrsIter<D>
where
    D: FusedIterator<Item: AsRef<FieldDescriptor>> + FieldDescriptorsOwned + ?Sized,
    for<'a> FieldDescriptorsItem<'a, D>: Into<ComponentId>,
{
}

impl<'a, D> FieldDescriptors<'a> for ErasedBundlePtrsIter<D>
where
    D: FieldDescriptors<'a> + ?Sized,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        self.descriptors().field_descriptors()
    }
}

impl<D> CovariantFieldDescriptors for ErasedBundlePtrsIter<D>
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
