use core::{
    borrow::Borrow,
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
};

use gpecs_erased::{
    data::ErasedMutSlice,
    ptr::slice::{ConstPtr, MutSliceItemPtr, SliceItemPtr},
};
use polonius_the_crab::{polonius, polonius_return};

use crate::{
    Component,
    erased::{
        ErasedComponentMutPtr, ErasedComponentMutSlicePtr, ErasedComponentPtr,
        ErasedComponentSlice, ErasedComponentSlicePtr,
        error::{DowncastError, NotRegisteredError, UpcastSliceError, check_downcast},
    },
    registry::{
        ComponentId, ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType},
    },
};

pub struct ErasedComponentMutSlice<'a, T>
where
    T: SliceItemPtr,
{
    component_id: ComponentId,
    erased: ErasedMutSlice<'a, T>,
}

impl<'a, T> ErasedComponentMutSlice<'a, T>
where
    T: SliceItemPtr,
{
    #[inline]
    pub unsafe fn from_parts(component_id: ComponentId, erased: ErasedMutSlice<'a, T>) -> Self {
        Self {
            component_id,
            erased,
        }
    }

    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedComponentMutSlicePtr<T>) -> Self {
        let (component_id, erased) = ptr.into_parts();
        let erased = unsafe { erased.as_mut_unchecked() };
        unsafe { Self::from_parts(component_id, erased) }
    }

    #[inline]
    pub fn component_id(&self) -> ComponentId {
        let Self { component_id, .. } = *self;
        component_id
    }

    #[inline]
    pub fn erased(&self) -> &ErasedMutSlice<'a, T> {
        let Self { erased, .. } = self;
        erased
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
    pub fn as_mut_component_slice_ptr(&mut self) -> ErasedComponentMutSlicePtr<T> {
        let Self {
            ref mut erased,
            component_id,
        } = *self;

        let erased = erased.mut_slice_ptr();
        unsafe { ErasedComponentMutSlicePtr::from_parts(component_id, erased) }
    }

    #[inline]
    pub fn as_mut_component_ptr(&mut self) -> ErasedComponentMutPtr<T> {
        let Self {
            ref mut erased,
            component_id,
        } = *self;

        let field = erased.mut_ptr();
        unsafe { ErasedComponentMutPtr::from_parts(component_id, field) }
    }

    #[inline]
    pub fn into_parts(self) -> (ComponentId, ErasedMutSlice<'a, T>) {
        let Self {
            component_id,
            erased,
        } = self;
        (component_id, erased)
    }
}

impl<'a, T> ErasedComponentMutSlice<'a, T>
where
    T: MutSliceItemPtr,
{
    #[inline]
    pub fn upcast<C, U>(
        components: &ComponentRegistryView<impl Sized, U>,
        component: &'a mut [C],
    ) -> Result<Self, UpcastSliceError>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let component_id = components
            .component_id::<C>()
            .ok_or_else(NotRegisteredError::of::<C>)?;
        let erased = ErasedMutSlice::upcast(component)?;

        let me = unsafe { Self::from_parts(component_id, erased) };
        Ok(me)
    }

    #[inline]
    pub fn downcast<C, U>(
        self,
        components: &ComponentRegistryView<impl Sized, U>,
    ) -> Result<&'a mut [C], DowncastError<Self>>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let Self { component_id, .. } = self;
        let Self { erased, .. } = check_downcast::<C, U, _>(components, component_id, self)?;

        let into_self = |erased| unsafe { Self::from_parts(component_id, erased) };
        let slice = unsafe { erased.downcast() }.map_err(|err| err.map_value(into_self))?;
        Ok(slice)
    }

    #[inline]
    pub fn downcast_ref<C, U>(
        &self,
        components: &ComponentRegistryView<impl Sized, U>,
    ) -> Result<&[C], DowncastError<&Self>>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let Self { component_id, .. } = *self;
        let Self { erased, .. } = check_downcast::<C, U, _>(components, component_id, self)?;

        let slice = unsafe { erased.downcast_ref() }.map_err(|err| err.map_value(|_| self))?;
        Ok(slice)
    }

    #[inline]
    pub fn downcast_mut<C, U>(
        &mut self,
        components: &ComponentRegistryView<impl Sized, U>,
    ) -> Result<&mut [C], DowncastError<&mut Self>>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let Self { component_id, .. } = *self;
        let mut this = check_downcast::<C, U, _>(components, component_id, self)?;

        let source = polonius!(|this| -> Result<&'polonius mut [C], _> {
            match unsafe { this.erased.downcast_mut() } {
                Ok(component) => polonius_return!(Ok(component)),
                Err(error) => error.into_source().into(),
            }
        });
        Err(DowncastError::new(this, source))
    }

    #[inline]
    pub fn as_component_slice_ptr(&self) -> ErasedComponentSlicePtr<ConstPtr<T::Ptrs>> {
        let Self {
            ref erased,
            component_id,
        } = *self;

        let erased = erased.slice_ptr();
        unsafe { ErasedComponentSlicePtr::from_parts(component_id, erased) }
    }

    #[inline]
    pub fn as_component_ptr(&self) -> ErasedComponentPtr<ConstPtr<T::Ptrs>> {
        let Self {
            ref erased,
            component_id,
        } = *self;

        let field = erased.ptr();
        unsafe { ErasedComponentPtr::from_parts(component_id, field) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T::Item {
        let Self { erased, .. } = self;
        erased.as_ptr()
    }

    #[inline]
    pub unsafe fn as_mut_ptr(&mut self) -> *mut T::Item {
        let Self { erased, .. } = self;
        erased.as_mut_ptr()
    }

    #[inline]
    pub fn as_buffer(&self) -> &[T::Item] {
        let Self { erased, .. } = self;
        erased.as_buffer()
    }

    #[inline]
    pub unsafe fn as_mut_buffer(&mut self) -> &mut [T::Item] {
        let Self { erased, .. } = self;
        erased.as_mut_buffer()
    }

    #[inline]
    pub fn into_slice(self) -> ErasedComponentSlice<'a, ConstPtr<T::Ptrs>> {
        let Self {
            component_id,
            erased,
        } = self;

        let erased = erased.into_slice();
        unsafe { ErasedComponentSlice::from_parts(component_id, erased) }
    }
}

impl<T> Debug for ErasedComponentMutSlice<'_, T>
where
    T: MutSliceItemPtr<Item: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            component_id,
            erased,
        } = self;

        f.debug_struct("ErasedComponentMutSlice")
            .field("component_id", component_id)
            .field("erased", erased)
            .finish()
    }
}

impl<T, U> PartialEq<ErasedComponentMutSlice<'_, U>> for ErasedComponentMutSlice<'_, T>
where
    T: SliceItemPtr,
    U: SliceItemPtr,
{
    #[inline]
    fn eq(&self, other: &ErasedComponentMutSlice<'_, U>) -> bool {
        let Self { component_id, .. } = self;
        component_id.eq(other.borrow())
    }
}

impl<T> Eq for ErasedComponentMutSlice<'_, T> where T: SliceItemPtr {}

impl<T, U> PartialOrd<ErasedComponentMutSlice<'_, U>> for ErasedComponentMutSlice<'_, T>
where
    T: SliceItemPtr,
    U: SliceItemPtr,
{
    #[inline]
    fn partial_cmp(&self, other: &ErasedComponentMutSlice<'_, U>) -> Option<cmp::Ordering> {
        let Self { component_id, .. } = self;
        component_id.partial_cmp(other.borrow())
    }
}

impl<T> Ord for ErasedComponentMutSlice<'_, T>
where
    T: SliceItemPtr,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { component_id, .. } = self;
        component_id.cmp(other.borrow())
    }
}

impl<T> Hash for ErasedComponentMutSlice<'_, T>
where
    T: SliceItemPtr,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { component_id, .. } = self;
        component_id.hash(state);
    }
}

impl<T> Borrow<ComponentId> for ErasedComponentMutSlice<'_, T>
where
    T: SliceItemPtr,
{
    #[inline]
    fn borrow(&self) -> &ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }
}

impl<T> AsRef<[T::Item]> for ErasedComponentMutSlice<'_, T>
where
    T: MutSliceItemPtr,
{
    #[inline]
    fn as_ref(&self) -> &[T::Item] {
        self.as_buffer()
    }
}
