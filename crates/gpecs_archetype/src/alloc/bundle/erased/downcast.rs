use core::{
    error::Error,
    fmt::{self, Debug, Display},
    mem::MaybeUninit,
};

use gpecs_component::{
    erased::error::{
        ComponentMismatchError, DowncastErrorKind as ComponentDowncastErrorKind, NotRegisteredError,
    },
    registry::{
        ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType},
    },
};

use gpecs_soa_erased::{
    error::LayoutMismatchError,
    ptr::slice::{ConstSliceItemPtr, MutSliceItemPtr, NonNullSliceItemPtr, SliceItemPtrs},
    soa::traits::{RawSoaContext, ReadSoaContext, SoaContext},
    storage::AlignedStorage,
};

use crate::{
    bundle::{
        Bundle, BundleMutPtrs, BundleNonNullPtrs, BundlePtrs, BundleRefs, BundleRefsMut,
        BundleSliceMutPtrs, BundleSlicePtrs, BundleSlices, BundleSlicesMut,
        erased::{
            ErasedBundleKind, ErasedBundleMutPtrs, ErasedBundleMutRefs, ErasedBundleMutSlicePtrs,
            ErasedBundleMutSlices, ErasedBundleNonNullPtrs, ErasedBundlePtrs, ErasedBundleRefs,
            ErasedBundleSlicePtrs, ErasedBundleSlices,
            traits::{ErasedArchetypeKind, ErasedBundleDrop},
        },
    },
    erased::error::{DuplicateComponentError, IncompatibleArchetypeError, MissingComponentError},
};

impl<D, P> ErasedBundleMutPtrs<D, P>
where
    D: ErasedArchetypeKind,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn downcast<B, T>(
        mut self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleMutPtrs<B>, DowncastError<Self>>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        if let Err(error) = self.archetype().check_compatibility_of::<B, T>(components) {
            return Err(DowncastError::new(self, error.into()));
        }
        let ptrs = B::mut_ptrs_from_erased(components, self.iter_mut())
            .map_err(|error| DowncastError::new(self, error.into()))?;
        Ok(ptrs)
    }
}

impl<'a, D, P> ErasedBundleMutRefs<'a, D, P>
where
    D: ErasedArchetypeKind,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleRefsMut<'a, B>, DowncastError<Self>>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs) };
        let ptrs = self
            .into_ptrs()
            .downcast::<B, T>(components)
            .map_err(|error| error.map_value(into_self))?;

        let refs = unsafe { B::CONTEXT.mut_ptrs_to_mut_refs(ptrs) };
        Ok(refs)
    }
}

impl<D, P> ErasedBundleMutSlicePtrs<D, P>
where
    D: ErasedArchetypeKind,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleSliceMutPtrs<B>, DowncastError<Self>>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let len = self.len();
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs, len) };
        let ptrs = self
            .into_ptrs()
            .downcast::<B, T>(components)
            .map_err(|error| error.map_value(into_self))?;

        let slices = B::CONTEXT.mut_slice_ptrs_from_raw_parts(ptrs, len);
        Ok(slices)
    }
}

impl<'a, D, P> ErasedBundleMutSlices<'a, D, P>
where
    D: ErasedArchetypeKind,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleSlicesMut<'a, B>, DowncastError<Self>>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs) };
        let slices = self
            .into_ptrs()
            .downcast::<B, T>(components)
            .map_err(|error| error.map_value(into_self))?;

        let slices = unsafe { B::CONTEXT.mut_slice_ptrs_to_mut_slices(slices) };
        Ok(slices)
    }
}

impl<D, P> ErasedBundleNonNullPtrs<D, P>
where
    D: ErasedArchetypeKind,
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleNonNullPtrs<B>, DowncastError<Self>>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let into_self = |ptrs| unsafe { Self::new_unchecked(ptrs) };
        let ptrs = ErasedBundleMutPtrs::from(self)
            .downcast::<B, T>(components)
            .map_err(|error| error.map_value(into_self))?;

        let ptrs = unsafe { B::CONTEXT.ptrs_to_nonnull(ptrs) };
        Ok(ptrs)
    }
}

impl<D, P> ErasedBundlePtrs<D, P>
where
    D: ErasedArchetypeKind,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundlePtrs<B>, DowncastError<Self>>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        if let Err(error) = self.archetype().check_compatibility_of::<B, T>(components) {
            return Err(DowncastError::new(self, error.into()));
        }

        let ptrs = B::ptrs_from_erased(components, self.iter())
            .map_err(|error| DowncastError::new(self, error.into()))?;
        Ok(ptrs)
    }
}

impl<'a, D, P> ErasedBundleRefs<'a, D, P>
where
    D: ErasedArchetypeKind,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleRefs<'a, B>, DowncastError<Self>>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs) };
        let ptrs = self
            .into_ptrs()
            .downcast::<B, T>(components)
            .map_err(|error| error.map_value(into_self))?;

        let refs = unsafe { B::CONTEXT.ptrs_to_refs(ptrs) };
        Ok(refs)
    }
}

impl<D, P> ErasedBundleSlicePtrs<D, P>
where
    D: ErasedArchetypeKind,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleSlicePtrs<B>, DowncastError<Self>>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let len = self.len();
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs, len) };
        let ptrs = self
            .into_ptrs()
            .downcast::<B, T>(components)
            .map_err(|error| error.map_value(into_self))?;

        let slices = B::CONTEXT.slice_ptrs_from_raw_parts(ptrs, len);
        Ok(slices)
    }
}

impl<'a, D, P> ErasedBundleSlices<'a, D, P>
where
    D: ErasedArchetypeKind,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleSlices<'a, B>, DowncastError<Self>>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs) };
        let slices = self
            .into_ptrs()
            .downcast::<B, T>(components)
            .map_err(|error| error.map_value(into_self))?;

        let slices = unsafe { B::CONTEXT.slice_ptrs_to_slices(slices) };
        Ok(slices)
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
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DowncastError<T>
where
    T: ?Sized,
{
    pub source: DowncastErrorKind,
    pub value: T,
}

impl<T> DowncastError<T> {
    #[inline]
    fn new(value: T, source: DowncastErrorKind) -> Self {
        Self { source, value }
    }

    #[inline]
    pub fn map_value<U, F>(self, f: F) -> DowncastError<U>
    where
        F: FnOnce(T) -> U,
    {
        let Self { source, value } = self;
        DowncastError::new(f(value), source)
    }
}

impl<T> From<DowncastError<T>> for DowncastErrorKind {
    #[inline]
    fn from(error: DowncastError<T>) -> Self {
        let DowncastError { source, .. } = error;
        source
    }
}

impl<T> Display for DowncastError<T>
where
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { source, value } = self;
        write!(f, "failed to downcast {value} into bundle: {source}")
    }
}

impl<T> Error for DowncastError<T>
where
    T: Debug + Display + ?Sized,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { source, .. } = self;
        Some(source)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DowncastErrorKind {
    DuplicateComponent(DuplicateComponentError),
    MissingComponent(MissingComponentError),
    ComponentNotRegistered(NotRegisteredError),
    ComponentMismatch(ComponentMismatchError),
    LayoutMismatch(LayoutMismatchError),
}

impl From<DuplicateComponentError> for DowncastErrorKind {
    #[inline]
    fn from(error: DuplicateComponentError) -> Self {
        Self::DuplicateComponent(error)
    }
}

impl From<MissingComponentError> for DowncastErrorKind {
    #[inline]
    fn from(error: MissingComponentError) -> Self {
        Self::MissingComponent(error)
    }
}

impl From<NotRegisteredError> for DowncastErrorKind {
    #[inline]
    fn from(error: NotRegisteredError) -> Self {
        Self::ComponentNotRegistered(error)
    }
}

impl From<ComponentMismatchError> for DowncastErrorKind {
    #[inline]
    fn from(error: ComponentMismatchError) -> Self {
        Self::ComponentMismatch(error)
    }
}

impl From<LayoutMismatchError> for DowncastErrorKind {
    #[inline]
    fn from(error: LayoutMismatchError) -> Self {
        Self::LayoutMismatch(error)
    }
}

impl From<IncompatibleArchetypeError> for DowncastErrorKind {
    #[inline]
    fn from(error: IncompatibleArchetypeError) -> Self {
        use IncompatibleArchetypeError::{
            ComponentNotRegistered, DuplicateComponent, MissingComponent,
        };

        match error {
            DuplicateComponent(error) => Self::DuplicateComponent(error),
            MissingComponent(error) => Self::MissingComponent(error),
            ComponentNotRegistered(error) => Self::ComponentNotRegistered(error),
        }
    }
}

impl From<ComponentDowncastErrorKind> for DowncastErrorKind {
    #[inline]
    fn from(error: ComponentDowncastErrorKind) -> Self {
        use ComponentDowncastErrorKind::{
            ComponentMismatch, ComponentNotRegistered, LayoutMismatch,
        };

        match error {
            ComponentNotRegistered(error) => Self::ComponentNotRegistered(error),
            ComponentMismatch(error) => Self::ComponentMismatch(error),
            LayoutMismatch(error) => Self::LayoutMismatch(error),
        }
    }
}

impl Display for DowncastErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateComponent(error) => Display::fmt(error, f),
            Self::MissingComponent(error) => Display::fmt(error, f),
            Self::ComponentNotRegistered(error) => Display::fmt(error, f),
            Self::ComponentMismatch(error) => Display::fmt(error, f),
            Self::LayoutMismatch(error) => Display::fmt(error, f),
        }
    }
}

impl Error for DowncastErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::DuplicateComponent(error) => Some(error),
            Self::MissingComponent(error) => Some(error),
            Self::ComponentNotRegistered(error) => Some(error),
            Self::ComponentMismatch(error) => Some(error),
            Self::LayoutMismatch(error) => Some(error),
        }
    }
}
