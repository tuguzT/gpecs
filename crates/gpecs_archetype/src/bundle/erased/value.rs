use core::{
    alloc::Layout,
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
    ptr,
};

use gpecs_component::{
    erased::{ErasedComponent, ErasedComponentMutRef, ErasedComponentRef, WithErasedDrop},
    registry::{ComponentId, traits::WithComponentId},
};
use gpecs_soa_erased::{
    CovariantFieldDescriptors, ErasedSoa, ErasedSoaIntoFields,
    error::FromFieldsDescriptorsError,
    ptr::slice::SliceItemPtrs,
    soa::field::{FieldDescriptors, FieldDescriptorsItem, FieldDescriptorsOutput},
    storage::{AlignedStorage, AlignedStorageFromLayout},
};
use itertools::equal;

use crate::{
    bundle::erased::{
        ErasedBundleMutPtrs, ErasedBundleMutRefs, ErasedBundleMutRefsIter, ErasedBundlePtrs,
        ErasedBundleRefs, ErasedBundleRefsIter,
        error::{ShuffleError, ShuffleErrorKind},
        traits::{ErasedArchetypeKind, ErasedBundleDrop, IntoErasedArchetypeIterator},
    },
    erased::{ErasedArchetypeView, Iter},
};

pub type ErasedBorrowedViewBundle<'a, Meta, D, S, P> =
    ErasedBundleKind<ErasedArchetypeView<'a, Meta>, D, S, P>;

pub struct ErasedBundleKind<T, D, S, P>
where
    T: ErasedArchetypeKind + ?Sized,
    D: ErasedBundleDrop<T::Meta>,
    S: AlignedStorage,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
{
    phantom: PhantomData<D>,
    inner: ErasedSoa<S, T, P>,
}

impl<T, D, S, P> ErasedBundleKind<T, D, S, P>
where
    T: ErasedArchetypeKind,
    D: ErasedBundleDrop<T::Meta>,
    S: AlignedStorage,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
{
    #[inline]
    pub unsafe fn from_inner(inner: ErasedSoa<S, T, P>) -> Self {
        let phantom = PhantomData;
        Self { phantom, inner }
    }

    #[inline]
    pub fn into_inner(self) -> ErasedSoa<S, T, P> {
        let me = ManuallyDrop::new(self);
        unsafe { ptr::read(&raw const me.inner) }
    }
}

impl<T, D, S, P> ErasedBundleKind<T, D, S, P>
where
    T: ErasedArchetypeKind + ?Sized,
    D: ErasedBundleDrop<T::Meta>,
    S: AlignedStorage,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
{
    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { inner, .. } = self;
        inner.layout()
    }

    #[inline]
    pub fn archetype(&self) -> ErasedArchetypeView<'_, T::Meta> {
        let Self { inner, .. } = self;
        inner.field_descriptors()
    }

    #[inline]
    pub fn as_ptr(&self) -> *const P::Item {
        let Self { inner, .. } = self;
        inner.as_ptr()
    }

    #[inline]
    pub unsafe fn as_mut_ptr(&mut self) -> *mut P::Item {
        let Self { inner, .. } = self;
        inner.as_mut_ptr()
    }

    #[inline]
    pub fn as_buffer(&self) -> &[P::Item] {
        let Self { inner, .. } = self;
        inner.as_buffer()
    }

    #[inline]
    pub unsafe fn as_mut_buffer(&mut self) -> &mut [P::Item] {
        let Self { inner, .. } = self;
        inner.as_mut_buffer()
    }

    #[inline]
    pub fn as_ptrs(&self) -> ErasedBundlePtrs<ErasedArchetypeView<'_, T::Meta>, P::Const> {
        let (ptrs, _) = self.as_ptrs_with_archetype();
        ptrs
    }

    #[inline]
    #[expect(clippy::type_complexity)]
    pub fn as_ptrs_with_archetype(
        &self,
    ) -> (
        ErasedBundlePtrs<ErasedArchetypeView<'_, T::Meta>, P::Const>,
        ErasedArchetypeView<'_, T::Meta>,
    ) {
        let Self { inner, .. } = self;

        let (inner, descriptors) = inner.as_ptrs_with_descriptors();
        let ptrs = unsafe { ErasedBundlePtrs::from_inner(inner) };
        (ptrs, descriptors.field_descriptors())
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> ErasedBundleMutPtrs<ErasedArchetypeView<'_, T::Meta>, P::Mut> {
        let (ptrs, _) = self.as_mut_ptrs_with_archetype();
        ptrs
    }

    #[inline]
    #[expect(clippy::type_complexity)]
    pub fn as_mut_ptrs_with_archetype(
        &mut self,
    ) -> (
        ErasedBundleMutPtrs<ErasedArchetypeView<'_, T::Meta>, P::Mut>,
        ErasedArchetypeView<'_, T::Meta>,
    ) {
        let Self { inner, .. } = self;

        let (inner, descriptors) = inner.as_mut_ptrs_with_descriptors();
        let ptrs = unsafe { ErasedBundleMutPtrs::from_inner(inner) };
        (ptrs, descriptors.field_descriptors())
    }

    #[inline]
    pub fn as_refs(&self) -> ErasedBundleRefs<'_, ErasedArchetypeView<'_, T::Meta>, P::Const> {
        let (refs, _) = self.as_refs_with_archetype();
        refs
    }

    #[inline]
    #[expect(clippy::type_complexity)]
    pub fn as_refs_with_archetype(
        &self,
    ) -> (
        ErasedBundleRefs<'_, ErasedArchetypeView<'_, T::Meta>, P::Const>,
        ErasedArchetypeView<'_, T::Meta>,
    ) {
        let (ptrs, descriptors) = self.as_ptrs_with_archetype();
        let refs = unsafe { ptrs.as_ref_unchecked() };
        (refs, descriptors)
    }

    #[inline]
    pub fn as_mut_refs(
        &mut self,
    ) -> ErasedBundleMutRefs<'_, ErasedArchetypeView<'_, T::Meta>, P::Mut> {
        let (refs, _) = self.as_mut_refs_with_archetype();
        refs
    }

    #[inline]
    #[expect(clippy::type_complexity)]
    pub fn as_mut_refs_with_archetype(
        &mut self,
    ) -> (
        ErasedBundleMutRefs<'_, ErasedArchetypeView<'_, T::Meta>, P::Mut>,
        ErasedArchetypeView<'_, T::Meta>,
    ) {
        let (ptrs, descriptors) = self.as_mut_ptrs_with_archetype();
        let refs = unsafe { ptrs.as_mut_unchecked() };
        (refs, descriptors)
    }

    #[inline]
    pub fn iter(&self) -> ErasedBundleRefsIter<'_, Iter<'_, T::Meta>, P::Const> {
        self.as_refs().into_iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> ErasedBundleMutRefsIter<'_, Iter<'_, T::Meta>, P::Mut> {
        self.as_mut_refs().into_iter()
    }
}

pub enum ShuffledBundle<Original, Other, D, S, P>
where
    Original: ErasedArchetypeKind,
    Other: ErasedArchetypeKind<Meta = Original::Meta>,
    D: ErasedBundleDrop<Original::Meta>,
    S: AlignedStorageFromLayout,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
{
    Original(ErasedBundleKind<Original, D, S, P>),
    Other(ErasedBundleKind<Other, D, S, P>),
}

impl<Original, Other, D, S, P> Debug for ShuffledBundle<Original, Other, D, S, P>
where
    Original: ErasedArchetypeKind,
    Other: ErasedArchetypeKind<Meta = Original::Meta>,
    D: ErasedBundleDrop<Original::Meta>,
    S: AlignedStorageFromLayout,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Original(bundle) => f.debug_tuple("Original").field(bundle).finish(),
            Self::Other(bundle) => f.debug_tuple("Other").field(bundle).finish(),
        }
    }
}

impl<Original, D, S, P> ErasedBundleKind<Original, D, S, P>
where
    Original: ErasedArchetypeKind,
    D: ErasedBundleDrop<Original::Meta>,
    S: AlignedStorageFromLayout<Item: Copy>,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
{
    #[inline]
    #[expect(clippy::type_complexity)]
    pub fn shuffle<Other>(
        self,
        archetype: Other,
    ) -> Result<ShuffledBundle<Original, Other, D, S, P>, ShuffleError<Self, Other, S::Error>>
    where
        Other: ErasedArchetypeKind<Meta = Original::Meta>,
    {
        let this = self.archetype();
        let other = archetype.field_descriptors();
        if let Err(error) = this.check_exact_compatibility(other.as_view()) {
            let error = ShuffleError {
                bundle: self,
                archetype,
                source: error.into(),
            };
            return Err(error);
        }

        if equal(
            this.iter().map(ComponentId::from),
            other.iter().map(ComponentId::from),
        ) {
            let shuffled = ShuffledBundle::Original(self);
            return Ok(shuffled);
        }

        let refs = self.as_refs();
        let fields = other.iter().map(|component| {
            let component_id = component.into();
            refs.get(component_id).expect("component should be present")
        });

        let result = ErasedSoa::<_, _, P>::try_from_fields_descriptors(fields, other);
        let inner = match result.map_err(into_shuffle_error_kind) {
            Ok(inner) => inner,
            Err(source) => {
                let error = ShuffleError {
                    bundle: self,
                    archetype,
                    source,
                };
                return Err(error);
            }
        };
        let _ = self.into_inner();

        let (storage, _) = inner.into_parts();
        let inner = unsafe { ErasedSoa::from_parts(storage, archetype) };
        let other = unsafe { ErasedBundleKind::from_inner(inner) };

        let shuffled = ShuffledBundle::Other(other);
        Ok(shuffled)
    }
}

#[inline]
fn into_shuffle_error_kind<E>(error: FromFieldsDescriptorsError<E>) -> ShuffleErrorKind<E> {
    match error {
        FromFieldsDescriptorsError::FromLayout(error) => ShuffleErrorKind::FromLayout(error),
        FromFieldsDescriptorsError::InvalidLayout(error) => error.into(),
        FromFieldsDescriptorsError::LenMismatch(error) => {
            unreachable!("failed to shuffle bundle: {error}")
        }
        FromFieldsDescriptorsError::InsufficientAlign(error) => {
            unreachable!("failed to shuffle bundle: {error}")
        }
    }
}

impl<T, D, S, P> Debug for ErasedBundleKind<T, D, S, P>
where
    T: ErasedArchetypeKind + ?Sized,
    D: ErasedBundleDrop<T::Meta>,
    S: AlignedStorage,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let components = &self.into_iter();
        f.debug_struct("ErasedBundle")
            .field("components", components)
            .finish()
    }
}

impl<T, D, S, P> AsRef<[P::Item]> for ErasedBundleKind<T, D, S, P>
where
    T: ErasedArchetypeKind + ?Sized,
    D: ErasedBundleDrop<T::Meta>,
    S: AlignedStorage,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
{
    #[inline]
    fn as_ref(&self) -> &[P::Item] {
        self.as_buffer()
    }
}

impl<T, D, S, P> Drop for ErasedBundleKind<T, D, S, P>
where
    T: ErasedArchetypeKind + ?Sized,
    D: ErasedBundleDrop<T::Meta>,
    S: AlignedStorage,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
{
    fn drop(&mut self) {
        let (mut ptrs, archetype) = self.as_mut_ptrs_with_archetype();
        unsafe { D::ptrs_drop_in_place(&archetype, &mut ptrs) }
    }
}

impl<'a, T, D, S, P> IntoIterator for &'a ErasedBundleKind<T, D, S, P>
where
    T: ErasedArchetypeKind + ?Sized,
    D: ErasedBundleDrop<T::Meta>,
    S: AlignedStorage,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
{
    type Item = ErasedComponentRef<'a, P::Const>;
    type IntoIter = ErasedBundleRefsIter<'a, Iter<'a, T::Meta>, P::Const>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, D, S, P> IntoIterator for &'a mut ErasedBundleKind<T, D, S, P>
where
    T: ErasedArchetypeKind + ?Sized,
    D: ErasedBundleDrop<T::Meta>,
    S: AlignedStorage,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
{
    type Item = ErasedComponentMutRef<'a, P::Mut>;
    type IntoIter = ErasedBundleMutRefsIter<'a, Iter<'a, T::Meta>, P::Mut>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T, D, S, P> IntoIterator for ErasedBundleKind<T, D, S, P>
where
    T: ErasedArchetypeKind + IntoErasedArchetypeIterator,
    D: ErasedBundleDrop<T::Meta>,
    S: AlignedStorageFromLayout<Item: Copy>,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
    for<'a> FieldDescriptorsItem<'a, T::IntoIter>: WithErasedDrop,
{
    type Item = Result<ErasedComponent<S, P>, S::Error>;
    type IntoIter = ErasedBundleIntoIterKind<S, T, S, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let inner = self.into_inner();

        let inner = inner.into_iter();
        ErasedBundleIntoIterKind { inner }
    }
}

impl<'a, T, D, S, P> FieldDescriptors<'a> for ErasedBundleKind<T, D, S, P>
where
    T: ErasedArchetypeKind + ?Sized,
    D: ErasedBundleDrop<T::Meta>,
    S: AlignedStorage,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
{
    type Output = ErasedArchetypeView<'a, T::Meta>;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        self.archetype()
    }
}

impl<T, D, S, P> CovariantFieldDescriptors for ErasedBundleKind<T, D, S, P>
where
    T: ErasedArchetypeKind + ?Sized,
    D: ErasedBundleDrop<T::Meta>,
    S: AlignedStorage,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        from
    }
}

pub type ErasedBorrowedViewBundleIntoIter<'a, S, Meta, F, P> =
    ErasedBundleIntoIterKind<S, ErasedArchetypeView<'a, Meta>, F, P>;

pub struct ErasedBundleIntoIterKind<S, T, F, P>
where
    T: ErasedArchetypeKind + IntoIterator,
{
    inner: ErasedSoaIntoFields<S, T::IntoIter, F, P>,
}

impl<S, T, F, P> Iterator for ErasedBundleIntoIterKind<S, T, F, P>
where
    S: AlignedStorage<Item: Copy>,
    T: ErasedArchetypeKind + IntoErasedArchetypeIterator,
    F: AlignedStorageFromLayout<Item = S::Item>,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
    for<'a> FieldDescriptorsItem<'a, T::IntoIter>: WithErasedDrop,
{
    type Item = Result<ErasedComponent<F, P>, F::Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        use gpecs_soa_erased::data::error::FromLayoutDataError::{
            FromLayout, InsufficientAlign, LenMismatch,
        };

        let Self { inner } = self;

        let component = inner.field_descriptors().into_iter().next()?;
        let erased_drop = component.erased_drop();
        let id = component.component_id();
        drop(component);

        let item = match inner.next()? {
            Ok(field) => {
                let component = unsafe { ErasedComponent::from_parts(id, field, erased_drop) };
                Ok(component)
            }
            Err(error) => match error {
                FromLayout(error) => Err(error),
                LenMismatch(error) => unreachable!("failed to create erased data: {error}"),
                InsufficientAlign(error) => unreachable!("failed to create erased data: {error}"),
            },
        };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<S, T, F, P> ExactSizeIterator for ErasedBundleIntoIterKind<S, T, F, P>
where
    S: AlignedStorage<Item: Copy>,
    T: ErasedArchetypeKind + IntoErasedArchetypeIterator,
    F: AlignedStorageFromLayout<Item = S::Item>,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
    T::IntoIter: ExactSizeIterator,
    for<'a> FieldDescriptorsItem<'a, T::IntoIter>: WithErasedDrop,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<S, T, F, P> FusedIterator for ErasedBundleIntoIterKind<S, T, F, P>
where
    S: AlignedStorage<Item: Copy>,
    T: ErasedArchetypeKind + IntoErasedArchetypeIterator,
    F: AlignedStorageFromLayout<Item = S::Item>,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
    T::IntoIter: FusedIterator,
    for<'a> FieldDescriptorsItem<'a, T::IntoIter>: WithErasedDrop,
{
}
