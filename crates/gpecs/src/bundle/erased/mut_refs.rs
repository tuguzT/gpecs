use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::MaybeUninit,
    ptr,
};

use gpecs_soa_erased::{CovariantFieldDescriptors, ErasedSoaMutRefs, ErasedSoaMutRefsIter};

use crate::{
    archetype::{
        erased::{ErasedArchetype, ErasedArchetypeIter},
        error::IncompatibleArchetypeError,
    },
    bundle::{
        Bundle, BundleRefsMut,
        erased::{ErasedBundleMutPtrs, ErasedBundleRefs, ErasedBundleRefsIter},
    },
    component::{
        erased::{ErasedComponentMutRef, ErasedComponentRef},
        registry::ComponentRegistry,
    },
    soa::{
        field::{FieldDescriptor, FieldDescriptors},
        traits::SoaContext,
    },
};

type Inner<'data, 'a, Meta> =
    ErasedSoaMutRefs<'data, &'a ErasedArchetype<Meta>, *mut MaybeUninit<u8>>;

#[derive(Debug)]
pub struct ErasedBundleMutRefs<'data, 'a, Meta> {
    inner: Inner<'data, 'a, Meta>,
}

impl<'data, 'a, Meta> ErasedBundleMutRefs<'data, 'a, Meta> {
    #[inline]
    pub fn from_inner(inner: Inner<'data, 'a, Meta>) -> Self {
        Self { inner }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedBundleMutPtrs<'a, Meta>) -> Self {
        let inner = ptrs.into_inner();
        let inner = unsafe { inner.deref_mut() };
        Self::from_inner(inner)
    }

    #[inline]
    pub fn as_buffer(&self) -> &[MaybeUninit<u8>] {
        let Self { inner } = self;
        inner.as_buffer()
    }

    #[inline]
    pub fn as_mut_buffer(&mut self) -> &mut [MaybeUninit<u8>] {
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
    pub fn into_inner(self) -> Inner<'data, 'a, Meta> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedBundleMutPtrs<'a, Meta> {
        let Self { inner } = self;

        let inner = inner.into_ptrs();
        ErasedBundleMutPtrs::from_inner(inner)
    }
}

impl<'data, Meta> ErasedBundleMutRefs<'data, '_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    pub fn downcast<B>(
        self,
        components: &ComponentRegistry,
    ) -> Result<BundleRefsMut<'data, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        let slices = self.into_ptrs().downcast::<B>(components)?;
        let slices = unsafe { B::CONTEXT.mut_ptrs_to_mut_refs(slices) };
        Ok(slices)
    }

    #[inline]
    pub fn iter(&self) -> ErasedBundleRefsIter<'_, '_, Meta> {
        let Self { inner } = self;

        let inner = inner.iter();
        ErasedBundleRefsIter::from_inner(inner)
    }

    #[inline]
    pub fn iter_mut(&mut self) -> ErasedBundleMutRefsIter<'_, '_, Meta> {
        let Self { inner } = self;

        let inner = inner.iter_mut();
        ErasedBundleMutRefsIter::from_inner(inner)
    }
}

impl<'a, Meta> IntoIterator for &'a ErasedBundleMutRefs<'_, '_, Meta>
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

impl<'a, Meta> IntoIterator for &'a mut ErasedBundleMutRefs<'_, '_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Item = ErasedComponentMutRef<'a>;
    type IntoIter = ErasedBundleMutRefsIter<'a, 'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'data, 'a, Meta> IntoIterator for ErasedBundleMutRefs<'data, 'a, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Item = ErasedComponentMutRef<'data>;
    type IntoIter = ErasedBundleMutRefsIter<'data, 'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        ErasedBundleMutRefsIter::from_inner(inner)
    }
}

impl<'data, 'a, Meta> From<ErasedBundleMutRefs<'data, 'a, Meta>>
    for ErasedBundleRefs<'data, 'a, Meta>
{
    #[inline]
    fn from(refs: ErasedBundleMutRefs<'data, 'a, Meta>) -> Self {
        let inner = refs.into_inner();
        let inner = inner.into();
        Self::from_inner(inner)
    }
}

impl<'me, 'a, Meta> FieldDescriptors<'me> for ErasedBundleMutRefs<'_, 'a, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Output = &'a ErasedArchetype<Meta>;

    #[inline]
    fn field_descriptors(&'me self) -> Self::Output {
        self.archetype()
    }
}

impl<Meta> CovariantFieldDescriptors for ErasedBundleMutRefs<'_, '_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: <Self as FieldDescriptors<'long>>::Output,
    ) -> <Self as FieldDescriptors<'short>>::Output {
        from
    }
}

type InnerIter<'data, 'a, Meta> =
    ErasedSoaMutRefsIter<'data, ErasedArchetypeIter<'a, Meta>, *mut MaybeUninit<u8>>;

pub struct ErasedBundleMutRefsIter<'data, 'a, Meta> {
    inner: InnerIter<'data, 'a, Meta>,
}

impl<'data, 'a, Meta> ErasedBundleMutRefsIter<'data, 'a, Meta> {
    #[inline]
    pub(super) fn from_inner(inner: InnerIter<'data, 'a, Meta>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn as_buffer(&self) -> &[MaybeUninit<u8>] {
        let Self { inner } = self;
        inner.as_buffer()
    }

    #[inline]
    pub fn as_mut_buffer(&mut self) -> &mut [MaybeUninit<u8>] {
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

impl<Meta> Debug for ErasedBundleMutRefsIter<'_, '_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = unsafe { ptr::from_ref(self).read() };
        f.debug_set().entries(entries).finish()
    }
}

impl<'data, Meta> Iterator for ErasedBundleMutRefsIter<'data, '_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Item = ErasedComponentMutRef<'data>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;

        let component_id = inner.descriptors().clone().next()?.into();
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

impl<Meta> ExactSizeIterator for ErasedBundleMutRefsIter<'_, '_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<Meta> FusedIterator for ErasedBundleMutRefsIter<'_, '_, Meta> where Meta: AsRef<FieldDescriptor>
{}

impl<'me, 'a, Meta> FieldDescriptors<'me> for ErasedBundleMutRefsIter<'_, 'a, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Output = ErasedArchetypeIter<'a, Meta>;

    #[inline]
    fn field_descriptors(&'me self) -> Self::Output {
        self.descriptors()
    }
}

impl<Meta> CovariantFieldDescriptors for ErasedBundleMutRefsIter<'_, '_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: <Self as FieldDescriptors<'long>>::Output,
    ) -> <Self as FieldDescriptors<'short>>::Output {
        from
    }
}
