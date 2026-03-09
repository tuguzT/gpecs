use std::{
    alloc::Layout,
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::{ManuallyDrop, MaybeUninit},
    ptr,
};

use gpecs_soa_erased::{
    CovariantFieldDescriptors, ErasedSoa, ErasedSoaIntoFields,
    ptr::slice::CoreSliceItemPtrs,
    storage::{AllocError, BoxedAlignedUninitStorage},
};
use itertools::{equal, zip_eq};

use crate::{
    archetype::{
        collect::try_collect_components,
        erased::{ErasedArchetype, FromComponentInfo},
    },
    bundle::{
        Bundle, BundleRefs, BundleRefsMut,
        erased::{
            ErasedBundleMutPtrs, ErasedBundleMutRefs, ErasedBundleMutRefsIter, ErasedBundlePtrs,
            ErasedBundleRefs, ErasedBundleRefsIter,
            error::{DowncastError, FromBundleError, FromComponentsError, ShuffleError},
        },
    },
    component::{
        erased::{ErasedComponent, ErasedComponentMutRef, ErasedComponentRef},
        registry::{ComponentId, ComponentRegistry, DropFn},
    },
    hash::IndexSet,
    soa::{
        field::{FieldDescriptor, FieldDescriptors},
        traits::ReadSoaContext,
    },
};

pub trait ErasedArchetypeKind:
    private::Sealed
    + for<'a> FieldDescriptors<'a, Output = &'a ErasedArchetype<Self::Meta>>
    + for<'a> IntoIterator<
        Item: AsRef<FieldDescriptor>,
        IntoIter: FieldDescriptors<
            'a,
            Output: IntoIterator<
                Item: AsRef<FieldDescriptor> + AsRef<Option<DropFn>> + Into<ComponentId>,
            > + Clone,
        >,
    >
{
    type Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>> + 'static;
}

impl<Meta> ErasedArchetypeKind for ErasedArchetype<Meta>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>> + 'static,
{
    type Meta = Meta;
}

impl<Meta> ErasedArchetypeKind for &ErasedArchetype<Meta>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>> + 'static,
{
    type Meta = Meta;
}

mod private {
    use super::ErasedArchetype;

    pub trait Sealed {}

    impl<Meta> Sealed for ErasedArchetype<Meta> {}
    impl<Meta> Sealed for &ErasedArchetype<Meta> {}
}

pub type ErasedBundle<Meta> = ErasedBundleKind<ErasedArchetype<Meta>>;
pub type ErasedBorrowedBundle<'a, Meta> = ErasedBundleKind<&'a ErasedArchetype<Meta>>;

pub struct ErasedBundleKind<T>
where
    T: ErasedArchetypeKind,
{
    inner: Inner<T>,
}

type Inner<T> = ErasedSoa<BoxedAlignedUninitStorage, T, CoreSliceItemPtrs<MaybeUninit<u8>>>;

impl<Meta> ErasedBundle<Meta>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>> + FromComponentInfo + 'static,
{
    #[inline]
    pub fn try_from<B>(
        registry: &mut ComponentRegistry,
        bundle: B,
    ) -> Result<Self, FromBundleError<B>>
    where
        B: Bundle,
    {
        let archetype = match ErasedArchetype::register::<B>(registry) {
            Ok(archetype) => archetype,
            Err(reason) => return Err(FromBundleError::new(bundle, reason.into())),
        };
        let inner = Inner::try_from_descriptors_value::<B, _>(archetype, B::CONTEXT, bundle)
            .map_err(|error| {
                use gpecs_soa_erased::error::{
                    FromDescriptorsValueError, FromDescriptorsValueErrorKind::FromLayout,
                };

                let FromDescriptorsValueError { value, reason, .. } = error;
                match reason {
                    FromLayout(reason) => FromBundleError::new(value, reason.into()),
                    _ => unreachable!("{reason}"),
                }
            })?;

        let me = unsafe { Self::from_inner(inner) };
        Ok(me)
    }
}

pub trait FromErasedComponent: Sized {
    fn from_erased_component(component: &ErasedComponent) -> Self;
}

impl<Meta> ErasedBundle<Meta>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>> + FromErasedComponent + 'static,
{
    #[inline]
    pub fn from_components<I>(components: I) -> Result<Self, FromComponentsError>
    where
        I: IntoIterator<Item = ErasedComponent>,
    {
        let components =
            try_collect_components(components, IndexSet::insert, ErasedComponent::component_id)?;

        let iter = components.iter().map(|component| {
            let id = component.component_id();
            let meta = Meta::from_erased_component(component);
            (id, meta)
        });
        let archetype = unsafe { ErasedArchetype::with_meta_unchecked(iter) };

        let fields = components.into_iter().map(ErasedComponent::into_field);
        let inner = Inner::try_from_fields_descriptors(fields, archetype).map_err(|error| {
            use gpecs_soa_erased::error::FromFieldsDescriptorsError::{FromLayout, InvalidLayout};

            match error {
                InvalidLayout(error) => FromComponentsError::from(error),
                FromLayout(error) => FromComponentsError::from(error),
                _ => unreachable!("failed to create erased bundle from components: {error}"),
            }
        })?;

        let me = unsafe { Self::from_inner(inner) };
        Ok(me)
    }
}

impl<T> ErasedBundleKind<T>
where
    T: ErasedArchetypeKind,
{
    #[inline]
    pub unsafe fn from_inner(inner: Inner<T>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn downcast<B>(self, registry: &ComponentRegistry) -> Result<B, DowncastError<Self>>
    where
        B: Bundle,
    {
        let src = match self.as_ptrs().downcast::<B>(registry) {
            Ok(src) => src,
            Err(reason) => return Err(DowncastError::new(self, reason)),
        };

        let bundle = unsafe { B::CONTEXT.read(src) };
        let _ = self.into_inner();
        Ok(bundle)
    }

    #[inline]
    pub fn downcast_ref<B>(
        &self,
        registry: &ComponentRegistry,
    ) -> Result<BundleRefs<'_, B>, DowncastError<&Self>>
    where
        B: Bundle,
    {
        self.as_refs()
            .downcast::<B>(registry)
            .map_err(|reason| DowncastError::new(self, reason))
    }

    #[inline]
    pub fn downcast_mut<B>(
        &mut self,
        registry: &ComponentRegistry,
    ) -> Result<BundleRefsMut<'_, B>, DowncastError<&mut Self>>
    where
        B: Bundle,
    {
        unsafe { self.as_mut_ptrs().deref_mut() }
            .downcast::<B>(registry)
            .map_err(|reason| DowncastError::new(self, reason))
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { inner } = self;
        inner.layout()
    }

    #[inline]
    pub fn archetype(&self) -> &ErasedArchetype<T::Meta> {
        let Self { inner } = self;
        inner.field_descriptors()
    }

    #[inline]
    pub fn as_ptr(&self) -> *const MaybeUninit<u8> {
        let Self { inner } = self;
        inner.as_ptr()
    }

    #[inline]
    pub unsafe fn as_mut_ptr(&mut self) -> *mut MaybeUninit<u8> {
        let Self { inner } = self;
        inner.as_mut_ptr()
    }

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
    pub fn as_ptrs(&self) -> ErasedBundlePtrs<'_, T::Meta> {
        let Self { inner } = self;

        let inner = inner.as_ptrs();
        unsafe { ErasedBundlePtrs::from_inner(inner) }
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> ErasedBundleMutPtrs<'_, T::Meta> {
        let Self { inner } = self;

        let inner = inner.as_mut_ptrs();
        unsafe { ErasedBundleMutPtrs::from_inner(inner) }
    }

    #[inline]
    pub fn as_refs(&self) -> ErasedBundleRefs<'_, '_, T::Meta> {
        unsafe { self.as_ptrs().deref() }
    }

    #[inline]
    pub fn as_mut_refs(&mut self) -> ErasedBundleMutRefs<'_, '_, T::Meta> {
        unsafe { self.as_mut_ptrs().deref_mut() }
    }

    #[inline]
    pub fn iter(&self) -> ErasedBundleRefsIter<'_, '_, T::Meta> {
        self.as_refs().into_iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> ErasedBundleMutRefsIter<'_, '_, T::Meta> {
        self.as_mut_refs().into_iter()
    }

    #[inline]
    pub fn into_inner(self) -> Inner<T> {
        let me = ManuallyDrop::new(self);
        unsafe { ptr::read(&raw const me.inner) }
    }
}

#[derive(Debug)]
pub enum ShuffledBundle<Original, Other>
where
    Original: ErasedArchetypeKind,
    Other: ErasedArchetypeKind<Meta = Original::Meta>,
{
    Original(ErasedBundleKind<Original>),
    Other(ErasedBundleKind<Other>),
}

impl<Original> ErasedBundleKind<Original>
where
    Original: ErasedArchetypeKind,
{
    #[inline]
    pub fn shuffle<Other>(
        self,
        archetype: Other,
    ) -> Result<ShuffledBundle<Original, Other>, ShuffleError<Self, Other>>
    where
        Other: ErasedArchetypeKind<Meta = Original::Meta>,
    {
        let this = self.archetype();
        let other = archetype.field_descriptors();
        if let Err(error) = this.check_exact_compatibility(other) {
            let error = ShuffleError {
                bundle: self,
                archetype,
                reason: error.into(),
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
        let result = Inner::try_from_fields_descriptors(fields, other);

        let result = result.map_err(|error| {
            use gpecs_soa_erased::error::FromFieldsDescriptorsError::{FromLayout, InvalidLayout};

            match error {
                FromLayout(error) => error.into(),
                InvalidLayout(error) => error.into(),
                _ => unreachable!("failed to shuffle bundle: {error}"),
            }
        });
        let inner = match result {
            Ok(inner) => inner,
            Err(reason) => {
                let error = ShuffleError {
                    bundle: self,
                    archetype,
                    reason,
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

impl<'a, Meta> From<ErasedBorrowedBundle<'a, Meta>> for ErasedBundle<Meta>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>> + Clone + 'static,
{
    #[inline]
    fn from(bundle: ErasedBorrowedBundle<'a, Meta>) -> Self {
        let (storage, archetype) = bundle.into_inner().into_parts();
        let archetype = archetype.clone();

        let inner = unsafe { Inner::from_parts(storage, archetype) };
        unsafe { Self::from_inner(inner) }
    }
}

impl<T> Debug for ErasedBundleKind<T>
where
    T: ErasedArchetypeKind,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let components = &self.into_iter();
        f.debug_struct("ErasedBundle")
            .field("components", components)
            .finish()
    }
}

impl<T> AsRef<[MaybeUninit<u8>]> for ErasedBundleKind<T>
where
    T: ErasedArchetypeKind,
{
    #[inline]
    fn as_ref(&self) -> &[MaybeUninit<u8>] {
        self.as_buffer()
    }
}

impl<T> AsRef<ErasedArchetype<T::Meta>> for ErasedBundleKind<T>
where
    T: ErasedArchetypeKind,
{
    #[inline]
    fn as_ref(&self) -> &ErasedArchetype<T::Meta> {
        self.archetype()
    }
}

impl<T> Drop for ErasedBundleKind<T>
where
    T: ErasedArchetypeKind,
{
    fn drop(&mut self) {
        let Self { inner } = self;

        let ptrs = inner.as_mut_ptrs().into_iter();
        let components = ptrs.descriptors().clone();
        for (ptr, component) in zip_eq(ptrs, components) {
            let Some(drop_fn) = component.as_ref() else {
                continue;
            };

            let ptr = ptr.as_mut_ptr().cast();
            unsafe { drop_fn(ptr) }
        }
    }
}

impl<'a, T> IntoIterator for &'a ErasedBundleKind<T>
where
    T: ErasedArchetypeKind,
{
    type Item = ErasedComponentRef<'a>;
    type IntoIter = ErasedBundleRefsIter<'a, 'a, T::Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut ErasedBundleKind<T>
where
    T: ErasedArchetypeKind,
{
    type Item = ErasedComponentMutRef<'a>;
    type IntoIter = ErasedBundleMutRefsIter<'a, 'a, T::Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> IntoIterator for ErasedBundleKind<T>
where
    T: ErasedArchetypeKind,
{
    type Item = Result<ErasedComponent, AllocError>;
    type IntoIter = ErasedBundleIntoIterKind<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let inner = self.into_inner();

        let inner = inner.into_iter();
        ErasedBundleIntoIterKind { inner }
    }
}

impl<'a, T> FieldDescriptors<'a> for ErasedBundleKind<T>
where
    T: ErasedArchetypeKind,
{
    type Output = &'a ErasedArchetype<T::Meta>;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        self.archetype()
    }
}

impl<T> CovariantFieldDescriptors for ErasedBundleKind<T>
where
    T: ErasedArchetypeKind,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: <Self as FieldDescriptors<'long>>::Output,
    ) -> <Self as FieldDescriptors<'short>>::Output {
        from
    }
}

pub type ErasedBundleIntoIter<Meta> = ErasedBundleIntoIterKind<ErasedArchetype<Meta>>;
pub type ErasedBorrowedBundleIntoIter<'a, Meta> =
    ErasedBundleIntoIterKind<&'a ErasedArchetype<Meta>>;

pub struct ErasedBundleIntoIterKind<T>
where
    T: ErasedArchetypeKind,
{
    inner: InnerIter<T::IntoIter>,
}

type InnerIter<T> = ErasedSoaIntoFields<
    BoxedAlignedUninitStorage,
    T,
    BoxedAlignedUninitStorage,
    CoreSliceItemPtrs<MaybeUninit<u8>>,
>;

impl<T> Iterator for ErasedBundleIntoIterKind<T>
where
    T: ErasedArchetypeKind,
{
    type Item = Result<ErasedComponent, AllocError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        use gpecs_soa_erased::data::error::FromLayoutDataError::FromLayout;

        let Self { inner } = self;

        let component = inner.field_descriptors().clone().into_iter().next()?;
        let &drop_fn = component.as_ref();
        let id = component.into();

        let item = match inner.next()? {
            Ok(field) => {
                let component = unsafe { ErasedComponent::from_parts(id, field, drop_fn) };
                Ok(component)
            }
            Err(error) => match error {
                FromLayout(error) => Err(error),
                _ => unreachable!("failed to create erased data: {error}"),
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

impl<T> ExactSizeIterator for ErasedBundleIntoIterKind<T>
where
    T: ErasedArchetypeKind,
    T::IntoIter: ExactSizeIterator,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<T> FusedIterator for ErasedBundleIntoIterKind<T>
where
    T: ErasedArchetypeKind,
    T::IntoIter: FusedIterator,
{
}
