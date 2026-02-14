use core::{
    alloc::Layout,
    fmt::{self, Debug},
    marker::PhantomData,
    mem::{MaybeUninit, forget},
    ptr, slice,
};

use crate::{
    bytes_to_items::item_count,
    error::{check_layout, check_len, check_sufficient_align},
    field::{
        ErasedFieldMutPtr, ErasedFieldPtr, ErasedFieldRef, ErasedFieldRefMut,
        error::{
            DowncastError, FromDescDataError, FromStorageError, FromValueError, FromValueErrorKind,
            check_downcast,
        },
    },
    slice_item_ptr::{ConstSliceItemPtr, MutSliceItemPtr, SliceItemPtrs},
    soa::field::FieldDescriptor,
    storage::{AlignedInitStorage, AlignedStorage, AlignedStorageFromLayout},
    uninit::write_copy_of_slice,
};

#[cfg(feature = "alloc")]
use crate::storage::BoxedAlignedUninitStorage;

#[cfg(feature = "alloc")]
pub type BoxedErasedField<P> = ErasedField<BoxedAlignedUninitStorage, P>;

pub struct ErasedField<T, P>
where
    T: ?Sized,
{
    phantom: PhantomData<P>,
    storage: AlignedInitStorage<T>,
}

impl<T, P> ErasedField<T, P>
where
    T: AlignedStorage<Item: Copy>,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    #[inline]
    pub fn try_from_storage_desc_data<V>(
        mut storage: T,
        desc: FieldDescriptor,
        data: V,
    ) -> Result<Self, FromStorageError<T>>
    where
        V: AsRef<[T::Item]>,
    {
        let layout = storage.layout();
        if let Err(err) = check_sufficient_align(layout, Layout::new::<T::Item>()) {
            return Err(FromStorageError::new(err.into(), storage));
        }

        let data = data.as_ref();
        let len = size_of_val(data);
        if let Err(err) = check_len(len, layout.size()) {
            return Err(FromStorageError::new(err.into(), storage));
        }

        let expected_layout = desc.layout();
        if let Err(err) = check_len(len, expected_layout.size()) {
            return Err(FromStorageError::new(err.into(), storage));
        }
        if let Err(err) = check_layout(layout, expected_layout) {
            return Err(FromStorageError::new(err.into(), storage));
        }

        if let Err(err) = write_copy_of_slice(storage.as_mut_uninit_slice(), data) {
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
        let desc = FieldDescriptor::of::<V>();

        let data = ptr::from_ref(&value).cast();
        let len = item_count::<T::Item>(desc);
        let data = unsafe { slice::from_raw_parts(data, len) };

        match Self::try_from_storage_desc_data(storage, desc, data) {
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

impl<T, P> ErasedField<T, P>
where
    T: AlignedStorage,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    #[inline]
    pub unsafe fn downcast<V>(self) -> Result<V, DowncastError<Self>> {
        let layout = self.descriptor().layout();
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
    pub fn into_parts(self) -> (AlignedInitStorage<T>, FieldDescriptor) {
        let Self { storage, .. } = self;
        let desc = storage_descriptor(&storage);
        (storage, desc)
    }
}

impl<T, P> ErasedField<T, P>
where
    T: AlignedStorageFromLayout<Item: Copy>,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    #[inline]
    pub fn try_from_desc_data<V>(
        desc: FieldDescriptor,
        data: V,
    ) -> Result<Self, FromDescDataError<T::Error>>
    where
        V: AsRef<[T::Item]>,
    {
        let layout = desc.layout();
        check_sufficient_align(layout, Layout::new::<T::Item>())?;

        let data = data.as_ref();
        check_len(size_of_val(data), layout.size())?;

        let mut storage = T::from_layout(layout).map_err(FromDescDataError::FromLayout)?;
        write_copy_of_slice(storage.as_mut_uninit_slice(), data)?;

        let me = Self {
            storage: unsafe { AlignedInitStorage::new_unchecked(storage) },
            phantom: PhantomData,
        };
        Ok(me)
    }

    #[inline]
    pub fn try_from<V>(value: V) -> Result<Self, FromValueError<T::Error, V>> {
        let desc = FieldDescriptor::of::<V>();

        let data = ptr::from_ref(&value).cast();
        let len = item_count::<T::Item>(desc);
        let data = unsafe { slice::from_raw_parts(data, len) };

        match Self::try_from_desc_data(desc, data) {
            Ok(me) => {
                forget(value);
                Ok(me)
            }
            Err(FromDescDataError::LenMismatch(error)) => unreachable!("{error}"),
            Err(FromDescDataError::InsufficientAlign(error)) => {
                let error = FromValueError::new(error.into(), value);
                Err(error)
            }
            Err(FromDescDataError::FromLayout(error)) => {
                let reason = FromValueErrorKind::FromLayout(error);
                let error = FromValueError::new(reason, value);
                Err(error)
            }
        }
    }
}

impl<T, P> ErasedField<T, P>
where
    T: AlignedStorage + ?Sized,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    #[inline]
    pub fn descriptor(&self) -> FieldDescriptor {
        let Self { storage, .. } = self;
        storage_descriptor(storage)
    }

    #[inline]
    pub unsafe fn downcast_ref<V>(&self) -> Result<&V, DowncastError<&Self>> {
        let layout = self.descriptor().layout();
        let Self { storage, .. } = check_downcast::<V, _>(layout, self)?;

        let ptr = storage.as_ptr().cast();
        Ok(unsafe { &*ptr })
    }

    #[inline]
    pub unsafe fn downcast_mut<V>(&mut self) -> Result<&mut V, DowncastError<&mut Self>> {
        let layout = self.descriptor().layout();
        let Self { storage, .. } = check_downcast::<V, _>(layout, self)?;

        let ptr = storage.as_mut_ptr().cast();
        Ok(unsafe { &mut *ptr })
    }

    #[inline]
    pub fn as_field(&self) -> ErasedFieldRef<'_, P::Const> {
        unsafe { self.as_field_ptr().deref() }
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldPtr<P::Const> {
        let Self { storage, .. } = self;

        let desc = storage_descriptor(storage);
        let buffer = storage.as_uninit_slice();
        let ptr = unsafe { ConstSliceItemPtr::from_slice(buffer, 0) };
        unsafe { ErasedFieldPtr::from_parts(desc, ptr) }
    }

    #[inline]
    pub fn as_mut_field(&mut self) -> ErasedFieldRefMut<'_, P::Mut> {
        unsafe { self.as_mut_field_ptr().deref_mut() }
    }

    #[inline]
    pub fn as_mut_field_ptr(&mut self) -> ErasedFieldMutPtr<P::Mut> {
        let Self { storage, .. } = self;

        let desc = storage_descriptor(storage);
        let buffer = storage.as_mut_uninit_slice();
        let ptr = unsafe { MutSliceItemPtr::from_slice(buffer, 0) };
        unsafe { ErasedFieldMutPtr::from_parts(desc, ptr) }
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

impl<T, P> Debug for ErasedField<T, P>
where
    T: AlignedStorage<Item: Debug> + ?Sized,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = &self.descriptor();
        let data = &self.as_slice();
        f.debug_struct("ErasedField")
            .field("desc", desc)
            .field("data", data)
            .finish()
    }
}

impl<T, P> AsRef<[T::Item]> for ErasedField<T, P>
where
    T: AlignedStorage + ?Sized,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    #[inline]
    fn as_ref(&self) -> &[T::Item] {
        self.as_slice()
    }
}

impl<T, P> AsMut<[T::Item]> for ErasedField<T, P>
where
    T: AlignedStorage + ?Sized,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T::Item] {
        self.as_mut_slice()
    }
}

#[inline]
fn storage_descriptor<T>(storage: &T) -> FieldDescriptor
where
    T: AlignedStorage + ?Sized,
{
    FieldDescriptor::new(storage.layout())
}
