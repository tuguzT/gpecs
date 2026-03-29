use core::{
    borrow::Borrow,
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
};

use gpecs_erased::{data::ErasedRef, ptr::slice::ConstSliceItemPtr};

use crate::{
    Component,
    erased::{
        ErasedComponentPtr,
        error::{DowncastError, NotRegisteredError, TryFromPtrError, check_downcast},
    },
    registry::{
        ComponentId, ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType},
    },
};

#[derive(Clone, Copy)]
pub struct ErasedComponentRef<'a, T>
where
    T: ConstSliceItemPtr,
{
    component_id: ComponentId,
    field: ErasedRef<'a, T>,
}

impl<'a, T> ErasedComponentRef<'a, T>
where
    T: ConstSliceItemPtr,
{
    #[inline]
    pub fn try_from<C, U>(
        components: &ComponentRegistryView<impl Sized, U>,
        component: &'a C,
    ) -> Result<Self, TryFromPtrError>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let component_id = components
            .component_id::<C>()
            .ok_or_else(NotRegisteredError::of::<C>)?;
        let field = ErasedRef::try_from(component)?;

        let me = unsafe { Self::from_parts(component_id, field) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_parts(component_id: ComponentId, field: ErasedRef<'a, T>) -> Self {
        Self {
            component_id,
            field,
        }
    }

    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedComponentPtr<T>) -> Self {
        let (component_id, field) = ptr.into_parts();
        let field = unsafe { ErasedRef::from_ptr(field) };
        unsafe { Self::from_parts(component_id, field) }
    }

    #[inline]
    pub fn downcast<C, U>(
        self,
        components: &ComponentRegistryView<impl Sized, U>,
    ) -> Result<&'a C, DowncastError<Self>>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let Self { component_id, .. } = self;
        let Self { field, .. } = check_downcast::<C, U, _>(components, component_id, self)?;

        let into_self = |field| unsafe { Self::from_parts(component_id, field) };
        let component = unsafe { field.downcast() }.map_err(|err| err.map_value(into_self))?;
        Ok(component)
    }

    #[inline]
    pub fn downcast_ref<C, U>(
        &self,
        components: &ComponentRegistryView<impl Sized, U>,
    ) -> Result<&C, DowncastError<&Self>>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let Self { component_id, .. } = *self;
        let Self { field, .. } = check_downcast::<C, U, _>(components, component_id, self)?;

        let component = unsafe { field.downcast_ref() }.map_err(|err| err.map_value(|_| self))?;
        Ok(component)
    }

    #[inline]
    pub fn component_id(&self) -> ComponentId {
        let Self { component_id, .. } = *self;
        component_id
    }

    #[inline]
    pub fn field(&self) -> &ErasedRef<'a, T> {
        let Self { field, .. } = self;
        field
    }

    #[inline]
    pub fn as_component_ptr(&self) -> ErasedComponentPtr<T> {
        let Self {
            ref field,
            component_id,
        } = *self;

        let field = field.as_field_ptr();
        unsafe { ErasedComponentPtr::from_parts(component_id, field) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T::Item {
        let Self { field, .. } = self;
        field.as_ptr()
    }

    #[inline]
    pub fn as_buffer(&self) -> &[T::Item] {
        let Self { field, .. } = self;
        field.as_buffer()
    }

    #[inline]
    pub fn into_parts(self) -> (ComponentId, ErasedRef<'a, T>) {
        let Self {
            component_id,
            field,
        } = self;
        (component_id, field)
    }
}

impl<T> Debug for ErasedComponentRef<'_, T>
where
    T: ConstSliceItemPtr<Item: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            component_id,
            field,
        } = self;

        f.debug_struct("ErasedComponentRef")
            .field("component_id", component_id)
            .field("field", field)
            .finish()
    }
}

impl<T, U> PartialEq<ErasedComponentRef<'_, U>> for ErasedComponentRef<'_, T>
where
    T: ConstSliceItemPtr,
    U: ConstSliceItemPtr,
{
    #[inline]
    fn eq(&self, other: &ErasedComponentRef<'_, U>) -> bool {
        let Self { component_id, .. } = self;
        component_id.eq(other.borrow())
    }
}

impl<T> Eq for ErasedComponentRef<'_, T> where T: ConstSliceItemPtr {}

impl<T, U> PartialOrd<ErasedComponentRef<'_, U>> for ErasedComponentRef<'_, T>
where
    T: ConstSliceItemPtr,
    U: ConstSliceItemPtr,
{
    #[inline]
    fn partial_cmp(&self, other: &ErasedComponentRef<'_, U>) -> Option<cmp::Ordering> {
        let Self { component_id, .. } = self;
        component_id.partial_cmp(other.borrow())
    }
}

impl<T> Ord for ErasedComponentRef<'_, T>
where
    T: ConstSliceItemPtr,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { component_id, .. } = self;
        component_id.cmp(other.borrow())
    }
}

impl<T> Hash for ErasedComponentRef<'_, T>
where
    T: ConstSliceItemPtr,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { component_id, .. } = self;
        component_id.hash(state);
    }
}

impl<T> Borrow<ComponentId> for ErasedComponentRef<'_, T>
where
    T: ConstSliceItemPtr,
{
    #[inline]
    fn borrow(&self) -> &ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }
}

impl<T> AsRef<[T::Item]> for ErasedComponentRef<'_, T>
where
    T: ConstSliceItemPtr,
{
    #[inline]
    fn as_ref(&self) -> &[T::Item] {
        self.as_buffer()
    }
}
