use core::{
    alloc::Layout,
    fmt::{self, Debug},
    marker::PhantomData,
    mem::{MaybeUninit, forget},
    ptr, slice,
};

use crate::{
    data::{
        ErasedMutPtr, ErasedMutRef, ErasedPtr, ErasedRef,
        error::{
            DowncastError, FromLayoutDataError, FromStorageError, FromValueError,
            FromValueErrorKind, check_downcast,
        },
        try_init_copy_from_slice,
    },
    error::{check_layout, check_len, check_sufficient_align},
    layout::bytes_to_items,
    ptr::slice::{ConstSliceItemPtr, MutSliceItemPtr, SliceItemPtrs},
    storage::{AlignedInitStorage, AlignedStorage, AlignedStorageFromLayout},
};

#[cfg(feature = "alloc")]
use crate::storage::BoxedAlignedUninitStorage;

#[cfg(feature = "alloc")]
pub type BoxedErased<P> = Erased<BoxedAlignedUninitStorage, P>;

pub struct Erased<T, P>
where
    T: ?Sized,
{
    phantom: PhantomData<P>,
    storage: AlignedInitStorage<T>,
}

impl<T, P> Erased<T, P>
where
    T: AlignedStorage<Item: Copy>,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    #[inline]
    pub fn try_from_storage_layout_data<V>(
        mut storage: T,
        layout: Layout,
        data: V,
    ) -> Result<Self, FromStorageError<T>>
    where
        V: AsRef<[T::Item]>,
    {
        let expected_layout = layout;
        let layout = storage.layout();
        if let Err(err) = check_sufficient_align(layout, Layout::new::<T::Item>()) {
            return Err(FromStorageError::new(err.into(), storage));
        }

        let data = data.as_ref();
        let len = size_of_val(data);
        if let Err(err) = check_len(len, layout.size()) {
            return Err(FromStorageError::new(err.into(), storage));
        }

        if let Err(err) = check_len(len, expected_layout.size()) {
            return Err(FromStorageError::new(err.into(), storage));
        }
        if let Err(err) = check_layout(layout, expected_layout) {
            return Err(FromStorageError::new(err.into(), storage));
        }

        if let Err(err) = try_init_copy_from_slice(storage.as_mut_uninit_slice(), data) {
            return Err(FromStorageError::new(err.into(), storage));
        }

        let me = Self {
            storage: unsafe { AlignedInitStorage::new_unchecked(storage) },
            phantom: PhantomData,
        };
        Ok(me)
    }

    #[inline]
    pub fn try_from_storage_value<V>(
        storage: T,
        value: V,
    ) -> Result<Self, FromStorageError<(T, V)>> {
        let layout = Layout::new::<V>();

        let data = ptr::from_ref(&value).cast();
        let len = bytes_to_items::<T::Item>(layout.size());
        let data = unsafe { slice::from_raw_parts(data, len) };

        match Self::try_from_storage_layout_data(storage, layout, data) {
            Ok(me) => {
                forget(value);
                Ok(me)
            }
            Err(err) => {
                let FromStorageError { reason, storage } = err;
                let err = FromStorageError::new(reason, (storage, value));
                Err(err)
            }
        }
    }
}

impl<T, P> Erased<T, P>
where
    T: AlignedStorage,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    #[inline]
    pub unsafe fn downcast<V>(self) -> Result<V, DowncastError<Self>> {
        let layout = self.layout();
        let Self { storage, .. } = check_downcast::<V, _>(layout, self)?;

        let src = storage.as_ptr().cast();
        Ok(unsafe { ptr::read(src) })
    }

    #[inline]
    pub fn into_storage(self) -> AlignedInitStorage<T> {
        let Self { storage, .. } = self;
        storage
    }

    #[inline]
    pub fn into_parts(self) -> (AlignedInitStorage<T>, Layout) {
        let Self { storage, .. } = self;
        let layout = storage.layout();
        (storage, layout)
    }
}

impl<T, P> Erased<T, P>
where
    T: AlignedStorageFromLayout<Item: Copy>,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    #[inline]
    pub fn try_from_layout_data<V>(
        layout: Layout,
        data: V,
    ) -> Result<Self, FromLayoutDataError<T::Error>>
    where
        V: AsRef<[T::Item]>,
    {
        check_sufficient_align(layout, Layout::new::<T::Item>())?;

        let data = data.as_ref();
        check_len(size_of_val(data), layout.size())?;

        let mut storage = T::from_layout(layout).map_err(FromLayoutDataError::FromLayout)?;
        try_init_copy_from_slice(storage.as_mut_uninit_slice(), data)?;

        let me = Self {
            storage: unsafe { AlignedInitStorage::new_unchecked(storage) },
            phantom: PhantomData,
        };
        Ok(me)
    }

    #[inline]
    pub fn try_from<V>(value: V) -> Result<Self, FromValueError<T::Error, V>> {
        let layout = Layout::new::<V>();

        let data = ptr::from_ref(&value).cast();
        let len = bytes_to_items::<T::Item>(layout.size());
        let data = unsafe { slice::from_raw_parts(data, len) };

        match Self::try_from_layout_data(layout, data) {
            Ok(me) => {
                forget(value);
                Ok(me)
            }
            Err(FromLayoutDataError::LenMismatch(error)) => unreachable!("{error}"),
            Err(FromLayoutDataError::InsufficientAlign(error)) => {
                let error = FromValueError::new(error.into(), value);
                Err(error)
            }
            Err(FromLayoutDataError::FromLayout(error)) => {
                let reason = FromValueErrorKind::FromLayout(error);
                let error = FromValueError::new(reason, value);
                Err(error)
            }
        }
    }
}

impl<T, P> Erased<T, P>
where
    T: AlignedStorage + ?Sized,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { storage, .. } = self;
        storage.layout()
    }

    #[inline]
    pub unsafe fn downcast_ref<V>(&self) -> Result<&V, DowncastError<&Self>> {
        let layout = self.layout();
        let Self { storage, .. } = check_downcast::<V, _>(layout, self)?;

        let ptr = storage.as_ptr().cast();
        Ok(unsafe { &*ptr })
    }

    #[inline]
    pub unsafe fn downcast_mut<V>(&mut self) -> Result<&mut V, DowncastError<&mut Self>> {
        let layout = self.layout();
        let Self { storage, .. } = check_downcast::<V, _>(layout, self)?;

        let ptr = storage.as_mut_ptr().cast();
        Ok(unsafe { &mut *ptr })
    }

    #[inline]
    pub fn as_erased(&self) -> ErasedRef<'_, P::Const> {
        unsafe { self.as_erased_ptr().deref() }
    }

    #[inline]
    pub fn as_erased_ptr(&self) -> ErasedPtr<P::Const> {
        let Self { storage, .. } = self;

        let layout = storage.layout();
        let buffer = storage.as_uninit_slice();
        let ptr = unsafe { ConstSliceItemPtr::from_slice(buffer, 0) };
        unsafe { ErasedPtr::from_parts(layout, ptr) }
    }

    #[inline]
    pub fn as_mut_erased(&mut self) -> ErasedMutRef<'_, P::Mut> {
        unsafe { self.as_mut_erased_ptr().deref_mut() }
    }

    #[inline]
    pub fn as_mut_erased_ptr(&mut self) -> ErasedMutPtr<P::Mut> {
        let Self { storage, .. } = self;

        let layout = storage.layout();
        let buffer = storage.as_mut_uninit_slice();
        let ptr = unsafe { MutSliceItemPtr::from_slice(buffer, 0) };
        unsafe { ErasedMutPtr::from_parts(layout, ptr) }
    }

    #[inline]
    pub fn as_slice(&self) -> &[T::Item] {
        let Self { storage, .. } = self;
        storage.as_slice()
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T::Item {
        let Self { storage, .. } = self;
        storage.as_ptr()
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T::Item] {
        let Self { storage, .. } = self;
        storage.as_mut_slice()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T::Item {
        let Self { storage, .. } = self;
        storage.as_mut_ptr()
    }
}

impl<T, P> Debug for Erased<T, P>
where
    T: AlignedStorage<Item: Debug> + ?Sized,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = &self.layout();
        let data = &self.as_slice();
        f.debug_struct("ErasedField")
            .field("desc", desc)
            .field("data", data)
            .finish()
    }
}

impl<T, P> AsRef<[T::Item]> for Erased<T, P>
where
    T: AlignedStorage + ?Sized,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    #[inline]
    fn as_ref(&self) -> &[T::Item] {
        self.as_slice()
    }
}

impl<T, P> AsMut<[T::Item]> for Erased<T, P>
where
    T: AlignedStorage + ?Sized,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T::Item] {
        self.as_mut_slice()
    }
}
