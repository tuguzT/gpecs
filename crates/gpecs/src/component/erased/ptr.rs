use std::{
    borrow::Borrow,
    cmp,
    hash::{self, Hash},
    mem::MaybeUninit,
};

use gpecs_soa_erased::{
    data::ErasedPtr,
    ptr::slice::{CastMutPtr, ConstSliceItemPtr},
};

use crate::{
    component::{
        Component,
        erased::{
            ErasedComponentMutPtr, ErasedComponentRef,
            error::{
                DanglingError, DowncastError, NotRegisteredError, TryFromPtrError, check_downcast,
            },
        },
        registry::{
            ComponentId, ComponentRegistryView,
            traits::{ComponentIdFrom, FromComponentType},
        },
    },
    soa::field::FieldDescriptor,
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedComponentPtr<T = *const MaybeUninit<u8>> {
    component_id: ComponentId,
    field: ErasedPtr<T>,
}

impl<T> ErasedComponentPtr<T> {
    #[inline]
    pub unsafe fn from_parts(component_id: ComponentId, field: ErasedPtr<T>) -> Self {
        Self {
            component_id,
            field,
        }
    }

    #[inline]
    pub fn component_id(self) -> ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }

    #[inline]
    pub fn field(self) -> ErasedPtr<T> {
        let Self { field, .. } = self;
        field
    }

    #[inline]
    pub fn into_parts(self) -> (ComponentId, ErasedPtr<T>) {
        let Self {
            component_id,
            field,
        } = self;
        (component_id, field)
    }
}

impl<T> ErasedComponentPtr<T>
where
    T: ConstSliceItemPtr,
{
    #[inline]
    pub fn dangling(
        components: &ComponentRegistryView<impl AsRef<FieldDescriptor>, impl ?Sized>,
        component_id: ComponentId,
    ) -> Result<Self, DanglingError> {
        let component_info = components
            .get_component_info(component_id)
            .ok_or(NotRegisteredError)?;

        let layout = component_info.as_meta().as_ref().layout();
        let field = ErasedPtr::dangling(layout)?;

        let me = unsafe { Self::from_parts(component_id, field) };
        Ok(me)
    }

    #[inline]
    pub fn try_from<C, U>(
        registry: &ComponentRegistryView<impl Sized, U>,
        component: *const C,
    ) -> Result<Self, TryFromPtrError>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let component_id = registry.component_id::<C>().ok_or(NotRegisteredError)?;
        let field = ErasedPtr::try_from(component)?;

        let me = unsafe { Self::from_parts(component_id, field) };
        Ok(me)
    }

    #[inline]
    pub fn dangling_of<C, M, U>(
        components: &ComponentRegistryView<M, U>,
    ) -> Result<Self, DanglingError>
    where
        C: Component,
        M: AsRef<FieldDescriptor>,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let component_id = components.component_id::<C>().ok_or(NotRegisteredError)?;

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
        let Self { field, .. } = check_downcast::<C, U, _>(components, component_id, self)?;

        let component = field.downcast().map_err(|err| err.map_value(|_| self))?;
        Ok(component)
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedComponentMutPtr<CastMutPtr<T>> {
        let Self {
            component_id,
            field,
        } = self;

        let field = field.cast_mut();
        unsafe { ErasedComponentMutPtr::from_parts(component_id, field) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedComponentRef<'a, T> {
        unsafe { ErasedComponentRef::from_ptr(self) }
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self {
            component_id,
            field,
        } = self;

        let ptr = unsafe { field.add(count) };
        unsafe { Self::from_parts(component_id, ptr) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn offset_from(self, origin: Self) -> isize {
        let Self { field, .. } = self;

        let origin = origin.field();
        unsafe { field.offset_from(origin) }
    }

    #[inline]
    pub fn as_ptr(self) -> *const T::Item {
        let Self { field, .. } = self;
        field.as_ptr()
    }

    #[inline]
    pub fn as_buffer(self) -> *const [T::Item] {
        let Self { field, .. } = self;
        field.as_buffer()
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
