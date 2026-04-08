use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::MaybeUninit,
};

use gpecs_soa_erased::{CovariantFieldDescriptors, ErasedSoaMutRefs, ErasedSoaMutRefsIter};

use crate::{
    archetype::erased::{ErasedArchetypeView, Iter, error::IncompatibleArchetypeError},
    bundle::{
        Bundle, BundleRefsMut,
        erased::{
            ErasedBundleMutPtrs, ErasedBundleRefs, ErasedBundleRefsIter,
            traits::{ErasedArchetypeIterator, ErasedArchetypeKind, IntoErasedArchetypeIterator},
        },
    },
    component::{
        erased::{ErasedComponentMutRef, ErasedComponentRef},
        registry::{
            ComponentId, ComponentRegistryView,
            traits::{ComponentIdFrom, FromComponentType},
        },
    },
    soa::{
        field::{
            FieldDescriptors, FieldDescriptorsIter, FieldDescriptorsOutput, FieldDescriptorsOwned,
        },
        traits::SoaContext,
    },
};

type Inner<'a, D> = ErasedSoaMutRefs<'a, D, *mut MaybeUninit<u8>>;

#[derive(Debug)]
pub struct ErasedBundleMutRefs<'a, D>
where
    D: ?Sized,
{
    inner: Inner<'a, D>,
}

impl<'a, D> ErasedBundleMutRefs<'a, D> {
    #[inline]
    pub unsafe fn from_inner(inner: Inner<'a, D>) -> Self {
        Self { inner }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedBundleMutPtrs<D>) -> Self {
        let inner = ptrs.into_inner();
        let inner = unsafe { inner.deref_mut() };
        unsafe { Self::from_inner(inner) }
    }

    #[inline]
    pub fn into_inner(self) -> Inner<'a, D> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedBundleMutPtrs<D> {
        let Self { inner } = self;

        let inner = inner.into_ptrs();
        unsafe { ErasedBundleMutPtrs::from_inner(inner) }
    }
}

impl<D> ErasedBundleMutRefs<'_, D>
where
    D: ?Sized,
{
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
    pub fn descriptors(&self) -> &D {
        let Self { inner } = self;
        inner.descriptors()
    }
}

impl<'a, D> ErasedBundleMutRefs<'a, D>
where
    D: ErasedArchetypeKind,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleRefsMut<'a, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let slices = self.into_ptrs().downcast::<B, T>(components)?;
        let slices = unsafe { B::CONTEXT.mut_ptrs_to_mut_refs(slices) };
        Ok(slices)
    }
}

impl<D> ErasedBundleMutRefs<'_, D>
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
    pub fn iter_mut(&mut self) -> ErasedBundleMutRefsIter<'_, Iter<'_, D::Meta>> {
        let Self { inner } = self;

        let inner = inner.iter_mut();
        unsafe { ErasedBundleMutRefsIter::from_inner(inner) }
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<ErasedComponentRef<'_>> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter().nth(index)
    }

    #[inline]
    pub fn get_mut(&mut self, component_id: ComponentId) -> Option<ErasedComponentMutRef<'_>> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter_mut().nth(index)
    }
}

impl<'a, D> IntoIterator for &'a ErasedBundleMutRefs<'_, D>
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

impl<'a, D> IntoIterator for &'a mut ErasedBundleMutRefs<'_, D>
where
    D: ErasedArchetypeKind + ?Sized,
{
    type Item = ErasedComponentMutRef<'a>;
    type IntoIter = ErasedBundleMutRefsIter<'a, Iter<'a, D::Meta>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, D> IntoIterator for ErasedBundleMutRefs<'a, D>
where
    D: IntoErasedArchetypeIterator,
{
    type Item = ErasedComponentMutRef<'a>;
    type IntoIter = ErasedBundleMutRefsIter<'a, D::IntoIter>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ErasedBundleMutRefsIter::from_inner(inner) }
    }
}

impl<'a, D> From<ErasedBundleMutRefs<'a, D>> for ErasedBundleRefs<'a, D> {
    #[inline]
    fn from(refs: ErasedBundleMutRefs<'a, D>) -> Self {
        let inner = refs.into_inner();
        let inner = inner.into();
        unsafe { Self::from_inner(inner) }
    }
}

impl<'a, D> FieldDescriptors<'a> for ErasedBundleMutRefs<'_, D>
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

impl<D> CovariantFieldDescriptors for ErasedBundleMutRefs<'_, D>
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

type InnerIter<'a, D> = ErasedSoaMutRefsIter<'a, D, *mut MaybeUninit<u8>>;

pub struct ErasedBundleMutRefsIter<'a, D>
where
    D: ?Sized,
{
    inner: InnerIter<'a, D>,
}

impl<'a, D> ErasedBundleMutRefsIter<'a, D> {
    #[inline]
    pub(super) unsafe fn from_inner(inner: InnerIter<'a, D>) -> Self {
        Self { inner }
    }
}

impl<D> ErasedBundleMutRefsIter<'_, D>
where
    D: ?Sized,
{
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
    pub fn descriptors(&self) -> &D {
        let Self { inner, .. } = self;
        inner.descriptors()
    }
}

impl<'a, D> ErasedBundleMutRefsIter<'_, D>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedBundleMutRefsIter<'a, FieldDescriptorsIter<'a, D>> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleMutRefsIter::from_inner(inner) }
    }
}

impl<'a, D> IntoIterator for &'a ErasedBundleMutRefsIter<'_, D>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
{
    type Item = ErasedComponentMutRef<'a>;
    type IntoIter = ErasedBundleMutRefsIter<'a, FieldDescriptorsIter<'a, D>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D> Debug for ErasedBundleMutRefsIter<'_, D>
where
    D: FieldDescriptorsOwned<Output: IntoErasedArchetypeIterator> + ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl<'a, D> Iterator for ErasedBundleMutRefsIter<'a, D>
where
    D: ErasedArchetypeIterator + ?Sized,
{
    type Item = ErasedComponentMutRef<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;

        let component_id = inner.field_descriptors().into_iter().next()?.into();
        let fields = inner.next()?;
        let item = unsafe { ErasedComponentMutRef::from_parts(component_id, fields) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<D> ExactSizeIterator for ErasedBundleMutRefsIter<'_, D>
where
    D: ErasedArchetypeIterator + ExactSizeIterator + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<D> FusedIterator for ErasedBundleMutRefsIter<'_, D> where
    D: ErasedArchetypeIterator + FusedIterator + ?Sized
{
}

impl<'a, D> FieldDescriptors<'a> for ErasedBundleMutRefsIter<'_, D>
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

impl<D> CovariantFieldDescriptors for ErasedBundleMutRefsIter<'_, D>
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
