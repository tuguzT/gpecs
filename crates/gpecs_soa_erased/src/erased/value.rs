use alloc::{boxed::Box, vec::Vec};
use core::{
    fmt::{self, Debug},
    ptr, slice,
};

use crate::{
    aligned_bytes::{AlignedBoxedByteSlice, AllocError},
    erased::{ErasedSoaRefs, ErasedSoaRefsMut, error::IntoValueError},
    error::{LenMismatchError, check_layout, check_len},
    field::{BoxedErasedField, error::ErasedFieldFromDescError},
    soa::{
        traits::{FieldDescriptor, Soa, buffer_layout, buffer_offsets},
        vec::SoaVec,
    },
};

pub type ErasedSoaVec = SoaVec<ErasedSoa>;

pub struct ErasedSoa {
    buffer: AlignedBoxedByteSlice,
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
                check_len(src.as_ref().len(), desc.layout().size())?;
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

        let layout = buffer_layout(&descriptors, 1)
            .expect("buffer layout size should not exceed `isize::MAX`");
        let mut buffer = AlignedBoxedByteSlice::new(layout).unwrap();

        let offsets = buffer_offsets(&descriptors, 1).map(Result::unwrap);
        for ((desc, src), offset) in descriptors.iter().zip(fields).zip(offsets) {
            let src = src.as_ref().as_ptr();
            let dst = unsafe { buffer.as_mut_ptr().add(offset) };

            let len = desc.layout().size();
            unsafe { ptr::copy_nonoverlapping(src, dst, len) }
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
        let descriptors: Box<[_]> = T::field_descriptors(context)
            .into_iter()
            .map(|desc| *desc.as_ref())
            .collect();

        let layout = T::buffer_layout(context, 1)
            .expect("buffer layout size should not exceed `isize::MAX`");
        let mut buffer = AlignedBoxedByteSlice::new(layout).unwrap();

        unsafe {
            let dst = T::ptrs_from_buffer(context, buffer.as_mut_ptr(), 1);
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
        } = &self;
        let result = T::field_descriptors(context)
            .into_iter()
            .zip(descriptors)
            .try_fold(0, |len, (desc, self_desc)| {
                check_layout(self_desc.layout(), desc.as_ref().layout())?;
                Ok(len + 1)
            })
            .and_then(|len| {
                check_len(len, descriptors.len())?;
                Ok(())
            });
        if let Err(error) = result {
            return Err(IntoValueError::new(self, error));
        }

        let layout = T::buffer_layout(context, 1)
            .expect("buffer layout size should not exceed `isize::MAX`");
        if let Err(error) = check_len(layout.size(), buffer.layout().size()) {
            return Err(IntoValueError::new(self, error.into()));
        }

        let Self { mut buffer, .. } = self;
        let value = unsafe {
            let src = T::ptrs_from_buffer(context, buffer.as_mut_ptr(), 1);
            T::ptrs_read(context, T::ptrs_cast_const(context, src))
        };
        Ok(value)
    }

    #[inline]
    pub fn into_fields(self) -> Result<Box<[BoxedErasedField]>, AllocError> {
        let Self {
            buffer,
            ref descriptors,
        } = self;

        let layout = buffer_layout(descriptors, 1)
            .expect("buffer layout size should not exceed `isize::MAX`");
        check_len(layout.size(), buffer.layout().size()).expect("buffer length should match");

        let offsets = buffer_offsets(descriptors, 1).map(Result::unwrap);
        descriptors
            .iter()
            .zip(offsets)
            .map(|(&desc, offset)| {
                let data = unsafe { buffer.as_ptr().add(offset) };
                let data = unsafe { slice::from_raw_parts(data, desc.layout().size()) };
                match BoxedErasedField::from_desc(desc, data) {
                    Ok(field) => Ok(field),
                    Err(ErasedFieldFromDescError::FromDesc(err)) => Err(err),
                    Err(ErasedFieldFromDescError::LenMismatch(err)) => unreachable!("{err}"),
                }
            })
            .collect()
    }

    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_ref()
    }

    #[inline]
    pub fn as_refs(&self) -> ErasedSoaRefs<'_, '_> {
        let Self {
            buffer,
            descriptors,
        } = self;
        unsafe { ErasedSoaRefs::new_unchecked(descriptors, buffer.as_ptr(), 1, 0) }
    }

    #[inline]
    pub fn as_refs_mut(&mut self) -> ErasedSoaRefsMut<'_, '_> {
        let &mut Self {
            ref mut buffer,
            ref descriptors,
        } = self;
        unsafe { ErasedSoaRefsMut::new_unchecked(descriptors, buffer.as_mut_ptr(), 1, 0) }
    }
}

impl Debug for ErasedSoa {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = &self.as_refs().into_iter();
        f.debug_struct("ErasedSoa").field("fields", fields).finish()
    }
}
