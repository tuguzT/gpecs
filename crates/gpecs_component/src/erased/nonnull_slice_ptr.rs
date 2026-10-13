use core::{
    borrow::Borrow,
    cmp,
    hash::{self, Hash},
    ptr::NonNull,
};

use gpecs_erased::{
    data::ErasedNonNullSlicePtr,
    ptr::slice::{ConstPtr, MutPtr, NonNullSliceItemPtr},
};

use crate::{
    Component,
    erased::{
        ErasedComponentMutSlicePtr, ErasedComponentNonNullPtr, ErasedComponentSlicePtr,
        error::{DowncastError, NotRegisteredError, UpcastSliceError, check_downcast},
    },
    registry::{
        ComponentId, ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType},
    },
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedComponentNonNullSlicePtr<T> {
    component_id: ComponentId,
    fields: ErasedNonNullSlicePtr<T>,
}

impl<T> ErasedComponentNonNullSlicePtr<T> {
    #[inline]
    pub unsafe fn from_parts(component_id: ComponentId, fields: ErasedNonNullSlicePtr<T>) -> Self {
        Self {
            component_id,
            fields,
        }
    }

    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedComponentNonNullPtr<T>, len: usize) -> Self {
        let (component_id, field) = ptr.into_parts();
        let fields = unsafe { ErasedNonNullSlicePtr::from_parts(field, len) };
        unsafe { Self::from_parts(component_id, fields) }
    }

    #[inline]
    pub fn component_id(self) -> ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }

    #[inline]
    pub fn fields(self) -> ErasedNonNullSlicePtr<T> {
        let Self { fields, .. } = self;
        fields
    }

    #[inline]
    pub fn component_ptr(self) -> ErasedComponentNonNullPtr<T> {
        let Self {
            component_id,
            fields,
        } = self;

        let field = fields.field_ptr();
        unsafe { ErasedComponentNonNullPtr::from_parts(component_id, field) }
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
    pub fn into_parts(self) -> (ComponentId, ErasedNonNullSlicePtr<T>) {
        let Self {
            component_id,
            fields,
        } = self;
        (component_id, fields)
    }
}

impl<T> ErasedComponentNonNullSlicePtr<T>
where
    T: NonNullSliceItemPtr,
{
    #[inline]
    pub fn upcast<C, U>(
        components: &ComponentRegistryView<impl Sized, U>,
        component: NonNull<[C]>,
    ) -> Result<Self, UpcastSliceError>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let component_id = components
            .component_id::<C>()
            .ok_or_else(NotRegisteredError::of::<C>)?;
        let fields = ErasedNonNullSlicePtr::upcast(component)?;

        let me = unsafe { Self::from_parts(component_id, fields) };
        Ok(me)
    }

    #[inline]
    pub fn downcast<C, U>(
        self,
        components: &ComponentRegistryView<impl Sized, U>,
    ) -> Result<NonNull<[C]>, DowncastError<Self>>
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
    pub fn into_slice_ptr(self) -> ErasedComponentSlicePtr<ConstPtr<T::Ptrs>> {
        let Self {
            component_id,
            fields,
        } = self;

        let fields = fields.into_slice_ptr();
        unsafe { ErasedComponentSlicePtr::from_parts(component_id, fields) }
    }

    #[inline]
    pub fn into_mut_slice_ptr(self) -> ErasedComponentMutSlicePtr<MutPtr<T::Ptrs>> {
        let Self {
            component_id,
            fields,
        } = self;

        let fields = fields.into_mut_slice_ptr();
        unsafe { ErasedComponentMutSlicePtr::from_parts(component_id, fields) }
    }

    #[inline]
    pub fn as_ptr(self) -> NonNull<T::Item> {
        let Self { fields, .. } = self;
        fields.as_ptr()
    }

    #[inline]
    pub fn as_buffer(self) -> NonNull<[T::Item]> {
        let Self { fields, .. } = self;
        fields.as_buffer()
    }
}

impl<T, U> PartialEq<ErasedComponentNonNullSlicePtr<U>> for ErasedComponentNonNullSlicePtr<T> {
    #[inline]
    fn eq(&self, other: &ErasedComponentNonNullSlicePtr<U>) -> bool {
        let Self { component_id, .. } = self;
        component_id.eq(other.borrow())
    }
}

impl<T> Eq for ErasedComponentNonNullSlicePtr<T> {}

impl<T, U> PartialOrd<ErasedComponentNonNullSlicePtr<U>> for ErasedComponentNonNullSlicePtr<T> {
    #[inline]
    fn partial_cmp(&self, other: &ErasedComponentNonNullSlicePtr<U>) -> Option<cmp::Ordering> {
        let Self { component_id, .. } = self;
        component_id.partial_cmp(other.borrow())
    }
}

impl<T> Ord for ErasedComponentNonNullSlicePtr<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { component_id, .. } = self;
        component_id.cmp(other.borrow())
    }
}

impl<T> Hash for ErasedComponentNonNullSlicePtr<T> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { component_id, .. } = self;
        component_id.hash(state);
    }
}

impl<T> Borrow<ComponentId> for ErasedComponentNonNullSlicePtr<T> {
    #[inline]
    fn borrow(&self) -> &ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }
}
