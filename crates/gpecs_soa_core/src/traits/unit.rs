use core::{
    ptr::{self, NonNull},
    slice,
};

use crate::traits::{
    SoaCloneToUninitContext, SoaContext, SoaRaw, SoaRawContext, SoaReadContext, SoaWriteContext,
};

unsafe impl SoaRawContext<()> for () {
    type Ptrs<'a> = *const ();

    #[inline]
    fn ptrs_upcast<'short, 'long: 'short>(from: Self::Ptrs<'long>) -> Self::Ptrs<'short> {
        from
    }

    #[inline]
    fn ptrs_dangling(&self) -> Self::Ptrs<'_> {
        ptr::dangling()
    }

    #[inline]
    #[expect(clippy::zst_offset, reason = "as a reference for other manual impls")]
    unsafe fn ptrs_add<'a>(&'a self, ptrs: Self::Ptrs<'a>, count: usize) -> Self::Ptrs<'a> {
        unsafe { ptrs.add(count) }
    }

    #[inline]
    unsafe fn ptrs_offset_from(&self, ptrs: Self::Ptrs<'_>, origin: Self::Ptrs<'_>) -> isize {
        unsafe { ptrs.offset_from(origin) }
    }

    type MutPtrs<'a> = *mut ();

    #[inline]
    fn mut_ptrs_upcast<'short, 'long: 'short>(from: Self::MutPtrs<'long>) -> Self::MutPtrs<'short> {
        from
    }

    #[inline]
    fn mut_ptrs_dangling(&self) -> Self::MutPtrs<'_> {
        ptr::dangling_mut()
    }

    #[inline]
    #[expect(clippy::zst_offset, reason = "as a reference for other manual impls")]
    unsafe fn mut_ptrs_add<'a>(
        &'a self,
        ptrs: Self::MutPtrs<'a>,
        count: usize,
    ) -> Self::MutPtrs<'a> {
        unsafe { ptrs.add(count) }
    }

    #[inline]
    unsafe fn mut_ptrs_offset_from(
        &self,
        ptrs: Self::MutPtrs<'_>,
        origin: Self::Ptrs<'_>,
    ) -> isize {
        unsafe { ptrs.offset_from(origin) }
    }

    #[inline]
    fn ptrs_cast_const<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::Ptrs<'a> {
        ptrs.cast_const()
    }

    #[inline]
    fn ptrs_cast_mut<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::MutPtrs<'a> {
        ptrs.cast_mut()
    }

    #[inline]
    unsafe fn ptrs_swap_nonoverlapping(
        &self,
        x: Self::MutPtrs<'_>,
        y: Self::MutPtrs<'_>,
        count: usize,
    ) {
        unsafe { ptr::swap_nonoverlapping(x, y, count) }
    }

    #[inline]
    unsafe fn ptrs_copy_nonoverlapping(
        &self,
        src: Self::Ptrs<'_>,
        dst: Self::MutPtrs<'_>,
        count: usize,
    ) {
        unsafe { ptr::copy_nonoverlapping(src, dst, count) }
    }

    #[inline]
    #[allow(dropping_copy_types, reason = "as a reference for other manual impls")]
    unsafe fn ptrs_drop_in_place(&self, to_drop: Self::MutPtrs<'_>) {
        unsafe { ptr::drop_in_place(to_drop) }
    }

    type NonNullPtrs<'a> = NonNull<()>;

    #[inline]
    fn nonnull_ptrs_upcast<'short, 'long: 'short>(
        from: Self::NonNullPtrs<'long>,
    ) -> Self::NonNullPtrs<'short> {
        from
    }

    #[inline]
    fn nonnull_ptrs_dangling(&self) -> Self::NonNullPtrs<'_> {
        NonNull::dangling()
    }

    #[inline]
    unsafe fn nonnull_ptrs_from_ptrs<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::NonNullPtrs<'a> {
        unsafe { NonNull::new_unchecked(ptrs.cast_mut()) }
    }

    #[inline]
    unsafe fn nonnull_ptrs_from_mut_ptrs<'a>(
        &'a self,
        ptrs: Self::MutPtrs<'a>,
    ) -> Self::NonNullPtrs<'a> {
        unsafe { NonNull::new_unchecked(ptrs) }
    }

    #[inline]
    #[expect(clippy::zst_offset, reason = "as a reference for other manual impls")]
    unsafe fn nonnull_ptrs_add<'a>(
        &'a self,
        ptrs: Self::NonNullPtrs<'a>,
        count: usize,
    ) -> Self::NonNullPtrs<'a> {
        unsafe { ptrs.add(count) }
    }

    #[inline]
    unsafe fn nonnull_ptrs_offset_from(
        &self,
        ptrs: Self::NonNullPtrs<'_>,
        origin: Self::NonNullPtrs<'_>,
    ) -> isize {
        unsafe { ptrs.offset_from(origin) }
    }

    #[inline]
    fn nonnull_ptrs_as_ptrs<'a>(&'a self, ptrs: Self::NonNullPtrs<'a>) -> Self::Ptrs<'a> {
        ptrs.as_ptr().cast_const()
    }

    #[inline]
    fn nonnull_ptrs_as_mut_ptrs<'a>(&'a self, ptrs: Self::NonNullPtrs<'a>) -> Self::MutPtrs<'a> {
        ptrs.as_ptr()
    }

    #[inline]
    unsafe fn nonnull_ptrs_swap_nonoverlapping(
        &self,
        x: Self::NonNullPtrs<'_>,
        y: Self::NonNullPtrs<'_>,
        count: usize,
    ) {
        let x = x.as_ptr();
        let y = y.as_ptr();
        unsafe { ptr::swap_nonoverlapping(x, y, count) }
    }

    #[inline]
    unsafe fn nonnull_ptrs_copy_nonoverlapping(
        &self,
        src: Self::NonNullPtrs<'_>,
        dst: Self::NonNullPtrs<'_>,
        count: usize,
    ) {
        unsafe { dst.copy_from_nonoverlapping(src, count) }
    }

    #[inline]
    unsafe fn nonnull_ptrs_drop_in_place(&self, to_drop: Self::NonNullPtrs<'_>) {
        unsafe { to_drop.drop_in_place() }
    }

    type SlicePtrs<'a> = *const [()];

    #[inline]
    fn slice_ptrs_upcast<'short, 'long: 'short>(
        from: Self::SlicePtrs<'long>,
    ) -> Self::SlicePtrs<'short> {
        from
    }

    #[inline]
    fn slice_ptrs_from_raw_parts<'a>(
        &'a self,
        data: Self::Ptrs<'a>,
        len: usize,
    ) -> Self::SlicePtrs<'a> {
        ptr::slice_from_raw_parts(data, len)
    }

    #[inline]
    fn slice_ptrs_len(&self, slices: &Self::SlicePtrs<'_>) -> usize {
        slices.len()
    }

    #[inline]
    fn slice_ptrs_as_ptrs<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::Ptrs<'a> {
        slices.cast() // should be `slices.as_ptr()` but it's unstable
    }

    type SliceMutPtrs<'a> = *mut [()];

    #[inline]
    fn mut_slice_ptrs_upcast<'short, 'long: 'short>(
        from: Self::SliceMutPtrs<'long>,
    ) -> Self::SliceMutPtrs<'short> {
        from
    }

    #[inline]
    fn mut_slice_ptrs_from_raw_parts<'a>(
        &'a self,
        data: Self::MutPtrs<'a>,
        len: usize,
    ) -> Self::SliceMutPtrs<'a> {
        ptr::slice_from_raw_parts_mut(data, len)
    }

    #[inline]
    fn mut_slice_ptrs_len(&self, slices: &Self::SliceMutPtrs<'_>) -> usize {
        slices.len()
    }

    #[inline]
    fn mut_slice_ptrs_as_ptrs<'a>(&'a self, slices: Self::SliceMutPtrs<'a>) -> Self::Ptrs<'a> {
        slices.cast_const().cast() // should be `slices.as_ptr()` but it's unstable
    }

    #[inline]
    fn mut_slice_ptrs_as_mut_ptrs<'a>(
        &'a self,
        slices: Self::SliceMutPtrs<'a>,
    ) -> Self::MutPtrs<'a> {
        slices.cast() // should be `slices.as_mut_ptr()` but it's unstable
    }

    #[inline]
    fn slice_ptrs_cast_const<'a>(&'a self, slices: Self::SliceMutPtrs<'a>) -> Self::SlicePtrs<'a> {
        slices.cast_const()
    }

    #[inline]
    fn slice_ptrs_cast_mut<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::SliceMutPtrs<'a> {
        slices.cast_mut()
    }

    #[inline]
    unsafe fn slices_drop_in_place(&self, slices_to_drop: Self::SliceMutPtrs<'_>) {
        unsafe { ptr::drop_in_place(slices_to_drop) }
    }

    type SliceNonNullPtrs<'a> = NonNull<[()]>;

    #[inline]
    fn nonnull_slice_ptrs_upcast<'short, 'long: 'short>(
        from: Self::SliceNonNullPtrs<'long>,
    ) -> Self::SliceNonNullPtrs<'short> {
        from
    }

    #[inline]
    fn nonnull_slice_ptrs_from_raw_parts<'a>(
        &'a self,
        data: Self::NonNullPtrs<'a>,
        len: usize,
    ) -> Self::SliceNonNullPtrs<'a> {
        NonNull::slice_from_raw_parts(data, len)
    }

    #[inline]
    fn nonnull_slice_ptrs_len(&self, slices: &Self::SliceNonNullPtrs<'_>) -> usize {
        slices.len()
    }
}

unsafe impl SoaRaw for () {
    type Context = ();
    type Fields = ();
}

unsafe impl SoaCloneToUninitContext<()> for () {
    #[inline]
    unsafe fn ptrs_clone_to_uninit(&self, _src: Self::Ptrs<'_>, _dst: Self::MutPtrs<'_>) {}

    #[inline]
    unsafe fn nonnull_ptrs_clone_to_uninit(
        &self,
        _src: Self::NonNullPtrs<'_>,
        _dst: Self::NonNullPtrs<'_>,
    ) {
    }
}

unsafe impl<'a> SoaReadContext<'a, ()> for () {
    #[inline]
    unsafe fn ptrs_read(&'a self, src: Self::Ptrs<'a>) {
        unsafe { src.read() }
    }

    #[inline]
    unsafe fn mut_ptrs_read(&'a self, src: Self::MutPtrs<'a>) {
        unsafe { src.read() }
    }

    #[inline]
    unsafe fn nonnull_ptrs_read(&'a self, src: Self::NonNullPtrs<'a>) {
        unsafe { src.read() }
    }
}

unsafe impl SoaWriteContext<(), ()> for () {
    #[inline]
    unsafe fn ptrs_write(&self, dst: Self::MutPtrs<'_>, value: ()) {
        unsafe { dst.write(value) }
    }

    #[inline]
    unsafe fn nonnull_ptrs_write(&self, dst: Self::NonNullPtrs<'_>, value: ()) {
        unsafe { dst.write(value) }
    }
}

unsafe impl<'data> SoaContext<'data, ()> for () {
    type Refs<'a> = &'data ();

    #[inline]
    fn refs_upcast<'short, 'long: 'short>(from: Self::Refs<'long>) -> Self::Refs<'short> {
        from
    }

    #[inline]
    unsafe fn refs_from_ptrs<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::Refs<'a> {
        unsafe { ptrs.as_ref_unchecked() }
    }

    #[inline]
    fn refs_as_ptrs<'a>(&'a self, refs: Self::Refs<'a>) -> Self::Ptrs<'a> {
        ptr::from_ref(refs)
    }

    type RefsMut<'a> = &'data mut ();

    #[inline]
    fn mut_refs_upcast<'short, 'long: 'short>(from: Self::RefsMut<'long>) -> Self::RefsMut<'short> {
        from
    }

    #[inline]
    unsafe fn mut_refs_from_mut_ptrs<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::RefsMut<'a> {
        unsafe { ptrs.as_mut_unchecked() }
    }

    #[inline]
    fn mut_refs_as_ptrs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::Ptrs<'a> {
        ptr::from_ref(refs)
    }

    #[inline]
    fn mut_refs_as_mut_ptrs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::MutPtrs<'a> {
        ptr::from_mut(refs)
    }

    #[inline]
    fn mut_refs_as_refs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::Refs<'a> {
        refs
    }

    type Slices<'a> = &'data [()];

    #[inline]
    fn slices_upcast<'short, 'long: 'short>(from: Self::Slices<'long>) -> Self::Slices<'short> {
        from
    }

    #[inline]
    unsafe fn slices_from_slice_ptrs<'a>(
        &'a self,
        slices: Self::SlicePtrs<'a>,
    ) -> Self::Slices<'a> {
        unsafe { slices.as_ref_unchecked() }
    }

    #[inline]
    unsafe fn slices_from_raw_parts<'a>(
        &'a self,
        data: Self::Ptrs<'a>,
        len: usize,
    ) -> Self::Slices<'a> {
        unsafe { slice::from_raw_parts(data, len) }
    }

    #[inline]
    fn slices_as_slice_ptrs<'a>(&'a self, slices: Self::Slices<'a>) -> Self::SlicePtrs<'a> {
        ptr::from_ref(slices)
    }

    #[inline]
    fn slices_as_ptrs<'a>(&'a self, slices: Self::Slices<'a>) -> Self::Ptrs<'a> {
        slices.as_ptr()
    }

    #[inline]
    fn slices_len(&self, slices: &Self::Slices<'_>) -> usize {
        slices.len()
    }

    type SlicesMut<'a> = &'data mut [()];

    #[inline]
    fn mut_slices_upcast<'short, 'long: 'short>(
        from: Self::SlicesMut<'long>,
    ) -> Self::SlicesMut<'short> {
        from
    }

    #[inline]
    unsafe fn mut_slices_from_mut_slice_ptrs<'a>(
        &'a self,
        slices: Self::SliceMutPtrs<'a>,
    ) -> Self::SlicesMut<'a> {
        unsafe { slices.as_mut_unchecked() }
    }

    #[inline]
    unsafe fn mut_slices_from_raw_parts<'a>(
        &'a self,
        data: Self::MutPtrs<'a>,
        len: usize,
    ) -> Self::SlicesMut<'a> {
        unsafe { slice::from_raw_parts_mut(data, len) }
    }

    #[inline]
    fn mut_slices_as_slice_ptrs<'a>(&'a self, slices: Self::SlicesMut<'a>) -> Self::SlicePtrs<'a> {
        ptr::from_ref(slices)
    }

    #[inline]
    fn mut_slices_as_mut_slice_ptrs<'a>(
        &'a self,
        slices: Self::SlicesMut<'a>,
    ) -> Self::SliceMutPtrs<'a> {
        ptr::from_mut(slices)
    }

    #[inline]
    fn mut_slices_as_ptrs<'a>(&'a self, slices: Self::SlicesMut<'a>) -> Self::Ptrs<'a> {
        slices.as_ptr()
    }

    #[inline]
    fn mut_slices_as_mut_ptrs<'a>(&'a self, slices: Self::SlicesMut<'a>) -> Self::MutPtrs<'a> {
        slices.as_mut_ptr()
    }

    #[inline]
    fn mut_slices_len(&self, slices: &Self::SlicesMut<'_>) -> usize {
        slices.len()
    }

    #[inline]
    fn mut_slices_as_slices<'a>(&'a self, slices: Self::SlicesMut<'a>) -> Self::Slices<'a> {
        slices
    }
}
