use core::{
    ptr::{self, NonNull},
    slice,
};

use crate::ptr::{align_cast_then_advance, align_up};

#[allow(clippy::missing_safety_doc)]
pub unsafe trait Soa: Sized {
    type Ptrs: Copy;
    type MutPtrs: Copy;
    type NonNullPtrs: Copy;

    type Refs<'a>
    where
        Self: 'a;

    type RefsMut<'a>
    where
        Self: 'a;

    type SlicePtrs: Copy;
    type SliceMutPtrs: Copy;

    type Slices<'a>
    where
        Self: 'a;

    type SlicesMut<'a>
    where
        Self: 'a;

    fn min_size_of_components() -> usize;
    fn len_in_bytes_unaligned(initial: usize, len: usize) -> usize;

    fn ptrs_dangling() -> Self::MutPtrs;
    unsafe fn ptrs(ptr: *mut u8, len: usize) -> Self::MutPtrs;
    unsafe fn ptrs_to_nonnull(ptrs: Self::MutPtrs) -> Self::NonNullPtrs;

    fn ptrs_cast_const(ptrs: Self::MutPtrs) -> Self::Ptrs;
    fn ptrs_cast_mut(ptrs: Self::Ptrs) -> Self::MutPtrs;

    unsafe fn ptrs_add(ptrs: Self::Ptrs, offset: usize) -> Self::Ptrs;
    unsafe fn ptrs_add_mut(ptrs: Self::MutPtrs, offset: usize) -> Self::MutPtrs;
    unsafe fn ptrs_swap(a: Self::MutPtrs, b: Self::MutPtrs);
    unsafe fn ptrs_copy(src: Self::Ptrs, dst: Self::MutPtrs, len: usize);
    unsafe fn ptrs_copy_rev(src: Self::Ptrs, dst: Self::MutPtrs, len: usize);
    unsafe fn ptrs_copy_nonoverlapping(src: Self::Ptrs, dst: Self::MutPtrs, len: usize);
    unsafe fn ptrs_read(src: Self::Ptrs) -> Self;
    unsafe fn ptrs_write(dst: Self::MutPtrs, value: Self);
    unsafe fn ptrs_drop_in_place(ptrs: Self::MutPtrs);

    unsafe fn as_refs<'a>(ptrs: Self::Ptrs) -> Self::Refs<'a>;
    unsafe fn as_mut_refs<'a>(ptrs: Self::MutPtrs) -> Self::RefsMut<'a>;

    fn refs_as_ptrs(refs: Self::Refs<'_>) -> Self::Ptrs;
    fn mut_refs_as_ptrs(refs: Self::RefsMut<'_>) -> Self::MutPtrs;
    fn mut_refs_as_refs(refs: Self::RefsMut<'_>) -> Self::Refs<'_>;

    fn slices_from_raw_parts(ptrs: Self::Ptrs, len: usize) -> Self::SlicePtrs;
    fn slices_from_raw_parts_mut(ptrs: Self::MutPtrs, len: usize) -> Self::SliceMutPtrs;

    unsafe fn slices_as_refs<'a>(slices: Self::SlicePtrs) -> Self::Slices<'a>;
    unsafe fn mut_slices_as_refs<'a>(slices: Self::SliceMutPtrs) -> Self::SlicesMut<'a>;

    fn slice_refs_as_ptrs(slices: Self::Slices<'_>) -> Self::SlicePtrs;
    fn mut_slice_refs_as_ptrs(slices: Self::SlicesMut<'_>) -> Self::SliceMutPtrs;

    unsafe fn slices_drop_in_place(slices: Self::SliceMutPtrs);
}

unsafe impl<T, U, V> Soa for (T, U, V) {
    type Ptrs = (*const T, *const U, *const V);
    type MutPtrs = (*mut T, *mut U, *mut V);
    type NonNullPtrs = (NonNull<T>, NonNull<U>, NonNull<V>);

    type Refs<'a> = (&'a T, &'a U, &'a V)
    where
        Self: 'a;

    type RefsMut<'a> = (&'a mut T, &'a mut U, &'a mut V)
    where
        Self: 'a;

    type SlicePtrs = (*const [T], *const [U], *const [V]);
    type SliceMutPtrs = (*mut [T], *mut [U], *mut [V]);

    type Slices<'a> = (&'a [T], &'a [U], &'a [V])
    where
        Self: 'a;

    type SlicesMut<'a> = (&'a mut [T], &'a mut [U], &'a mut [V])
    where
        Self: 'a;

    fn min_size_of_components() -> usize {
        size_of::<T>() + size_of::<U>() + size_of::<V>()
    }

    fn len_in_bytes_unaligned(initial: usize, len: usize) -> usize {
        let mut len_in_bytes = initial;
        len_in_bytes = align_up::<T>(len_in_bytes) + (len * size_of::<T>());
        len_in_bytes = align_up::<U>(len_in_bytes) + (len * size_of::<U>());
        len_in_bytes = align_up::<V>(len_in_bytes) + (len * size_of::<V>());

        len_in_bytes
    }

    fn ptrs_dangling() -> Self::MutPtrs {
        (
            NonNull::dangling().as_ptr(),
            NonNull::dangling().as_ptr(),
            NonNull::dangling().as_ptr(),
        )
    }

    unsafe fn ptrs(ptr: *mut u8, len: usize) -> Self::MutPtrs {
        let (t_ptr, ptr) = unsafe { align_cast_then_advance(ptr, len) };
        let (u_ptr, ptr) = unsafe { align_cast_then_advance(ptr, len) };
        let (v_ptr, _) = unsafe { align_cast_then_advance(ptr, len) };
        (t_ptr, u_ptr, v_ptr)
    }

    unsafe fn ptrs_to_nonnull(ptrs: Self::MutPtrs) -> Self::NonNullPtrs {
        let (t_ptr, u_ptr, v_ptr) = ptrs;
        unsafe {
            (
                NonNull::new_unchecked(t_ptr),
                NonNull::new_unchecked(u_ptr),
                NonNull::new_unchecked(v_ptr),
            )
        }
    }

    fn ptrs_cast_const(ptrs: Self::MutPtrs) -> Self::Ptrs {
        let (t_ptr, u_ptr, v_ptr) = ptrs;
        (t_ptr.cast_const(), u_ptr.cast_const(), v_ptr.cast_const())
    }

    fn ptrs_cast_mut(ptrs: Self::Ptrs) -> Self::MutPtrs {
        let (t_ptr, u_ptr, v_ptr) = ptrs;
        (t_ptr.cast_mut(), u_ptr.cast_mut(), v_ptr.cast_mut())
    }

    unsafe fn ptrs_add(ptrs: Self::Ptrs, offset: usize) -> Self::Ptrs {
        let (t_ptr, u_ptr, v_ptr) = ptrs;

        unsafe {
            let t_ptr = t_ptr.add(offset);
            let u_ptr = u_ptr.add(offset);
            let v_ptr = v_ptr.add(offset);
            (t_ptr, u_ptr, v_ptr)
        }
    }

    unsafe fn ptrs_add_mut(ptrs: Self::MutPtrs, offset: usize) -> Self::MutPtrs {
        let (t_ptr, u_ptr, v_ptr) = ptrs;

        unsafe {
            let t_ptr = t_ptr.add(offset);
            let u_ptr = u_ptr.add(offset);
            let v_ptr = v_ptr.add(offset);
            (t_ptr, u_ptr, v_ptr)
        }
    }

    unsafe fn ptrs_swap(a: Self::MutPtrs, b: Self::MutPtrs) {
        let (a_t_ptr, a_u_ptr, a_v_ptr) = a;
        let (b_t_ptr, b_u_ptr, b_v_ptr) = b;

        unsafe {
            ptr::swap(a_t_ptr, b_t_ptr);
            ptr::swap(a_u_ptr, b_u_ptr);
            ptr::swap(a_v_ptr, b_v_ptr);
        }
    }

    unsafe fn ptrs_copy(src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
        unsafe {
            ptr::copy(src.0, dst.0, len);
            ptr::copy(src.1, dst.1, len);
            ptr::copy(src.2, dst.2, len);
        }
    }

    unsafe fn ptrs_copy_rev(src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
        unsafe {
            ptr::copy(src.2, dst.2, len);
            ptr::copy(src.1, dst.1, len);
            ptr::copy(src.0, dst.0, len);
        }
    }

    unsafe fn ptrs_copy_nonoverlapping(src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
        unsafe {
            ptr::copy_nonoverlapping(src.0, dst.0, len);
            ptr::copy_nonoverlapping(src.1, dst.1, len);
            ptr::copy_nonoverlapping(src.2, dst.2, len);
        }
    }

    unsafe fn ptrs_read(ptrs: Self::Ptrs) -> Self {
        let (t_ptr, u_ptr, v_ptr) = ptrs;

        unsafe {
            let t = ptr::read(t_ptr);
            let u = ptr::read(u_ptr);
            let v = ptr::read(v_ptr);
            (t, u, v)
        }
    }

    unsafe fn ptrs_write(dst: Self::MutPtrs, value: Self) {
        let (t_ptr, u_ptr, v_ptr) = dst;
        let (t, u, v) = value;

        unsafe {
            ptr::write(t_ptr, t);
            ptr::write(u_ptr, u);
            ptr::write(v_ptr, v);
        }
    }

    unsafe fn ptrs_drop_in_place(ptrs: Self::MutPtrs) {
        let (t_ptr, u_ptr, v_ptr) = ptrs;

        unsafe {
            ptr::drop_in_place(t_ptr);
            ptr::drop_in_place(u_ptr);
            ptr::drop_in_place(v_ptr);
        }
    }

    unsafe fn as_refs<'a>(ptrs: Self::Ptrs) -> Self::Refs<'a> {
        let (t_ptr, u_ptr, v_ptr) = ptrs;

        unsafe {
            let t_ref = &*t_ptr;
            let u_ref = &*u_ptr;
            let v_ref = &*v_ptr;
            (t_ref, u_ref, v_ref)
        }
    }

    unsafe fn as_mut_refs<'a>(ptrs: Self::MutPtrs) -> Self::RefsMut<'a> {
        let (t_ptr, u_ptr, v_ptr) = ptrs;

        unsafe {
            let t_ref = &mut *t_ptr;
            let u_ref = &mut *u_ptr;
            let v_ref = &mut *v_ptr;
            (t_ref, u_ref, v_ref)
        }
    }

    fn refs_as_ptrs(refs: Self::Refs<'_>) -> Self::Ptrs {
        let (t_ref, u_ref, v_ref) = refs;
        (t_ref, u_ref, v_ref)
    }

    fn mut_refs_as_ptrs(refs: Self::RefsMut<'_>) -> Self::MutPtrs {
        let (t_ref, u_ref, v_ref) = refs;
        (t_ref, u_ref, v_ref)
    }

    fn mut_refs_as_refs(refs: Self::RefsMut<'_>) -> Self::Refs<'_> {
        let (t_ref, u_ref, v_ref) = refs;
        (t_ref, u_ref, v_ref)
    }

    fn slices_from_raw_parts(ptrs: Self::Ptrs, len: usize) -> Self::SlicePtrs {
        let (t_ptr, u_ptr, v_ptr) = ptrs;

        let t_slice = ptr::slice_from_raw_parts(t_ptr, len);
        let u_slice = ptr::slice_from_raw_parts(u_ptr, len);
        let v_slice = ptr::slice_from_raw_parts(v_ptr, len);
        (t_slice, u_slice, v_slice)
    }

    fn slices_from_raw_parts_mut(ptrs: Self::MutPtrs, len: usize) -> Self::SliceMutPtrs {
        let (t_ptr, u_ptr, v_ptr) = ptrs;

        let t_slice = ptr::slice_from_raw_parts_mut(t_ptr, len);
        let u_slice = ptr::slice_from_raw_parts_mut(u_ptr, len);
        let v_slice = ptr::slice_from_raw_parts_mut(v_ptr, len);
        (t_slice, u_slice, v_slice)
    }

    unsafe fn slices_as_refs<'a>(slices: Self::SlicePtrs) -> Self::Slices<'a> {
        let (t_slice, u_slice, v_slice) = slices;

        unsafe {
            let t_slice = slice::from_raw_parts(t_slice.cast(), t_slice.len());
            let u_slice = slice::from_raw_parts(u_slice.cast(), u_slice.len());
            let v_slice = slice::from_raw_parts(v_slice.cast(), v_slice.len());
            (t_slice, u_slice, v_slice)
        }
    }

    unsafe fn mut_slices_as_refs<'a>(slices: Self::SliceMutPtrs) -> Self::SlicesMut<'a> {
        let (t_slice, u_slice, v_slice) = slices;

        unsafe {
            let t_slice = slice::from_raw_parts_mut(t_slice.cast(), t_slice.len());
            let u_slice = slice::from_raw_parts_mut(u_slice.cast(), u_slice.len());
            let v_slice = slice::from_raw_parts_mut(v_slice.cast(), v_slice.len());
            (t_slice, u_slice, v_slice)
        }
    }

    fn slice_refs_as_ptrs(slices: Self::Slices<'_>) -> Self::SlicePtrs {
        let (t_slice, u_slice, v_slice) = slices;
        (t_slice, u_slice, v_slice)
    }

    fn mut_slice_refs_as_ptrs(slices: Self::SlicesMut<'_>) -> Self::SliceMutPtrs {
        let (t_slice, u_slice, v_slice) = slices;
        (t_slice, u_slice, v_slice)
    }

    unsafe fn slices_drop_in_place(slices: Self::SliceMutPtrs) {
        let (t_slice, u_slice, v_slice) = slices;

        unsafe {
            ptr::drop_in_place(t_slice);
            ptr::drop_in_place(u_slice);
            ptr::drop_in_place(v_slice);
        }
    }
}
