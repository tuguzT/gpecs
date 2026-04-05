use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::MaybeUninit,
};

use gpecs_soa_erased::{CovariantFieldDescriptors, ErasedSoaRefs, ErasedSoaRefsIter};

use crate::{
    archetype::erased::{ErasedArchetype, Iter, error::IncompatibleArchetypeError},
    bundle::{Bundle, BundleRefs, erased::ErasedBundlePtrs},
    component::{
        erased::ErasedComponentRef,
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
    ErasedSoaRefs<'data, &'a ErasedArchetype<Meta>, *const MaybeUninit<u8>>;

#[derive(Debug)]
pub struct ErasedBundleRefs<'data, 'a, Meta> {
    inner: Inner<'data, 'a, Meta>,
}

impl<'data, 'a, Meta> ErasedBundleRefs<'data, 'a, Meta> {
    #[inline]
    pub unsafe fn from_inner(inner: Inner<'data, 'a, Meta>) -> Self {
        Self { inner }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedBundlePtrs<'a, Meta>) -> Self {
        let inner = ptrs.into_inner();
        let inner = unsafe { inner.deref() };
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
    pub fn into_inner(self) -> Inner<'data, 'a, Meta> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedBundlePtrs<'a, Meta> {
        let Self { inner } = self;

        let inner = inner.into_ptrs();
        unsafe { ErasedBundlePtrs::from_inner(inner) }
    }
}

impl<'data, Meta> ErasedBundleRefs<'data, '_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleRefs<'data, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let slices = self.into_ptrs().downcast::<B, T>(components)?;
        let slices = unsafe { B::CONTEXT.ptrs_to_refs(slices) };
        Ok(slices)
    }

    #[inline]
    pub fn iter(&self) -> ErasedBundleRefsIter<'_, '_, Meta> {
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

impl<Meta> Clone for ErasedBundleRefs<'_, '_, Meta> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<Meta> Copy for ErasedBundleRefs<'_, '_, Meta> {}

impl<'a, Meta> IntoIterator for &'a ErasedBundleRefs<'_, '_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Item = ErasedComponentRef<'a>;
    type IntoIter = ErasedBundleRefsIter<'a, 'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'data, 'a, Meta> IntoIterator for ErasedBundleRefs<'data, 'a, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Item = ErasedComponentRef<'data>;
    type IntoIter = ErasedBundleRefsIter<'data, 'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ErasedBundleRefsIter::from_inner(inner) }
    }
}

impl<'a, Meta> FieldDescriptors<'a> for ErasedBundleRefs<'_, '_, Meta>
where
    Meta: AsRef<FieldDescriptor> + 'a,
{
    type Output = &'a ErasedArchetype<Meta>;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        self.archetype()
    }
}

impl<Meta> CovariantFieldDescriptors for ErasedBundleRefs<'_, '_, Meta>
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

type InnerIter<'data, 'a, Meta> = ErasedSoaRefsIter<'data, Iter<'a, Meta>, *const MaybeUninit<u8>>;

pub struct ErasedBundleRefsIter<'data, 'a, Meta> {
    inner: InnerIter<'data, 'a, Meta>,
}

impl<'data, 'a, Meta> ErasedBundleRefsIter<'data, 'a, Meta> {
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

impl<Meta> Debug for ErasedBundleRefsIter<'_, '_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.clone();
        f.debug_set().entries(entries).finish()
    }
}

impl<Meta> Clone for ErasedBundleRefsIter<'_, '_, Meta> {
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        Self { inner }
    }
}

impl<'data, Meta> Iterator for ErasedBundleRefsIter<'data, '_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Item = ErasedComponentRef<'data>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;

        let component_id = inner.descriptors().clone().next()?.into();
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

impl<Meta> ExactSizeIterator for ErasedBundleRefsIter<'_, '_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<Meta> FusedIterator for ErasedBundleRefsIter<'_, '_, Meta> where Meta: AsRef<FieldDescriptor> {}

impl<'a, Meta> FieldDescriptors<'a> for ErasedBundleRefsIter<'_, '_, Meta>
where
    Meta: AsRef<FieldDescriptor> + 'a,
{
    type Output = Iter<'a, Meta>;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        self.descriptors()
    }
}

impl<Meta> CovariantFieldDescriptors for ErasedBundleRefsIter<'_, '_, Meta>
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
