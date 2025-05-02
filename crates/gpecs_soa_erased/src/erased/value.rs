use alloc::{boxed::Box, vec::Vec};
use core::{
    fmt::{self, Debug},
    ptr, slice,
};

use crate::{
    aligned_bytes::AlignedBytes,
    assert::{check_same_layout, check_same_len},
    error::LenMismatchError,
    field::{ErasedField, ErasedFieldRef, ErasedFieldRefMut},
    soa::{
        traits::{buffer_layout, Soa},
        FieldDescriptor,
    },
};

use super::{error::IntoValueError, ErasedSoaRefs, ErasedSoaRefsMut};

pub struct ErasedSoa {
    buffer: AlignedBytes,
    descriptors: Box<[FieldDescriptor]>,
}

impl ErasedSoa {
    #[inline]
    pub fn new<I, F>(fields: I) -> Result<Self, LenMismatchError>
    where
        I: IntoIterator<Item = (FieldDescriptor, F)>,
        F: AsRef<[u8]>,
    {
        let fields = fields
            .into_iter()
            .map(|(desc, src)| {
                check_same_len(src.as_ref().len(), desc.layout().size())?;
                Ok((desc, src))
            })
            .collect::<Result<Box<_>, _>>()?;
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

        let mut buffer = AlignedBytes::new(buffer_layout);
        for ((desc, src), offset) in descriptors.iter().zip(fields).zip(offsets) {
            let src = src.as_ref().as_ptr();
            let dst = unsafe { buffer.as_mut_ptr().cast::<u8>().add(offset) };

            let len = desc.layout().size();
            unsafe {
                ptr::copy_nonoverlapping(src, dst, len);
            }
        }
        Self {
            buffer,
            descriptors,
        }
    }

    #[inline]
    pub fn from<T>(context: &T::Context, value: T) -> Self
    where
        T: Soa,
    {
        let descriptors = T::field_descriptors(context)
            .into_iter()
            .map(|desc| desc.as_ref().clone())
            .collect();

        let (buffer_layout, offsets) =
            T::buffer_layout(context, 1).expect("layout size should not exceed `isize::MAX`");

        let mut buffer = AlignedBytes::new(buffer_layout);
        unsafe {
            let dst = {
                let buffer = buffer.as_mut_ptr().cast::<u8>();
                let ptrs = offsets.into_iter().map(|offset| buffer.add(offset));
                T::ptrs_restore_mut(context, ptrs)
            };
            T::ptrs_write(context, dst, value);
        }

        Self {
            buffer,
            descriptors,
        }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> Result<T, IntoValueError<Self>>
    where
        T: Soa,
    {
        let Self {
            buffer,
            descriptors,
            ..
        } = &self;
        let result = T::field_descriptors(context)
            .into_iter()
            .zip(descriptors)
            .try_fold(0, |len, (desc, self_desc)| {
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
        let buffer_len = buffer_layout.size();
        if let Err(error) = check_same_len(buffer_len, buffer.layout().size()) {
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
    pub fn into_fields(self) -> Box<[ErasedField]> {
        let Self {
            buffer,
            descriptors,
            ..
        } = self;

        let field_layouts = descriptors.iter().map(FieldDescriptor::layout);
        let (buffer_layout, offsets): (_, Box<[_]>) =
            buffer_layout(field_layouts, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout.size();
        check_same_len(buffer_len, buffer.layout().size()).expect("buffer length should match");

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
    pub fn as_refs(&self) -> ErasedSoaRefs<'_> {
        let Self {
            buffer,
            descriptors,
            ..
        } = self;

        let field_layouts = descriptors.iter().map(FieldDescriptor::layout);
        let (buffer_layout, offsets): (_, Box<[_]>) =
            buffer_layout(field_layouts, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout.size();
        check_same_len(buffer_len, buffer.layout().size()).expect("buffer length should match");

        let refs = descriptors.iter().zip(offsets).map(|(desc, offset)| {
            let data = unsafe { buffer.as_ptr().cast::<u8>().add(offset) };
            let len = desc.layout().size();
            let r#ref = unsafe { slice::from_raw_parts(data, len) };
            unsafe { ErasedFieldRef::new_unchecked(desc.clone(), r#ref) }
        });
        ErasedSoaRefs::new(refs)
    }

    #[inline]
    pub fn as_refs_mut(&mut self) -> ErasedSoaRefsMut<'_> {
        let Self {
            buffer,
            descriptors,
            ..
        } = self;

        let field_layouts = descriptors.iter().map(FieldDescriptor::layout);
        let (buffer_layout, offsets): (_, Box<[_]>) =
            buffer_layout(field_layouts, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout.size();
        check_same_len(buffer_len, buffer.layout().size()).expect("buffer length should match");

        let refs = descriptors.iter().zip(offsets).map(|(desc, offset)| {
            let data = unsafe { buffer.as_mut_ptr().cast::<u8>().add(offset) };
            let len = desc.layout().size();
            let r#ref = unsafe { slice::from_raw_parts_mut(data, len) };
            unsafe { ErasedFieldRefMut::new_unchecked(desc.clone(), r#ref) }
        });
        ErasedSoaRefsMut::new(refs)
    }
}

impl Debug for ErasedSoa {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let refs = self.as_refs();
        let fields = &refs.field_refs();
        f.debug_struct("ErasedSoa").field("fields", fields).finish()
    }
}
