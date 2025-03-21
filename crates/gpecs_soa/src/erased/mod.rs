use alloc::{
    boxed::Box,
    vec::{self, Vec},
};
use core::{
    alloc::{Layout, LayoutError},
    iter,
    ptr::{self, NonNull},
};

use crate::traits::{buffer_layout, FieldDescriptor, Soa};

use self::{
    assert::assert_layouts,
    byte::ErasedByte,
    field::{
        ErasedFieldMutPtr, ErasedFieldNonNullPtr, ErasedFieldPtr, ErasedFieldSliceMutPtr,
        ErasedFieldSlicePtr,
    },
};

pub use self::{
    context::ErasedSoaContext,
    nonnull_ptrs::ErasedSoaNonNullPtrs,
    ptrs::ErasedSoaPtrs,
    ptrs_mut::ErasedSoaMutPtrs,
    refs::ErasedSoaRefs,
    refs_mut::ErasedSoaRefsMut,
    slice_ptrs::{ErasedSoaSlicePtrs, ErasedSoaSlicePtrsIter},
    slice_ptrs_mut::{ErasedSoaSliceMutPtrs, ErasedSoaSliceMutPtrsIter},
    slices::{ErasedSoaSlices, ErasedSoaSlicesIter},
    slices_mut::{ErasedSoaSlicesIterMut, ErasedSoaSlicesMut},
    value::ErasedSoa,
    vecs::{ErasedFieldVec, ErasedSoaVecs},
};

pub mod field;

mod assert;
mod byte;
mod context;
mod nonnull_ptrs;
mod ptrs;
mod ptrs_mut;
mod refs;
mod refs_mut;
mod slice_ptrs;
mod slice_ptrs_mut;
mod slices;
mod slices_mut;
mod value;
mod vecs;

unsafe impl<Fields> Soa for ErasedSoa<Fields> {
    type Context = ErasedSoaContext<Fields>;

    type Fields = Fields;

    type FieldDescriptors<'a> = &'a [FieldDescriptor];

    fn field_descriptors(context: &Self::Context) -> Self::FieldDescriptors<'_> {
        context.field_descriptors()
    }

    type FieldOffsets<'a> = Box<[usize]>;

    fn buffer_layout(
        context: &Self::Context,
        capacity: usize,
    ) -> Result<(Layout, Self::FieldOffsets<'_>), LayoutError> {
        let field_layouts = context
            .field_descriptors()
            .iter()
            .map(FieldDescriptor::layout);
        buffer_layout(field_layouts, capacity)
    }

    type Ptrs = ErasedSoaPtrs<Fields>;
    type MutPtrs = ErasedSoaMutPtrs<Fields>;

    type ErasedPtrs = iter::Map<vec::IntoIter<ErasedFieldPtr>, fn(ErasedFieldPtr) -> *const u8>;
    type ErasedMutPtrs =
        iter::Map<vec::IntoIter<ErasedFieldMutPtr>, fn(ErasedFieldMutPtr) -> *mut u8>;

    fn ptrs_erase(context: &Self::Context, ptrs: Self::Ptrs) -> Self::ErasedPtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), ptrs.fields().len());

        ptrs.into_fields()
            .into_vec()
            .into_iter()
            .map(ErasedFieldPtr::into_ptr)
    }

    fn ptrs_erase_mut(context: &Self::Context, ptrs: Self::MutPtrs) -> Self::ErasedMutPtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), ptrs.fields().len());

        ptrs.into_fields()
            .into_vec()
            .into_iter()
            .map(ErasedFieldMutPtr::into_ptr)
    }

    fn ptrs_restore(
        context: &Self::Context,
        ptrs: impl IntoIterator<Item = *const u8>,
    ) -> Self::Ptrs {
        let descriptors = context.field_descriptors();

        let ptrs: Box<[_]> = descriptors
            .iter()
            .zip(ptrs)
            .map(|(desc, ptr)| {
                let buffer = ptr::slice_from_raw_parts(ptr, desc.layout().size());
                ErasedFieldPtr::new(*desc, buffer)
            })
            .collect();
        assert_eq!(descriptors.len(), ptrs.len());

        ErasedSoaPtrs::new(ptrs)
    }

    fn ptrs_restore_mut(
        context: &Self::Context,
        ptrs: impl IntoIterator<Item = *mut u8>,
    ) -> Self::MutPtrs {
        let descriptors = context.field_descriptors();

        let ptrs: Box<[_]> = descriptors
            .iter()
            .zip(ptrs)
            .map(|(desc, ptr)| {
                let buffer = ptr::slice_from_raw_parts_mut(ptr, desc.layout().size());
                ErasedFieldMutPtr::new(*desc, buffer)
            })
            .collect();
        assert_eq!(descriptors.len(), ptrs.len());

        ErasedSoaMutPtrs::new(ptrs)
    }

    fn ptrs_dangling(context: &Self::Context) -> Self::MutPtrs {
        let ptrs = context
            .field_descriptors()
            .iter()
            .copied()
            .map(ErasedFieldMutPtr::dangling);
        ErasedSoaMutPtrs::new(ptrs)
    }

    fn ptrs_cast_const(context: &Self::Context, ptrs: Self::MutPtrs) -> Self::Ptrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), ptrs.fields().len());

        let ptrs = descriptors
            .iter()
            .zip(ptrs.into_fields())
            .inspect(|(desc, ptr)| assert_layouts(desc.layout(), ptr.descriptor().layout()))
            .map(|(_, ptr)| ptr.cast_const());
        ErasedSoaPtrs::new(ptrs)
    }

    fn ptrs_cast_mut(context: &Self::Context, ptrs: Self::Ptrs) -> Self::MutPtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), ptrs.fields().len());

        let ptrs = descriptors
            .iter()
            .zip(ptrs.into_fields())
            .inspect(|(desc, ptr)| assert_layouts(desc.layout(), ptr.descriptor().layout()))
            .map(|(_, ptr)| ptr.cast_mut());
        ErasedSoaMutPtrs::new(ptrs)
    }

    unsafe fn ptrs_add(context: &Self::Context, ptrs: Self::Ptrs, offset: usize) -> Self::Ptrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), ptrs.fields().len());

        let ptrs = descriptors
            .iter()
            .zip(ptrs.into_fields())
            .inspect(|(desc, ptr)| assert_layouts(desc.layout(), ptr.descriptor().layout()))
            .map(|(_, ptr)| unsafe { ptr.add(offset) });
        ErasedSoaPtrs::new(ptrs)
    }

    unsafe fn ptrs_add_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
        offset: usize,
    ) -> Self::MutPtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), ptrs.fields().len());

        let ptrs = descriptors
            .iter()
            .zip(ptrs.into_fields())
            .inspect(|(desc, ptr)| assert_layouts(desc.layout(), ptr.descriptor().layout()))
            .map(|(_, ptr)| unsafe { ptr.add(offset) });
        ErasedSoaMutPtrs::new(ptrs)
    }

    unsafe fn ptrs_offset_from(
        context: &Self::Context,
        ptrs: Self::Ptrs,
        origin: Self::Ptrs,
    ) -> isize {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), ptrs.fields().len());
        assert_eq!(ptrs.fields().len(), origin.fields().len());

        let mut offsets = descriptors
            .iter()
            .zip(ptrs.into_fields())
            .zip(origin.into_fields())
            .inspect(|((desc, ptr), origin)| {
                assert_layouts(desc.layout(), ptr.descriptor().layout());
                assert_layouts(desc.layout(), origin.descriptor().layout());
            })
            .map(|((_, ptr), origin)| unsafe { ptr.offset_from(origin) });

        let offset = offsets.next().expect("self should not be a ZST");
        assert!(offsets.all(|item| item == offset));
        offset
    }

    unsafe fn ptrs_offset_from_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
        origin: Self::Ptrs,
    ) -> isize {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), ptrs.fields().len());
        assert_eq!(ptrs.fields().len(), origin.fields().len());

        let mut offsets = descriptors
            .iter()
            .zip(ptrs.into_fields())
            .zip(origin.into_fields())
            .inspect(|((desc, ptr), origin)| {
                assert_layouts(desc.layout(), ptr.descriptor().layout());
                assert_layouts(desc.layout(), origin.descriptor().layout());
            })
            .map(|((_, ptr), origin)| unsafe { ptr.offset_from(origin) });

        let offset = offsets.next().expect("self should not be a ZST");
        assert!(offsets.all(|item| item == offset));
        offset
    }

    unsafe fn ptrs_swap(context: &Self::Context, a: Self::MutPtrs, b: Self::MutPtrs) {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), a.fields().len());
        assert_eq!(a.fields().len(), b.fields().len());

        let mut temp = Vec::new();
        descriptors
            .iter()
            .zip(a.into_fields())
            .zip(b.into_fields())
            .inspect(|((desc, a), b)| {
                assert_layouts(desc.layout(), a.descriptor().layout());
                assert_layouts(desc.layout(), b.descriptor().layout());
            })
            .for_each(|((_, a), b)| {
                let count = a.descriptor().layout().size();
                temp.reserve(count);

                unsafe {
                    temp.set_len(count);
                    a.swap(b, &mut temp);
                }
                temp.clear();
            })
    }

    unsafe fn ptrs_copy(context: &Self::Context, src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), src.fields().len());
        assert_eq!(src.fields().len(), dst.fields().len());

        let mut temp = Vec::new();
        descriptors
            .iter()
            .zip(src.into_fields())
            .zip(dst.into_fields())
            .inspect(|((desc, src), dst)| {
                assert_layouts(desc.layout(), src.descriptor().layout());
                assert_layouts(desc.layout(), dst.descriptor().layout());
            })
            .for_each(|((_, src), dst)| {
                let count = len * src.descriptor().layout().size();
                temp.reserve(count);

                unsafe {
                    temp.set_len(count);
                    dst.copy_from(src, len, &mut temp);
                }
                temp.clear();
            })
    }

    unsafe fn ptrs_copy_rev(
        context: &Self::Context,
        src: Self::Ptrs,
        dst: Self::MutPtrs,
        len: usize,
    ) {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), src.fields().len());
        assert_eq!(src.fields().len(), dst.fields().len());

        let mut temp = Vec::new();
        descriptors
            .iter()
            .zip(src.into_fields())
            .zip(dst.into_fields())
            .rev()
            .inspect(|((desc, src), dst)| {
                assert_layouts(desc.layout(), src.descriptor().layout());
                assert_layouts(desc.layout(), dst.descriptor().layout());
            })
            .for_each(|((_, src), dst)| {
                let count = len * src.descriptor().layout().size();
                temp.reserve(count);

                unsafe {
                    temp.set_len(count);
                    dst.copy_from(src, len, &mut temp);
                }
                temp.clear();
            })
    }

    unsafe fn ptrs_copy_nonoverlapping(
        context: &Self::Context,
        src: Self::Ptrs,
        dst: Self::MutPtrs,
        len: usize,
    ) {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), src.fields().len());
        assert_eq!(src.fields().len(), dst.fields().len());

        descriptors
            .iter()
            .zip(src.into_fields())
            .zip(dst.into_fields())
            .inspect(|((desc, src), dst)| {
                assert_layouts(desc.layout(), src.descriptor().layout());
                assert_layouts(desc.layout(), dst.descriptor().layout());
            })
            .for_each(|((_, src), dst)| unsafe { dst.copy_from_nonoverlapping(src, len) })
    }

    unsafe fn ptrs_read(context: &Self::Context, src: Self::Ptrs) -> Self {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), src.fields().len());

        let fields = descriptors
            .iter()
            .zip(src.into_fields())
            .inspect(|(desc, src)| assert_layouts(desc.layout(), src.descriptor().layout()))
            .map(|(&field_layout, src)| (field_layout, unsafe { src.deref().into_buffer() }));
        Self::new(fields)
    }

    unsafe fn ptrs_write(context: &Self::Context, dst: Self::MutPtrs, value: Self) {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), dst.fields().len());
        assert!(value
            .field_descriptors()
            .iter()
            .map(FieldDescriptor::layout)
            .eq(descriptors.iter().map(FieldDescriptor::layout)));

        descriptors
            .iter()
            .zip(dst.into_fields())
            .zip(value.as_refs().into_fields())
            .inspect(|((desc, dst), _)| assert_layouts(desc.layout(), dst.descriptor().layout()))
            .for_each(|((_, dst), src)| {
                let src = src.as_field_ptr();
                unsafe { dst.copy_from_nonoverlapping(src, 1) }
            })
    }

    unsafe fn ptrs_drop_in_place(context: &Self::Context, ptrs: Self::MutPtrs) {
        let iter = iter::once(ptrs.fields());
        context.drop_in_place(iter);
    }

    type NonNullPtrs = ErasedSoaNonNullPtrs<Fields>;

    unsafe fn ptrs_to_nonnull(context: &Self::Context, ptrs: Self::MutPtrs) -> Self::NonNullPtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), ptrs.fields().len());

        let ptrs = descriptors
            .iter()
            .zip(ptrs.into_fields())
            .inspect(|(desc, ptr)| assert_layouts(desc.layout(), ptr.descriptor().layout()))
            .map(|(_, ptr)| {
                let desc = ptr.descriptor();
                let buffer = unsafe { NonNull::new_unchecked(ptr.buffer()) };
                ErasedFieldNonNullPtr::new(desc, buffer)
            });
        ErasedSoaNonNullPtrs::new(ptrs)
    }

    fn nonnull_to_ptrs(context: &Self::Context, ptrs: Self::NonNullPtrs) -> Self::MutPtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), ptrs.fields().len());

        let ptrs = descriptors
            .iter()
            .zip(ptrs.into_fields())
            .inspect(|(desc, ptr)| assert_layouts(desc.layout(), ptr.descriptor().layout()))
            .map(|(_, ptr)| {
                let desc = ptr.descriptor();
                let buffer = ptr.buffer().as_ptr();
                ErasedFieldMutPtr::new(desc, buffer)
            });
        ErasedSoaMutPtrs::new(ptrs)
    }

    type Vecs = ErasedSoaVecs<Fields>;

    fn vecs_with_capacity(context: &Self::Context, capacity: usize) -> Self::Vecs {
        let vecs = context
            .field_descriptors()
            .iter()
            .map(|&desc| {
                let capacity =
                    (capacity * desc.layout().size()).div_ceil(size_of::<ErasedByte<Fields>>());
                let buffer = Vec::with_capacity(capacity);
                ErasedFieldVec { buffer, desc }
            })
            .collect();
        ErasedSoaVecs { len: 0, vecs }
    }

    fn vecs_as_ptrs(context: &Self::Context, vecs: &Self::Vecs) -> Self::Ptrs {
        let descriptors = context.field_descriptors();
        let ErasedSoaVecs { vecs, .. } = vecs;
        assert_eq!(descriptors.len(), vecs.len());

        let ptrs = descriptors
            .iter()
            .zip(vecs)
            .inspect(|(desc, vec)| assert_layouts(desc.layout(), vec.desc.layout()))
            .map(|(_, vec)| {
                let ErasedFieldVec { buffer, desc, .. } = vec;

                let data = buffer.as_ptr().cast();
                let len = desc.layout().size();
                let buffer = ptr::slice_from_raw_parts(data, len);
                ErasedFieldPtr::new(*desc, buffer)
            });
        ErasedSoaPtrs::new(ptrs)
    }

    fn mut_vecs_as_ptrs(context: &Self::Context, vecs: &mut Self::Vecs) -> Self::MutPtrs {
        let descriptors = context.field_descriptors();
        let ErasedSoaVecs { vecs, .. } = vecs;
        assert_eq!(descriptors.len(), vecs.len());

        let ptrs = descriptors
            .iter()
            .zip(vecs)
            .inspect(|(desc, vec)| assert_layouts(desc.layout(), vec.desc.layout()))
            .map(|(_, vec)| {
                let ErasedFieldVec { buffer, desc, .. } = vec;

                let data = buffer.as_mut_ptr().cast();
                let len = desc.layout().size();
                let buffer = ptr::slice_from_raw_parts_mut(data, len);
                ErasedFieldMutPtr::new(*desc, buffer)
            });
        ErasedSoaMutPtrs::new(ptrs)
    }

    fn vecs_len(context: &Self::Context, vecs: &Self::Vecs) -> usize {
        let descriptors = context.field_descriptors();
        let ErasedSoaVecs { ref vecs, len, .. } = *vecs;
        assert_eq!(descriptors.len(), vecs.len());

        len
    }

    unsafe fn vecs_set_len(context: &Self::Context, vecs: &mut Self::Vecs, len: usize) {
        let descriptors = context.field_descriptors();
        let ErasedSoaVecs {
            vecs,
            len: vecs_len,
            ..
        } = vecs;
        assert_eq!(descriptors.len(), vecs.len());

        descriptors
            .iter()
            .zip(vecs)
            .inspect(|(desc, vec)| assert_layouts(desc.layout(), vec.desc.layout()))
            .for_each(|(_, vec)| {
                let ErasedFieldVec { buffer, desc } = vec;
                let len = (len * desc.layout().size()).div_ceil(size_of::<ErasedByte<Fields>>());
                unsafe { buffer.set_len(len) }
            });
        *vecs_len = len;
    }

    type Refs<'a>
        = ErasedSoaRefs<'a, Fields>
    where
        Self: 'a;

    type RefsMut<'a>
        = ErasedSoaRefsMut<'a, Fields>
    where
        Self: 'a;

    unsafe fn ptrs_to_refs<'a>(context: &Self::Context, ptrs: Self::Ptrs) -> Self::Refs<'a> {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), ptrs.fields().len());

        let refs = descriptors
            .iter()
            .zip(ptrs.into_fields())
            .inspect(|(desc, ptr)| assert_layouts(desc.layout(), ptr.descriptor().layout()))
            .map(|(_, ptr)| unsafe { ptr.deref() });
        ErasedSoaRefs::new(refs)
    }

    unsafe fn ptrs_to_refs_mut<'a>(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
    ) -> Self::RefsMut<'a> {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), ptrs.fields().len());

        let refs = descriptors
            .iter()
            .zip(ptrs.into_fields())
            .inspect(|(desc, ptr)| assert_layouts(desc.layout(), ptr.descriptor().layout()))
            .map(|(_, ptr)| unsafe { ptr.deref_mut() });
        ErasedSoaRefsMut::new(refs)
    }

    fn refs_as_ptrs(context: &Self::Context, refs: Self::Refs<'_>) -> Self::Ptrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), refs.fields().len());

        let ptrs = descriptors
            .iter()
            .zip(refs.into_fields())
            .inspect(|(desc, r#ref)| assert_layouts(desc.layout(), r#ref.descriptor().layout()))
            .map(|(_, r#ref)| r#ref.as_field_ptr());
        ErasedSoaPtrs::new(ptrs)
    }

    fn mut_refs_as_ptrs(context: &Self::Context, refs: Self::RefsMut<'_>) -> Self::MutPtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), refs.fields().len());

        let ptrs = descriptors
            .iter()
            .zip(refs.into_fields())
            .inspect(|(desc, r#ref)| assert_layouts(desc.layout(), r#ref.descriptor().layout()))
            .map(|(_, mut r#ref)| r#ref.as_field_mut_ptr());
        ErasedSoaMutPtrs::new(ptrs)
    }

    fn mut_refs_as_refs<'a>(context: &Self::Context, refs: Self::RefsMut<'a>) -> Self::Refs<'a> {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), refs.fields().len());

        let refs = descriptors
            .iter()
            .zip(refs.into_fields())
            .inspect(|(desc, r#ref)| assert_layouts(desc.layout(), r#ref.descriptor().layout()))
            .map(|(_, r#ref)| From::from(r#ref));
        ErasedSoaRefs::new(refs)
    }

    type SlicePtrs = ErasedSoaSlicePtrs<Fields>;

    type SliceMutPtrs = ErasedSoaSliceMutPtrs<Fields>;

    fn slices_from_raw_parts(
        context: &Self::Context,
        ptrs: Self::Ptrs,
        len: usize,
    ) -> Self::SlicePtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), ptrs.fields().len());

        let slices = descriptors
            .iter()
            .zip(ptrs.into_fields())
            .inspect(|(desc, ptr)| assert_layouts(desc.layout(), ptr.descriptor().layout()))
            .map(|(_, ptr)| {
                let desc = ptr.descriptor();
                let len = len * desc.layout().size();
                let buffer = ptr::slice_from_raw_parts(ptr.as_ptr(), len);
                ErasedFieldSlicePtr::new(desc, buffer)
            });
        ErasedSoaSlicePtrs::new(len, slices)
    }

    fn slices_from_raw_parts_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
        len: usize,
    ) -> Self::SliceMutPtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), ptrs.fields().len());

        let slices = descriptors
            .iter()
            .zip(ptrs.into_fields())
            .inspect(|(desc, ptr)| assert_layouts(desc.layout(), ptr.descriptor().layout()))
            .map(|(_, ptr)| {
                let desc = ptr.descriptor();
                let len = len * desc.layout().size();
                let buffer = ptr::slice_from_raw_parts_mut(ptr.as_ptr(), len);
                ErasedFieldSliceMutPtr::new(desc, buffer)
            });
        ErasedSoaSliceMutPtrs::new(len, slices)
    }

    fn slice_ptrs_cast_const(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::SlicePtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.fields().len());

        let len = slices.len();
        let slices = descriptors
            .iter()
            .zip(slices.into_fields())
            .inspect(|(desc, slice)| assert_layouts(desc.layout(), slice.descriptor().layout()))
            .map(|(_, slice)| slice.cast_const());
        ErasedSoaSlicePtrs::new(len, slices)
    }

    fn slice_ptrs_cast_mut(context: &Self::Context, slices: Self::SlicePtrs) -> Self::SliceMutPtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.fields().len());

        let len = slices.len();
        let slices = descriptors
            .iter()
            .zip(slices.into_fields())
            .inspect(|(desc, slice)| assert_layouts(desc.layout(), slice.descriptor().layout()))
            .map(|(_, slice)| slice.cast_mut());
        ErasedSoaSliceMutPtrs::new(len, slices)
    }

    fn slice_ptrs_len(context: &Self::Context, slices: Self::SlicePtrs) -> usize {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.fields().len());

        slices.len()
    }

    fn slice_ptrs_len_mut(context: &Self::Context, slices: Self::SliceMutPtrs) -> usize {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.fields().len());

        slices.len()
    }

    fn slice_ptrs_as_ptrs(context: &Self::Context, slices: Self::SlicePtrs) -> Self::Ptrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.fields().len());

        let ptrs = descriptors
            .iter()
            .zip(slices.into_fields())
            .inspect(|(desc, slice)| assert_layouts(desc.layout(), slice.descriptor().layout()))
            .map(|(_, slice)| slice.as_field_ptr());
        ErasedSoaPtrs::new(ptrs)
    }

    fn mut_slice_ptrs_as_ptrs(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::MutPtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.fields().len());

        let ptrs = descriptors
            .iter()
            .zip(slices.into_fields())
            .inspect(|(desc, slice)| assert_layouts(desc.layout(), slice.descriptor().layout()))
            .map(|(_, slice)| slice.as_field_ptr());
        ErasedSoaMutPtrs::new(ptrs)
    }

    type Slices<'a>
        = ErasedSoaSlices<'a, Fields>
    where
        Self: 'a;

    type SlicesMut<'a>
        = ErasedSoaSlicesMut<'a, Fields>
    where
        Self: 'a;

    unsafe fn slice_ptrs_to_slices<'a>(
        context: &Self::Context,
        slices: Self::SlicePtrs,
    ) -> Self::Slices<'a> {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.fields().len());

        let len = slices.len();
        let slices = descriptors
            .iter()
            .zip(slices.into_fields())
            .inspect(|(desc, slice)| assert_layouts(desc.layout(), slice.descriptor().layout()))
            .map(|(_, slice)| unsafe { slice.deref() });
        ErasedSoaSlices::new(len, slices)
    }

    unsafe fn slice_ptrs_to_slices_mut<'a>(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::SlicesMut<'a> {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.fields().len());

        let len = slices.len();
        let slices = descriptors
            .iter()
            .zip(slices.into_fields())
            .inspect(|(desc, slice)| assert_layouts(desc.layout(), slice.descriptor().layout()))
            .map(|(_, slice)| unsafe { slice.deref_mut() });
        ErasedSoaSlicesMut::new(len, slices)
    }

    fn slices_len(context: &Self::Context, slices: &Self::Slices<'_>) -> usize {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.fields().len());

        slices.len()
    }

    fn slices_len_mut(context: &Self::Context, slices: &Self::SlicesMut<'_>) -> usize {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.fields().len());

        slices.len()
    }

    fn slice_refs_as_slice_ptrs(
        context: &Self::Context,
        slices: Self::Slices<'_>,
    ) -> Self::SlicePtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.fields().len());

        let len = slices.len();
        let slices = descriptors
            .iter()
            .zip(slices.into_fields())
            .inspect(|(desc, slice)| assert_layouts(desc.layout(), slice.descriptor().layout()))
            .map(|(_, slice)| slice.as_field_slice_ptr());
        ErasedSoaSlicePtrs::new(len, slices)
    }

    fn mut_slice_refs_as_slice_ptrs(
        context: &Self::Context,
        slices: Self::SlicesMut<'_>,
    ) -> Self::SliceMutPtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.fields().len());

        let len = slices.len();
        let slices = descriptors
            .iter()
            .zip(slices.into_fields())
            .inspect(|(desc, slice)| assert_layouts(desc.layout(), slice.descriptor().layout()))
            .map(|(_, mut slice)| slice.as_field_slice_mut_ptr());
        ErasedSoaSliceMutPtrs::new(len, slices)
    }

    fn mut_slices_as_slices<'a>(
        context: &Self::Context,
        slices: Self::SlicesMut<'a>,
    ) -> Self::Slices<'a> {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.fields().len());

        let len = slices.len();
        let slices = descriptors
            .iter()
            .zip(slices.into_fields())
            .inspect(|(desc, slice)| assert_layouts(desc.layout(), slice.descriptor().layout()))
            .map(|(_, slice)| From::from(slice));
        ErasedSoaSlices::new(len, slices)
    }

    fn slice_refs_as_ptrs(context: &Self::Context, slices: Self::Slices<'_>) -> Self::Ptrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.fields().len());

        let ptrs = descriptors
            .iter()
            .zip(slices.into_fields())
            .inspect(|(desc, slice)| assert_layouts(desc.layout(), slice.descriptor().layout()))
            .map(|(_, slice)| slice.as_field_ptr());
        ErasedSoaPtrs::new(ptrs)
    }

    fn mut_slice_refs_as_ptrs(
        context: &Self::Context,
        slices: Self::SlicesMut<'_>,
    ) -> Self::MutPtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.fields().len());

        let ptrs = descriptors
            .iter()
            .zip(slices.into_fields())
            .inspect(|(desc, slice)| assert_layouts(desc.layout(), slice.descriptor().layout()))
            .map(|(_, mut slice)| slice.as_field_mut_ptr());
        ErasedSoaMutPtrs::new(ptrs)
    }

    unsafe fn slices_drop_in_place(context: &Self::Context, slices: Self::SliceMutPtrs) {
        let iter = slices.into_iter().map(ErasedSoaMutPtrs::into_fields);
        context.drop_in_place(iter);
    }
}
