use alloc::{boxed::Box, vec::Vec};
use core::{ptr, slice};

use crate::{
    align::Aligned,
    assert::{check_same_layout, check_same_len, validate_layout},
    byte::ErasedByte,
    error::InvalidLayoutError,
    field::{ErasedField, ErasedFieldRef, ErasedFieldRefMut},
    soa::traits::{buffer_layout, FieldDescriptor, Soa},
};

use super::{
    error::{ErasedSoaError, FromValueError, IntoValueError},
    ErasedSoaRefs, ErasedSoaRefsMut,
};

pub struct ErasedSoa<Fields> {
    buffer: Box<[ErasedByte<Aligned<Fields>>]>,
    descriptors: Box<[FieldDescriptor]>,
}

impl<Fields> ErasedSoa<Fields> {
    #[inline]
    pub fn new<I, F>(fields: I) -> Result<Self, ErasedSoaError>
    where
        I: IntoIterator<Item = (FieldDescriptor, F)>,
        F: AsRef<[u8]>,
    {
        let fields = fields
            .into_iter()
            .map(|(desc, src)| {
                validate_layout::<Fields>(desc.layout())?;
                check_same_len(src.as_ref().len(), desc.layout().size())?;
                Ok((desc, src))
            })
            .collect::<Result<Box<_>, ErasedSoaError>>()?;
        let me = unsafe { Self::actual_new(fields) };
        Ok(me)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked<I, F>(fields: I) -> Self
    where
        I: IntoIterator<Item = (FieldDescriptor, F)>,
        F: AsRef<[u8]>,
    {
        if cfg!(debug_assertions) {
            return Self::new(fields).expect("incorrect inputs");
        }
        unsafe { Self::actual_new(fields) }
    }

    #[inline]
    unsafe fn actual_new<I, F>(fields: I) -> Self
    where
        I: IntoIterator<Item = (FieldDescriptor, F)>,
        F: AsRef<[u8]>,
    {
        let (descriptors, fields): (Vec<_>, Vec<_>) = fields.into_iter().unzip();
        let descriptors = descriptors.into_boxed_slice();

        let (buffer_layout, offsets): (_, Box<[_]>) =
            buffer_layout(descriptors.iter().map(FieldDescriptor::layout), 1)
                .expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Aligned<Fields>>>());

        let mut buffer = Box::new_uninit_slice(buffer_len);
        for ((desc, src), offset) in descriptors.iter().zip(fields).zip(offsets) {
            let src = src.as_ref().as_ptr();
            let dst = unsafe { buffer.as_mut_ptr().cast::<u8>().add(offset) };

            let len = desc.layout().size();
            unsafe {
                ptr::copy_nonoverlapping(src, dst, len);
            }
        }
        Self {
            buffer: unsafe { buffer.assume_init() },
            descriptors,
        }
    }

    #[inline]
    pub fn from<T>(context: &T::Context, value: T) -> Result<Self, FromValueError<T>>
    where
        T: Soa<Fields = Fields>,
    {
        let descriptors = T::field_descriptors(context)
            .into_iter()
            .map(|desc| {
                validate_layout::<Fields>(desc.as_ref().layout())?;
                Ok(desc.as_ref().clone())
            })
            .collect::<Result<Box<[_]>, InvalidLayoutError>>();
        let descriptors = match descriptors {
            Ok(descriptors) => descriptors,
            Err(error) => return Err(FromValueError::new(value, error)),
        };

        let (buffer_layout, offsets) =
            T::buffer_layout(context, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Aligned<Fields>>>());

        let mut buffer = Box::new_uninit_slice(buffer_len);
        unsafe {
            let dst = {
                let buffer = buffer.as_mut_ptr().cast::<u8>();
                let ptrs = offsets.into_iter().map(|offset| buffer.add(offset));
                T::ptrs_restore_mut(context, ptrs)
            };
            T::ptrs_write(context, dst, value);
        }

        Ok(Self {
            buffer: unsafe { buffer.assume_init() },
            descriptors,
        })
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> Result<T, IntoValueError<Self>>
    where
        T: Soa<Fields = Fields>,
    {
        let Self {
            buffer,
            descriptors,
        } = &self;
        let result = T::field_descriptors(context)
            .into_iter()
            .zip(descriptors)
            .try_fold(0, |len, (desc, self_desc)| {
                validate_layout::<T::Fields>(desc.as_ref().layout())?;
                check_same_layout(self_desc.layout(), desc.as_ref().layout())?;
                Ok(len + 1)
            })
            .and_then(|len| {
                check_same_len(len, descriptors.len())?;
                Ok(())
            });
        if let Err(error) = result {
            return Err(IntoValueError::new(self, error));
        }

        let (buffer_layout, offsets) =
            T::buffer_layout(context, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Aligned<Fields>>>());
        if let Err(error) = check_same_len(buffer_len, buffer.len()) {
            return Err(IntoValueError::new(self, error.into()));
        }

        let value = unsafe {
            let src = {
                let buffer = buffer.as_ptr().cast::<u8>();
                let ptrs = offsets.into_iter().map(|offset| buffer.add(offset));
                T::ptrs_restore(context, ptrs)
            };
            T::ptrs_read(context, src)
        };
        Ok(value)
    }

    #[inline]
    pub fn into_fields(self) -> Box<[ErasedField<Aligned<Fields>>]> {
        let Self {
            buffer,
            descriptors,
        } = self;

        let field_layouts = descriptors.iter().map(FieldDescriptor::layout);
        let (buffer_layout, offsets): (_, Box<[_]>) =
            buffer_layout(field_layouts, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Aligned<Fields>>>());
        check_same_len(buffer_len, buffer.len()).expect("buffer length should match");

        descriptors
            .into_vec()
            .into_iter()
            .zip(offsets)
            .map(|(desc, offset)| {
                let data = unsafe { buffer.as_ptr().cast::<u8>().add(offset) };
                let buffer = unsafe { slice::from_raw_parts(data, desc.layout().size()) };
                unsafe { ErasedField::new_unchecked(desc, buffer) }
            })
            .collect()
    }

    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_ref()
    }

    #[inline]
    pub fn as_refs(&self) -> ErasedSoaRefs<'_, Fields> {
        let Self {
            buffer,
            descriptors,
        } = self;

        let field_layouts = descriptors.iter().map(FieldDescriptor::layout);
        let (buffer_layout, offsets): (_, Box<[_]>) =
            buffer_layout(field_layouts, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Aligned<Fields>>>());
        check_same_len(buffer_len, buffer.len()).expect("buffer length should match");

        let refs = descriptors.iter().zip(offsets).map(|(desc, offset)| {
            let data = unsafe { buffer.as_ptr().cast::<u8>().add(offset) };
            let len = desc.layout().size();
            let r#ref = unsafe { slice::from_raw_parts(data, len) };
            unsafe { ErasedFieldRef::new_unchecked(desc.clone(), r#ref) }
        });
        unsafe { ErasedSoaRefs::new_unchecked(refs) }
    }

    #[inline]
    pub fn as_refs_mut(&mut self) -> ErasedSoaRefsMut<'_, Fields> {
        let Self {
            buffer,
            descriptors,
        } = self;

        let field_layouts = descriptors.iter().map(FieldDescriptor::layout);
        let (buffer_layout, offsets): (_, Box<[_]>) =
            buffer_layout(field_layouts, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Aligned<Fields>>>());
        check_same_len(buffer_len, buffer.len()).expect("buffer length should match");

        let refs = descriptors.iter().zip(offsets).map(|(desc, offset)| {
            let data = unsafe { buffer.as_mut_ptr().cast::<u8>().add(offset) };
            let len = desc.layout().size();
            let r#ref = unsafe { slice::from_raw_parts_mut(data, len) };
            unsafe { ErasedFieldRefMut::new_unchecked(desc.clone(), r#ref) }
        });
        unsafe { ErasedSoaRefsMut::new_unchecked(refs) }
    }
}
