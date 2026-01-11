use core::{
    alloc::{Layout, LayoutError},
    ptr::{self, NonNull},
    slice,
};

use crate::{
    field::FieldDescriptor,
    traits::{
        MutPtrs, Ptrs, RawSoa, RawSoaContext, SliceMutPtrs, SlicePtrs, Soa, SoaCloneToUninit,
        SoaRead, SoaTrustedFields, SoaWrite,
    },
};

unsafe impl RawSoaContext for () {
    type FieldDescriptors<'a> = [FieldDescriptor; 1];

    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: Self::FieldDescriptors<'long>,
    ) -> Self::FieldDescriptors<'short> {
        from
    }

    #[inline]
    fn field_descriptors(&self) -> Self::FieldDescriptors<'_> {
        [FieldDescriptor::of::<()>()]
    }

    #[inline]
    fn buffer_layout(&self, capacity: usize) -> Result<Layout, LayoutError> {
        Layout::array::<()>(capacity)
    }

    #[inline]
    fn capacity_from(&self, _buffer_layout: Layout) -> usize {
        usize::MAX
    }

    type Ptrs<'a> = *const ();

    #[inline]
    fn upcast_ptrs<'short, 'long: 'short>(from: Self::Ptrs<'long>) -> Self::Ptrs<'short> {
        from
    }

    #[inline]
    fn ptrs_dangling(&self) -> Self::Ptrs<'_> {
        ptr::dangling()
    }

    #[inline]
    unsafe fn ptrs_from_buffer(&self, buffer: *const u8, _capacity: usize) -> Self::Ptrs<'_> {
        buffer.cast()
    }

    #[inline]
    #[expect(clippy::zst_offset, reason = "reference to other manual impls")]
    unsafe fn ptrs_add<'a>(&'a self, ptrs: Self::Ptrs<'a>, offset: usize) -> Self::Ptrs<'a> {
        unsafe { ptrs.add(offset) }
    }

    #[inline]
    unsafe fn ptrs_offset_from(&self, ptrs: Self::Ptrs<'_>, origin: Self::Ptrs<'_>) -> isize {
        unsafe { ptrs.offset_from(origin) }
    }

    type MutPtrs<'a> = *mut ();

    #[inline]
    fn upcast_mut_ptrs<'short, 'long: 'short>(from: Self::MutPtrs<'long>) -> Self::MutPtrs<'short> {
        from
    }

    #[inline]
    fn ptrs_dangling_mut(&self) -> Self::MutPtrs<'_> {
        ptr::dangling_mut()
    }

    #[inline]
    unsafe fn ptrs_from_buffer_mut(&self, buffer: *mut u8, _capacity: usize) -> Self::MutPtrs<'_> {
        buffer.cast()
    }

    #[inline]
    #[expect(clippy::zst_offset, reason = "reference to other manual impls")]
    unsafe fn ptrs_add_mut<'a>(
        &'a self,
        ptrs: Self::MutPtrs<'a>,
        offset: usize,
    ) -> Self::MutPtrs<'a> {
        unsafe { ptrs.add(offset) }
    }

    #[inline]
    unsafe fn ptrs_offset_from_mut(
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
    unsafe fn ptrs_swap(&self, a: Self::MutPtrs<'_>, b: Self::MutPtrs<'_>) {
        unsafe { ptr::swap(a, b) }
    }

    #[inline]
    unsafe fn ptrs_copy(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, len: usize) {
        unsafe { ptr::copy(src, dst, len) }
    }

    #[inline]
    unsafe fn ptrs_copy_rev(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, len: usize) {
        unsafe { ptr::copy(src, dst, len) }
    }

    #[inline]
    unsafe fn ptrs_copy_nonoverlapping(
        &self,
        src: Self::Ptrs<'_>,
        dst: Self::MutPtrs<'_>,
        len: usize,
    ) {
        unsafe { ptr::copy_nonoverlapping(src, dst, len) }
    }

    #[inline]
    unsafe fn ptrs_drop_in_place(&self, ptrs: Self::MutPtrs<'_>) {
        unsafe { ptr::drop_in_place(ptrs) }
    }

    type NonNullPtrs<'a> = NonNull<()>;

    #[inline]
    fn upcast_nonnull_ptrs<'short, 'long: 'short>(
        from: Self::NonNullPtrs<'long>,
    ) -> Self::NonNullPtrs<'short> {
        from
    }

    #[inline]
    unsafe fn ptrs_to_nonnull<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::NonNullPtrs<'a> {
        unsafe { NonNull::new_unchecked(ptrs) }
    }

    #[inline]
    fn nonnull_to_ptrs<'a>(&'a self, ptrs: Self::NonNullPtrs<'a>) -> Self::MutPtrs<'a> {
        ptrs.as_ptr()
    }

    type SlicePtrs<'a> = *const [()];

    #[inline]
    fn upcast_slice_ptrs<'short, 'long: 'short>(
        from: Self::SlicePtrs<'long>,
    ) -> Self::SlicePtrs<'short> {
        from
    }

    #[inline]
    fn slice_ptrs_from_raw_parts<'a>(
        &'a self,
        ptrs: Self::Ptrs<'a>,
        len: usize,
    ) -> Self::SlicePtrs<'a> {
        ptr::slice_from_raw_parts(ptrs, len)
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
    fn upcast_mut_slice_ptrs<'short, 'long: 'short>(
        from: SliceMutPtrs<'long, Self>,
    ) -> SliceMutPtrs<'short, Self> {
        from
    }

    #[inline]
    fn mut_slice_ptrs_from_raw_parts<'a>(
        &'a self,
        ptrs: Self::MutPtrs<'a>,
        len: usize,
    ) -> Self::SliceMutPtrs<'a> {
        ptr::slice_from_raw_parts_mut(ptrs, len)
    }

    #[inline]
    fn mut_slice_ptrs_len(&self, slices: &Self::SliceMutPtrs<'_>) -> usize {
        slices.len()
    }

    #[inline]
    fn mut_slice_ptrs_as_ptrs<'a>(&'a self, slices: Self::SliceMutPtrs<'a>) -> Self::MutPtrs<'a> {
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
    unsafe fn slices_drop_in_place(&self, slices: Self::SliceMutPtrs<'_>) {
        unsafe { ptr::drop_in_place(slices) }
    }
}

unsafe impl RawSoa for () {
    type Context = ();
    type Fields = ();
}

unsafe impl<'a> Soa<'a> for () {
    type Refs<'ctx> = &'a Self;

    #[inline]
    fn upcast_refs<'short, 'long: 'short>(from: Self::Refs<'long>) -> Self::Refs<'short> {
        from
    }

    type RefsMut<'ctx> = &'a mut Self;

    #[inline]
    fn upcast_refs_mut<'short, 'long: 'short>(from: Self::RefsMut<'long>) -> Self::RefsMut<'short> {
        from
    }

    #[inline]
    unsafe fn ptrs_to_refs<'ctx>(
        _context: &'ctx Self::Context,
        ptrs: Ptrs<'ctx, Self>,
    ) -> Self::Refs<'ctx> {
        unsafe { &*ptrs }
    }

    #[inline]
    unsafe fn ptrs_to_refs_mut<'ctx>(
        _context: &'ctx Self::Context,
        ptrs: MutPtrs<'ctx, Self>,
    ) -> Self::RefsMut<'ctx> {
        unsafe { &mut *ptrs }
    }

    #[inline]
    fn refs_as_ptrs<'ctx>(
        _context: &'ctx Self::Context,
        refs: Self::Refs<'ctx>,
    ) -> Ptrs<'ctx, Self> {
        ptr::from_ref(refs)
    }

    #[inline]
    fn refs_mut_as_ptrs<'ctx>(
        _context: &'ctx Self::Context,
        refs: Self::RefsMut<'ctx>,
    ) -> MutPtrs<'ctx, Self> {
        ptr::from_mut(refs)
    }

    #[inline]
    fn refs_mut_as_refs<'ctx>(
        _context: &'ctx Self::Context,
        refs: Self::RefsMut<'ctx>,
    ) -> Self::Refs<'ctx> {
        &*refs
    }

    #[inline]
    fn value_as_refs(_context: &'a Self::Context, value: &'a Self) -> Self::Refs<'a> {
        value
    }

    #[inline]
    fn mut_value_as_refs(_context: &'a Self::Context, value: &'a mut Self) -> Self::RefsMut<'a> {
        value
    }

    type Slices<'ctx> = &'a [Self];

    #[inline]
    fn upcast_slices<'short, 'long: 'short>(from: Self::Slices<'long>) -> Self::Slices<'short> {
        from
    }

    type SlicesMut<'ctx> = &'a mut [Self];

    #[inline]
    fn upcast_mut_slices<'short, 'long: 'short>(
        from: Self::SlicesMut<'long>,
    ) -> Self::SlicesMut<'short> {
        from
    }

    #[inline]
    unsafe fn slice_ptrs_to_slices<'ctx>(
        context: &'ctx Self::Context,
        slices: SlicePtrs<'ctx, Self>,
    ) -> Self::Slices<'ctx> {
        let data = Self::slice_ptrs_as_ptrs(context, slices);
        let len = Self::slice_ptrs_len(context, &slices);
        unsafe { slice::from_raw_parts(data, len) }
    }

    #[inline]
    unsafe fn mut_slice_ptrs_to_mut_slices<'ctx>(
        context: &'ctx Self::Context,
        slices: SliceMutPtrs<'ctx, Self>,
    ) -> Self::SlicesMut<'ctx> {
        let data = Self::mut_slice_ptrs_as_ptrs(context, slices);
        let len = Self::mut_slice_ptrs_len(context, &slices);
        unsafe { slice::from_raw_parts_mut(data, len) }
    }

    #[inline]
    fn slices_len(_context: &Self::Context, slices: &Self::Slices<'_>) -> usize {
        slices.len()
    }

    #[inline]
    fn mut_slices_len(_context: &Self::Context, slices: &Self::SlicesMut<'_>) -> usize {
        slices.len()
    }

    #[inline]
    fn slices_as_slice_ptrs<'ctx>(
        _context: &'ctx Self::Context,
        slices: Self::Slices<'ctx>,
    ) -> SlicePtrs<'ctx, Self>
    where
        Self: 'a,
    {
        ptr::from_ref(slices)
    }

    #[inline]
    fn mut_slices_as_slice_ptrs<'ctx>(
        _context: &'ctx Self::Context,
        slices: Self::SlicesMut<'ctx>,
    ) -> SliceMutPtrs<'ctx, Self> {
        ptr::from_mut(slices)
    }

    #[inline]
    fn mut_slices_as_slices<'ctx>(
        _context: &'ctx Self::Context,
        slices: Self::SlicesMut<'ctx>,
    ) -> Self::Slices<'ctx> {
        &*slices
    }

    #[inline]
    fn slices_as_ptrs<'ctx>(
        _context: &'ctx Self::Context,
        slices: Self::Slices<'ctx>,
    ) -> Ptrs<'ctx, Self> {
        slices.as_ptr()
    }

    #[inline]
    fn mut_slices_as_ptrs<'ctx>(
        _context: &'ctx Self::Context,
        slices: Self::SlicesMut<'ctx>,
    ) -> MutPtrs<'ctx, Self> {
        slices.as_mut_ptr()
    }
}

unsafe impl SoaRead for () {
    #[inline]
    unsafe fn read(_context: &Self::Context, ptrs: Ptrs<'_, Self>) -> Self {
        unsafe { ptr::read(ptrs) }
    }
}

unsafe impl SoaWrite for () {
    #[inline]
    unsafe fn write(_context: &Self::Context, ptrs: MutPtrs<'_, Self>, value: Self) {
        unsafe { ptr::write(ptrs, value) }
    }
}

unsafe impl SoaCloneToUninit for () {
    #[inline]
    unsafe fn clone_to_uninit(
        _context: &Self::Context,
        _src: Ptrs<'_, Self>,
        _dst: MutPtrs<'_, Self>,
    ) {
    }
}

unsafe impl SoaTrustedFields for () {}
