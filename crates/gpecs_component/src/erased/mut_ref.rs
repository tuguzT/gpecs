use core::{
    borrow::Borrow,
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
};

use gpecs_erased::{
    data::ErasedMutRef,
    ptr::slice::{CastConst, MutSliceItemPtr},
};
use polonius_the_crab::{polonius, polonius_return};

use crate::{
    Component,
    erased::{
        ErasedComponentMutPtr, ErasedComponentPtr, ErasedComponentRef,
        error::{DowncastError, NotRegisteredError, TryFromPtrError, check_downcast},
    },
    registry::{
        ComponentId, ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType},
    },
};

pub struct ErasedComponentMutRef<'a, T>
where
    T: MutSliceItemPtr,
{
    component_id: ComponentId,
    field: ErasedMutRef<'a, T>,
}

impl<'a, T> ErasedComponentMutRef<'a, T>
where
    T: MutSliceItemPtr,
{
    #[inline]
    pub fn try_from<C, U>(
        components: &ComponentRegistryView<impl Sized, U>,
        component: &'a mut C,
    ) -> Result<Self, TryFromPtrError>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let component_id = components
            .component_id::<C>()
            .ok_or_else(NotRegisteredError::of::<C>)?;
        let field = ErasedMutRef::try_from(component)?;

        let me = unsafe { Self::from_parts(component_id, field) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_parts(component_id: ComponentId, field: ErasedMutRef<'a, T>) -> Self {
        Self {
            component_id,
            field,
        }
    }

    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedComponentMutPtr<T>) -> Self {
        let (component_id, field) = ptr.into_parts();
        let field = unsafe { ErasedMutRef::from_ptr(field) };
        unsafe { Self::from_parts(component_id, field) }
    }

    #[inline]
    pub fn downcast<C, U>(
        self,
        components: &ComponentRegistryView<impl Sized, U>,
    ) -> Result<&'a mut C, DowncastError<Self>>
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
    pub fn downcast_mut<C, U>(
        &mut self,
        components: &ComponentRegistryView<impl Sized, U>,
    ) -> Result<&mut C, DowncastError<&mut Self>>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let Self { component_id, .. } = *self;
        let mut this = check_downcast::<C, U, _>(components, component_id, self)?;

        let source = polonius!(|this| -> Result<&'polonius mut C, _> {
            match unsafe { this.field.downcast_mut() } {
                Ok(component) => polonius_return!(Ok(component)),
                Err(error) => error.source.into(),
            }
        });
        Err(DowncastError::new(this, source))
    }

    #[inline]
    pub fn component_id(&self) -> ComponentId {
        let Self { component_id, .. } = *self;
        component_id
    }

    #[inline]
    pub fn field(&self) -> &ErasedMutRef<'a, T> {
        let Self { field, .. } = self;
        field
    }

    #[inline]
    pub fn as_component_ptr(&self) -> ErasedComponentPtr<CastConst<T>> {
        let Self {
            ref field,
            component_id,
        } = *self;

        let field = field.as_field_ptr();
        unsafe { ErasedComponentPtr::from_parts(component_id, field) }
    }

    #[inline]
    pub fn as_mut_component_ptr(&mut self) -> ErasedComponentMutPtr<T> {
        let Self {
            ref mut field,
            component_id,
        } = *self;

        let field = field.as_mut_field_ptr();
        unsafe { ErasedComponentMutPtr::from_parts(component_id, field) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T::Item {
        let Self { field, .. } = self;
        field.as_ptr()
    }

    #[inline]
    pub unsafe fn as_mut_ptr(&mut self) -> *mut T::Item {
        let Self { field, .. } = self;
        field.as_mut_ptr()
    }

    #[inline]
    pub fn as_buffer(&self) -> &[T::Item] {
        let Self { field, .. } = self;
        field.as_buffer()
    }

    #[inline]
    pub unsafe fn as_mut_buffer(&mut self) -> &mut [T::Item] {
        let Self { field, .. } = self;
        field.as_mut_buffer()
    }

    #[inline]
    pub fn into_parts(self) -> (ComponentId, ErasedMutRef<'a, T>) {
        let Self {
            component_id,
            field,
        } = self;
        (component_id, field)
    }
}

impl<T> Debug for ErasedComponentMutRef<'_, T>
where
    T: MutSliceItemPtr<Item: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            component_id,
            field,
        } = self;

        f.debug_struct("ErasedComponentMutRef")
            .field("component_id", component_id)
            .field("field", field)
            .finish()
    }
}

impl<T, U> PartialEq<ErasedComponentMutRef<'_, U>> for ErasedComponentMutRef<'_, T>
where
    T: MutSliceItemPtr,
    U: MutSliceItemPtr,
{
    #[inline]
    fn eq(&self, other: &ErasedComponentMutRef<'_, U>) -> bool {
        let Self { component_id, .. } = self;
        component_id.eq(other.borrow())
    }
}

impl<T> Eq for ErasedComponentMutRef<'_, T> where T: MutSliceItemPtr {}

impl<T, U> PartialOrd<ErasedComponentMutRef<'_, U>> for ErasedComponentMutRef<'_, T>
where
    T: MutSliceItemPtr,
    U: MutSliceItemPtr,
{
    #[inline]
    fn partial_cmp(&self, other: &ErasedComponentMutRef<'_, U>) -> Option<cmp::Ordering> {
        let Self { component_id, .. } = self;
        component_id.partial_cmp(other.borrow())
    }
}

impl<T> Ord for ErasedComponentMutRef<'_, T>
where
    T: MutSliceItemPtr,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { component_id, .. } = self;
        component_id.cmp(other.borrow())
    }
}

impl<T> Hash for ErasedComponentMutRef<'_, T>
where
    T: MutSliceItemPtr,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { component_id, .. } = self;
        component_id.hash(state);
    }
}

impl<T> Borrow<ComponentId> for ErasedComponentMutRef<'_, T>
where
    T: MutSliceItemPtr,
{
    #[inline]
    fn borrow(&self) -> &ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }
}

impl<T> AsRef<[T::Item]> for ErasedComponentMutRef<'_, T>
where
    T: MutSliceItemPtr,
{
    #[inline]
    fn as_ref(&self) -> &[T::Item] {
        self.as_buffer()
    }
}

impl<'a, T> From<ErasedComponentMutRef<'a, T>> for ErasedComponentRef<'a, CastConst<T>>
where
    T: MutSliceItemPtr,
{
    #[inline]
    fn from(r#ref: ErasedComponentMutRef<'a, T>) -> Self {
        let (component_id, field) = r#ref.into_parts();
        let field = field.into();
        unsafe { Self::from_parts(component_id, field) }
    }
}
