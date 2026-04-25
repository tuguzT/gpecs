use core::{
    borrow::Borrow,
    cmp,
    hash::{self, Hash},
};

use gpecs_erased::{
    data::ErasedMutPtr,
    layout::WithLayout,
    ptr::slice::{CastConst, MutSliceItemPtr},
};

use crate::{
    Component,
    erased::{
        ErasedComponentMutRef, ErasedComponentPtr, ErasedComponentRef, WithErasedDrop,
        error::{
            DanglingError, DowncastError, NotRegisteredError, TryFromPtrError, check_downcast,
        },
    },
    registry::{
        ComponentId, ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType},
    },
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedComponentMutPtr<T> {
    component_id: ComponentId,
    field: ErasedMutPtr<T>,
}

impl<T> ErasedComponentMutPtr<T> {
    #[inline]
    pub unsafe fn from_parts(component_id: ComponentId, field: ErasedMutPtr<T>) -> Self {
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
    pub fn field(self) -> ErasedMutPtr<T> {
        let Self { field, .. } = self;
        field
    }

    #[inline]
    pub fn into_parts(self) -> (ComponentId, ErasedMutPtr<T>) {
        let Self {
            component_id,
            field,
        } = self;
        (component_id, field)
    }
}

impl<T> ErasedComponentMutPtr<T>
where
    T: MutSliceItemPtr,
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
        let field = ErasedMutPtr::dangling(layout)?;

        let me = unsafe { Self::from_parts(component_id, field) };
        Ok(me)
    }

    #[inline]
    pub fn try_from<C, U>(
        components: &ComponentRegistryView<impl Sized, U>,
        component: *mut C,
    ) -> Result<Self, TryFromPtrError>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let component_id = components
            .component_id::<C>()
            .ok_or_else(NotRegisteredError::of::<C>)?;
        let field = ErasedMutPtr::try_from(component)?;

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
        components: &ComponentRegistryView<impl Sized, U>,
    ) -> Result<*mut C, DowncastError<Self>>
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
    pub fn cast_const(self) -> ErasedComponentPtr<CastConst<T>> {
        let Self {
            component_id,
            field,
        } = self;

        let ptr = field.cast_const();
        unsafe { ErasedComponentPtr::from_parts(component_id, ptr) }
    }

    #[inline]
    pub unsafe fn as_ref_unchecked<'a>(self) -> ErasedComponentRef<'a, CastConst<T>> {
        unsafe { self.cast_const().as_ref_unchecked() }
    }

    #[inline]
    pub unsafe fn as_mut_unchecked<'a>(self) -> ErasedComponentMutRef<'a, T> {
        unsafe { ErasedComponentMutRef::from_ptr(self) }
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
    pub unsafe fn offset_from(self, origin: ErasedComponentPtr<CastConst<T>>) -> isize {
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
    pub unsafe fn copy_from(self, src: ErasedComponentPtr<CastConst<T>>, count: usize) {
        let Self { field, .. } = self;

        let src = src.field();
        unsafe { field.copy_from(src, count) }
    }

    #[inline]
    pub unsafe fn copy_from_nonoverlapping(
        self,
        src: ErasedComponentPtr<CastConst<T>>,
        count: usize,
    ) {
        let Self { field, .. } = self;

        let src = src.field();
        unsafe { field.copy_from_nonoverlapping(src, count) }
    }

    #[inline]
    pub unsafe fn drop_in_place(
        self,
        components: &ComponentRegistryView<impl WithErasedDrop, impl ?Sized>,
    ) -> Result<(), NotRegisteredError> {
        let component_desc = components
            .get_component_descriptor(self.component_id())
            .ok_or_else(NotRegisteredError::new)?;
        let Some(erased_drop) = component_desc.erased_drop() else {
            return Ok(());
        };

        unsafe { erased_drop.drop_in_place(self) }
        Ok(())
    }

    #[inline]
    pub fn as_ptr(self) -> *const T::Item {
        let Self { field, .. } = self;
        field.as_ptr()
    }

    #[inline]
    pub unsafe fn as_mut_ptr(self) -> *mut T::Item {
        let Self { field, .. } = self;
        field.as_mut_ptr()
    }

    #[inline]
    pub fn as_buffer(self) -> *const [T::Item] {
        let Self { field, .. } = self;
        field.as_buffer()
    }

    #[inline]
    pub unsafe fn as_mut_buffer(self) -> *mut [T::Item] {
        let Self { field, .. } = self;
        field.as_mut_buffer()
    }
}

impl<T, U> PartialEq<ErasedComponentMutPtr<U>> for ErasedComponentMutPtr<T> {
    #[inline]
    fn eq(&self, other: &ErasedComponentMutPtr<U>) -> bool {
        let Self { component_id, .. } = self;
        component_id.eq(other.borrow())
    }
}

impl<T> Eq for ErasedComponentMutPtr<T> {}

impl<T, U> PartialOrd<ErasedComponentMutPtr<U>> for ErasedComponentMutPtr<T> {
    #[inline]
    fn partial_cmp(&self, other: &ErasedComponentMutPtr<U>) -> Option<cmp::Ordering> {
        let Self { component_id, .. } = self;
        component_id.partial_cmp(other.borrow())
    }
}

impl<T> Ord for ErasedComponentMutPtr<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { component_id, .. } = self;
        component_id.cmp(other.borrow())
    }
}

impl<T> Hash for ErasedComponentMutPtr<T> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { component_id, .. } = self;
        component_id.hash(state);
    }
}

impl<T> Borrow<ComponentId> for ErasedComponentMutPtr<T> {
    #[inline]
    fn borrow(&self) -> &ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }
}
