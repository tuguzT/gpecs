use core::{
    alloc::Layout,
    fmt::{self, Debug},
    marker::PhantomData,
    mem::{MaybeUninit, forget},
    ptr, slice,
};

use crate::{
    error::{check_layout, check_len, check_sufficient_align},
    field::{
        ErasedFieldMutPtr, ErasedFieldPtr, ErasedFieldRef, ErasedFieldRefMut,
        error::{
            ErasedFieldFromDescDataError, ErasedFieldFromStorageError, ErasedFieldFromValueError,
            ErasedFieldFromValueErrorKind, ErasedFieldIntoValueError, check_into_layout,
        },
    },
    fmt::SliceUpperHex,
    slice_item_ptr::{ConstSliceItemPtr, MutSliceItemPtr, SliceItemPtrs},
    soa::field::FieldDescriptor,
    storage::{AddressableUnit, AlignedInitStorage, AlignedStorage, AlignedStorageFromLayout},
    uninit::write_copy_of_slice,
};

#[cfg(feature = "alloc")]
use crate::storage::BoxedAlignedUninitStorage;

#[cfg(feature = "alloc")]
pub type BoxedErasedField<P> = ErasedField<BoxedAlignedUninitStorage, P, u8>;

pub struct ErasedField<T, P, A>
where
    T: ?Sized,
    A: AddressableUnit,
{
    phantom: PhantomData<fn() -> P>,
    storage: AlignedInitStorage<T, A>,
}

impl<T, P, A> ErasedField<T, P, A>
where
    T: AlignedStorage<A>,
    P: SliceItemPtrs<MaybeUninit<A>>,
    A: AddressableUnit,
{
    #[inline]
    pub fn try_from_storage_desc_data<V>(
        mut storage: T,
        desc: FieldDescriptor,
        data: V,
    ) -> Result<Self, ErasedFieldFromStorageError<T>>
    where
        V: AsRef<[A]>,
    {
        let layout = storage.layout();
        if let Err(err) = check_sufficient_align(layout, Layout::new::<A>()) {
            return Err(ErasedFieldFromStorageError::new(err.into(), storage));
        }

        let data = data.as_ref();
        let len = size_of_val(data);
        if let Err(err) = check_len(len, layout.size()) {
            return Err(ErasedFieldFromStorageError::new(err.into(), storage));
        }

        let expected_layout = desc.layout();
        if let Err(err) = check_len(len, expected_layout.size()) {
            return Err(ErasedFieldFromStorageError::new(err.into(), storage));
        }
        if let Err(err) = check_layout(layout, expected_layout) {
            return Err(ErasedFieldFromStorageError::new(err.into(), storage));
        }

        if let Err(err) = write_copy_of_slice(storage.as_mut_uninit_slice(), data) {
            return Err(ErasedFieldFromStorageError::new(err.into(), storage));
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
    ) -> Result<Self, ErasedFieldFromStorageError<(T, V)>> {
        let desc = FieldDescriptor::of::<V>();

        let data = ptr::from_ref(&value).cast();
        let len = desc.layout().size().div_ceil(size_of::<A>());
        let data = unsafe { slice::from_raw_parts(data, len) };

        match Self::try_from_storage_desc_data(storage, desc, data) {
            Ok(me) => {
                forget(value);
                Ok(me)
            }
            Err(err) => {
                let ErasedFieldFromStorageError { reason, storage } = err;
                let err = ErasedFieldFromStorageError::new(reason, (storage, value));
                Err(err)
            }
        }
    }

    #[inline]
    pub unsafe fn try_into<V>(self) -> Result<V, ErasedFieldIntoValueError<Self>> {
        let desc = self.descriptor();
        let me = check_into_layout::<V, _>(desc.layout(), self)?;
        let Self { storage, .. } = me;

        let src = storage.as_ptr().cast();
        Ok(unsafe { ptr::read(src) })
    }

    #[inline]
    pub fn into_storage(self) -> AlignedInitStorage<T, A> {
        let Self { storage, .. } = self;
        storage
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, AlignedInitStorage<T, A>) {
        let Self { storage, .. } = self;
        let desc = storage_descriptor(&storage);
        (desc, storage)
    }
}

impl<T, P, A> ErasedField<T, P, A>
where
    T: AlignedStorageFromLayout<A>,
    P: SliceItemPtrs<MaybeUninit<A>>,
    A: AddressableUnit,
{
    #[inline]
    pub fn try_from_desc_data<V>(
        desc: FieldDescriptor,
        data: V,
    ) -> Result<Self, ErasedFieldFromDescDataError<T, A>>
    where
        V: AsRef<[A]>,
    {
        let layout = desc.layout();
        check_sufficient_align(layout, Layout::new::<A>())?;

        let data = data.as_ref();
        check_len(size_of_val(data), layout.size())?;

        let mut storage =
            T::from_layout(layout).map_err(ErasedFieldFromDescDataError::FromLayout)?;
        write_copy_of_slice(storage.as_mut_uninit_slice(), data)?;

        let me = Self {
            storage: unsafe { AlignedInitStorage::new_unchecked(storage) },
            phantom: PhantomData,
        };
        Ok(me)
    }

    #[inline]
    pub fn try_from<V>(value: V) -> Result<Self, ErasedFieldFromValueError<T, V, A>> {
        let desc = FieldDescriptor::of::<V>();

        let data = ptr::from_ref(&value).cast();
        let len = desc.layout().size().div_ceil(size_of::<A>());
        let data = unsafe { slice::from_raw_parts(data, len) };

        match Self::try_from_desc_data(desc, data) {
            Ok(me) => {
                forget(value);
                Ok(me)
            }
            Err(ErasedFieldFromDescDataError::LenMismatch(error)) => unreachable!("{error}"),
            Err(ErasedFieldFromDescDataError::InsufficientAlign(error)) => {
                let error = ErasedFieldFromValueError::new(error.into(), value);
                Err(error)
            }
            Err(ErasedFieldFromDescDataError::FromLayout(error)) => {
                let reason = ErasedFieldFromValueErrorKind::FromLayout(error);
                let error = ErasedFieldFromValueError::new(reason, value);
                Err(error)
            }
        }
    }
}

impl<T, P, A> ErasedField<T, P, A>
where
    T: AlignedStorage<A> + ?Sized,
    P: SliceItemPtrs<MaybeUninit<A>>,
    A: AddressableUnit,
{
    #[inline]
    pub fn descriptor(&self) -> FieldDescriptor {
        let Self { storage, .. } = self;
        storage_descriptor(storage)
    }

    #[inline]
    pub unsafe fn cast<V>(&self) -> Result<&V, ErasedFieldIntoValueError<&Self>> {
        let desc = self.descriptor();
        let me = check_into_layout::<V, _>(desc.layout(), self)?;
        let Self { storage, .. } = me;

        let ptr = storage.as_ptr().cast();
        Ok(unsafe { &*ptr })
    }

    #[inline]
    pub unsafe fn cast_mut<V>(&mut self) -> Result<&mut V, ErasedFieldIntoValueError<&mut Self>> {
        let desc = self.descriptor();
        let me = check_into_layout::<V, _>(desc.layout(), self)?;
        let Self { storage, .. } = me;

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
    pub fn as_slice(&self) -> &[A] {
        let Self { storage, .. } = self;
        storage.as_slice()
    }

    #[inline]
    pub fn as_ptr(&self) -> *const A {
        let Self { storage, .. } = self;
        storage.as_ptr()
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [A] {
        let Self { storage, .. } = self;
        storage.as_mut_slice()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut A {
        let Self { storage, .. } = self;
        storage.as_mut_ptr()
    }
}

impl<T, P, A> Debug for ErasedField<T, P, A>
where
    T: AlignedStorage<A> + ?Sized,
    P: SliceItemPtrs<MaybeUninit<A>>,
    A: AddressableUnit,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = &self.descriptor();
        let data = &SliceUpperHex(self.as_slice());
        f.debug_struct("ErasedField")
            .field("desc", desc)
            .field("data", data)
            .finish()
    }
}

impl<T, P, A> AsRef<[A]> for ErasedField<T, P, A>
where
    T: AlignedStorage<A> + ?Sized,
    P: SliceItemPtrs<MaybeUninit<A>>,
    A: AddressableUnit,
{
    #[inline]
    fn as_ref(&self) -> &[A] {
        self.as_slice()
    }
}

impl<T, P, A> AsMut<[A]> for ErasedField<T, P, A>
where
    T: AlignedStorage<A> + ?Sized,
    P: SliceItemPtrs<MaybeUninit<A>>,
    A: AddressableUnit,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [A] {
        self.as_mut_slice()
    }
}

#[inline]
fn storage_descriptor<T, A>(storage: &T) -> FieldDescriptor
where
    A: AddressableUnit,
    T: AlignedStorage<A> + ?Sized,
{
    FieldDescriptor::new(storage.layout())
}
