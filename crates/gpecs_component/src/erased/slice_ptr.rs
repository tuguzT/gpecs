use core::{
    borrow::Borrow,
    cmp,
    hash::{self, Hash},
};

use gpecs_erased::{
    data::ErasedSlicePtr,
    ptr::slice::{ConstSliceItemPtr, MutPtr, SliceItemPtr},
};

use crate::{
    Component,
    erased::{
        ErasedComponentMutSlicePtr, ErasedComponentPtr, ErasedComponentSlice,
        error::{DowncastError, NotRegisteredError, UpcastSliceError, check_downcast},
    },
    registry::{
        ComponentId, ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType},
    },
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedComponentSlicePtr<T> {
    component_id: ComponentId,
    erased: ErasedSlicePtr<T>,
}

impl<T> ErasedComponentSlicePtr<T> {
    #[inline]
    pub unsafe fn from_parts(component_id: ComponentId, erased: ErasedSlicePtr<T>) -> Self {
        Self {
            component_id,
            erased,
        }
    }

    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedComponentPtr<T>, len: usize) -> Self {
        let (component_id, field) = ptr.into_parts();
        let erased = unsafe { ErasedSlicePtr::from_parts(field, len) };
        unsafe { Self::from_parts(component_id, erased) }
    }

    #[inline]
    pub fn component_id(self) -> ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }

    #[inline]
    pub fn erased(self) -> ErasedSlicePtr<T> {
        let Self { erased, .. } = self;
        erased
    }

    #[inline]
    pub fn component_ptr(self) -> ErasedComponentPtr<T> {
        let Self {
            component_id,
            erased,
        } = self;

        let field = erased.ptr();
        unsafe { ErasedComponentPtr::from_parts(component_id, field) }
    }

    #[inline]
    pub fn len(self) -> usize {
        let Self { erased, .. } = self;
        erased.len()
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn into_parts(self) -> (ComponentId, ErasedSlicePtr<T>) {
        let Self {
            component_id,
            erased,
        } = self;
        (component_id, erased)
    }
}

impl<T> ErasedComponentSlicePtr<T>
where
    T: SliceItemPtr,
{
    #[inline]
    pub unsafe fn as_ref_unchecked<'a>(self) -> ErasedComponentSlice<'a, T> {
        unsafe { ErasedComponentSlice::from_ptr(self) }
    }
}

impl<T> ErasedComponentSlicePtr<T>
where
    T: ConstSliceItemPtr,
{
    #[inline]
    pub fn upcast<C, U>(
        components: &ComponentRegistryView<impl Sized, U>,
        component: *const [C],
    ) -> Result<Self, UpcastSliceError>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let component_id = components
            .component_id::<C>()
            .ok_or_else(NotRegisteredError::of::<C>)?;
        let erased = ErasedSlicePtr::upcast(component)?;

        let me = unsafe { Self::from_parts(component_id, erased) };
        Ok(me)
    }

    #[inline]
    pub fn downcast<C, U>(
        self,
        components: &ComponentRegistryView<impl Sized, U>,
    ) -> Result<*const [C], DowncastError<Self>>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let Self { component_id, .. } = self;
        let Self { erased, .. } = check_downcast::<C, U, _>(components, component_id, self)?;

        let slice = erased.downcast().map_err(|err| err.map_value(|_| self))?;
        Ok(slice)
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedComponentMutSlicePtr<MutPtr<T::Ptrs>> {
        let Self {
            component_id,
            erased,
        } = self;

        let erased = erased.cast_mut();
        unsafe { ErasedComponentMutSlicePtr::from_parts(component_id, erased) }
    }

    #[inline]
    pub fn as_ptr(self) -> *const T::Item {
        let Self { erased, .. } = self;
        erased.as_ptr()
    }

    #[inline]
    pub fn as_buffer(self) -> *const [T::Item] {
        let Self { erased, .. } = self;
        erased.as_buffer()
    }
}

impl<T, U> PartialEq<ErasedComponentSlicePtr<U>> for ErasedComponentSlicePtr<T> {
    #[inline]
    fn eq(&self, other: &ErasedComponentSlicePtr<U>) -> bool {
        let Self { component_id, .. } = self;
        component_id.eq(other.borrow())
    }
}

impl<T> Eq for ErasedComponentSlicePtr<T> {}

impl<T, U> PartialOrd<ErasedComponentSlicePtr<U>> for ErasedComponentSlicePtr<T> {
    #[inline]
    fn partial_cmp(&self, other: &ErasedComponentSlicePtr<U>) -> Option<cmp::Ordering> {
        let Self { component_id, .. } = self;
        component_id.partial_cmp(other.borrow())
    }
}

impl<T> Ord for ErasedComponentSlicePtr<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { component_id, .. } = self;
        component_id.cmp(other.borrow())
    }
}

impl<T> Hash for ErasedComponentSlicePtr<T> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { component_id, .. } = self;
        component_id.hash(state);
    }
}

impl<T> Borrow<ComponentId> for ErasedComponentSlicePtr<T> {
    #[inline]
    fn borrow(&self) -> &ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }
}
