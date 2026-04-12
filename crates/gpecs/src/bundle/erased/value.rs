use std::{
    alloc::Layout,
    fmt::{self, Debug},
    iter::{FusedIterator, chain},
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
    ptr,
};

use gpecs_archetype::bundle::erased::{
    ErasedBundleMutPtrs, ErasedBundleMutRefs, ErasedBundlePtrs, ErasedBundleRefs,
};
use gpecs_soa_erased::{
    CovariantFieldDescriptors, ErasedSoa, ErasedSoaIntoFields,
    error::{FromDescriptorsValueError, FromDescriptorsValueErrorKind, FromFieldsDescriptorsError},
    ptr::slice::{CoreSliceItemPtrs, SliceItemPtrs},
    storage::{AlignedStorage, AlignedStorageFromLayout, BoxedAlignedUninitStorage},
};
use itertools::{equal, zip_eq};

use crate::{
    archetype::erased::{
        ComponentIds, ErasedArchetype, ErasedArchetypeView, FromComponentInfo, Iter,
        error::{AlreadyHasComponentError, MissingComponentError},
    },
    bundle::{
        Bundle, BundleRefs, BundleRefsMut,
        erased::{
            ErasedBundleMutRefsIter, ErasedBundleRefsIter,
            error::{
                DowncastError, FromBundleError, FromBundleErrorKind, FromComponentsError,
                InsertError, InsertErrorKind, RemoveError, RemoveErrorKind, ReplaceError,
                ReplaceErrorKind, ShuffleError, ShuffleErrorKind,
            },
            traits::{
                ErasedArchetypeKind, ErasedBundleDrop, IntoErasedArchetypeIterator, MustDrop,
            },
        },
    },
    component::{
        erased::{ErasedComponent, ErasedComponentMutRef, ErasedComponentRef, WithErasedDrop},
        registry::{
            ComponentId, ComponentRegistry, ComponentRegistryView,
            traits::{
                ComponentIdFrom, ComponentIdFromOrInsertWith, FromComponentType, PushBackArray,
                WithComponentId,
            },
        },
    },
    soa::{
        field::{FieldDescriptor, FieldDescriptors, FieldDescriptorsItem, FieldDescriptorsOutput},
        traits::ReadSoaContext,
    },
};

pub type ErasedBundle<Meta, D = MustDrop, S = St, P = SlicePtrs> =
    ErasedBundleKind<ErasedArchetype<Meta>, D, S, P>;
pub type ErasedBorrowedBundle<'a, Meta, D = MustDrop, S = St, P = SlicePtrs> =
    ErasedBundleKind<&'a ErasedArchetype<Meta>, D, S, P>;
pub type ErasedBorrowedViewBundle<'a, Meta, D = MustDrop, S = St, P = SlicePtrs> =
    ErasedBundleKind<ErasedArchetypeView<'a, Meta>, D, S, P>;

pub struct ErasedBundleKind<T, D = MustDrop, S = St, P = SlicePtrs>
where
    T: ErasedArchetypeKind + ?Sized,
    D: ErasedBundleDrop<T::Meta>,
    S: AlignedStorage,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
{
    phantom: PhantomData<D>,
    inner: ErasedSoa<S, T, P>,
}

type St = BoxedAlignedUninitStorage;
type SlicePtrs = CoreSliceItemPtrs<MaybeUninit<u8>>;

impl<Meta, D, S, P> ErasedBundle<Meta, D, S, P>
where
    Meta: AsRef<FieldDescriptor> + 'static,
    D: ErasedBundleDrop<Meta>,
    S: AlignedStorageFromLayout,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
{
    #[inline]
    pub fn try_from<'a, B, M, T>(
        components: &'a mut ComponentRegistry<M, T>,
        bundle: B,
    ) -> Result<Self, FromBundleError<B, S::Error>>
    where
        B: Bundle,
        Meta: FromComponentInfo<'a, M::Item>,
        M: PushBackArray<Item: FromComponentType>,
        T: ComponentIdFromOrInsertWith<Key: FromComponentType> + ?Sized,
    {
        let archetype = match ErasedArchetype::register::<B, M, T>(components) {
            Ok(archetype) => archetype,
            Err(source) => return Err(FromBundleError::new(bundle, source.into())),
        };
        let inner = ErasedSoa::try_from_descriptors_value::<B, B>(archetype, B::CONTEXT, bundle)
            .map_err(into_from_bundle_error)?;

        let me = unsafe { Self::from_inner(inner) };
        Ok(me)
    }
}

#[inline]
fn into_from_bundle_error<B, T>(error: FromDescriptorsValueError<B, T>) -> FromBundleError<B, T> {
    let FromDescriptorsValueError { value, source, .. } = error;
    let source = match source {
        FromDescriptorsValueErrorKind::FromLayout(error) => FromBundleErrorKind::FromLayout(error),
        FromDescriptorsValueErrorKind::InsufficientAlign(error) => error.into(),
        FromDescriptorsValueErrorKind::InvalidLayout(error) => error.into(),
        FromDescriptorsValueErrorKind::LenMismatch(error) => {
            unreachable!("failed to erase some bundle: {error}")
        }
        FromDescriptorsValueErrorKind::LayoutMismatch(error) => {
            unreachable!("failed to erase some bundle: {error}")
        }
    };
    FromBundleError::new(value, source)
}

pub trait FromErasedComponent<S, P>: Sized
where
    S: AlignedStorage,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
{
    fn from_erased_component(component: &ErasedComponent<S, P>) -> Self;
}

impl<Meta, D, S, P> ErasedBundle<Meta, D, S, P>
where
    Meta: AsRef<FieldDescriptor> + FromErasedComponent<S, P> + 'static,
    D: ErasedBundleDrop<Meta>,
    S: AlignedStorageFromLayout<Item: Copy>,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
{
    #[inline]
    pub fn from_components<I>(
        components: &ComponentRegistryView<impl Sized, impl ?Sized>,
        iter: I,
    ) -> Result<Self, FromComponentsError<S::Error>>
    where
        I: IntoIterator<Item = ErasedComponent<S, P>>,
    {
        let iter = iter
            .into_iter()
            .map(|component| (component.component_id(), component));
        let components = ErasedArchetype::from_iter(components, iter)?;

        let iter = components.iter().map(|component_info| {
            let component_id = component_info.component_id();
            let meta = Meta::from_erased_component(component_info.as_meta());
            (component_id, meta)
        });
        let archetype = unsafe { ErasedArchetype::from_iter_unchecked(iter) };

        let fields = components
            .into_iter()
            .map(|component_info| component_info.into_meta().into_field());

        let inner = ErasedSoa::try_from_fields_descriptors(fields, archetype)
            .map_err(into_from_components_error)?;

        let me = unsafe { Self::from_inner(inner) };
        Ok(me)
    }
}

#[inline]
fn into_from_components_error<T>(error: FromFieldsDescriptorsError<T>) -> FromComponentsError<T> {
    match error {
        FromFieldsDescriptorsError::FromLayout(error) => FromComponentsError::FromLayout(error),
        FromFieldsDescriptorsError::InvalidLayout(error) => error.into(),
        FromFieldsDescriptorsError::LenMismatch(error) => {
            unreachable!("failed to create erased bundle from components: {error}")
        }
        FromFieldsDescriptorsError::InsufficientAlign(error) => {
            unreachable!("failed to create erased bundle from components: {error}")
        }
    }
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

    #[inline]
    pub fn downcast<B, U>(
        self,
        registry: &ComponentRegistryView<impl Sized, U>,
    ) -> Result<B, DowncastError<Self>>
    where
        B: Bundle,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let src = match self.as_ptrs().downcast::<B, U>(registry) {
            Ok(src) => src,
            Err(error) => return Err(error.map_value(drop).map_value(|()| self)),
        };

        let bundle = unsafe { B::CONTEXT.read(src) };
        let _ = self.into_inner();
        Ok(bundle)
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
    pub fn downcast_ref<B, U>(
        &self,
        registry: &ComponentRegistryView<impl Sized, U>,
    ) -> Result<BundleRefs<'_, B>, DowncastError<&Self>>
    where
        B: Bundle,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        self.as_refs()
            .downcast::<B, U>(registry)
            .map_err(|error| error.map_value(|_| self))
    }

    #[inline]
    pub fn downcast_mut<B, U>(
        &mut self,
        registry: &ComponentRegistryView<impl Sized, U>,
    ) -> Result<BundleRefsMut<'_, B>, DowncastError<&mut Self>>
    where
        B: Bundle,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        match unsafe { self.as_mut_ptrs().deref_mut() }.downcast::<B, U>(registry) {
            Ok(refs) => Ok(refs),
            Err(error) => Err(error.map_value(drop).map_value(|()| self)),
        }
    }

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
        let refs = unsafe { ptrs.deref() };
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
        let refs = unsafe { ptrs.deref_mut() };
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

impl<T, D, S, P> ErasedBundleKind<T, D, S, P>
where
    T: ErasedArchetypeKind<Meta: Clone>,
    D: ErasedBundleDrop<T::Meta>,
    S: AlignedStorageFromLayout<Item: Copy>,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
{
    #[inline]
    #[expect(clippy::type_complexity)]
    // FIXME: can we optimize this?
    pub fn insert<ToInsert>(
        self,
        to_insert: ErasedBundleKind<ToInsert, D, S, P>,
    ) -> Result<
        ErasedBundle<T::Meta, D, S, P>,
        InsertError<Self, ErasedBundleKind<ToInsert, D, S, P>, S::Error>,
    >
    where
        ToInsert: ErasedArchetypeKind<Meta = T::Meta>,
    {
        if let Some(has_component_id) = to_insert
            .archetype()
            .component_ids()
            .find(|&id| self.archetype().contains(id))
        {
            let error = InsertError {
                source: AlreadyHasComponentError::new(has_component_id).into(),
                bundle: self,
                to_insert,
            };
            return Err(error);
        }

        let refs = chain(self.as_refs(), to_insert.as_refs());
        let iter = chain(self.archetype(), to_insert.archetype())
            .map(|component_info| component_info.map_meta(Clone::clone).into_parts());
        let archetype = unsafe { ErasedArchetype::from_iter_unchecked(iter) };

        let result = ErasedSoa::try_from_fields_descriptors(refs, archetype);
        let inner = match result.map_err(into_insert_error_kind) {
            Ok(inner) => inner,
            Err(source) => {
                let error = InsertError {
                    source,
                    bundle: self,
                    to_insert,
                };
                return Err(error);
            }
        };

        let _ = (self.into_inner(), to_insert.into_inner());
        let bundle = unsafe { ErasedBundle::from_inner(inner) };
        Ok(bundle)
    }
}

#[inline]
fn into_insert_error_kind<E>(error: FromFieldsDescriptorsError<E>) -> InsertErrorKind<E> {
    match error {
        FromFieldsDescriptorsError::FromLayout(error) => InsertErrorKind::FromLayout(error),
        FromFieldsDescriptorsError::InvalidLayout(error) => error.into(),
        FromFieldsDescriptorsError::LenMismatch(error) => {
            unreachable!("failed to insert some components into bundle: {error}")
        }
        FromFieldsDescriptorsError::InsufficientAlign(error) => {
            unreachable!("failed to insert some components into bundle: {error}")
        }
    }
}

impl<T, D, S, P> ErasedBundleKind<T, D, S, P>
where
    T: ErasedArchetypeKind<Meta: Clone + WithErasedDrop>,
    D: ErasedBundleDrop<T::Meta>,
    S: AlignedStorageFromLayout<Item: Copy>,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
{
    #[inline]
    #[expect(clippy::type_complexity)]
    // FIXME: can we optimize this?
    pub fn replace<ToReplace>(
        mut self,
        to_replace: ErasedBundleKind<ToReplace, D, S, P>,
    ) -> Result<
        ErasedBundle<T::Meta, D, S, P>,
        ReplaceError<Self, ErasedBundleKind<ToReplace, D, S, P>, S::Error>,
    >
    where
        ToReplace: ErasedArchetypeKind<Meta = T::Meta>,
    {
        let (ptrs, archetype) = self.as_mut_ptrs_with_archetype();

        let ptrs = zip_eq(ptrs, archetype).map(|(dst, component_info)| {
            if to_replace.archetype().contains(dst.component_id()) {
                if let Some(erased_drop) = component_info.erased_drop() {
                    unsafe { erased_drop.drop_in_place(dst) }
                }
                let src = to_replace
                    .as_ptrs()
                    .get(dst.component_id())
                    .expect("to replace archetype should contain component");
                unsafe { dst.copy_from_nonoverlapping(src, 1) }
            }
            dst.cast_const()
        });
        let ptrs_to_append = to_replace
            .as_ptrs()
            .into_iter()
            .filter(|ptr| !archetype.contains(ptr.component_id()));
        let refs = chain(ptrs, ptrs_to_append).map(|ptr| unsafe { ptr.deref() });

        let metas_to_append = to_replace
            .archetype()
            .into_iter()
            .filter(|component_info| !archetype.contains(component_info.component_id()));
        let iter = chain(archetype, metas_to_append)
            .map(|component_info| component_info.map_meta(Clone::clone).into_parts());
        let archetype = unsafe { ErasedArchetype::from_iter_unchecked(iter) };

        let result = ErasedSoa::try_from_fields_descriptors(refs, archetype);
        let inner = match result.map_err(into_replace_error_kind) {
            Ok(inner) => inner,
            Err(source) => {
                let error = ReplaceError {
                    source,
                    bundle: self,
                    to_replace,
                };
                return Err(error);
            }
        };

        let _ = (self.into_inner(), to_replace.into_inner());
        let bundle = unsafe { ErasedBundle::from_inner(inner) };
        Ok(bundle)
    }
}

#[inline]
fn into_replace_error_kind<E>(error: FromFieldsDescriptorsError<E>) -> ReplaceErrorKind<E> {
    match error {
        FromFieldsDescriptorsError::FromLayout(error) => ReplaceErrorKind::FromLayout(error),
        FromFieldsDescriptorsError::InvalidLayout(error) => error.into(),
        FromFieldsDescriptorsError::LenMismatch(error) => {
            unreachable!("failed to replace some components in bundle: {error}")
        }
        FromFieldsDescriptorsError::InsufficientAlign(error) => {
            unreachable!("failed to replace some components in bundle: {error}")
        }
    }
}

pub struct RemovePair<ToRemove, D, S, P>
where
    ToRemove: ErasedArchetypeKind,
    D: ErasedBundleDrop<ToRemove::Meta>,
    S: AlignedStorageFromLayout,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
{
    pub retained: ErasedBundle<ToRemove::Meta, D, S, P>,
    pub removed: ErasedBundleKind<ToRemove, D, S, P>,
}

impl<ToRemove, D, S, P> Debug for RemovePair<ToRemove, D, S, P>
where
    ToRemove: ErasedArchetypeKind,
    D: ErasedBundleDrop<ToRemove::Meta>,
    S: AlignedStorageFromLayout,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { retained, removed } = self;
        f.debug_struct("RemovePair")
            .field("retained", retained)
            .field("removed", removed)
            .finish()
    }
}

impl<T, D, S, P> ErasedBundleKind<T, D, S, P>
where
    T: ErasedArchetypeKind<Meta: Clone>,
    D: ErasedBundleDrop<T::Meta>,
    S: AlignedStorageFromLayout<Item: Copy>,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
{
    #[inline]
    #[expect(clippy::type_complexity)]
    // FIXME: can we optimize this?
    pub fn remove<ToRemove>(
        self,
        to_remove: ToRemove,
    ) -> Result<RemovePair<ToRemove, D, S, P>, RemoveError<Self, S::Error>>
    where
        ToRemove: ErasedArchetypeKind<Meta = T::Meta>,
    {
        let archetype_to_remove = to_remove.field_descriptors();
        let bundle = self.check_remove(archetype_to_remove.component_ids())?;

        let retained_refs = bundle
            .as_refs()
            .into_iter()
            .filter(|component| !archetype_to_remove.contains(component.component_id()));
        let retained_iter = bundle
            .archetype()
            .into_iter()
            .filter(|component_info| !archetype_to_remove.contains(component_info.component_id()))
            .map(|component_info| component_info.map_meta(Clone::clone).into_parts());
        let retained_archetype = unsafe { ErasedArchetype::from_iter_unchecked(retained_iter) };
        let result = ErasedSoa::try_from_fields_descriptors(retained_refs, retained_archetype);
        let retained_inner = match result.map_err(into_remove_error_kind) {
            Ok(inner) => inner,
            Err(source) => {
                let error = RemoveError { source, bundle };
                return Err(error);
            }
        };

        let removed_refs = bundle
            .as_refs()
            .into_iter()
            .filter(|component| archetype_to_remove.contains(component.component_id()));
        let result =
            ErasedSoa::<_, _, P>::try_from_fields_descriptors(removed_refs, archetype_to_remove);
        let removed_inner = match result.map_err(into_remove_error_kind) {
            Ok(inner) => inner,
            Err(source) => {
                let error = RemoveError { source, bundle };
                return Err(error);
            }
        };
        let (removed_storage, _) = removed_inner.into_parts();
        let removed_inner = unsafe { ErasedSoa::from_parts(removed_storage, to_remove) };

        let _ = bundle.into_inner();
        let pair = RemovePair {
            retained: unsafe { ErasedBundleKind::from_inner(retained_inner) },
            removed: unsafe { ErasedBundleKind::from_inner(removed_inner) },
        };
        Ok(pair)
    }
}

impl<T, D, S, P> ErasedBundleKind<T, D, S, P>
where
    T: ErasedArchetypeKind<Meta: WithErasedDrop + Clone>,
    D: ErasedBundleDrop<T::Meta>,
    S: AlignedStorageFromLayout<Item: Copy>,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
{
    #[inline]
    #[expect(clippy::type_complexity)]
    // FIXME: can we optimize this?
    pub fn destroy(
        self,
        to_destroy: ErasedArchetypeView<impl Sized>,
    ) -> Result<ErasedBundle<T::Meta, D, S, P>, RemoveError<Self, S::Error>> {
        let mut bundle = self.check_remove(to_destroy.component_ids())?;

        let (refs, archetype) = bundle.as_mut_refs_with_archetype();
        let fields = zip_eq(refs, archetype).filter_map(|(mut field, component_info)| {
            if to_destroy.contains(component_info.component_id()) {
                let to_drop = field.as_mut_component_ptr();
                if let Some(erased_drop) = component_info.erased_drop() {
                    unsafe { erased_drop.drop_in_place(to_drop) }
                }
                return None;
            }
            Some(field)
        });
        let iter = archetype.iter().filter_map(|component_info| {
            if to_destroy.contains(component_info.component_id()) {
                return None;
            }
            let component_info = component_info.map_meta(Clone::clone);
            Some(component_info.into_parts())
        });
        let archetype = unsafe { ErasedArchetype::from_iter_unchecked(iter) };
        let result = ErasedSoa::try_from_fields_descriptors(fields, archetype);
        let inner = match result.map_err(into_remove_error_kind) {
            Ok(inner) => inner,
            Err(source) => {
                let error = RemoveError { source, bundle };
                return Err(error);
            }
        };

        let _ = bundle.into_inner();
        let bundle = unsafe { ErasedBundle::from_inner(inner) };
        Ok(bundle)
    }
}

impl<T, D, S, P> ErasedBundleKind<T, D, S, P>
where
    T: ErasedArchetypeKind,
    D: ErasedBundleDrop<T::Meta>,
    S: AlignedStorage,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
{
    #[inline]
    fn check_remove<E>(
        self,
        mut to_remove: ComponentIds<'_>,
    ) -> Result<Self, RemoveError<Self, E>> {
        if let Some(missing_component_id) = to_remove.find(|&id| !self.archetype().contains(id)) {
            let error = RemoveError {
                source: MissingComponentError::new(missing_component_id).into(),
                bundle: self,
            };
            return Err(error);
        }
        Ok(self)
    }
}

#[inline]
fn into_remove_error_kind<E>(error: FromFieldsDescriptorsError<E>) -> RemoveErrorKind<E> {
    match error {
        FromFieldsDescriptorsError::FromLayout(error) => RemoveErrorKind::FromLayout(error),
        FromFieldsDescriptorsError::LenMismatch(error) => {
            unreachable!("failed to remove some components of bundle: {error}")
        }
        FromFieldsDescriptorsError::InsufficientAlign(error) => {
            unreachable!("failed to remove some components of bundle: {error}")
        }
        FromFieldsDescriptorsError::InvalidLayout(error) => {
            unreachable!("failed to remove some components of bundle: {error}")
        }
    }
}

impl<'a, Meta, D, S, P> From<ErasedBorrowedBundle<'a, Meta, D, S, P>>
    for ErasedBorrowedViewBundle<'a, Meta, D, S, P>
where
    Meta: AsRef<FieldDescriptor> + Clone + 'static,
    D: ErasedBundleDrop<Meta>,
    S: AlignedStorage,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
{
    #[inline]
    fn from(bundle: ErasedBorrowedBundle<'a, Meta, D, S, P>) -> Self {
        let (storage, archetype) = bundle.into_inner().into_parts();
        let archetype = archetype.as_view();

        let inner = unsafe { ErasedSoa::from_parts(storage, archetype) };
        unsafe { Self::from_inner(inner) }
    }
}

impl<'a, Meta, D, S, P> From<ErasedBorrowedBundle<'a, Meta, D, S, P>>
    for ErasedBundle<Meta, D, S, P>
where
    Meta: AsRef<FieldDescriptor> + Clone + 'static,
    D: ErasedBundleDrop<Meta>,
    S: AlignedStorage,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
{
    #[inline]
    fn from(bundle: ErasedBorrowedBundle<'a, Meta, D, S, P>) -> Self {
        let (storage, archetype) = bundle.into_inner().into_parts();
        let archetype = archetype.clone();

        let inner = unsafe { ErasedSoa::from_parts(storage, archetype) };
        unsafe { Self::from_inner(inner) }
    }
}

impl<'a, Meta, D, S, P> From<ErasedBorrowedViewBundle<'a, Meta, D, S, P>>
    for ErasedBundle<Meta, D, S, P>
where
    Meta: AsRef<FieldDescriptor> + Clone + 'static,
    D: ErasedBundleDrop<Meta>,
    S: AlignedStorage,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
{
    #[inline]
    fn from(bundle: ErasedBorrowedViewBundle<'a, Meta, D, S, P>) -> Self {
        let (storage, archetype) = bundle.into_inner().into_parts();
        let archetype = archetype.into();

        let inner = unsafe { ErasedSoa::from_parts(storage, archetype) };
        unsafe { Self::from_inner(inner) }
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

pub type ErasedBundleIntoIter<S, Meta, F, P> =
    ErasedBundleIntoIterKind<S, ErasedArchetype<Meta>, F, P>;
pub type ErasedBorrowedBundleIntoIter<'a, S, Meta, F, P> =
    ErasedBundleIntoIterKind<S, &'a ErasedArchetype<Meta>, F, P>;
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
