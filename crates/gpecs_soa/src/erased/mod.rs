use alloc::{
    boxed::Box,
    vec::{self, Vec},
};
use core::{
    alloc::{Layout, LayoutError},
    borrow::Borrow,
    iter,
    marker::PhantomData,
    ptr::{self, NonNull},
    slice,
};

use crate::traits::{buffer_layout, Soa};

use self::{byte::ErasedByte, vecs::ErasedFieldVec};

pub use self::{
    context::ErasedSoaContext,
    nonnull_ptrs::{ErasedFieldNonNullPtr, ErasedSoaNonNullPtrs},
    ptrs::{ErasedFieldPtr, ErasedSoaPtrs},
    ptrs_mut::ErasedSoaMutPtrs,
    refs::{ErasedFieldRef, ErasedSoaRefs},
    refs_mut::{ErasedFieldRefMut, ErasedSoaRefsMut},
    slice_ptrs::ErasedSoaSlicePtrs,
    slice_ptrs_mut::ErasedSoaSliceMutPtrs,
    slices::ErasedSoaSlices,
    slices_mut::ErasedSoaSlicesMut,
    vecs::ErasedSoaVecs,
};

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
mod vecs;

type ErasedFields<Fields> = Box<[ErasedByte<Fields>]>;

pub struct ErasedSoa<Fields> {
    buffer: ErasedFields<Fields>,
    field_layouts: Box<[Layout]>,
}

impl<Fields> ErasedSoa<Fields> {
    #[inline]
    pub fn new<I, F>(fields: I) -> Self
    where
        I: IntoIterator<Item = (Layout, F)>,
        F: Borrow<[u8]>,
    {
        let (field_layouts, fields): (Vec<_>, Vec<_>) = fields
            .into_iter()
            .inspect(|(field_layout, src)| assert_eq!(field_layout.size(), src.borrow().len()))
            .unzip();
        let field_layouts = field_layouts.into_boxed_slice();

        let (buffer_layout, offsets): (_, Box<[_]>) =
            buffer_layout(&field_layouts, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Fields>>());

        let mut buffer = Box::new_uninit_slice(buffer_len);
        let buffer = unsafe {
            for ((field_layout, src), offset) in field_layouts.iter().zip(fields).zip(offsets) {
                let src = src.borrow().as_ptr();
                let dst = buffer.as_mut_ptr().cast::<u8>().add(offset);

                let len = field_layout.size();
                ptr::copy_nonoverlapping(src, dst, len);
            }
            buffer.assume_init()
        };
        Self {
            buffer,
            field_layouts,
        }
    }

    #[inline]
    pub fn from<T>(context: &T::Context, value: T) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let field_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<T::Fields, _>)
            .collect();

        let (buffer_layout, offsets) =
            T::buffer_layout(context, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Fields>>());

        let mut buffer = Box::new_uninit_slice(buffer_len);
        let buffer = unsafe {
            let dst = {
                let buffer = buffer.as_mut_ptr().cast::<u8>();
                let ptrs = offsets.into_iter().map(|offset| buffer.add(offset));
                T::ptrs_restore_mut(context, ptrs)
            };
            T::ptrs_write(context, dst, value);
            buffer.assume_init()
        };

        Self {
            buffer,
            field_layouts,
        }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> T
    where
        T: Soa<Fields = Fields>,
    {
        let Self {
            buffer,
            field_layouts,
        } = self;

        let target_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<T::Fields, _>);
        assert!(target_layouts.eq(field_layouts));

        let (buffer_layout, offsets) =
            T::buffer_layout(context, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Fields>>());
        assert_eq!(buffer_len, buffer.len());

        unsafe {
            let src = {
                let buffer = buffer.as_ptr().cast::<u8>();
                let ptrs = offsets.into_iter().map(|offset| buffer.add(offset));
                T::ptrs_restore(context, ptrs)
            };
            T::ptrs_read(context, src)
        }
    }

    #[inline]
    pub fn into_fields(self) -> Box<[(Layout, Box<[u8]>)]> {
        let Self {
            buffer,
            field_layouts,
        } = self;

        let (buffer_layout, offsets): (_, Box<[_]>) =
            buffer_layout(&field_layouts, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Fields>>());
        assert_eq!(buffer_len, buffer.len());

        iter::zip(field_layouts, offsets)
            .map(|(field_layout, offset)| {
                let data = unsafe { buffer.as_ptr().cast::<u8>().add(offset) };
                let len = field_layout.size();
                let r#ref = unsafe { slice::from_raw_parts(data, len) };
                (field_layout.clone(), r#ref.into())
            })
            .collect()
    }

    #[inline]
    pub fn layouts(&self) -> &[Layout] {
        let Self { field_layouts, .. } = self;
        field_layouts.as_ref()
    }

    #[inline]
    pub fn as_refs(&self) -> ErasedSoaRefs<'_, Fields> {
        let Self {
            buffer,
            field_layouts,
        } = self;

        let (buffer_layout, offsets): (_, Box<[_]>) =
            buffer_layout(field_layouts, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Fields>>());
        assert_eq!(buffer_len, buffer.len());

        let refs = field_layouts
            .iter()
            .zip(offsets)
            .map(|(field_layout, offset)| {
                let data = unsafe { buffer.as_ptr().cast::<u8>().add(offset) };
                let len = field_layout.size();
                let r#ref = unsafe { slice::from_raw_parts(data, len) };
                ErasedFieldRef::new(field_layout.clone(), r#ref)
            })
            .collect();
        ErasedSoaRefs {
            refs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn as_refs_mut(&mut self) -> ErasedSoaRefsMut<'_, Fields> {
        let Self {
            buffer,
            field_layouts,
        } = self;

        let (buffer_layout, offsets): (_, Box<[_]>) =
            buffer_layout(&*field_layouts, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Fields>>());
        assert_eq!(buffer_len, buffer.len());

        let refs = field_layouts
            .iter()
            .zip(offsets)
            .map(|(field_layout, offset)| {
                let data = unsafe { buffer.as_mut_ptr().cast::<u8>().add(offset) };
                let len = field_layout.size();
                let r#ref = unsafe { slice::from_raw_parts_mut(data, len) };
                ErasedFieldRefMut::new(field_layout.clone(), r#ref)
            })
            .collect();
        ErasedSoaRefsMut {
            refs,
            phantom: PhantomData,
        }
    }
}

unsafe impl<Fields> Soa for ErasedSoa<Fields> {
    type Context = ErasedSoaContext<Fields>;

    type Fields = Fields;

    type FieldLayouts<'a> = &'a [Layout];

    fn field_layouts(context: &Self::Context) -> Self::FieldLayouts<'_> {
        let ErasedSoaContext { field_layouts, .. } = context;
        field_layouts.as_ref()
    }

    type FieldOffsets<'a> = Box<[usize]>;

    fn buffer_layout(
        context: &Self::Context,
        capacity: usize,
    ) -> Result<(Layout, Self::FieldOffsets<'_>), LayoutError> {
        let ErasedSoaContext { field_layouts, .. } = context;
        buffer_layout(field_layouts, capacity)
    }

    type Ptrs = ErasedSoaPtrs<Fields>;
    type MutPtrs = ErasedSoaMutPtrs<Fields>;

    type ErasedPtrs = iter::Map<vec::IntoIter<ErasedFieldPtr>, fn(ErasedFieldPtr) -> *const u8>;
    type ErasedMutPtrs =
        iter::Map<vec::IntoIter<(Layout, *mut [u8])>, fn((Layout, *mut [u8])) -> *mut u8>;

    fn ptrs_erase(context: &Self::Context, ptrs: Self::Ptrs) -> Self::ErasedPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        ptrs.into_vec().into_iter().map(|ptr| ptr.buffer().cast())
    }

    fn ptrs_erase_mut(context: &Self::Context, ptrs: Self::MutPtrs) -> Self::ErasedMutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaMutPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        ptrs.into_vec().into_iter().map(|(_, ptr)| ptr.cast())
    }

    fn ptrs_restore(
        context: &Self::Context,
        ptrs: impl IntoIterator<Item = *const u8>,
    ) -> Self::Ptrs {
        let ErasedSoaContext { field_layouts, .. } = context;

        let ptrs: Box<[_]> = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let data = ptr.cast();
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts(data, len);
                ErasedFieldPtr::new(field_layout.clone(), ptr)
            })
            .collect();
        assert_eq!(field_layouts.len(), ptrs.len());

        ErasedSoaPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn ptrs_restore_mut(
        context: &Self::Context,
        ptrs: impl IntoIterator<Item = *mut u8>,
    ) -> Self::MutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;

        let ptrs: Box<[_]> = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let data = ptr.cast();
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts_mut(data, len);
                (field_layout.clone(), ptr)
            })
            .collect();
        assert_eq!(field_layouts.len(), ptrs.len());

        ErasedSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn ptrs_dangling(context: &Self::Context) -> Self::MutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;

        let ptrs = field_layouts
            .iter()
            .map(|field_layout| {
                let data = ptr::without_provenance_mut(field_layout.align());
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts_mut(data, len);
                (field_layout.clone(), ptr)
            })
            .collect();
        ErasedSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn ptrs_cast_const(context: &Self::Context, ptrs: Self::MutPtrs) -> Self::Ptrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaMutPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, (layout, ptr))| {
                assert_eq!(field_layout, &layout);
                ErasedFieldPtr::new(layout, ptr.cast_const())
            })
            .collect();
        ErasedSoaPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn ptrs_cast_mut(context: &Self::Context, ptrs: Self::Ptrs) -> Self::MutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, field_ptr)| {
                assert_eq!(*field_layout, field_ptr.layout());
                (*field_layout, field_ptr.buffer().cast_mut())
            })
            .collect();
        ErasedSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    unsafe fn ptrs_add(context: &Self::Context, ptrs: Self::Ptrs, offset: usize) -> Self::Ptrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                assert_eq!(*field_layout, ptr.layout());
                // assert_eq!(field_layout.size(), ptr.len());

                let count = offset * field_layout.pad_to_align().size();
                let data = unsafe { ptr.buffer().cast::<u8>().add(count) };
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts(data, len);
                ErasedFieldPtr::new(*field_layout, ptr)
            })
            .collect();
        ErasedSoaPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    unsafe fn ptrs_add_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
        offset: usize,
    ) -> Self::MutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaMutPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, (layout, ptr))| {
                assert_eq!(field_layout, &layout);
                // assert_eq!(field_layout.size(), ptr.len());

                let count = offset * field_layout.pad_to_align().size();
                let data = unsafe { ptr.cast::<u8>().add(count) };
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts_mut(data, len);
                (layout, ptr)
            })
            .collect();
        ErasedSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    unsafe fn ptrs_offset_from(
        context: &Self::Context,
        ptrs: Self::Ptrs,
        origin: Self::Ptrs,
    ) -> isize {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaPtrs { ptrs, .. } = ptrs;
        let ErasedSoaPtrs { ptrs: origin, .. } = origin;

        assert_eq!(field_layouts.len(), ptrs.len());
        assert_eq!(ptrs.len(), origin.len());

        let mut offsets =
            field_layouts
                .iter()
                .zip(ptrs)
                .zip(origin)
                .map(|((field_layout, ptr), origin)| {
                    assert_eq!(*field_layout, ptr.layout());
                    assert_eq!(*field_layout, origin.layout());
                    assert_eq!(field_layout.size(), ptr.buffer().len());
                    assert_eq!(ptr.buffer().len(), origin.buffer().len());

                    let offset = unsafe {
                        ptr.buffer()
                            .cast::<u8>()
                            .offset_from(origin.buffer().cast())
                    };
                    let field_size = field_layout
                        .size()
                        .try_into()
                        .expect("layout size should not exceed `isize::MAX`");
                    offset
                        .checked_div(field_size)
                        .expect("self should not be a ZST")
                });

        let offset = offsets.next().expect("self should not be a ZST");
        assert!(offsets.all(|item| item == offset));
        offset
    }

    unsafe fn ptrs_offset_from_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
        origin: Self::Ptrs,
    ) -> isize {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaMutPtrs { ptrs, .. } = ptrs;
        let ErasedSoaPtrs { ptrs: origin, .. } = origin;

        assert_eq!(field_layouts.len(), ptrs.len());
        assert_eq!(ptrs.len(), origin.len());

        let mut offsets = field_layouts.iter().zip(ptrs).zip(origin).map(
            |((field_layout, (ptr_layout, ptr)), origin)| {
                assert_eq!(field_layout, &ptr_layout);
                assert_eq!(*field_layout, origin.layout());
                assert_eq!(field_layout.size(), ptr.len());
                assert_eq!(ptr.len(), origin.buffer().len());

                let offset = unsafe { ptr.cast::<u8>().offset_from(origin.buffer().cast()) };
                let field_size = field_layout
                    .size()
                    .try_into()
                    .expect("layout size should not exceed `isize::MAX`");
                offset
                    .checked_div(field_size)
                    .expect("self should not be a ZST")
            },
        );

        let offset = offsets.next().expect("self should not be a ZST");
        assert!(offsets.all(|item| item == offset));
        offset
    }

    unsafe fn ptrs_swap(context: &Self::Context, a: Self::MutPtrs, b: Self::MutPtrs) {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaMutPtrs { ptrs: a, .. } = a;
        let ErasedSoaMutPtrs { ptrs: b, .. } = b;

        assert_eq!(field_layouts.len(), a.len());
        assert_eq!(a.len(), b.len());

        let mut temp = Vec::new();
        for ((field_layout, (a_layout, a)), (b_layout, b)) in field_layouts.iter().zip(a).zip(b) {
            assert_eq!(field_layout, &a_layout);
            assert_eq!(field_layout, &b_layout);
            assert_eq!(field_layout.size(), a.len());
            assert_eq!(a.len(), b.len());

            let a = a.cast::<u8>();
            let b = b.cast();

            let len = field_layout.size();
            temp.reserve(len);
            unsafe {
                ptr::copy_nonoverlapping(a, temp.as_mut_ptr(), len);
                temp.set_len(len);

                ptr::copy(b, a, len);
                ptr::copy_nonoverlapping(temp.as_ptr(), b, len);
            }
            temp.clear();
        }
    }

    unsafe fn ptrs_copy(context: &Self::Context, src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaPtrs { ptrs: src, .. } = src;
        let ErasedSoaMutPtrs { ptrs: dst, .. } = dst;

        assert_eq!(field_layouts.len(), src.len());
        assert_eq!(src.len(), dst.len());

        let mut temp = Vec::new();
        for ((field_layout, src), (dst_layout, dst)) in field_layouts.iter().zip(src).zip(dst) {
            assert_eq!(*field_layout, src.layout());
            assert_eq!(field_layout, &dst_layout);
            assert_eq!(field_layout.size(), src.buffer().len());
            assert_eq!(src.buffer().len(), dst.len());

            let src = src.buffer().cast::<u8>();
            let dst = dst.cast();

            let len = len * field_layout.size();
            temp.reserve(len);
            unsafe {
                ptr::copy_nonoverlapping(src, temp.as_mut_ptr(), len);
                temp.set_len(len);

                ptr::copy_nonoverlapping(temp.as_ptr(), dst, len);
            }
            temp.clear();
        }
    }

    unsafe fn ptrs_copy_rev(
        context: &Self::Context,
        src: Self::Ptrs,
        dst: Self::MutPtrs,
        len: usize,
    ) {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaPtrs { ptrs: src, .. } = src;
        let ErasedSoaMutPtrs { ptrs: dst, .. } = dst;

        assert_eq!(field_layouts.len(), src.len());
        assert_eq!(src.len(), dst.len());

        let mut temp = Vec::new();
        for ((field_layout, src), (dst_layout, dst)) in field_layouts.iter().zip(src).zip(dst).rev()
        {
            assert_eq!(*field_layout, src.layout());
            assert_eq!(field_layout, &dst_layout);
            assert_eq!(field_layout.size(), src.buffer().len());
            assert_eq!(src.buffer().len(), dst.len());

            let src = src.buffer().cast::<u8>();
            let dst = dst.cast();

            let len = len * field_layout.size();
            temp.reserve(len);
            unsafe {
                ptr::copy_nonoverlapping(src, temp.as_mut_ptr(), len);
                temp.set_len(len);

                ptr::copy_nonoverlapping(temp.as_ptr(), dst, len);
            }
            temp.clear();
        }
    }

    unsafe fn ptrs_copy_nonoverlapping(
        context: &Self::Context,
        src: Self::Ptrs,
        dst: Self::MutPtrs,
        len: usize,
    ) {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaPtrs { ptrs: src, .. } = src;
        let ErasedSoaMutPtrs { ptrs: dst, .. } = dst;

        assert_eq!(field_layouts.len(), src.len());
        assert_eq!(src.len(), dst.len());

        for ((field_layout, src), (dst_layout, dst)) in field_layouts.iter().zip(src).zip(dst) {
            assert_eq!(*field_layout, src.layout());
            assert_eq!(field_layout, &dst_layout);

            let src = src.buffer().cast::<u8>();
            let dst = dst.cast();

            let len = len * field_layout.size();
            unsafe {
                ptr::copy_nonoverlapping(src, dst, len);
            }
        }
    }

    unsafe fn ptrs_read(context: &Self::Context, src: Self::Ptrs) -> Self {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaPtrs { ptrs: src, .. } = src;
        assert_eq!(field_layouts.len(), src.len());

        let (buffer_layout, offsets) =
            Self::buffer_layout(context, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Fields>>());

        let mut buffer = Box::new_uninit_slice(buffer_len);
        for ((field_layout, src), offset) in field_layouts.iter().zip(src).zip(offsets) {
            assert_eq!(*field_layout, src.layout());
            let src = src.buffer().cast();
            let dst = unsafe { buffer.as_mut_ptr().cast::<u8>().add(offset) };

            let len = field_layout.size();
            unsafe {
                ptr::copy_nonoverlapping(src, dst, len);
            }
        }
        let buffer = unsafe { buffer.assume_init() };
        Self {
            buffer,
            field_layouts: field_layouts.clone(),
        }
    }

    unsafe fn ptrs_write(context: &Self::Context, dst: Self::MutPtrs, value: Self) {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaMutPtrs { ptrs: dst, .. } = dst;
        let Self {
            buffer,
            field_layouts: value_layouts,
        } = value;

        assert_eq!(field_layouts.len(), dst.len());
        assert_eq!(field_layouts.as_ref(), value_layouts.as_ref());

        let (buffer_layout, offsets) =
            Self::buffer_layout(context, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Fields>>());
        assert_eq!(buffer_len, buffer.len());

        for ((field_layout, (dst_layout, dst)), offset) in
            field_layouts.iter().zip(dst).zip(offsets)
        {
            assert_eq!(field_layout, &dst_layout);

            let src = unsafe { buffer.as_ptr().cast::<u8>().add(offset) };
            let dst = dst.cast();

            let len = field_layout.size();
            unsafe {
                ptr::copy_nonoverlapping(src, dst, len);
            }
        }
    }

    unsafe fn ptrs_drop_in_place(context: &Self::Context, ptrs: Self::MutPtrs) {
        let ErasedSoaContext {
            field_layouts,
            drop_fields,
            ..
        } = context;
        let Some(drop_fields) = drop_fields else {
            return;
        };

        let ErasedSoaMutPtrs { ptrs, .. } = ptrs;
        assert_eq!(field_layouts.len(), ptrs.len());

        drop_fields(ptrs.as_ref());
    }

    type NonNullPtrs = ErasedSoaNonNullPtrs<Fields>;

    unsafe fn ptrs_to_nonnull(context: &Self::Context, ptrs: Self::MutPtrs) -> Self::NonNullPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaMutPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, (layout, ptr))| {
                assert_eq!(field_layout, &layout);
                // assert_eq!(field_layout.size(), ptr.len());
                ErasedFieldNonNullPtr::new(layout, unsafe { NonNull::new_unchecked(ptr) })
            })
            .collect();
        ErasedSoaNonNullPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn nonnull_to_ptrs(context: &Self::Context, ptrs: Self::NonNullPtrs) -> Self::MutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaNonNullPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                assert_eq!(*field_layout, ptr.layout());
                // assert_eq!(field_layout.size(), ptr.len());
                (field_layout.clone(), ptr.buffer().as_ptr())
            })
            .collect();
        ErasedSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    type Vecs = ErasedSoaVecs<Fields>;

    fn vecs_with_capacity(context: &Self::Context, capacity: usize) -> Self::Vecs {
        let ErasedSoaContext { field_layouts, .. } = context;

        let vecs = field_layouts
            .iter()
            .map(|field_layout| {
                let capacity =
                    (capacity * field_layout.size()).div_ceil(size_of::<ErasedByte<Fields>>());
                ErasedFieldVec {
                    buffer: Vec::with_capacity(capacity),
                    layout: field_layout.clone(),
                }
            })
            .collect();
        ErasedSoaVecs { len: 0, vecs }
    }

    fn vecs_as_ptrs(context: &Self::Context, vecs: &Self::Vecs) -> Self::Ptrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaVecs { vecs, .. } = vecs;

        assert_eq!(field_layouts.len(), vecs.len());

        let ptrs = field_layouts
            .iter()
            .zip(vecs)
            .map(|(field_layout, vec)| {
                let ErasedFieldVec {
                    buffer,
                    layout: vec_field_layout,
                    ..
                } = vec;
                assert_eq!(field_layout, vec_field_layout);

                let data = buffer.as_ptr().cast();
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts(data, len);
                ErasedFieldPtr::new(field_layout.clone(), ptr)
            })
            .collect();
        ErasedSoaPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn mut_vecs_as_ptrs(context: &Self::Context, vecs: &mut Self::Vecs) -> Self::MutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaVecs { vecs, .. } = vecs;

        assert_eq!(field_layouts.len(), vecs.len());

        let ptrs = field_layouts
            .iter()
            .zip(vecs)
            .map(|(field_layout, vec)| {
                let ErasedFieldVec {
                    buffer,
                    layout: vec_field_layout,
                    ..
                } = vec;
                assert_eq!(field_layout, vec_field_layout);

                let data = buffer.as_mut_ptr().cast();
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts_mut(data, len);
                (field_layout.clone(), ptr)
            })
            .collect();
        ErasedSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn vecs_len(context: &Self::Context, vecs: &Self::Vecs) -> usize {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaVecs { vecs, len, .. } = vecs;

        assert_eq!(field_layouts.len(), vecs.len());

        // let mut lens = field_layouts.iter().zip(vecs).map(|(field_layout, vec)| {
        //     let ErasedFieldVec {
        //         buffer,
        //         layout: vec_field_layout,
        //         ..
        //     } = vec;
        //     assert_eq!(field_layout, vec_field_layout);
        //     *len
        // });
        // let len = lens.next().unwrap_or(0);
        // assert!(lens.all(|item| item == len));
        *len
    }

    unsafe fn vecs_set_len(context: &Self::Context, vecs: &mut Self::Vecs, len: usize) {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaVecs {
            vecs, len: vec_len, ..
        } = vecs;

        assert_eq!(field_layouts.len(), vecs.len());

        for (field_layout, vec) in field_layouts.iter().zip(vecs) {
            let ErasedFieldVec {
                buffer: field_buffer,
                layout: vec_field_layout,
            } = vec;
            assert_eq!(field_layout, vec_field_layout);

            *vec_len = len;
            let len = (len * vec_field_layout.size()).div_ceil(size_of::<ErasedByte<Fields>>());
            unsafe {
                field_buffer.set_len(len);
            }
        }
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
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let refs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                assert_eq!(*field_layout, ptr.layout());
                assert_eq!(field_layout.size(), ptr.buffer().len());

                let buffer = ptr.buffer();
                let r#ref = unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) };
                ErasedFieldRef::new(field_layout.clone(), r#ref)
            })
            .collect();
        ErasedSoaRefs {
            refs,
            phantom: PhantomData,
        }
    }

    unsafe fn ptrs_to_refs_mut<'a>(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
    ) -> Self::RefsMut<'a> {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaMutPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let refs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, (layout, ptr))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(field_layout.size(), ptr.len());

                let r#ref = unsafe { slice::from_raw_parts_mut(ptr.cast(), ptr.len()) };
                ErasedFieldRefMut::new(layout, r#ref)
            })
            .collect();
        ErasedSoaRefsMut {
            refs,
            phantom: PhantomData,
        }
    }

    fn refs_as_ptrs(context: &Self::Context, refs: Self::Refs<'_>) -> Self::Ptrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaRefs { refs, .. } = refs;

        assert_eq!(field_layouts.len(), refs.len());

        let ptrs = field_layouts
            .iter()
            .zip(refs)
            .map(|(field_layout, r#ref)| {
                assert_eq!(*field_layout, r#ref.layout());
                assert_eq!(field_layout.size(), r#ref.buffer().len());

                ErasedFieldPtr::new(r#ref.layout(), ptr::from_ref(r#ref.buffer()))
            })
            .collect();
        ErasedSoaPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn mut_refs_as_ptrs(context: &Self::Context, refs: Self::RefsMut<'_>) -> Self::MutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaRefsMut { refs, .. } = refs;

        assert_eq!(field_layouts.len(), refs.len());

        let ptrs = field_layouts
            .iter()
            .zip(refs)
            .map(|(field_layout, mut r#ref)| {
                assert_eq!(*field_layout, r#ref.layout());
                assert_eq!(field_layout.size(), r#ref.buffer().len());
                (*field_layout, ptr::from_mut(r#ref.buffer_mut()))
            })
            .collect();
        ErasedSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn mut_refs_as_refs<'a>(context: &Self::Context, refs: Self::RefsMut<'a>) -> Self::Refs<'a> {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaRefsMut { refs, .. } = refs;

        assert_eq!(field_layouts.len(), refs.len());

        let refs = field_layouts
            .iter()
            .zip(refs)
            .map(|(field_layout, r#ref)| {
                assert_eq!(*field_layout, r#ref.layout());
                assert_eq!(field_layout.size(), r#ref.buffer().len());
                ErasedFieldRef::new(*field_layout, r#ref.into_parts().1)
            })
            .collect();
        ErasedSoaRefs {
            refs,
            phantom: PhantomData,
        }
    }

    type SlicePtrs = ErasedSoaSlicePtrs<Fields>;

    type SliceMutPtrs = ErasedSoaSliceMutPtrs<Fields>;

    fn slices_from_raw_parts(
        context: &Self::Context,
        ptrs: Self::Ptrs,
        len: usize,
    ) -> Self::SlicePtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let slices = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                assert_eq!(*field_layout, ptr.layout());
                // assert_eq!(field_layout.size(), ptr.len());

                let data = ptr.buffer().cast();
                let len = len * field_layout.size();
                let slice = ptr::slice_from_raw_parts(data, len);
                (*field_layout, slice)
            })
            .collect();
        ErasedSoaSlicePtrs {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    fn slices_from_raw_parts_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
        len: usize,
    ) -> Self::SliceMutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaMutPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let slices = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, (layout, ptr))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(field_layout.size(), ptr.len());

                let data = ptr.cast();
                let len = len * field_layout.size();
                let slice = ptr::slice_from_raw_parts_mut(data, len);
                (layout, slice)
            })
            .collect();
        ErasedSoaSliceMutPtrs {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    fn slice_ptrs_cast_const(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::SlicePtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSliceMutPtrs { slices, len, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                (layout, slice.cast_const())
            })
            .collect();
        ErasedSoaSlicePtrs {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    fn slice_ptrs_cast_mut(context: &Self::Context, slices: Self::SlicePtrs) -> Self::SliceMutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSlicePtrs { slices, len, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                (layout, slice.cast_mut())
            })
            .collect();
        ErasedSoaSliceMutPtrs {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    fn slice_ptrs_len(context: &Self::Context, slices: Self::SlicePtrs) -> usize {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSlicePtrs { slices, len, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        // let mut lens = field_layouts
        //     .iter()
        //     .zip(slices)
        //     .map(|(field_layout, slice)| {
        //         assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);
        //         slice.len().checked_div(field_layout.size()).unwrap_or(0)
        //     });
        // let len = lens.next().unwrap_or(0);
        // assert!(lens.all(|item| item == len));
        len
    }

    fn slice_ptrs_len_mut(context: &Self::Context, slices: Self::SliceMutPtrs) -> usize {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSliceMutPtrs { slices, len, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        // let mut lens = field_layouts
        //     .iter()
        //     .zip(slices)
        //     .map(|(field_layout, (layout, slice))| {
        //         assert_eq!(field_layout, &layout);
        //         assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

        //         slice.len().checked_div(field_layout.size()).unwrap_or(0)
        //     });
        // let len = lens.next().unwrap_or(0);
        // assert!(lens.all(|item| item == len));
        len
    }

    fn slice_ptrs_as_ptrs(context: &Self::Context, slices: Self::SlicePtrs) -> Self::Ptrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSlicePtrs { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let ptrs = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                let data = slice.cast();
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts(data, len);
                ErasedFieldPtr::new(layout, ptr)
            })
            .collect();
        ErasedSoaPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn mut_slice_ptrs_as_ptrs(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::MutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSliceMutPtrs { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let ptrs = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                let data = slice.cast();
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts_mut(data, len);
                (layout, ptr)
            })
            .collect();
        ErasedSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
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
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSlicePtrs { slices, len, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                let data = slice.cast();
                let len = slice.len();
                let slice = unsafe { slice::from_raw_parts(data, len) };
                (layout, slice)
            })
            .collect();
        ErasedSoaSlices {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    unsafe fn slice_ptrs_to_slices_mut<'a>(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::SlicesMut<'a> {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSliceMutPtrs { slices, len, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                let data = slice.cast();
                let len = slice.len();
                let slice = unsafe { slice::from_raw_parts_mut(data, len) };
                (layout, slice)
            })
            .collect();
        ErasedSoaSlicesMut {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    fn slices_len(context: &Self::Context, slices: &Self::Slices<'_>) -> usize {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSlices { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let mut lens = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                slice.len().checked_div(field_layout.size()).unwrap_or(0)
            });
        let len = lens.next().unwrap_or(0);
        assert!(lens.all(|item| item == len));
        len
    }

    fn slices_len_mut(context: &Self::Context, slices: &Self::SlicesMut<'_>) -> usize {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSlicesMut { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let mut lens = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                slice.len().checked_div(field_layout.size()).unwrap_or(0)
            });
        let len = lens.next().unwrap_or(0);
        assert!(lens.all(|item| item == len));
        len
    }

    fn slice_refs_as_slice_ptrs(
        context: &Self::Context,
        slices: Self::Slices<'_>,
    ) -> Self::SlicePtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSlices { slices, len, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                (layout, ptr::from_ref(slice))
            })
            .collect();
        ErasedSoaSlicePtrs {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    fn mut_slice_refs_as_slice_ptrs(
        context: &Self::Context,
        slices: Self::SlicesMut<'_>,
    ) -> Self::SliceMutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSlicesMut { slices, len, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                (layout, ptr::from_mut(slice))
            })
            .collect();
        ErasedSoaSliceMutPtrs {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    fn mut_slices_as_slices<'a>(
        context: &Self::Context,
        slices: Self::SlicesMut<'a>,
    ) -> Self::Slices<'a> {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSlicesMut { slices, len, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);
                (layout, &*slice)
            })
            .collect();
        ErasedSoaSlices {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    fn slice_refs_as_ptrs(context: &Self::Context, slices: Self::Slices<'_>) -> Self::Ptrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSlices { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let ptrs = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                let data = slice.as_ptr();
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts(data, len);
                ErasedFieldPtr::new(layout, ptr)
            })
            .collect();
        ErasedSoaPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn mut_slice_refs_as_ptrs(
        context: &Self::Context,
        slices: Self::SlicesMut<'_>,
    ) -> Self::MutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSlicesMut { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let ptrs = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                let data = slice.as_mut_ptr();
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts_mut(data, len);
                (layout, ptr)
            })
            .collect();
        ErasedSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    unsafe fn slices_drop_in_place(context: &Self::Context, slices: Self::SliceMutPtrs) {
        let ErasedSoaContext {
            field_layouts,
            drop_fields,
            ..
        } = context;
        let Some(drop_fields) = drop_fields else {
            return;
        };

        let ErasedSoaSliceMutPtrs {
            mut slices, len, ..
        } = slices;
        assert_eq!(field_layouts.len(), slices.len());

        for ((ref layout, ref mut slice), field_layout) in iter::zip(&mut slices, field_layouts) {
            assert_eq!(layout, field_layout);
            assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

            let data = slice.cast();
            let len = slice.len().checked_div(field_layout.size()).unwrap_or(0);
            *slice = ptr::slice_from_raw_parts_mut(data, len);
        }

        for _ in 0..len {
            drop_fields(slices.as_ref());

            for (ref field_layout, ref mut slice) in slices.iter_mut() {
                let len = field_layout.size();
                let data = unsafe { slice.cast::<u8>().add(len) };
                *slice = ptr::slice_from_raw_parts_mut(data, len);
            }
        }
    }
}

#[inline]
fn validate_layout<Fields, I>(item: I) -> Layout
where
    I: Borrow<Layout>,
{
    let layout: &Layout = item.borrow();

    let input_align = layout.align();
    let max_align = align_of::<Fields>();
    assert!(
        input_align <= max_align,
        "input alignment must be less than or equal to {max_align}, but got {input_align}",
    );
    layout.clone()
}

#[cold]
#[track_caller]
#[inline(never)]
fn assert_value_buffer_len_failed(buffer_len: usize, layout_size: usize) -> ! {
    panic!("buffer len {buffer_len} should match layout size {layout_size}")
}

#[inline]
#[track_caller]
fn assert_value_buffer_len(buffer_len: usize, layout_size: usize) {
    if buffer_len == layout_size {
        return;
    }
    assert_value_buffer_len_failed(buffer_len, layout_size)
}

#[cold]
#[track_caller]
#[inline(never)]
fn assert_value_buffer_align_failed(layout_align: usize) -> ! {
    panic!("buffer should be aligned to {layout_align}")
}

#[inline]
#[track_caller]
fn assert_value_buffer_align(buffer: *const u8, layout_align: usize) {
    if buffer.align_offset(layout_align) == 0 {
        return;
    }
    assert_value_buffer_align_failed(layout_align)
}
