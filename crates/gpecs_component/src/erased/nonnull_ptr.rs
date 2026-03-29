use core::{
    borrow::Borrow,
    cmp,
    hash::{self, Hash},
    ptr::NonNull,
};

use gpecs_erased::{
    data::ErasedNonNullPtr,
    layout::WithLayout,
    ptr::slice::{NonNullAsPtr, NonNullSliceItemPtr},
};

use crate::{
    Component,
    erased::{
        ErasedComponentMutPtr,
        error::{
            DanglingError, DowncastError, NotRegisteredError, TryFromPtrError, check_downcast,
        },
    },
    registry::{
        ComponentId, ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType},
    },
};

type Field<T> = ErasedNonNullPtr<T>;

#[derive(Debug, Clone, Copy)]
pub struct ErasedComponentNonNullPtr<T> {
    component_id: ComponentId,
    field: Field<T>,
}

impl<T> ErasedComponentNonNullPtr<T> {
    #[inline]
    pub unsafe fn from_parts(component_id: ComponentId, field: Field<T>) -> Self {
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
    pub fn field(self) -> Field<T> {
        let Self { field, .. } = self;
        field
    }

    #[inline]
    pub fn into_parts(self) -> (ComponentId, Field<T>) {
        let Self {
            component_id,
            field,
        } = self;
        (component_id, field)
    }
}

impl<T> ErasedComponentNonNullPtr<T>
where
    T: NonNullSliceItemPtr,
{
    #[inline]
    pub fn new(ptr: ErasedComponentMutPtr<NonNullAsPtr<T>>) -> Option<Self> {
        let (component_id, field) = ptr.into_parts();
        let field = Field::new(field)?;

        let me = unsafe { Self::from_parts(component_id, field) };
        Some(me)
    }

    #[inline]
    pub unsafe fn new_unchecked(ptr: ErasedComponentMutPtr<NonNullAsPtr<T>>) -> Self {
        let (component_id, field) = ptr.into_parts();
        let field = unsafe { Field::new_unchecked(field) };

        unsafe { Self::from_parts(component_id, field) }
    }

    #[inline]
    pub fn dangling(
        components: &ComponentRegistryView<impl WithLayout, impl ?Sized>,
        component_id: ComponentId,
    ) -> Result<Self, DanglingError> {
        let component_info = components
            .get_component_info(component_id)
            .ok_or_else(NotRegisteredError::new)?;

        let layout = component_info.as_meta().layout();
        let field = Field::dangling(layout)?;

        let me = unsafe { Self::from_parts(component_id, field) };
        Ok(me)
    }

    #[inline]
    pub fn try_from<C, U>(
        registry: &ComponentRegistryView<impl Sized, U>,
        component: NonNull<C>,
    ) -> Result<Self, TryFromPtrError>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let component_id = registry
            .component_id::<C>()
            .ok_or_else(NotRegisteredError::of::<C>)?;
        let field = Field::try_from(component)?;

        let me = unsafe { Self::from_parts(component_id, field) };
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
        let Self { field, .. } = check_downcast::<C, U, _>(registry, component_id, self)?;

        let component = field.downcast().map_err(|err| err.map_value(|_| self))?;
        Ok(component)
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self {
            component_id,
            field,
        } = self;

        let field = unsafe { field.add(count) };
        unsafe { Self::from_parts(component_id, field) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn offset_from(self, origin: Self) -> isize {
        let Self { field, .. } = self;

        let origin = origin.field();
        unsafe { field.offset_from(origin) }
    }

    #[inline]
    pub unsafe fn swap(self, with: Self) {
        let Self { field, .. } = self;

        let with = with.field();
        unsafe { field.swap(with) }
    }

    #[inline]
    pub unsafe fn copy_from(self, src: Self, count: usize) {
        let Self { field, .. } = self;

        let src = src.field();
        unsafe { field.copy_from(src, count) }
    }

    #[inline]
    pub unsafe fn copy_from_nonoverlapping(self, src: Self, count: usize) {
        let Self { field, .. } = self;

        let src = src.field();
        unsafe { field.copy_from_nonoverlapping(src, count) }
    }

    #[inline]
    pub fn as_ptr(self) -> NonNull<T::Item> {
        let Self { field, .. } = self;
        field.as_ptr()
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

impl<T> From<ErasedComponentNonNullPtr<T>> for ErasedComponentMutPtr<NonNullAsPtr<T>>
where
    T: NonNullSliceItemPtr,
{
    #[inline]
    fn from(ptr: ErasedComponentNonNullPtr<T>) -> Self {
        let (component_id, field) = ptr.into_parts();
        let field = field.into();
        unsafe { Self::from_parts(component_id, field) }
    }
}
