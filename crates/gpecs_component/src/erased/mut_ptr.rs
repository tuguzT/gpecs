use core::{
    borrow::Borrow,
    cmp,
    hash::{self, Hash},
};

use gpecs_erased::{
    data::ErasedMutPtr,
    layout::WithLayout,
    ptr::slice::{ConstPtr, MutSliceItemPtr, SliceItemPtr},
};

use crate::{
    Component,
    erased::{
        ErasedComponentMutRef, ErasedComponentPtr, ErasedComponentRef, WithErasedDrop,
        error::{DanglingError, DowncastError, NotRegisteredError, UpcastPtrError, check_downcast},
    },
    registry::{
        ComponentId, ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType},
    },
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedComponentMutPtr<T> {
    component_id: ComponentId,
    erased: ErasedMutPtr<T>,
}

impl<T> ErasedComponentMutPtr<T> {
    #[inline]
    pub unsafe fn from_parts(component_id: ComponentId, erased: ErasedMutPtr<T>) -> Self {
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
    pub fn erased(self) -> ErasedMutPtr<T> {
        let Self { erased, .. } = self;
        erased
    }

    #[inline]
    pub fn into_parts(self) -> (ComponentId, ErasedMutPtr<T>) {
        let Self {
            component_id,
            erased,
        } = self;
        (component_id, erased)
    }
}

impl<T> ErasedComponentMutPtr<T>
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

    #[inline]
    pub unsafe fn as_mut_unchecked<'a>(self) -> ErasedComponentMutRef<'a, T> {
        unsafe { ErasedComponentMutRef::from_ptr(self) }
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
        let erased = ErasedMutPtr::dangling(layout)?;

        let me = unsafe { Self::from_parts(component_id, erased) };
        Ok(me)
    }

    #[inline]
    pub fn upcast<C, U>(
        components: &ComponentRegistryView<impl Sized, U>,
        component: *mut C,
    ) -> Result<Self, UpcastPtrError>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let component_id = components
            .component_id::<C>()
            .ok_or_else(NotRegisteredError::of::<C>)?;
        let erased = ErasedMutPtr::upcast(component)?;

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
    ) -> Result<*mut C, DowncastError<Self>>
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
    pub fn cast_const(self) -> ErasedComponentPtr<ConstPtr<T::Ptrs>> {
        let Self {
            component_id,
            erased,
        } = self;

        let ptr = erased.cast_const();
        unsafe { ErasedComponentPtr::from_parts(component_id, ptr) }
    }

    #[inline]
    pub unsafe fn as_ref_unchecked<'a>(self) -> ErasedComponentRef<'a, ConstPtr<T::Ptrs>> {
        unsafe { self.cast_const().as_ref_unchecked() }
    }

    #[inline]
    pub unsafe fn swap(self, with: Self) {
        let Self { erased, .. } = self;

        let with = with.erased();
        unsafe { erased.swap(with) }
    }

    #[inline]
    pub unsafe fn copy_from(self, src: ErasedComponentPtr<ConstPtr<T::Ptrs>>, count: usize) {
        let Self { erased, .. } = self;

        let src = src.erased();
        unsafe { erased.copy_from(src, count) }
    }

    #[inline]
    pub unsafe fn copy_from_nonoverlapping(
        self,
        src: ErasedComponentPtr<ConstPtr<T::Ptrs>>,
        count: usize,
    ) {
        let Self { erased, .. } = self;

        let src = src.erased();
        unsafe { erased.copy_from_nonoverlapping(src, count) }
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
        let Self { erased, .. } = self;
        erased.as_ptr()
    }

    #[inline]
    pub unsafe fn as_mut_ptr(self) -> *mut T::Item {
        let Self { erased, .. } = self;
        erased.as_mut_ptr()
    }

    #[inline]
    pub fn as_buffer(self) -> *const [T::Item] {
        let Self { erased, .. } = self;
        erased.as_buffer()
    }

    #[inline]
    pub unsafe fn as_mut_buffer(self) -> *mut [T::Item] {
        let Self { erased, .. } = self;
        erased.as_mut_buffer()
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
