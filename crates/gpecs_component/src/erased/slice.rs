use core::{
    borrow::Borrow,
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr::NonNull,
};

use gpecs_erased::{
    data::ErasedSlice,
    ptr::slice::{ConstPtr, NonNullSliceItemPtr, SliceItemPtr},
};

use crate::{
    Component,
    erased::{
        ErasedComponentNonNullPtr, ErasedComponentNonNullSlicePtr, ErasedComponentPtr,
        ErasedComponentSlicePtr,
        error::{DowncastError, NotRegisteredError, UpcastSliceError, check_downcast},
    },
    registry::{
        ComponentId, ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType},
    },
};

#[derive(Clone, Copy)]
pub struct ErasedComponentSlice<'a, T>
where
    T: SliceItemPtr,
{
    component_id: ComponentId,
    erased: ErasedSlice<'a, T>,
}

impl<'a, T> ErasedComponentSlice<'a, T>
where
    T: SliceItemPtr,
{
    #[inline]
    pub unsafe fn from_parts(component_id: ComponentId, erased: ErasedSlice<'a, T>) -> Self {
        Self {
            component_id,
            erased,
        }
    }

    #[inline]
    pub unsafe fn from_nonnull_slice_ptr(ptr: ErasedComponentNonNullSlicePtr<T>) -> Self {
        let (component_id, erased) = ptr.into_parts();
        let erased = unsafe { erased.as_ref() };
        unsafe { Self::from_parts(component_id, erased) }
    }

    #[inline]
    pub fn component_id(&self) -> ComponentId {
        let Self { component_id, .. } = *self;
        component_id
    }

    #[inline]
    pub fn as_erased(&self) -> &ErasedSlice<'a, T> {
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
    pub fn into_parts(self) -> (ComponentId, ErasedSlice<'a, T>) {
        let Self {
            component_id,
            erased,
        } = self;
        (component_id, erased)
    }

    #[inline]
    pub fn as_nonnull_slice_ptr(&self) -> ErasedComponentNonNullSlicePtr<T> {
        let Self {
            ref erased,
            component_id,
        } = *self;

        let erased = erased.as_nonnull_slice_ptr();
        unsafe { ErasedComponentNonNullSlicePtr::from_parts(component_id, erased) }
    }

    #[inline]
    pub fn as_nonnull_ptr(&self) -> ErasedComponentNonNullPtr<T> {
        let Self {
            ref erased,
            component_id,
        } = *self;

        let erased = erased.as_nonnull_ptr();
        unsafe { ErasedComponentNonNullPtr::from_parts(component_id, erased) }
    }
}

impl<'a, T> ErasedComponentSlice<'a, T>
where
    T: NonNullSliceItemPtr,
{
    #[inline]
    pub unsafe fn from_slice_ptr(ptr: ErasedComponentSlicePtr<ConstPtr<T::Ptrs>>) -> Option<Self> {
        let (component_id, erased) = ptr.into_parts();
        let erased = unsafe { ErasedSlice::from_slice_ptr(erased) }?;
        let me = unsafe { Self::from_parts(component_id, erased) };
        Some(me)
    }

    #[inline]
    pub unsafe fn from_slice_ptr_unchecked(
        ptr: ErasedComponentSlicePtr<ConstPtr<T::Ptrs>>,
    ) -> Self {
        let (component_id, erased) = ptr.into_parts();
        let erased = unsafe { ErasedSlice::from_slice_ptr_unchecked(erased) };
        unsafe { Self::from_parts(component_id, erased) }
    }

    #[inline]
    pub fn upcast<C, U>(
        components: &ComponentRegistryView<impl Sized, U>,
        component: &'a [C],
    ) -> Result<Self, UpcastSliceError>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let component_id = components
            .component_id::<C>()
            .ok_or_else(NotRegisteredError::of::<C>)?;
        let erased = ErasedSlice::upcast(component)?;

        let me = unsafe { Self::from_parts(component_id, erased) };
        Ok(me)
    }

    #[inline]
    pub fn downcast<C, U>(
        self,
        components: &ComponentRegistryView<impl Sized, U>,
    ) -> Result<&'a [C], DowncastError<Self>>
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
    pub fn as_slice_ptr(&self) -> ErasedComponentSlicePtr<ConstPtr<T::Ptrs>> {
        let Self {
            ref erased,
            component_id,
        } = *self;

        let erased = erased.as_slice_ptr();
        unsafe { ErasedComponentSlicePtr::from_parts(component_id, erased) }
    }

    #[inline]
    pub fn as_ptr(&self) -> ErasedComponentPtr<ConstPtr<T::Ptrs>> {
        let Self {
            ref erased,
            component_id,
        } = *self;

        let erased = erased.as_ptr();
        unsafe { ErasedComponentPtr::from_parts(component_id, erased) }
    }

    #[inline]
    pub fn as_nonnull_item_ptr(&self) -> NonNull<T::Item> {
        let Self { erased, .. } = self;
        erased.as_nonnull_item_ptr()
    }

    #[inline]
    pub fn as_item_ptr(&self) -> *const T::Item {
        let Self { erased, .. } = self;
        erased.as_item_ptr()
    }

    #[inline]
    pub fn as_items(&self) -> &[T::Item] {
        let Self { erased, .. } = self;
        erased.as_items()
    }

    #[inline]
    pub fn into_items(self) -> &'a [T::Item] {
        let Self { erased, .. } = self;
        erased.into_items()
    }
}

impl<T> Debug for ErasedComponentSlice<'_, T>
where
    T: NonNullSliceItemPtr<Item: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            component_id,
            erased,
        } = self;

        f.debug_struct("ErasedComponentSlice")
            .field("component_id", component_id)
            .field("erased", erased)
            .finish()
    }
}

impl<T, U> PartialEq<ErasedComponentSlice<'_, U>> for ErasedComponentSlice<'_, T>
where
    T: SliceItemPtr,
    U: SliceItemPtr,
{
    #[inline]
    fn eq(&self, other: &ErasedComponentSlice<'_, U>) -> bool {
        let Self { component_id, .. } = self;
        component_id.eq(other.borrow())
    }
}

impl<T> Eq for ErasedComponentSlice<'_, T> where T: SliceItemPtr {}

impl<T, U> PartialOrd<ErasedComponentSlice<'_, U>> for ErasedComponentSlice<'_, T>
where
    T: SliceItemPtr,
    U: SliceItemPtr,
{
    #[inline]
    fn partial_cmp(&self, other: &ErasedComponentSlice<'_, U>) -> Option<cmp::Ordering> {
        let Self { component_id, .. } = self;
        component_id.partial_cmp(other.borrow())
    }
}

impl<T> Ord for ErasedComponentSlice<'_, T>
where
    T: SliceItemPtr,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { component_id, .. } = self;
        component_id.cmp(other.borrow())
    }
}

impl<T> Hash for ErasedComponentSlice<'_, T>
where
    T: SliceItemPtr,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { component_id, .. } = self;
        component_id.hash(state);
    }
}

impl<T> Borrow<ComponentId> for ErasedComponentSlice<'_, T>
where
    T: SliceItemPtr,
{
    #[inline]
    fn borrow(&self) -> &ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }
}

impl<T> AsRef<[T::Item]> for ErasedComponentSlice<'_, T>
where
    T: NonNullSliceItemPtr,
{
    #[inline]
    fn as_ref(&self) -> &[T::Item] {
        self.as_items()
    }
}
