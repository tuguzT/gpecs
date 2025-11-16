use core::{
    alloc::{Layout, LayoutError},
    any::type_name,
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    ptr::{self, NonNull},
    slice,
};

use crate::{
    field::FieldDescriptor,
    traits::{
        MutPtrs, Ptrs, SliceMutPtrs, SlicePtrs, Soa, SoaContext, SoaRead, SoaToOwned,
        SoaTrustedFields, SoaWrite,
    },
};

#[inline]
#[track_caller]
pub fn debug_assert_ptr_is_aligned<T>(ptr: *const T) {
    debug_assert!(
        ptr.is_aligned(),
        "pointer {:p} of {} is not aligned to {} [its current align offset (in bytes) is {}]",
        ptr,
        type_name::<T>(),
        align_of::<T>(),
        ptr.cast::<u8>().align_offset(align_of::<T>()),
    );
}

unsafe impl SoaContext for () {
    type Fields = ();

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
    fn upcast_slice_mut_ptrs<'short, 'long: 'short>(
        from: SliceMutPtrs<'long, Self>,
    ) -> SliceMutPtrs<'short, Self> {
        from
    }

    #[inline]
    fn slice_mut_ptrs_from_raw_parts<'a>(
        &'a self,
        ptrs: Self::MutPtrs<'a>,
        len: usize,
    ) -> Self::SliceMutPtrs<'a> {
        ptr::slice_from_raw_parts_mut(ptrs, len)
    }

    #[inline]
    fn slice_mut_ptrs_len(&self, slices: &Self::SliceMutPtrs<'_>) -> usize {
        slices.len()
    }

    #[inline]
    fn slice_mut_ptrs_as_ptrs<'a>(&'a self, slices: Self::SliceMutPtrs<'a>) -> Self::MutPtrs<'a> {
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

unsafe impl Soa for () {
    type Context = ();

    type Refs<'context, 'a>
        = &'a Self
    where
        Self: 'a;

    #[inline]
    fn upcast_refs<'short, 'long: 'short, 'a_short, 'a_long: 'a_short>(
        from: Self::Refs<'long, 'a_long>,
    ) -> Self::Refs<'short, 'a_short> {
        from
    }

    type RefsMut<'context, 'a>
        = &'a mut Self
    where
        Self: 'a;

    #[inline]
    fn upcast_refs_mut<'short, 'long: 'short, 'a_short, 'a_long: 'a_short>(
        from: Self::RefsMut<'long, 'a_long>,
    ) -> Self::RefsMut<'short, 'a_short> {
        from
    }

    #[inline]
    unsafe fn ptrs_to_refs<'context, 'a>(
        _context: &'context Self::Context,
        ptrs: Ptrs<'context, Self>,
    ) -> Self::Refs<'context, 'a> {
        unsafe { &*ptrs }
    }

    #[inline]
    unsafe fn ptrs_to_refs_mut<'context, 'a>(
        _context: &'context Self::Context,
        ptrs: MutPtrs<'context, Self>,
    ) -> Self::RefsMut<'context, 'a> {
        unsafe { &mut *ptrs }
    }

    #[inline]
    fn refs_as_ptrs<'context, 'a>(
        _context: &'context Self::Context,
        refs: Self::Refs<'context, 'a>,
    ) -> Ptrs<'context, Self>
    where
        Self: 'a,
    {
        ptr::from_ref(refs)
    }

    #[inline]
    fn refs_mut_as_ptrs<'context, 'a>(
        _context: &'context Self::Context,
        refs: Self::RefsMut<'context, 'a>,
    ) -> MutPtrs<'context, Self>
    where
        Self: 'a,
    {
        ptr::from_mut(refs)
    }

    #[inline]
    fn refs_mut_as_refs<'context, 'a>(
        _context: &'context Self::Context,
        refs: Self::RefsMut<'context, 'a>,
    ) -> Self::Refs<'context, 'a> {
        &*refs
    }

    #[inline]
    fn value_as_refs<'a>(_context: &'a Self::Context, value: &'a Self) -> Self::Refs<'a, 'a>
    where
        Self: 'a,
    {
        value
    }

    #[inline]
    fn mut_value_as_refs<'a>(
        _context: &'a Self::Context,
        value: &'a mut Self,
    ) -> Self::RefsMut<'a, 'a>
    where
        Self: 'a,
    {
        value
    }

    type Slices<'context, 'a>
        = &'a [Self]
    where
        Self: 'a;

    #[inline]
    fn upcast_slices<'short, 'long: 'short, 'a_short, 'a_long: 'a_short>(
        from: Self::Slices<'long, 'a_long>,
    ) -> Self::Slices<'short, 'a_short> {
        from
    }

    type SlicesMut<'context, 'a>
        = &'a mut [Self]
    where
        Self: 'a;

    #[inline]
    fn upcast_slices_mut<'short, 'long: 'short, 'a_short, 'a_long: 'a_short>(
        from: Self::SlicesMut<'long, 'a_long>,
    ) -> Self::SlicesMut<'short, 'a_short> {
        from
    }

    #[inline]
    unsafe fn slice_ptrs_to_slices<'context, 'a>(
        context: &'context Self::Context,
        slices: SlicePtrs<'context, Self>,
    ) -> Self::Slices<'context, 'a> {
        let data = Self::slice_ptrs_as_ptrs(context, slices);
        let len = Self::slice_ptrs_len(context, &slices);
        unsafe { slice::from_raw_parts(data, len) }
    }

    #[inline]
    unsafe fn slice_mut_ptrs_to_slices<'context, 'a>(
        context: &'context Self::Context,
        slices: SliceMutPtrs<'context, Self>,
    ) -> Self::SlicesMut<'context, 'a> {
        let data = Self::slice_mut_ptrs_as_ptrs(context, slices);
        let len = Self::slice_mut_ptrs_len(context, &slices);
        unsafe { slice::from_raw_parts_mut(data, len) }
    }

    #[inline]
    fn slices_len<'a>(_context: &Self::Context, slices: &Self::Slices<'_, 'a>) -> usize
    where
        Self: 'a,
    {
        slices.len()
    }

    #[inline]
    fn slices_mut_len<'a>(_context: &Self::Context, slices: &Self::SlicesMut<'_, 'a>) -> usize
    where
        Self: 'a,
    {
        slices.len()
    }

    #[inline]
    fn slices_as_slice_ptrs<'context, 'a>(
        _context: &'context Self::Context,
        slices: Self::Slices<'context, 'a>,
    ) -> SlicePtrs<'context, Self>
    where
        Self: 'a,
    {
        ptr::from_ref(slices)
    }

    #[inline]
    fn slices_mut_as_slice_ptrs<'context, 'a>(
        _context: &'context Self::Context,
        slices: Self::SlicesMut<'context, 'a>,
    ) -> SliceMutPtrs<'context, Self>
    where
        Self: 'a,
    {
        ptr::from_mut(slices)
    }

    #[inline]
    fn slices_mut_as_slices<'context, 'a>(
        _context: &'context Self::Context,
        slices: Self::SlicesMut<'context, 'a>,
    ) -> Self::Slices<'context, 'a> {
        &*slices
    }

    #[inline]
    fn slices_as_ptrs<'context, 'a>(
        _context: &'context Self::Context,
        slices: Self::Slices<'context, 'a>,
    ) -> Ptrs<'context, Self>
    where
        Self: 'a,
    {
        slices.as_ptr()
    }

    #[inline]
    fn slices_mut_as_ptrs<'context, 'a>(
        _context: &'context Self::Context,
        slices: Self::SlicesMut<'context, 'a>,
    ) -> MutPtrs<'context, Self>
    where
        Self: 'a,
    {
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

impl<'a> SoaToOwned<'_, 'a> for &'a () {
    type Owned = ();

    #[inline]
    fn to_owned(&self, _context: &<Self::Owned as Soa>::Context) -> Self::Owned {}

    #[inline]
    fn clone_into(&self, _context: &<Self::Owned as Soa>::Context, _target: &mut Self::Owned) {}

    #[inline]
    unsafe fn clone_into_ptrs(
        &self,
        _context: &<Self::Owned as Soa>::Context,
        _target: MutPtrs<'_, Self::Owned>,
    ) {
    }

    #[inline]
    fn clone_into_refs<'context>(
        &self,
        _context: &'context <Self::Owned as Soa>::Context,
        _target: <Self::Owned as Soa>::RefsMut<'context, '_>,
    ) {
    }
}

// https://veykril.github.io/tlborm/decl-macros/building-blocks/counting.html#enum-counting
#[macro_export]
#[doc(hidden)]
macro_rules! count_idents {
    ($($idents:ident),* $(,)*) => {
        {
            #[expect(dead_code)]
            #[repr(usize)]
            enum Idents { $($idents,)* __CountIdentsLast }

            const COUNT: usize = Idents::__CountIdentsLast as usize;
            COUNT
        }
    };
}

#[doc(hidden)]
pub use count_idents;

pub struct TupleContext<T>(PhantomData<fn() -> T>);

impl<T> TupleContext<T> {
    #[inline]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Debug for TupleContext<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TupleContext").finish_non_exhaustive()
    }
}

impl<T> Default for TupleContext<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for TupleContext<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for TupleContext<T> {}

impl<T> PartialEq for TupleContext<T> {
    fn eq(&self, other: &Self) -> bool {
        let Self(this) = self;
        let Self(other) = other;
        this == other
    }
}

impl<T> Eq for TupleContext<T> {}

impl<T> PartialOrd for TupleContext<T> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for TupleContext<T> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self(this) = self;
        let Self(other) = other;
        this.cmp(other)
    }
}

impl<T> Hash for TupleContext<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self(this) = self;
        this.hash(state);
    }
}

#[inline]
#[must_use]
#[doc(hidden)]
pub const fn permutation<const N: usize>() -> [usize; N] {
    let mut permutation = [0; _];
    let mut i = 0;
    while i < permutation.len() {
        permutation[i] = i;
        i += 1;
    }
    permutation
}

#[inline]
#[must_use]
#[doc(hidden)]
pub const fn layout_permutation<const N: usize>(layouts: [Layout; N]) -> [usize; N] {
    let mut permutation = permutation();
    let mut i = 1;
    while i < permutation.len() {
        let mut j = i;
        while j > 0 && layouts[permutation[j - 1]].align() > layouts[permutation[j]].align() {
            permutation.swap(j - 1, j);
            j -= 1;
        }
        i += 1;
    }
    permutation
}

macro_rules! soa_tuple_impl {
    ($($types:ident index $indices:tt),* $(,)?) => {
        impl<$($types,)*> TupleContext<($($types,)*)> {
            #[doc(hidden)]
            pub const PERMUTATION: [usize; count_idents!($($types,)*)] = {
                let layouts = [$(Layout::new::<$types>(),)*];
                layout_permutation(layouts)
            };
            #[doc(hidden)]
            pub const FIELD_DESCRIPTORS: [FieldDescriptor; count_idents!($($types,)*)] = {
                let permutation = Self::PERMUTATION;
                let descriptors = [$(FieldDescriptor::of::<$types>(),)*];
                [$(descriptors[permutation[$indices]],)*]
            };
        }

        unsafe impl<$($types,)*> super::SoaContext for TupleContext<($($types,)*)> {
            type Fields = ($($types,)*);

            type FieldDescriptors<'a> = [FieldDescriptor; count_idents!($($types,)*)];

            #[inline]
            fn upcast_field_descriptors<'short, 'long: 'short>(
                from: Self::FieldDescriptors<'long>,
            ) -> Self::FieldDescriptors<'short> {
                from
            }

            #[inline]
            fn field_descriptors(&self) -> Self::FieldDescriptors<'_> {
                Self::FIELD_DESCRIPTORS
            }

            #[inline]
            fn buffer_layout(&self, capacity: usize) -> Result<Layout, LayoutError> {
                let permutation = Self::PERMUTATION;

                let mut layout = Layout::new::<()>();
                let regions = [$(Layout::array::<$types>(capacity)?,)*];
                $((layout, _) = layout.extend(regions[permutation[$indices]])?;)*

                Ok(layout)
            }

            type Ptrs<'a> = ($(*const $types,)*);

            #[inline]
            fn upcast_ptrs<'short, 'long: 'short>(from: Self::Ptrs<'long>) -> Self::Ptrs<'short> {
                from
            }

            #[inline]
            fn ptrs_dangling(&self) -> Self::Ptrs<'_> {
                let ptrs = ($(ptr::dangling::<$types>(),)*);
                ptrs
            }

            #[inline]
            unsafe fn ptrs_from_buffer(&self, buffer: *const u8, capacity: usize) -> Self::Ptrs<'_> {
                let permutation = Self::PERMUTATION;

                let mut layout = Layout::new::<()>();
                let mut offsets = [0; count_idents!($($types,)*)];

                let regions = unsafe { [$(Layout::array::<$types>(capacity).unwrap_unchecked(),)*] };
                $((layout, offsets[permutation[$indices]]) = unsafe { layout.extend(regions[permutation[$indices]]).unwrap_unchecked() };)*
                let _ = layout;

                let ptrs = unsafe { ($(buffer.add(offsets[$indices]).cast(),)*) };
                $(debug_assert_ptr_is_aligned(ptrs.$indices);)*
                ptrs
            }

            #[inline]
            unsafe fn ptrs_add<'a>(&'a self, ptrs: Self::Ptrs<'a>, offset: usize) -> Self::Ptrs<'a> {
                let ptrs = unsafe { ($(ptrs.$indices.add(offset),)*) };
                ptrs
            }

            #[inline]
            unsafe fn ptrs_offset_from(&self, ptrs: Self::Ptrs<'_>, origin: Self::Ptrs<'_>) -> isize {
                let offsets = unsafe { [$(ptrs.$indices.offset_from(origin.$indices),)*] };
                assert!(offsets.iter().all(|offset| offsets[0].eq(offset)));
                offsets[0]
            }

            type MutPtrs<'a> = ($(*mut $types,)*);

            #[inline]
            fn upcast_mut_ptrs<'short, 'long: 'short>(from: Self::MutPtrs<'long>) -> Self::MutPtrs<'short> {
                from
            }

            #[inline]
            fn ptrs_dangling_mut(&self) -> Self::MutPtrs<'_> {
                let ptrs = ($(ptr::dangling_mut::<$types>(),)*);
                ptrs
            }

            #[inline]
            unsafe fn ptrs_from_buffer_mut(&self, buffer: *mut u8, capacity: usize) -> Self::MutPtrs<'_> {
                let permutation = Self::PERMUTATION;

                let mut layout = Layout::new::<()>();
                let mut offsets = [0; count_idents!($($types,)*)];

                let regions = unsafe { [$(Layout::array::<$types>(capacity).unwrap_unchecked(),)*] };
                $((layout, offsets[permutation[$indices]]) = unsafe { layout.extend(regions[permutation[$indices]]).unwrap_unchecked() };)*
                let _ = layout;

                let ptrs = unsafe { ($(buffer.add(offsets[$indices]).cast(),)*) };
                $(debug_assert_ptr_is_aligned(ptrs.$indices);)*
                ptrs
            }

            #[inline]
            unsafe fn ptrs_add_mut<'a>(
                &'a self,
                ptrs: Self::MutPtrs<'a>,
                offset: usize,
            ) -> Self::MutPtrs<'a> {
                let ptrs = unsafe { ($(ptrs.$indices.add(offset),)*) };
                ptrs
            }

            #[inline]
            unsafe fn ptrs_offset_from_mut(
                &self,
                ptrs: Self::MutPtrs<'_>,
                origin: Self::Ptrs<'_>,
            ) -> isize {
                let offsets = unsafe { [$(ptrs.$indices.offset_from(origin.$indices),)*] };
                assert!(offsets.iter().all(|offset| offsets[0].eq(offset)));
                offsets[0]
            }

            #[inline]
            fn ptrs_cast_const<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::Ptrs<'a> {
                let ptrs = ($(ptrs.$indices.cast_const(),)*);
                ptrs
            }

            #[inline]
            fn ptrs_cast_mut<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::MutPtrs<'a> {
                let ptrs = ($(ptrs.$indices.cast_mut(),)*);
                ptrs
            }

            #[inline]
            unsafe fn ptrs_swap(&self, a: Self::MutPtrs<'_>, b: Self::MutPtrs<'_>) {
                let permutation = Self::PERMUTATION;

                let closures = ($(|| unsafe { ptr::swap(a.$indices, b.$indices) },)*);
                let closures: [&dyn Fn(); count_idents!($($types,)*)] = [$(&closures.$indices,)*];

                for index in 0..count_idents!($($types,)*) {
                    closures[permutation[index]]();
                }
            }

            #[inline]
            unsafe fn ptrs_copy(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, len: usize) {
                let permutation = Self::PERMUTATION;

                let closures = ($(|| unsafe { ptr::copy(src.$indices, dst.$indices, len) },)*);
                let closures: [&dyn Fn(); count_idents!($($types,)*)] = [$(&closures.$indices,)*];

                for index in 0..count_idents!($($types,)*) {
                    closures[permutation[index]]();
                }
            }

            #[inline]
            unsafe fn ptrs_copy_rev(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, len: usize) {
                let permutation = Self::PERMUTATION;

                let closures = ($(|| unsafe { ptr::copy(src.$indices, dst.$indices, len) },)*);
                let closures: [&dyn Fn(); count_idents!($($types,)*)] = [$(&closures.$indices,)*];

                for index in (0..count_idents!($($types,)*)).rev() {
                    closures[permutation[index]]();
                }
            }

            #[inline]
            unsafe fn ptrs_copy_nonoverlapping(
                &self,
                src: Self::Ptrs<'_>,
                dst: Self::MutPtrs<'_>,
                len: usize,
            ) {
                // because source and destination are non-overlapping, we can copy them in any order
                unsafe { $(ptr::copy_nonoverlapping(src.$indices, dst.$indices, len);)* }
            }

            #[inline]
            unsafe fn ptrs_drop_in_place(&self, ptrs: Self::MutPtrs<'_>) {
                unsafe { $(ptr::drop_in_place(ptrs.$indices);)* }
            }

            type NonNullPtrs<'a> = ($(NonNull<$types>,)*);

            #[inline]
            fn upcast_nonnull_ptrs<'short, 'long: 'short>(
                from: Self::NonNullPtrs<'long>,
            ) -> Self::NonNullPtrs<'short> {
                from
            }

            #[inline]
            unsafe fn ptrs_to_nonnull<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::NonNullPtrs<'a> {
                let ptrs = unsafe { ($(NonNull::new_unchecked(ptrs.$indices),)*) };
                ptrs
            }

            #[inline]
            fn nonnull_to_ptrs<'a>(&'a self, ptrs: Self::NonNullPtrs<'a>) -> Self::MutPtrs<'a> {
                let ptrs = ($(ptrs.$indices.as_ptr(),)*);
                ptrs
            }

            type SlicePtrs<'a> = ($(*const [$types],)*);

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
                let slices = ($(ptr::slice_from_raw_parts(ptrs.$indices, len),)*);
                slices
            }

            #[inline]
            fn slice_ptrs_len(&self, slices: &Self::SlicePtrs<'_>) -> usize {
                let lens = [$(slices.$indices.len(),)*];
                assert!(lens.iter().all(|len| lens[0].eq(len)));
                lens[0]
            }

            #[inline]
            fn slice_ptrs_as_ptrs<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::Ptrs<'a> {
                let slices = ($(slices.$indices.cast(),)*); // should be `slices.$indices.as_ptr()` but it's unstable
                slices
            }

            type SliceMutPtrs<'a> = ($(*mut [$types],)*);

            #[inline]
            fn upcast_slice_mut_ptrs<'short, 'long: 'short>(
                from: Self::SliceMutPtrs<'long>,
            ) -> Self::SliceMutPtrs<'short> {
                from
            }

            #[inline]
            fn slice_mut_ptrs_from_raw_parts<'a>(
                &'a self,
                ptrs: Self::MutPtrs<'a>,
                len: usize,
            ) -> Self::SliceMutPtrs<'a> {
                let slices = ($(ptr::slice_from_raw_parts_mut(ptrs.$indices, len),)*);
                slices
            }

            #[inline]
            fn slice_mut_ptrs_len(&self, slices: &Self::SliceMutPtrs<'_>) -> usize {
                let lens = [$(slices.$indices.len(),)*];
                assert!(lens.iter().all(|len| lens[0].eq(len)));
                lens[0]
            }

            #[inline]
            fn slice_mut_ptrs_as_ptrs<'a>(&'a self, slices: Self::SliceMutPtrs<'a>) -> Self::MutPtrs<'a> {
                let slices = ($(slices.$indices.cast(),)*); // should be `slices.$indices.as_mut_ptr()` but it's unstable
                slices
            }

            #[inline]
            fn slice_ptrs_cast_const<'a>(&'a self, slices: Self::SliceMutPtrs<'a>) -> Self::SlicePtrs<'a> {
                let slices = ($(slices.$indices.cast_const(),)*);
                slices
            }

            #[inline]
            fn slice_ptrs_cast_mut<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::SliceMutPtrs<'a> {
                let slices = ($(slices.$indices.cast_mut(),)*);
                slices
            }

            #[inline]
            unsafe fn slices_drop_in_place(&self, slices: Self::SliceMutPtrs<'_>) {
                unsafe { $(ptr::drop_in_place(slices.$indices);)* }
            }
        }

        unsafe impl<$($types,)*> Soa for ($($types,)*) {
            type Context = TupleContext<($($types,)*)>;

            type Refs<'context, 'a>
                = ($(&'a $types,)*)
            where
                Self: 'a;

            #[inline]
            fn upcast_refs<'short, 'long: 'short, 'a_short, 'a_long: 'a_short>(
                from: Self::Refs<'long, 'a_long>,
            ) -> Self::Refs<'short, 'a_short> {
                from
            }

            type RefsMut<'context, 'a>
                = ($(&'a mut $types,)*)
            where
                Self: 'a;

            #[inline]
            fn upcast_refs_mut<'short, 'long: 'short, 'a_short, 'a_long: 'a_short>(
                from: Self::RefsMut<'long, 'a_long>,
            ) -> Self::RefsMut<'short, 'a_short> {
                from
            }

            #[inline]
            unsafe fn ptrs_to_refs<'context, 'a>(
                _context: &'context Self::Context,
                ptrs: Ptrs<'context, Self>,
            ) -> Self::Refs<'context, 'a> {
                let refs = unsafe { ($(&*ptrs.$indices,)*) };
                refs
            }

            #[inline]
            unsafe fn ptrs_to_refs_mut<'context, 'a>(
                _context: &'context Self::Context,
                ptrs: MutPtrs<'context, Self>,
            ) -> Self::RefsMut<'context, 'a> {
                let refs = unsafe { ($(&mut *ptrs.$indices,)*) };
                refs
            }

            #[inline]
            fn refs_as_ptrs<'context, 'a>(
                _context: &'context Self::Context,
                refs: Self::Refs<'context, 'a>,
            ) -> Ptrs<'context, Self>
            where
                Self: 'a,
            {
                let ptrs = ($(ptr::from_ref(refs.$indices),)*);
                ptrs
            }

            #[inline]
            fn refs_mut_as_ptrs<'context, 'a>(
                _context: &'context Self::Context,
                refs: Self::RefsMut<'context, 'a>,
            ) -> MutPtrs<'context, Self>
            where
                Self: 'a,
            {
                let ptrs = ($(ptr::from_mut(refs.$indices),)*);
                ptrs
            }

            #[inline]
            fn refs_mut_as_refs<'context, 'a>(
                _context: &'context Self::Context,
                refs: Self::RefsMut<'context, 'a>,
            ) -> Self::Refs<'context, 'a> {
                let refs = ($(&*refs.$indices,)*);
                refs
            }

            #[inline]
            fn value_as_refs<'a>(_context: &'a Self::Context, value: &'a Self) -> Self::Refs<'a, 'a>
            where
                Self: 'a,
            {
                let refs = ($(&value.$indices,)*);
                refs
            }

            #[inline]
            fn mut_value_as_refs<'a>(
                _context: &'a Self::Context,
                value: &'a mut Self,
            ) -> Self::RefsMut<'a, 'a>
            where
                Self: 'a,
            {
                let refs = ($(&mut value.$indices,)*);
                refs
            }

            type Slices<'context, 'a>
                = ($(&'a [$types],)*)
            where
                Self: 'a;

            #[inline]
            fn upcast_slices<'short, 'long: 'short, 'a_short, 'a_long: 'a_short>(
                from: Self::Slices<'long, 'a_long>,
            ) -> Self::Slices<'short, 'a_short> {
                from
            }

            type SlicesMut<'context, 'a>
                = ($(&'a mut [$types],)*)
            where
                Self: 'a;

            #[inline]
            fn upcast_slices_mut<'short, 'long: 'short, 'a_short, 'a_long: 'a_short>(
                from: Self::SlicesMut<'long, 'a_long>,
            ) -> Self::SlicesMut<'short, 'a_short> {
                from
            }

            #[inline]
            unsafe fn slice_ptrs_to_slices<'context, 'a>(
                context: &'context Self::Context,
                slices: SlicePtrs<'context, Self>,
            ) -> Self::Slices<'context, 'a> {
                let data = context.slice_ptrs_as_ptrs(slices);
                let len = context.slice_ptrs_len(&slices);
                let slices = unsafe { ($(slice::from_raw_parts(data.$indices, len),)*) };
                slices
            }

            #[inline]
            unsafe fn slice_mut_ptrs_to_slices<'context, 'a>(
                context: &'context Self::Context,
                slices: SliceMutPtrs<'context, Self>,
            ) -> Self::SlicesMut<'context, 'a> {
                let data = context.slice_mut_ptrs_as_ptrs(slices);
                let len = context.slice_mut_ptrs_len(&slices);
                let slices = unsafe { ($(slice::from_raw_parts_mut(data.$indices, len),)*) };
                slices
            }

            #[inline]
            fn slices_len<'a>(_context: &Self::Context, slices: &Self::Slices<'_, 'a>) -> usize
            where
                Self: 'a,
            {
                let lens = [$(slices.$indices.len(),)*];
                assert!(lens.iter().all(|len| lens[0].eq(len)));
                lens[0]
            }

            #[inline]
            fn slices_mut_len<'a>(_context: &Self::Context, slices: &Self::SlicesMut<'_, 'a>) -> usize
            where
                Self: 'a,
            {
                let lens = [$(slices.$indices.len(),)*];
                assert!(lens.iter().all(|len| lens[0].eq(len)));
                lens[0]
            }

            #[inline]
            fn slices_as_slice_ptrs<'context, 'a>(
                _context: &'context Self::Context,
                slices: Self::Slices<'context, 'a>,
            ) -> SlicePtrs<'context, Self>
            where
                Self: 'a,
            {
                let slices = ($(ptr::from_ref(slices.$indices),)*);
                slices
            }

            #[inline]
            fn slices_mut_as_slice_ptrs<'context, 'a>(
                _context: &'context Self::Context,
                slices: Self::SlicesMut<'context, 'a>,
            ) -> SliceMutPtrs<'context, Self>
            where
                Self: 'a,
            {
                let slices = ($(ptr::from_mut(slices.$indices),)*);
                slices
            }

            #[inline]
            fn slices_mut_as_slices<'context, 'a>(
                _context: &'context Self::Context,
                slices: Self::SlicesMut<'context, 'a>,
            ) -> Self::Slices<'context, 'a> {
                let slices = ($(&*slices.$indices,)*);
                slices
            }

            #[inline]
            fn slices_as_ptrs<'context, 'a>(
                _context: &'context Self::Context,
                slices: Self::Slices<'context, 'a>,
            ) -> Ptrs<'context, Self>
            where
                Self: 'a,
            {
                let slices = ($(slices.$indices.as_ptr(),)*);
                slices
            }

            #[inline]
            fn slices_mut_as_ptrs<'context, 'a>(
                _context: &'context Self::Context,
                slices: Self::SlicesMut<'context, 'a>,
            ) -> MutPtrs<'context, Self>
            where
                Self: 'a,
            {
                let slices = ($(slices.$indices.as_mut_ptr(),)*);
                slices
            }
        }

        unsafe impl<$($types,)*> SoaRead for ($($types,)*) {
            #[inline]
            unsafe fn read(_context: &Self::Context, ptrs: Ptrs<'_, Self>) -> Self {
                unsafe { ($(ptr::read(ptrs.$indices),)*) }
            }
        }

        unsafe impl<$($types,)*> SoaWrite for ($($types,)*) {
            #[inline]
            unsafe fn write(_context: &Self::Context, dst: MutPtrs<'_ , Self>, value: Self) {
                unsafe { $(ptr::write(dst.$indices, value.$indices);)* }
            }
        }

        impl<'a, $($types,)*> SoaToOwned<'_, 'a> for ($(&'a $types,)*)
        where
            $($types: Clone,)*
        {
            type Owned = ($($types,)*);

            #[inline]
            fn to_owned(&self, _context: &<Self::Owned as Soa>::Context) -> Self::Owned {
                let owned = ($(self.$indices.clone(),)*);
                owned
            }

            #[inline]
            fn clone_into(&self, _context: &<Self::Owned as Soa>::Context, target: &mut Self::Owned) {
                $(target.$indices.clone_from(self.$indices);)*
            }

            #[inline]
            fn clone_into_refs<'context>(
                &self,
                _context: &'context <Self::Owned as Soa>::Context,
                target: <Self::Owned as Soa>::RefsMut<'context, '_>,
            ) {
                $(target.$indices.clone_from(self.$indices);)*
            }
        }

        unsafe impl<$($types,)*> SoaTrustedFields for ($($types,)*) {}
    };
}

soa_tuple_impl!(
    A index 0,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
    H index 7,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
    H index 7,
    I index 8,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
    H index 7,
    I index 8,
    J index 9,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
    H index 7,
    I index 8,
    J index 9,
    K index 10,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
    H index 7,
    I index 8,
    J index 9,
    K index 10,
    L index 11,
);
