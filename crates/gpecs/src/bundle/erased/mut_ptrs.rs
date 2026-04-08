use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::MaybeUninit,
};

use gpecs_soa_erased::{CovariantFieldDescriptors, ErasedSoaMutPtrs, ErasedSoaMutPtrsIter};

use crate::{
    archetype::erased::{
        ErasedArchetypeView, ErasedArchetypeViewExt, Iter, error::IncompatibleArchetypeError,
    },
    bundle::{
        Bundle, BundleMutPtrs,
        erased::{
            ErasedBundleKind, ErasedBundleMutRefs, ErasedBundlePtrs, ErasedBundlePtrsIter,
            ErasedBundleRefs,
            traits::{ErasedArchetypeIterator, ErasedArchetypeKind, IntoErasedArchetypeIterator},
        },
    },
    component::{
        erased::{
            ErasedComponentMutPtr, ErasedComponentPtr, WithErasedDrop, error::NotRegisteredError,
        },
        registry::{
            ComponentId, ComponentRegistryView,
            traits::{ComponentIdFrom, FromComponentType, WithComponentId},
        },
    },
    soa::field::{
        FieldDescriptors, FieldDescriptorsIter, FieldDescriptorsOutput, FieldDescriptorsOwned,
    },
};

type Inner<D> = ErasedSoaMutPtrs<D, *mut MaybeUninit<u8>>;

#[derive(Debug)]
pub struct ErasedBundleMutPtrs<D>
where
    D: ?Sized,
{
    inner: Inner<D>,
}

impl<D> ErasedBundleMutPtrs<D> {
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
    pub fn cast_const(self) -> ErasedBundlePtrs<D> {
        let Self { inner } = self;

        let inner = inner.cast_const();
        unsafe { ErasedBundlePtrs::from_inner(inner) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedBundleRefs<'a, D> {
        unsafe { self.cast_const().deref() }
    }

    #[inline]
    pub unsafe fn deref_mut<'a>(self) -> ErasedBundleMutRefs<'a, D> {
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

impl<D> ErasedBundleMutPtrs<D>
where
    D: ?Sized,
{
    #[inline]
    pub unsafe fn as_inner(&self) -> &Inner<D> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub unsafe fn as_mut_inner(&mut self) -> &mut Inner<D> {
        let Self { inner } = self;
        inner
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
    pub fn descriptors(&self) -> &D {
        let Self { inner } = self;
        inner.descriptors()
    }
}

impl<D> ErasedBundleMutPtrs<D>
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
        mut self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleMutPtrs<B>, IncompatibleArchetypeError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        self.archetype()
            .check_compatibility_of::<B, T>(components)?;

        let ptrs = B::mut_ptrs_from_erased(components, self.iter_mut())
            .expect("archetype compatibility should be already checked");
        Ok(ptrs)
    }
}

impl<D> ErasedBundleMutPtrs<D>
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
    pub fn iter_mut(&mut self) -> ErasedBundleMutPtrsIter<Iter<'_, D::Meta>> {
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
        &mut self,
        components: &ComponentRegistryView<impl WithErasedDrop, impl ?Sized>,
    ) -> Result<(), NotRegisteredError> {
        self.iter()
            .map(ErasedComponentPtr::component_id)
            .try_for_each(|id| {
                components
                    .get_component_info(id)
                    .map(drop)
                    .ok_or_else(NotRegisteredError::new)
            })?;

        self.iter_mut().for_each(|ptr| {
            if let Err(error) = unsafe { ptr.drop_in_place(components) } {
                unreachable!("{error}, but it was checked earlier to be registered")
            }
        });
        Ok(())
    }

    #[inline]
    #[track_caller]
    pub unsafe fn swap<N>(&mut self, with: &mut ErasedBundleMutPtrs<N>)
    where
        N: ErasedArchetypeKind + ?Sized,
    {
        let Self { inner } = self;

        let with = unsafe { with.as_mut_inner() };
        unsafe { inner.swap(with) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_forward<N>(&mut self, src: &ErasedBundlePtrs<N>, count: usize)
    where
        N: ErasedArchetypeKind + ?Sized,
    {
        let Self { inner } = self;

        let src = unsafe { src.as_inner() };
        unsafe { inner.copy_from_forward(src, count) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_backward<N>(&mut self, src: &ErasedBundlePtrs<N>, count: usize)
    where
        N: ErasedArchetypeKind + ?Sized,
    {
        let Self { inner } = self;

        let src = unsafe { src.as_inner() };
        unsafe { inner.copy_from_backward(src, count) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_nonoverlapping<N>(&mut self, src: &ErasedBundlePtrs<N>, count: usize)
    where
        N: ErasedArchetypeKind + ?Sized,
    {
        let Self { inner } = self;

        let src = unsafe { src.as_inner() };
        unsafe { inner.copy_from_nonoverlapping(src, count) }
    }
}

impl<D> ErasedBundleMutPtrs<D>
where
    D: ErasedArchetypeKind<Meta: WithErasedDrop> + ?Sized,
{
    #[inline]
    pub unsafe fn write<T>(&mut self, value: ErasedBundleKind<T>)
    where
        T: ErasedArchetypeKind<Meta = D::Meta>,
    {
        let Self { inner } = self;

        let value = value.into_inner();
        unsafe { inner.write(value) }
    }
}

impl<D> Clone for ErasedBundleMutPtrs<D>
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

impl<D> Copy for ErasedBundleMutPtrs<D> where D: Copy {}

impl<'a, D> IntoIterator for &'a ErasedBundleMutPtrs<D>
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

impl<'a, D> IntoIterator for &'a mut ErasedBundleMutPtrs<D>
where
    D: ErasedArchetypeKind + ?Sized,
{
    type Item = ErasedComponentMutPtr;
    type IntoIter = ErasedBundleMutPtrsIter<Iter<'a, D::Meta>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<D> IntoIterator for ErasedBundleMutPtrs<D>
where
    D: IntoErasedArchetypeIterator,
{
    type Item = ErasedComponentMutPtr;
    type IntoIter = ErasedBundleMutPtrsIter<D::IntoIter>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ErasedBundleMutPtrsIter::from_inner(inner) }
    }
}

impl<'a, D> FieldDescriptors<'a> for ErasedBundleMutPtrs<D>
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

impl<D> CovariantFieldDescriptors for ErasedBundleMutPtrs<D>
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

type InnerIter<D> = ErasedSoaMutPtrsIter<D, *mut MaybeUninit<u8>>;

pub struct ErasedBundleMutPtrsIter<D>
where
    D: ?Sized,
{
    inner: InnerIter<D>,
}

impl<D> ErasedBundleMutPtrsIter<D> {
    #[inline]
    pub(super) unsafe fn from_inner(inner: InnerIter<D>) -> Self {
        Self { inner }
    }
}

impl<D> ErasedBundleMutPtrsIter<D>
where
    D: ?Sized,
{
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
    pub fn descriptors(&self) -> &D {
        let Self { inner, .. } = self;
        inner.descriptors()
    }
}

impl<'a, D> ErasedBundleMutPtrsIter<D>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedBundleMutPtrsIter<FieldDescriptorsIter<'a, D>> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleMutPtrsIter::from_inner(inner) }
    }
}

impl<'a, D> IntoIterator for &'a ErasedBundleMutPtrsIter<D>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
{
    type Item = ErasedComponentMutPtr;
    type IntoIter = ErasedBundleMutPtrsIter<FieldDescriptorsIter<'a, D>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D> Debug for ErasedBundleMutPtrsIter<D>
where
    D: FieldDescriptorsOwned<Output: IntoErasedArchetypeIterator> + ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl<D> Clone for ErasedBundleMutPtrsIter<D>
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

impl<D> Iterator for ErasedBundleMutPtrsIter<D>
where
    D: ErasedArchetypeIterator + ?Sized,
{
    type Item = ErasedComponentMutPtr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;

        let component_id = inner.field_descriptors().into_iter().next()?.component_id();
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

impl<D> ExactSizeIterator for ErasedBundleMutPtrsIter<D>
where
    D: ErasedArchetypeIterator + ExactSizeIterator + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<D> FusedIterator for ErasedBundleMutPtrsIter<D> where
    D: ErasedArchetypeIterator + FusedIterator + ?Sized
{
}

impl<'a, D> FieldDescriptors<'a> for ErasedBundleMutPtrsIter<D>
where
    D: FieldDescriptors<'a> + ?Sized,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        self.descriptors().field_descriptors()
    }
}

impl<D> CovariantFieldDescriptors for ErasedBundleMutPtrsIter<D>
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
