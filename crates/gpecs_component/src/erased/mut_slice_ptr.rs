use core::{
    borrow::Borrow,
    cmp,
    hash::{self, Hash},
};

use gpecs_erased::{
    data::ErasedMutSlicePtr,
    ptr::slice::{CastConst, MutSliceItemPtr},
};

use crate::{
    Component,
    erased::{
        ErasedComponentMutPtr, ErasedComponentMutSlice, ErasedComponentSlice,
        ErasedComponentSlicePtr, WithErasedDrop,
        error::{DowncastError, NotRegisteredError, TryFromSlicePtrError, check_downcast},
    },
    registry::{
        ComponentId, ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType},
    },
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedComponentMutSlicePtr<T> {
    component_id: ComponentId,
    fields: ErasedMutSlicePtr<T>,
}

impl<T> ErasedComponentMutSlicePtr<T> {
    #[inline]
    pub unsafe fn from_parts(component_id: ComponentId, fields: ErasedMutSlicePtr<T>) -> Self {
        Self {
            component_id,
            fields,
        }
    }

    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedComponentMutPtr<T>, len: usize) -> Self {
        let (component_id, field) = ptr.into_parts();
        let fields = unsafe { ErasedMutSlicePtr::from_parts(field, len) };
        unsafe { Self::from_parts(component_id, fields) }
    }

    #[inline]
    pub fn component_id(self) -> ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }

    #[inline]
    pub fn fields(self) -> ErasedMutSlicePtr<T> {
        let Self { fields, .. } = self;
        fields
    }

    #[inline]
    pub fn component_ptr(self) -> ErasedComponentMutPtr<T> {
        let Self {
            component_id,
            fields,
        } = self;

        let field = fields.field_ptr();
        unsafe { ErasedComponentMutPtr::from_parts(component_id, field) }
    }

    #[inline]
    pub fn len(self) -> usize {
        let Self { fields, .. } = self;
        fields.len()
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn into_parts(self) -> (ComponentId, ErasedMutSlicePtr<T>) {
        let Self {
            component_id,
            fields,
        } = self;
        (component_id, fields)
    }
}

impl<T> ErasedComponentMutSlicePtr<T>
where
    T: MutSliceItemPtr,
{
    #[inline]
    pub fn try_from<C, U>(
        components: &ComponentRegistryView<impl Sized, U>,
        component: *mut [C],
    ) -> Result<Self, TryFromSlicePtrError>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let component_id = components
            .component_id::<C>()
            .ok_or_else(NotRegisteredError::of::<C>)?;
        let fields = ErasedMutSlicePtr::try_from(component)?;

        let me = unsafe { Self::from_parts(component_id, fields) };
        Ok(me)
    }

    #[inline]
    pub fn downcast<C, U>(
        self,
        components: &ComponentRegistryView<impl Sized, U>,
    ) -> Result<*mut [C], DowncastError<Self>>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let Self { component_id, .. } = self;
        let Self { fields, .. } = check_downcast::<C, U, _>(components, component_id, self)?;

        let component = fields.downcast().map_err(|err| err.map_value(|_| self))?;
        Ok(component)
    }

    #[inline]
    pub fn cast_const(self) -> ErasedComponentSlicePtr<CastConst<T>> {
        let Self {
            component_id,
            fields,
        } = self;

        let fields = fields.cast_const();
        unsafe { ErasedComponentSlicePtr::from_parts(component_id, fields) }
    }

    #[inline]
    pub unsafe fn as_ref_unchecked<'a>(self) -> ErasedComponentSlice<'a, CastConst<T>> {
        unsafe { self.cast_const().as_ref_unchecked() }
    }

    #[inline]
    pub unsafe fn as_mut_unchecked<'a>(self) -> ErasedComponentMutSlice<'a, T> {
        unsafe { ErasedComponentMutSlice::from_ptr(self) }
    }

    #[inline]
    pub unsafe fn drop_in_place(
        self,
        components: &ComponentRegistryView<impl WithErasedDrop, impl ?Sized>,
    ) -> Result<(), NotRegisteredError> {
        let component_info = components
            .get_component_info(self.component_id())
            .ok_or_else(NotRegisteredError::new)?;
        let Some(erased_drop) = component_info.erased_drop() else {
            return Ok(());
        };

        unsafe { erased_drop.drop_in_place_slice(self) }
        Ok(())
    }

    #[inline]
    pub fn as_ptr(self) -> *const T::Item {
        let Self { fields, .. } = self;
        fields.as_ptr()
    }

    #[inline]
    pub unsafe fn as_mut_ptr(self) -> *mut T::Item {
        let Self { fields, .. } = self;
        fields.as_mut_ptr()
    }

    #[inline]
    pub fn as_buffer(self) -> *const [T::Item] {
        let Self { fields, .. } = self;
        fields.as_buffer()
    }

    #[inline]
    pub unsafe fn as_mut_buffer(self) -> *mut [T::Item] {
        let Self { fields, .. } = self;
        fields.as_mut_buffer()
    }
}

impl<T, U> PartialEq<ErasedComponentMutSlicePtr<U>> for ErasedComponentMutSlicePtr<T> {
    #[inline]
    fn eq(&self, other: &ErasedComponentMutSlicePtr<U>) -> bool {
        let Self { component_id, .. } = self;
        component_id.eq(other.borrow())
    }
}

impl<T> Eq for ErasedComponentMutSlicePtr<T> {}

impl<T, U> PartialOrd<ErasedComponentMutSlicePtr<U>> for ErasedComponentMutSlicePtr<T> {
    #[inline]
    fn partial_cmp(&self, other: &ErasedComponentMutSlicePtr<U>) -> Option<cmp::Ordering> {
        let Self { component_id, .. } = self;
        component_id.partial_cmp(other.borrow())
    }
}

impl<T> Ord for ErasedComponentMutSlicePtr<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { component_id, .. } = self;
        component_id.cmp(other.borrow())
    }
}

impl<T> Hash for ErasedComponentMutSlicePtr<T> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { component_id, .. } = self;
        component_id.hash(state);
    }
}

impl<T> Borrow<ComponentId> for ErasedComponentMutSlicePtr<T> {
    #[inline]
    fn borrow(&self) -> &ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }
}
