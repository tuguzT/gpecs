use core::{
    borrow::Borrow,
    cmp,
    hash::{self, Hash},
    ptr::NonNull,
};

use gpecs_erased::{
    data::ErasedNonNullPtr,
    layout::WithLayout,
    ptr::slice::{ConstPtr, MutPtr, NonNullSliceItemPtr, SliceItemPtr},
};

use crate::{
    Component,
    erased::{
        ErasedComponentMutPtr, ErasedComponentPtr,
        error::{DanglingError, DowncastError, NotRegisteredError, UpcastPtrError, check_downcast},
    },
    registry::{
        ComponentId, ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType},
    },
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedComponentNonNullPtr<T> {
    component_id: ComponentId,
    erased: ErasedNonNullPtr<T>,
}

impl<T> ErasedComponentNonNullPtr<T> {
    #[inline]
    pub unsafe fn from_parts(component_id: ComponentId, erased: ErasedNonNullPtr<T>) -> Self {
        Self {
            component_id,
            erased,
        }
    }

    #[inline]
    pub fn component_id(self) -> ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }

    #[inline]
    pub fn erased(self) -> ErasedNonNullPtr<T> {
        let Self { erased, .. } = self;
        erased
    }

    #[inline]
    pub fn into_parts(self) -> (ComponentId, ErasedNonNullPtr<T>) {
        let Self {
            component_id,
            erased,
        } = self;
        (component_id, erased)
    }
}

impl<T> ErasedComponentNonNullPtr<T>
where
    T: SliceItemPtr,
{
    #[inline]
    #[must_use]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self {
            component_id,
            erased,
        } = self;

        let erased = unsafe { erased.add(count) };
        unsafe { Self::from_parts(component_id, erased) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn offset_from(self, origin: Self) -> isize {
        let Self { erased, .. } = self;

        let origin = origin.erased();
        unsafe { erased.offset_from(origin) }
    }
}

impl<T> ErasedComponentNonNullPtr<T>
where
    T: NonNullSliceItemPtr,
{
    #[inline]
    pub fn from_ptr(ptr: ErasedComponentPtr<ConstPtr<T::Ptrs>>) -> Option<Self> {
        let (component_id, erased) = ptr.into_parts();
        let erased = ErasedNonNullPtr::from_ptr(erased)?;

        let me = unsafe { Self::from_parts(component_id, erased) };
        Some(me)
    }

    #[inline]
    pub unsafe fn from_ptr_unchecked(ptr: ErasedComponentPtr<ConstPtr<T::Ptrs>>) -> Self {
        let (component_id, erased) = ptr.into_parts();
        let erased = unsafe { ErasedNonNullPtr::from_ptr_unchecked(erased) };

        unsafe { Self::from_parts(component_id, erased) }
    }

    #[inline]
    pub fn from_mut_ptr(ptr: ErasedComponentMutPtr<MutPtr<T::Ptrs>>) -> Option<Self> {
        let (component_id, erased) = ptr.into_parts();
        let erased = ErasedNonNullPtr::from_mut_ptr(erased)?;

        let me = unsafe { Self::from_parts(component_id, erased) };
        Some(me)
    }

    #[inline]
    pub unsafe fn from_mut_ptr_unchecked(ptr: ErasedComponentMutPtr<MutPtr<T::Ptrs>>) -> Self {
        let (component_id, erased) = ptr.into_parts();
        let erased = unsafe { ErasedNonNullPtr::from_mut_ptr_unchecked(erased) };

        unsafe { Self::from_parts(component_id, erased) }
    }

    #[inline]
    pub fn dangling(
        components: &ComponentRegistryView<impl WithLayout, impl ?Sized>,
        component_id: ComponentId,
    ) -> Result<Self, DanglingError> {
        let component_desc = components
            .get_component_descriptor(component_id)
            .ok_or_else(NotRegisteredError::new)?;

        let layout = component_desc.layout();
        let erased = ErasedNonNullPtr::dangling(layout)?;

        let me = unsafe { Self::from_parts(component_id, erased) };
        Ok(me)
    }

    #[inline]
    pub fn upcast<C, U>(
        registry: &ComponentRegistryView<impl Sized, U>,
        component: NonNull<C>,
    ) -> Result<Self, UpcastPtrError>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let component_id = registry
            .component_id::<C>()
            .ok_or_else(NotRegisteredError::of::<C>)?;
        let erased = ErasedNonNullPtr::upcast(component)?;

        let me = unsafe { Self::from_parts(component_id, erased) };
        Ok(me)
    }

    #[inline]
    pub fn dangling_of<C, M, U>(
        components: &ComponentRegistryView<M, U>,
    ) -> Result<Self, DanglingError>
    where
        C: Component,
        M: WithLayout,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let component_id = components
            .component_id::<C>()
            .ok_or_else(NotRegisteredError::of::<C>)?;

        let me = Self::dangling(components, component_id)?;
        Ok(me)
    }

    #[inline]
    pub fn downcast<C, U>(
        self,
        registry: &ComponentRegistryView<impl Sized, U>,
    ) -> Result<NonNull<C>, DowncastError<Self>>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let Self { component_id, .. } = self;
        let Self { erased, .. } = check_downcast::<C, U, _>(registry, component_id, self)?;

        let ptr = erased.downcast().map_err(|err| err.map_value(|_| self))?;
        Ok(ptr)
    }

    #[inline]
    pub unsafe fn swap(self, with: Self) {
        let Self { erased, .. } = self;

        let with = with.erased();
        unsafe { erased.swap(with) }
    }

    #[inline]
    pub unsafe fn copy_from(self, src: Self, count: usize) {
        let Self { erased, .. } = self;

        let src = src.erased();
        unsafe { erased.copy_from(src, count) }
    }

    #[inline]
    pub unsafe fn copy_from_nonoverlapping(self, src: Self, count: usize) {
        let Self { erased, .. } = self;

        let src = src.erased();
        unsafe { erased.copy_from_nonoverlapping(src, count) }
    }

    #[inline]
    pub fn as_ptr(self) -> NonNull<T::Item> {
        let Self { erased, .. } = self;
        erased.as_ptr()
    }

    #[inline]
    pub fn cast_const(self) -> ErasedComponentPtr<ConstPtr<T::Ptrs>> {
        let Self {
            component_id,
            erased,
        } = self;

        let erased = erased.cast_const();
        unsafe { ErasedComponentPtr::from_parts(component_id, erased) }
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedComponentMutPtr<MutPtr<T::Ptrs>> {
        let Self {
            component_id,
            erased,
        } = self;

        let erased = erased.cast_mut();
        unsafe { ErasedComponentMutPtr::from_parts(component_id, erased) }
    }
}

impl<T, U> PartialEq<ErasedComponentNonNullPtr<U>> for ErasedComponentNonNullPtr<T> {
    #[inline]
    fn eq(&self, other: &ErasedComponentNonNullPtr<U>) -> bool {
        let Self { component_id, .. } = self;
        component_id.eq(other.borrow())
    }
}

impl<T> Eq for ErasedComponentNonNullPtr<T> {}

impl<T, U> PartialOrd<ErasedComponentNonNullPtr<U>> for ErasedComponentNonNullPtr<T> {
    #[inline]
    fn partial_cmp(&self, other: &ErasedComponentNonNullPtr<U>) -> Option<cmp::Ordering> {
        let Self { component_id, .. } = self;
        component_id.partial_cmp(other.borrow())
    }
}

impl<T> Ord for ErasedComponentNonNullPtr<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { component_id, .. } = self;
        component_id.cmp(other.borrow())
    }
}

impl<T> Hash for ErasedComponentNonNullPtr<T> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { component_id, .. } = self;
        component_id.hash(state);
    }
}

impl<T> Borrow<ComponentId> for ErasedComponentNonNullPtr<T> {
    #[inline]
    fn borrow(&self) -> &ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }
}
