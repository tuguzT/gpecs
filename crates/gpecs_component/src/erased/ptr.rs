use core::{
    borrow::Borrow,
    cmp,
    hash::{self, Hash},
};

use gpecs_erased::{
    data::ErasedPtr,
    layout::WithLayout,
    ptr::slice::{ConstSliceItemPtr, MutPtr, NonNullPtr, SliceItemPtr},
};

use crate::{
    Component,
    erased::{
        ErasedComponentMutPtr, ErasedComponentNonNullPtr, ErasedComponentRef,
        error::{DanglingError, DowncastError, NotRegisteredError, UpcastPtrError, check_downcast},
    },
    registry::{
        ComponentId, ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType},
    },
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedComponentPtr<T> {
    component_id: ComponentId,
    erased: ErasedPtr<T>,
}

impl<T> ErasedComponentPtr<T> {
    #[inline]
    pub unsafe fn from_parts(component_id: ComponentId, erased: ErasedPtr<T>) -> Self {
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
    pub fn as_erased(self) -> ErasedPtr<T> {
        let Self { erased, .. } = self;
        erased
    }

    #[inline]
    pub fn into_parts(self) -> (ComponentId, ErasedPtr<T>) {
        let Self {
            component_id,
            erased,
        } = self;
        (component_id, erased)
    }
}

impl<T> ErasedComponentPtr<T>
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

        let origin = origin.as_erased();
        unsafe { erased.offset_from(origin) }
    }
}

impl<T> ErasedComponentPtr<T>
where
    T: ConstSliceItemPtr,
{
    #[inline]
    pub fn dangling(
        components: &ComponentRegistryView<impl WithLayout, impl ?Sized>,
        component_id: ComponentId,
    ) -> Result<Self, DanglingError> {
        let component_desc = components
            .get_component_descriptor(component_id)
            .ok_or_else(NotRegisteredError::new)?;

        let layout = component_desc.layout();
        let erased = ErasedPtr::dangling(layout)?;

        let me = unsafe { Self::from_parts(component_id, erased) };
        Ok(me)
    }

    #[inline]
    pub fn upcast<C, U>(
        registry: &ComponentRegistryView<impl Sized, U>,
        component: *const C,
    ) -> Result<Self, UpcastPtrError>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let component_id = registry
            .component_id::<C>()
            .ok_or_else(NotRegisteredError::of::<C>)?;
        let erased = ErasedPtr::upcast(component)?;

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
        components: &ComponentRegistryView<impl Sized, U>,
    ) -> Result<*const C, DowncastError<Self>>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let Self { component_id, .. } = self;
        let Self { erased, .. } = check_downcast::<C, U, _>(components, component_id, self)?;

        let ptr = erased.downcast().map_err(|err| err.map_value(|_| self))?;
        Ok(ptr)
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

    #[inline]
    pub fn cast_nonnull(self) -> Option<ErasedComponentNonNullPtr<NonNullPtr<T::Ptrs>>> {
        ErasedComponentNonNullPtr::from_ptr(self)
    }

    #[inline]
    pub unsafe fn cast_nonnull_unchecked(self) -> ErasedComponentNonNullPtr<NonNullPtr<T::Ptrs>> {
        unsafe { ErasedComponentNonNullPtr::from_ptr_unchecked(self) }
    }

    #[inline]
    pub unsafe fn as_ref_unchecked<'a>(self) -> ErasedComponentRef<'a, NonNullPtr<T::Ptrs>> {
        unsafe { self.cast_nonnull_unchecked().as_ref() }
    }

    #[inline]
    pub fn as_item_ptr(self) -> *const T::Item {
        let Self { erased, .. } = self;
        erased.as_item_ptr()
    }

    #[inline]
    pub fn as_items(self) -> *const [T::Item] {
        let Self { erased, .. } = self;
        erased.as_items()
    }
}

impl<T, U> PartialEq<ErasedComponentPtr<U>> for ErasedComponentPtr<T> {
    #[inline]
    fn eq(&self, other: &ErasedComponentPtr<U>) -> bool {
        let Self { component_id, .. } = self;
        component_id.eq(other.borrow())
    }
}

impl<T> Eq for ErasedComponentPtr<T> {}

impl<T, U> PartialOrd<ErasedComponentPtr<U>> for ErasedComponentPtr<T> {
    #[inline]
    fn partial_cmp(&self, other: &ErasedComponentPtr<U>) -> Option<cmp::Ordering> {
        let Self { component_id, .. } = self;
        component_id.partial_cmp(other.borrow())
    }
}

impl<T> Ord for ErasedComponentPtr<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { component_id, .. } = self;
        component_id.cmp(other.borrow())
    }
}

impl<T> Hash for ErasedComponentPtr<T> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { component_id, .. } = self;
        component_id.hash(state);
    }
}

impl<T> Borrow<ComponentId> for ErasedComponentPtr<T> {
    #[inline]
    fn borrow(&self) -> &ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }
}
