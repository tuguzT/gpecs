use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::MaybeUninit,
};

use gpecs_soa_erased::{CovariantFieldDescriptors, ErasedSoaRefs, ErasedSoaRefsIter};

use crate::{
    archetype::erased::{ErasedArchetypeView, Iter, error::IncompatibleArchetypeError},
    bundle::{
        Bundle, BundleRefs,
        erased::{ErasedArchetypeKind, ErasedBundlePtrs},
    },
    component::{
        erased::ErasedComponentRef,
        registry::{
            ComponentId, ComponentRegistryView,
            traits::{ComponentIdFrom, FromComponentType},
        },
    },
    soa::{
        field::{
            FieldDescriptor, FieldDescriptors, FieldDescriptorsItem, FieldDescriptorsOutput,
            FieldDescriptorsOwned,
        },
        traits::SoaContext,
    },
};

type Inner<'a, D> = ErasedSoaRefs<'a, D, *const MaybeUninit<u8>>;

#[derive(Debug)]
pub struct ErasedBundleRefs<'a, D>
where
    D: ?Sized,
{
    inner: Inner<'a, D>,
}

impl<'a, D> ErasedBundleRefs<'a, D> {
    #[inline]
    pub unsafe fn from_inner(inner: Inner<'a, D>) -> Self {
        Self { inner }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedBundlePtrs<D>) -> Self {
        let inner = ptrs.into_inner();
        let inner = unsafe { inner.deref() };
        unsafe { Self::from_inner(inner) }
    }

    #[inline]
    pub fn into_inner(self) -> Inner<'a, D> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedBundlePtrs<D> {
        let Self { inner } = self;

        let inner = inner.into_ptrs();
        unsafe { ErasedBundlePtrs::from_inner(inner) }
    }
}

impl<D> ErasedBundleRefs<'_, D>
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
        let Self { inner } = self;
        inner.descriptors()
    }
}

impl<'a, D> ErasedBundleRefs<'a, D>
where
    D: ErasedArchetypeKind,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleRefs<'a, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let slices = self.into_ptrs().downcast::<B, T>(components)?;
        let slices = unsafe { B::CONTEXT.ptrs_to_refs(slices) };
        Ok(slices)
    }
}

impl<D> ErasedBundleRefs<'_, D>
where
    D: ErasedArchetypeKind + ?Sized,
{
    #[inline]
    pub fn archetype(&self) -> ErasedArchetypeView<'_, D::Meta> {
        self.field_descriptors()
    }

    #[inline]
    pub fn iter(&self) -> ErasedBundleRefsIter<'_, Iter<'_, D::Meta>> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleRefsIter::from_inner(inner) }
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<ErasedComponentRef<'_>> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter().nth(index)
    }
}

impl<D> Clone for ErasedBundleRefs<'_, D>
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

impl<D> Copy for ErasedBundleRefs<'_, D> where D: Copy {}

impl<'a, D> IntoIterator for &'a ErasedBundleRefs<'_, D>
where
    D: ErasedArchetypeKind + ?Sized,
{
    type Item = ErasedComponentRef<'a>;
    type IntoIter = ErasedBundleRefsIter<'a, Iter<'a, D::Meta>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D> IntoIterator for ErasedBundleRefs<'a, D>
where
    D: IntoIterator<Item: AsRef<FieldDescriptor>, IntoIter: FieldDescriptorsOwned>,
    for<'b> FieldDescriptorsItem<'b, D::IntoIter>: Into<ComponentId>,
{
    type Item = ErasedComponentRef<'a>;
    type IntoIter = ErasedBundleRefsIter<'a, D::IntoIter>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ErasedBundleRefsIter::from_inner(inner) }
    }
}

impl<'a, D> FieldDescriptors<'a> for ErasedBundleRefs<'_, D>
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

impl<D> CovariantFieldDescriptors for ErasedBundleRefs<'_, D>
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

type InnerIter<'a, D> = ErasedSoaRefsIter<'a, D, *const MaybeUninit<u8>>;

pub struct ErasedBundleRefsIter<'a, D>
where
    D: ?Sized,
{
    inner: InnerIter<'a, D>,
}

impl<'a, D> ErasedBundleRefsIter<'a, D> {
    #[inline]
    pub(super) unsafe fn from_inner(inner: InnerIter<'a, D>) -> Self {
        Self { inner }
    }
}

impl<D> ErasedBundleRefsIter<'_, D>
where
    D: ?Sized,
{
    #[inline]
    pub fn as_buffer(&self) -> &[MaybeUninit<u8>] {
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

impl<D> Debug for ErasedBundleRefsIter<'_, D>
where
    D: Clone + Iterator<Item: AsRef<FieldDescriptor>> + FieldDescriptorsOwned,
    for<'a> FieldDescriptorsItem<'a, D>: Into<ComponentId>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.clone();
        f.debug_set().entries(entries).finish()
    }
}

impl<D> Clone for ErasedBundleRefsIter<'_, D>
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

impl<'a, D> Iterator for ErasedBundleRefsIter<'a, D>
where
    D: Iterator<Item: AsRef<FieldDescriptor>> + FieldDescriptorsOwned + ?Sized,
    for<'b> FieldDescriptorsItem<'b, D>: Into<ComponentId>,
{
    type Item = ErasedComponentRef<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;

        let component_id = inner.field_descriptors().into_iter().next()?.into();
        let fields = inner.next()?;
        let item = unsafe { ErasedComponentRef::from_parts(component_id, fields) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<D> ExactSizeIterator for ErasedBundleRefsIter<'_, D>
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

impl<D> FusedIterator for ErasedBundleRefsIter<'_, D>
where
    D: FusedIterator<Item: AsRef<FieldDescriptor>> + FieldDescriptorsOwned + ?Sized,
    for<'a> FieldDescriptorsItem<'a, D>: Into<ComponentId>,
{
}

impl<'a, D> FieldDescriptors<'a> for ErasedBundleRefsIter<'_, D>
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

impl<D> CovariantFieldDescriptors for ErasedBundleRefsIter<'_, D>
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
