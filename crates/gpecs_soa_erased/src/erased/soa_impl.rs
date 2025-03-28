use alloc::{
    boxed::Box,
    vec::{self, Vec},
};
use core::{
    alloc::{Layout, LayoutError},
    iter,
    ptr::{self, NonNull},
};

use crate::{
    align::Aligned,
    assert::check_same_layout,
    byte::ErasedByte,
    field::{
        field_slice_from_raw_parts, field_slice_from_raw_parts_mut, ErasedFieldMutPtr,
        ErasedFieldNonNullPtr, ErasedFieldPtr, ErasedFieldVec,
    },
    soa::traits::{buffer_layout, FieldDescriptor, Soa},
};

use super::{
    ErasedSoa, ErasedSoaContext, ErasedSoaMutPtrs, ErasedSoaNonNullPtrs, ErasedSoaPtrs,
    ErasedSoaRefs, ErasedSoaRefsMut, ErasedSoaSliceMutPtrs, ErasedSoaSlicePtrs, ErasedSoaSlices,
    ErasedSoaSlicesMut, ErasedSoaVecs,
};

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
        assert_eq!(descriptors.len(), ptrs.field_ptrs().len());

        ptrs.into_field_ptrs()
            .into_vec()
            .into_iter()
            .map(ErasedFieldPtr::into_ptr)
    }

    fn ptrs_erase_mut(context: &Self::Context, ptrs: Self::MutPtrs) -> Self::ErasedMutPtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), ptrs.field_ptrs().len());

        ptrs.into_field_ptrs()
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
                ErasedFieldPtr::new(*desc, buffer).expect("buffer should be aligned")
            })
            .collect();
        assert_eq!(descriptors.len(), ptrs.len());

        unsafe { ErasedSoaPtrs::new_unchecked(ptrs) }
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
                ErasedFieldMutPtr::new(*desc, buffer).expect("buffer should be aligned")
            })
            .collect();
        assert_eq!(descriptors.len(), ptrs.len());

        unsafe { ErasedSoaMutPtrs::new_unchecked(ptrs) }
    }

    fn ptrs_dangling(context: &Self::Context) -> Self::MutPtrs {
        let ptrs = context
            .field_descriptors()
            .iter()
            .copied()
            .map(ErasedFieldMutPtr::dangling);
        unsafe { ErasedSoaMutPtrs::new_unchecked(ptrs) }
    }

    fn ptrs_cast_const(context: &Self::Context, ptrs: Self::MutPtrs) -> Self::Ptrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), ptrs.field_ptrs().len());

        let ptrs = descriptors
            .iter()
            .zip(ptrs.into_field_ptrs())
            .inspect(|(desc, ptr)| {
                check_same_layout(ptr.descriptor().layout(), desc.layout())
                    .expect("layouts should match")
            })
            .map(|(_, ptr)| ptr.cast_const());
        unsafe { ErasedSoaPtrs::new_unchecked(ptrs) }
    }

    fn ptrs_cast_mut(context: &Self::Context, ptrs: Self::Ptrs) -> Self::MutPtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), ptrs.field_ptrs().len());

        let ptrs = descriptors
            .iter()
            .zip(ptrs.into_field_ptrs())
            .inspect(|(desc, ptr)| {
                check_same_layout(ptr.descriptor().layout(), desc.layout())
                    .expect("layouts should match")
            })
            .map(|(_, ptr)| ptr.cast_mut());
        unsafe { ErasedSoaMutPtrs::new_unchecked(ptrs) }
    }

    unsafe fn ptrs_add(context: &Self::Context, ptrs: Self::Ptrs, offset: usize) -> Self::Ptrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), ptrs.field_ptrs().len());

        let ptrs = descriptors
            .iter()
            .zip(ptrs.into_field_ptrs())
            .inspect(|(desc, ptr)| {
                check_same_layout(ptr.descriptor().layout(), desc.layout())
                    .expect("layouts should match")
            })
            .map(|(_, ptr)| unsafe { ptr.add(offset) });
        unsafe { ErasedSoaPtrs::new_unchecked(ptrs) }
    }

    unsafe fn ptrs_add_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
        offset: usize,
    ) -> Self::MutPtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), ptrs.field_ptrs().len());

        let ptrs = descriptors
            .iter()
            .zip(ptrs.into_field_ptrs())
            .inspect(|(desc, ptr)| {
                check_same_layout(ptr.descriptor().layout(), desc.layout())
                    .expect("layouts should match")
            })
            .map(|(_, ptr)| unsafe { ptr.add(offset) });
        unsafe { ErasedSoaMutPtrs::new_unchecked(ptrs) }
    }

    unsafe fn ptrs_offset_from(
        context: &Self::Context,
        ptrs: Self::Ptrs,
        origin: Self::Ptrs,
    ) -> isize {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), ptrs.field_ptrs().len());
        assert_eq!(ptrs.field_ptrs().len(), origin.field_ptrs().len());

        let mut offsets = descriptors
            .iter()
            .zip(ptrs.into_field_ptrs())
            .zip(origin.into_field_ptrs())
            .inspect(|((desc, ptr), origin)| {
                check_same_layout(ptr.descriptor().layout(), desc.layout())
                    .expect("layouts should match");
                check_same_layout(origin.descriptor().layout(), desc.layout())
                    .expect("layouts should match");
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
        assert_eq!(descriptors.len(), ptrs.field_ptrs().len());
        assert_eq!(ptrs.field_ptrs().len(), origin.field_ptrs().len());

        let mut offsets = descriptors
            .iter()
            .zip(ptrs.into_field_ptrs())
            .zip(origin.into_field_ptrs())
            .inspect(|((desc, ptr), origin)| {
                check_same_layout(ptr.descriptor().layout(), desc.layout())
                    .expect("layouts should match");
                check_same_layout(origin.descriptor().layout(), desc.layout())
                    .expect("layouts should match");
            })
            .map(|((_, ptr), origin)| unsafe { ptr.offset_from(origin) });

        let offset = offsets.next().expect("self should not be a ZST");
        assert!(offsets.all(|item| item == offset));
        offset
    }

    unsafe fn ptrs_swap(context: &Self::Context, a: Self::MutPtrs, b: Self::MutPtrs) {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), a.field_ptrs().len());
        assert_eq!(a.field_ptrs().len(), b.field_ptrs().len());

        let mut temp = Vec::new();
        descriptors
            .iter()
            .zip(a.into_field_ptrs())
            .zip(b.into_field_ptrs())
            .inspect(|((desc, a), b)| {
                check_same_layout(a.descriptor().layout(), desc.layout())
                    .expect("layouts should match");
                check_same_layout(b.descriptor().layout(), desc.layout())
                    .expect("layouts should match");
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
        assert_eq!(descriptors.len(), src.field_ptrs().len());
        assert_eq!(src.field_ptrs().len(), dst.field_ptrs().len());

        let mut temp = Vec::new();
        descriptors
            .iter()
            .zip(src.into_field_ptrs())
            .zip(dst.into_field_ptrs())
            .inspect(|((desc, src), dst)| {
                check_same_layout(src.descriptor().layout(), desc.layout())
                    .expect("layouts should match");
                check_same_layout(dst.descriptor().layout(), desc.layout())
                    .expect("layouts should match");
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
        assert_eq!(descriptors.len(), src.field_ptrs().len());
        assert_eq!(src.field_ptrs().len(), dst.field_ptrs().len());

        let mut temp = Vec::new();
        descriptors
            .iter()
            .zip(src.into_field_ptrs())
            .zip(dst.into_field_ptrs())
            .rev()
            .inspect(|((desc, src), dst)| {
                check_same_layout(src.descriptor().layout(), desc.layout())
                    .expect("layouts should match");
                check_same_layout(dst.descriptor().layout(), desc.layout())
                    .expect("layouts should match");
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
        assert_eq!(descriptors.len(), src.field_ptrs().len());
        assert_eq!(src.field_ptrs().len(), dst.field_ptrs().len());

        descriptors
            .iter()
            .zip(src.into_field_ptrs())
            .zip(dst.into_field_ptrs())
            .inspect(|((desc, src), dst)| {
                check_same_layout(src.descriptor().layout(), desc.layout())
                    .expect("layouts should match");
                check_same_layout(dst.descriptor().layout(), desc.layout())
                    .expect("layouts should match");
            })
            .for_each(|((_, src), dst)| unsafe { dst.copy_from_nonoverlapping(src, len) })
    }

    unsafe fn ptrs_read(context: &Self::Context, src: Self::Ptrs) -> Self {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), src.field_ptrs().len());

        let fields = descriptors
            .iter()
            .zip(src.into_field_ptrs())
            .inspect(|(desc, src)| {
                check_same_layout(src.descriptor().layout(), desc.layout())
                    .expect("layouts should match")
            })
            .map(|(desc, src)| (*desc, unsafe { src.deref().into_buffer() }));
        unsafe { Self::new_unchecked(fields) }
    }

    unsafe fn ptrs_write(context: &Self::Context, dst: Self::MutPtrs, value: Self) {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), dst.field_ptrs().len());
        assert_eq!(descriptors.len(), value.field_descriptors().len());

        descriptors
            .iter()
            .zip(dst.into_field_ptrs())
            .zip(value.as_refs().into_field_refs())
            .inspect(|((desc, dst), src)| {
                check_same_layout(src.descriptor().layout(), desc.layout())
                    .expect("layouts should match");
                check_same_layout(dst.descriptor().layout(), desc.layout())
                    .expect("layouts should match");
            })
            .for_each(|((_, dst), src)| {
                let src = src.as_field_ptr();
                unsafe { dst.copy_from_nonoverlapping(src, 1) }
            })
    }

    unsafe fn ptrs_drop_in_place(_: &Self::Context, _: Self::MutPtrs) {
        // do nothing; it's safe to not drop anything
    }

    type NonNullPtrs = ErasedSoaNonNullPtrs<Fields>;

    unsafe fn ptrs_to_nonnull(context: &Self::Context, ptrs: Self::MutPtrs) -> Self::NonNullPtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), ptrs.field_ptrs().len());

        let ptrs = descriptors
            .iter()
            .zip(ptrs.into_field_ptrs())
            .inspect(|(desc, ptr)| {
                check_same_layout(ptr.descriptor().layout(), desc.layout())
                    .expect("layouts should match")
            })
            .map(|(_, ptr)| {
                let desc = ptr.descriptor();
                let buffer = unsafe { NonNull::new_unchecked(ptr.buffer()) };
                ErasedFieldNonNullPtr::new(desc, buffer).expect("buffer should be aligned")
            });
        unsafe { ErasedSoaNonNullPtrs::new_unchecked(ptrs) }
    }

    fn nonnull_to_ptrs(context: &Self::Context, ptrs: Self::NonNullPtrs) -> Self::MutPtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), ptrs.field_ptrs().len());

        let ptrs = descriptors
            .iter()
            .zip(ptrs.into_field_ptrs())
            .inspect(|(desc, ptr)| {
                check_same_layout(ptr.descriptor().layout(), desc.layout())
                    .expect("layouts should match")
            })
            .map(|(_, ptr)| {
                let desc = ptr.descriptor();
                let buffer = ptr.buffer().as_ptr();
                ErasedFieldMutPtr::new(desc, buffer).expect("buffer should be aligned")
            });
        unsafe { ErasedSoaMutPtrs::new_unchecked(ptrs) }
    }

    type Vecs = ErasedSoaVecs<Fields>;

    fn vecs_with_capacity(context: &Self::Context, capacity: usize) -> Self::Vecs {
        let vecs = context
            .field_descriptors()
            .iter()
            .map(|&desc| {
                let capacity = (capacity * desc.layout().size())
                    .div_ceil(size_of::<ErasedByte<Aligned<Fields>>>());
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
            .inspect(|(desc, vec)| {
                check_same_layout(vec.desc.layout(), desc.layout()).expect("layouts should match")
            })
            .map(|(_, vec)| {
                let ErasedFieldVec { buffer, desc, .. } = vec;

                let data = buffer.as_ptr().cast();
                let len = desc.layout().size();
                let buffer = ptr::slice_from_raw_parts(data, len);
                ErasedFieldPtr::new(*desc, buffer).expect("buffer should be aligned")
            });
        unsafe { ErasedSoaPtrs::new_unchecked(ptrs) }
    }

    fn mut_vecs_as_ptrs(context: &Self::Context, vecs: &mut Self::Vecs) -> Self::MutPtrs {
        let descriptors = context.field_descriptors();
        let ErasedSoaVecs { vecs, .. } = vecs;
        assert_eq!(descriptors.len(), vecs.len());

        let ptrs = descriptors
            .iter()
            .zip(vecs)
            .inspect(|(desc, vec)| {
                check_same_layout(vec.desc.layout(), desc.layout()).expect("layouts should match")
            })
            .map(|(_, vec)| {
                let ErasedFieldVec { buffer, desc, .. } = vec;

                let data = buffer.as_mut_ptr().cast();
                let len = desc.layout().size();
                let buffer = ptr::slice_from_raw_parts_mut(data, len);
                ErasedFieldMutPtr::new(*desc, buffer).expect("buffer should be aligned")
            });
        unsafe { ErasedSoaMutPtrs::new_unchecked(ptrs) }
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
            .inspect(|(desc, vec)| {
                check_same_layout(vec.desc.layout(), desc.layout()).expect("layouts should match")
            })
            .for_each(|(_, vec)| {
                let ErasedFieldVec { buffer, desc } = vec;
                let len =
                    (len * desc.layout().size()).div_ceil(size_of::<ErasedByte<Aligned<Fields>>>());
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
        assert_eq!(descriptors.len(), ptrs.field_ptrs().len());

        let refs = descriptors
            .iter()
            .zip(ptrs.into_field_ptrs())
            .inspect(|(desc, ptr)| {
                check_same_layout(ptr.descriptor().layout(), desc.layout())
                    .expect("layouts should match")
            })
            .map(|(_, ptr)| unsafe { ptr.deref() });
        unsafe { ErasedSoaRefs::new_unchecked(refs) }
    }

    unsafe fn ptrs_to_refs_mut<'a>(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
    ) -> Self::RefsMut<'a> {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), ptrs.field_ptrs().len());

        let refs = descriptors
            .iter()
            .zip(ptrs.into_field_ptrs())
            .inspect(|(desc, ptr)| {
                check_same_layout(ptr.descriptor().layout(), desc.layout())
                    .expect("layouts should match")
            })
            .map(|(_, ptr)| unsafe { ptr.deref_mut() });
        unsafe { ErasedSoaRefsMut::new_unchecked(refs) }
    }

    fn refs_as_ptrs(context: &Self::Context, refs: Self::Refs<'_>) -> Self::Ptrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), refs.field_refs().len());

        let ptrs = descriptors
            .iter()
            .zip(refs.into_field_refs())
            .inspect(|(desc, r#ref)| {
                check_same_layout(r#ref.descriptor().layout(), desc.layout())
                    .expect("layouts should match")
            })
            .map(|(_, r#ref)| r#ref.as_field_ptr());
        unsafe { ErasedSoaPtrs::new_unchecked(ptrs) }
    }

    fn mut_refs_as_ptrs(context: &Self::Context, refs: Self::RefsMut<'_>) -> Self::MutPtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), refs.field_refs().len());

        let ptrs = descriptors
            .iter()
            .zip(refs.into_field_refs())
            .inspect(|(desc, r#ref)| {
                check_same_layout(r#ref.descriptor().layout(), desc.layout())
                    .expect("layouts should match")
            })
            .map(|(_, mut r#ref)| r#ref.as_field_mut_ptr());
        unsafe { ErasedSoaMutPtrs::new_unchecked(ptrs) }
    }

    fn mut_refs_as_refs<'a>(context: &Self::Context, refs: Self::RefsMut<'a>) -> Self::Refs<'a> {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), refs.field_refs().len());

        let refs = descriptors
            .iter()
            .zip(refs.into_field_refs())
            .inspect(|(desc, r#ref)| {
                check_same_layout(r#ref.descriptor().layout(), desc.layout())
                    .expect("layouts should match")
            })
            .map(|(_, r#ref)| From::from(r#ref));
        unsafe { ErasedSoaRefs::new_unchecked(refs) }
    }

    type SlicePtrs = ErasedSoaSlicePtrs<Fields>;
    type SliceMutPtrs = ErasedSoaSliceMutPtrs<Fields>;

    fn slices_from_raw_parts(
        context: &Self::Context,
        ptrs: Self::Ptrs,
        len: usize,
    ) -> Self::SlicePtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), ptrs.field_ptrs().len());

        let slices = descriptors
            .iter()
            .zip(ptrs.into_field_ptrs())
            .inspect(|(desc, ptr)| {
                check_same_layout(ptr.descriptor().layout(), desc.layout())
                    .expect("layouts should match")
            })
            .map(|(_, data)| field_slice_from_raw_parts(data, len));
        unsafe { ErasedSoaSlicePtrs::new_unchecked(len, slices) }
    }

    fn slices_from_raw_parts_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
        len: usize,
    ) -> Self::SliceMutPtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), ptrs.field_ptrs().len());

        let slices = descriptors
            .iter()
            .zip(ptrs.into_field_ptrs())
            .inspect(|(desc, ptr)| {
                check_same_layout(ptr.descriptor().layout(), desc.layout())
                    .expect("layouts should match")
            })
            .map(|(_, data)| field_slice_from_raw_parts_mut(data, len));
        unsafe { ErasedSoaSliceMutPtrs::new_unchecked(len, slices) }
    }

    fn slice_ptrs_cast_const(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::SlicePtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.field_slices().len());

        let len = slices.len();
        let slices = descriptors
            .iter()
            .zip(slices.into_field_slices())
            .inspect(|(desc, slice)| {
                check_same_layout(slice.descriptor().layout(), desc.layout())
                    .expect("layouts should match")
            })
            .map(|(_, slice)| slice.cast_const());
        unsafe { ErasedSoaSlicePtrs::new_unchecked(len, slices) }
    }

    fn slice_ptrs_cast_mut(context: &Self::Context, slices: Self::SlicePtrs) -> Self::SliceMutPtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.field_slices().len());

        let len = slices.len();
        let slices = descriptors
            .iter()
            .zip(slices.into_field_slices())
            .inspect(|(desc, slice)| {
                check_same_layout(slice.descriptor().layout(), desc.layout())
                    .expect("layouts should match")
            })
            .map(|(_, slice)| slice.cast_mut());
        unsafe { ErasedSoaSliceMutPtrs::new_unchecked(len, slices) }
    }

    fn slice_ptrs_len(context: &Self::Context, slices: Self::SlicePtrs) -> usize {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.field_slices().len());

        slices.len()
    }

    fn slice_ptrs_len_mut(context: &Self::Context, slices: Self::SliceMutPtrs) -> usize {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.field_slices().len());

        slices.len()
    }

    fn slice_ptrs_as_ptrs(context: &Self::Context, slices: Self::SlicePtrs) -> Self::Ptrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.field_slices().len());

        let ptrs = descriptors
            .iter()
            .zip(slices.into_field_slices())
            .inspect(|(desc, slice)| {
                check_same_layout(slice.descriptor().layout(), desc.layout())
                    .expect("layouts should match")
            })
            .map(|(_, slice)| slice.as_field_ptr());
        unsafe { ErasedSoaPtrs::new_unchecked(ptrs) }
    }

    fn mut_slice_ptrs_as_ptrs(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::MutPtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.field_slices().len());

        let ptrs = descriptors
            .iter()
            .zip(slices.into_field_slices())
            .inspect(|(desc, slice)| {
                check_same_layout(slice.descriptor().layout(), desc.layout())
                    .expect("layouts should match")
            })
            .map(|(_, slice)| slice.as_field_ptr());
        unsafe { ErasedSoaMutPtrs::new_unchecked(ptrs) }
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
        assert_eq!(descriptors.len(), slices.field_slices().len());

        let len = slices.len();
        let slices = descriptors
            .iter()
            .zip(slices.into_field_slices())
            .inspect(|(desc, slice)| {
                check_same_layout(slice.descriptor().layout(), desc.layout())
                    .expect("layouts should match")
            })
            .map(|(_, slice)| unsafe { slice.deref() });
        unsafe { ErasedSoaSlices::new_unchecked(len, slices) }
    }

    unsafe fn slice_ptrs_to_slices_mut<'a>(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::SlicesMut<'a> {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.field_slices().len());

        let len = slices.len();
        let slices = descriptors
            .iter()
            .zip(slices.into_field_slices())
            .inspect(|(desc, slice)| {
                check_same_layout(slice.descriptor().layout(), desc.layout())
                    .expect("layouts should match")
            })
            .map(|(_, slice)| unsafe { slice.deref_mut() });
        unsafe { ErasedSoaSlicesMut::new_unchecked(len, slices) }
    }

    fn slices_len(context: &Self::Context, slices: &Self::Slices<'_>) -> usize {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.field_slices().len());

        slices.len()
    }

    fn slices_len_mut(context: &Self::Context, slices: &Self::SlicesMut<'_>) -> usize {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.field_slices().len());

        slices.len()
    }

    fn slice_refs_as_slice_ptrs(
        context: &Self::Context,
        slices: Self::Slices<'_>,
    ) -> Self::SlicePtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.field_slices().len());

        let len = slices.len();
        let slices = descriptors
            .iter()
            .zip(slices.into_field_slices())
            .inspect(|(desc, slice)| {
                check_same_layout(slice.descriptor().layout(), desc.layout())
                    .expect("layouts should match")
            })
            .map(|(_, slice)| slice.as_field_slice_ptr());
        unsafe { ErasedSoaSlicePtrs::new_unchecked(len, slices) }
    }

    fn mut_slice_refs_as_slice_ptrs(
        context: &Self::Context,
        slices: Self::SlicesMut<'_>,
    ) -> Self::SliceMutPtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.field_slices().len());

        let len = slices.len();
        let slices = descriptors
            .iter()
            .zip(slices.into_field_slices())
            .inspect(|(desc, slice)| {
                check_same_layout(slice.descriptor().layout(), desc.layout())
                    .expect("layouts should match")
            })
            .map(|(_, mut slice)| slice.as_field_slice_mut_ptr());
        unsafe { ErasedSoaSliceMutPtrs::new_unchecked(len, slices) }
    }

    fn mut_slices_as_slices<'a>(
        context: &Self::Context,
        slices: Self::SlicesMut<'a>,
    ) -> Self::Slices<'a> {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.field_slices().len());

        let len = slices.len();
        let slices = descriptors
            .iter()
            .zip(slices.into_field_slices())
            .inspect(|(desc, slice)| {
                check_same_layout(slice.descriptor().layout(), desc.layout())
                    .expect("layouts should match")
            })
            .map(|(_, slice)| From::from(slice));
        unsafe { ErasedSoaSlices::new_unchecked(len, slices) }
    }

    fn slice_refs_as_ptrs(context: &Self::Context, slices: Self::Slices<'_>) -> Self::Ptrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.field_slices().len());

        let ptrs = descriptors
            .iter()
            .zip(slices.into_field_slices())
            .inspect(|(desc, slice)| {
                check_same_layout(slice.descriptor().layout(), desc.layout())
                    .expect("layouts should match")
            })
            .map(|(_, slice)| slice.as_field_ptr());
        unsafe { ErasedSoaPtrs::new_unchecked(ptrs) }
    }

    fn mut_slice_refs_as_ptrs(
        context: &Self::Context,
        slices: Self::SlicesMut<'_>,
    ) -> Self::MutPtrs {
        let descriptors = context.field_descriptors();
        assert_eq!(descriptors.len(), slices.field_slices().len());

        let ptrs = descriptors
            .iter()
            .zip(slices.into_field_slices())
            .inspect(|(desc, slice)| {
                check_same_layout(slice.descriptor().layout(), desc.layout())
                    .expect("layouts should match")
            })
            .map(|(_, mut slice)| slice.as_field_mut_ptr());
        unsafe { ErasedSoaMutPtrs::new_unchecked(ptrs) }
    }

    unsafe fn slices_drop_in_place(_: &Self::Context, _: Self::SliceMutPtrs) {
        // do nothing; it's safe to not drop anything
    }
}
