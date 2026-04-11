use core::{
    borrow::Borrow,
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    mem::{ManuallyDrop, MaybeUninit},
    ptr,
};

use gpecs_erased::{
    data::Erased,
    ptr::slice::SliceItemPtrs,
    storage::{AlignedStorage, AlignedStorageFromLayout},
};
use polonius_the_crab::{polonius, polonius_return};

use crate::{
    Component,
    erased::{
        ErasedComponentMutPtr, ErasedComponentMutRef, ErasedComponentPtr, ErasedComponentRef,
        ErasedDrop, WithErasedDrop,
        error::{
            DowncastError, FromComponentError, FromStorageError, NotRegisteredError, check_downcast,
        },
    },
    registry::{
        ComponentId, ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType},
    },
};

pub struct ErasedComponent<T, P>
where
    T: AlignedStorage,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    component_id: ComponentId,
    field: Erased<T, P>,
    erased_drop: Option<ErasedDrop>,
}

impl<T, P> ErasedComponent<T, P>
where
    T: AlignedStorage<Item: Copy>,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    #[inline]
    pub fn try_from_storage_component<C, M, U>(
        components: &ComponentRegistryView<M, U>,
        component: C,
        storage: T,
    ) -> Result<Self, FromStorageError<(T, C)>>
    where
        C: Component,
        M: WithErasedDrop,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let Some(component_info) = components.get_component_info_of::<C>() else {
            let source = NotRegisteredError::of::<C>().into();
            return Err(FromStorageError::new(source, (storage, component)));
        };

        let field = Erased::try_from_storage_value(storage, component)?;
        let component_id = component_info.component_id();
        let erased_drop = component_info.erased_drop();

        let me = unsafe { Self::from_parts(component_id, field, erased_drop) };
        Ok(me)
    }
}

impl<T, P> ErasedComponent<T, P>
where
    T: AlignedStorageFromLayout<Item: Copy>,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    #[inline]
    pub fn try_from<C, M, U>(
        components: &ComponentRegistryView<M, U>,
        component: C,
    ) -> Result<Self, FromComponentError<T::Error, C>>
    where
        C: Component,
        M: WithErasedDrop,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let Some(component_info) = components.get_component_info_of::<C>() else {
            let source = NotRegisteredError::of::<C>().into();
            return Err(FromComponentError::new(component, source));
        };

        let field = Erased::try_from(component)?;
        let component_id = component_info.component_id();
        let erased_drop = component_info.erased_drop();

        let me = unsafe { Self::from_parts(component_id, field, erased_drop) };
        Ok(me)
    }
}

impl<T, P> ErasedComponent<T, P>
where
    T: AlignedStorage,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    #[inline]
    pub unsafe fn from_parts(
        component_id: ComponentId,
        field: Erased<T, P>,
        erased_drop: Option<ErasedDrop>,
    ) -> Self {
        Self {
            component_id,
            field,
            erased_drop,
        }
    }

    #[inline]
    pub fn downcast<C, U>(
        self,
        components: &ComponentRegistryView<impl Sized, U>,
    ) -> Result<C, DowncastError<Self>>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let Self { component_id, .. } = self;
        let (_, field, erased_drop) =
            check_downcast::<C, U, _>(components, component_id, self)?.into_parts();

        let into_self = |field| unsafe { Self::from_parts(component_id, field, erased_drop) };
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
    pub fn as_field(&self) -> &Erased<T, P> {
        let Self { field, .. } = self;
        field
    }

    #[inline]
    pub unsafe fn as_mut_field(&mut self) -> &mut Erased<T, P> {
        let Self { field, .. } = self;
        field
    }

    #[inline]
    pub fn erased_drop(&self) -> Option<ErasedDrop> {
        let Self { erased_drop, .. } = *self;
        erased_drop
    }

    #[inline]
    pub fn as_erased_component_ptr(&self) -> ErasedComponentPtr<P::Const> {
        let Self {
            ref field,
            component_id,
            ..
        } = *self;

        let field = field.as_erased_ptr();
        unsafe { ErasedComponentPtr::from_parts(component_id, field) }
    }

    #[inline]
    pub fn as_mut_erased_component_ptr(&mut self) -> ErasedComponentMutPtr<P::Mut> {
        let Self {
            ref mut field,
            component_id,
            ..
        } = *self;

        let field = field.as_mut_erased_ptr();
        unsafe { ErasedComponentMutPtr::from_parts(component_id, field) }
    }

    #[inline]
    pub fn as_erased_component(&self) -> ErasedComponentRef<'_, P::Const> {
        unsafe { self.as_erased_component_ptr().deref() }
    }

    #[inline]
    pub fn as_mut_erased_component(&mut self) -> ErasedComponentMutRef<'_, P::Mut> {
        unsafe { self.as_mut_erased_component_ptr().deref_mut() }
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
    pub fn as_slice(&self) -> &[MaybeUninit<T::Item>] {
        let Self { field, .. } = self;
        field.as_slice()
    }

    #[inline]
    pub unsafe fn as_mut_slice(&mut self) -> &mut [MaybeUninit<T::Item>] {
        let Self { field, .. } = self;
        field.as_mut_slice()
    }

    #[inline]
    pub fn into_field(self) -> Erased<T, P> {
        let me = ManuallyDrop::new(self);
        unsafe { ptr::read(&raw const me.field) }
    }

    #[inline]
    pub fn into_parts(self) -> (ComponentId, Erased<T, P>, Option<ErasedDrop>) {
        let Self {
            component_id,
            erased_drop,
            ..
        } = self;

        let field = self.into_field();
        (component_id, field, erased_drop)
    }
}

impl<T, P> Debug for ErasedComponent<T, P>
where
    T: AlignedStorage<Item: Debug>,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            component_id,
            field,
            erased_drop,
        } = self;

        f.debug_struct("ErasedComponent")
            .field("component_id", component_id)
            .field("field", field)
            .field("erased_drop", erased_drop)
            .finish()
    }
}

impl<T, U, P, Z> PartialEq<ErasedComponent<U, Z>> for ErasedComponent<T, P>
where
    T: AlignedStorage,
    U: AlignedStorage,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
    Z: SliceItemPtrs<Item = MaybeUninit<U::Item>>,
{
    #[inline]
    fn eq(&self, other: &ErasedComponent<U, Z>) -> bool {
        let Self { component_id, .. } = self;
        component_id.eq(other.borrow())
    }
}

impl<T, P> Eq for ErasedComponent<T, P>
where
    T: AlignedStorage,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
}

impl<T, U, P, Z> PartialOrd<ErasedComponent<U, Z>> for ErasedComponent<T, P>
where
    T: AlignedStorage,
    U: AlignedStorage,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
    Z: SliceItemPtrs<Item = MaybeUninit<U::Item>>,
{
    #[inline]
    fn partial_cmp(&self, other: &ErasedComponent<U, Z>) -> Option<cmp::Ordering> {
        let Self { component_id, .. } = self;
        component_id.partial_cmp(other.borrow())
    }
}

impl<T, P> Ord for ErasedComponent<T, P>
where
    T: AlignedStorage,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { component_id, .. } = self;
        component_id.cmp(other.borrow())
    }
}

impl<T, P> Hash for ErasedComponent<T, P>
where
    T: AlignedStorage,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { component_id, .. } = self;
        component_id.hash(state);
    }
}

impl<T, P> Borrow<ComponentId> for ErasedComponent<T, P>
where
    T: AlignedStorage,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    #[inline]
    fn borrow(&self) -> &ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }
}

impl<T, P> AsRef<[MaybeUninit<T::Item>]> for ErasedComponent<T, P>
where
    T: AlignedStorage,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    #[inline]
    fn as_ref(&self) -> &[MaybeUninit<T::Item>] {
        self.as_slice()
    }
}

impl<T, P> Drop for ErasedComponent<T, P>
where
    T: AlignedStorage,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    #[inline]
    fn drop(&mut self) {
        let Some(drop) = self.erased_drop() else {
            return;
        };

        let to_drop = self.as_mut_erased_component_ptr();
        unsafe { drop.drop_in_place(to_drop) }
    }
}
