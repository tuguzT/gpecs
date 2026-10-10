mod identity;
mod tuple;
mod unit;

/// This trait is used to perform all raw pointer arithmetics for [SoA](SoaRaw) types.
pub unsafe trait SoaRawContext<T>
where
    T: ?Sized,
{
    /// Collection of pointers to each stored field.
    type Ptrs<'a>: Clone;

    /// Restricts [pointers](SoaRawContext::Ptrs) to each stored field
    /// to be covariant over generic lifetime.
    fn ptrs_upcast<'short, 'long: 'short>(from: Self::Ptrs<'long>) -> Self::Ptrs<'short>;

    /// Returns dangling [pointers](SoaRawContext::Ptrs) to each stored field.
    fn ptrs_dangling(&self) -> Self::Ptrs<'_> {
        let ptrs = self.nonnull_ptrs_dangling();
        self.nonnull_ptrs_as_ptrs(ptrs)
    }

    /// Adds an unsigned offset to each [pointer](SoaRawContext::Ptrs) of each stored field.
    ///
    /// All the safety requirements resulting from applying [`pointer::add()`] method to each pointer
    /// should be satisfied to be safe to call this method.
    ///
    /// [`pointer::add()`]: https://doc.rust-lang.org/stable/core/primitive.pointer.html#method.add
    unsafe fn ptrs_add<'a>(&'a self, ptrs: Self::Ptrs<'a>, count: usize) -> Self::Ptrs<'a>;

    /// Calculates the distance between two [pointers](SoaRawContext::Ptrs)
    /// to each stored field within the same allocation.
    ///
    /// All the safety requirements resulting from applying [`pointer::offset_from()`] method to each pointer
    /// should be satisfied to be safe to call this method.
    ///
    /// Note that resulting offsets should be the same for all the fields,
    /// or else this method could panic.
    ///
    /// [`pointer::offset_from()`]: https://doc.rust-lang.org/stable/core/primitive.pointer.html#method.offset_from
    unsafe fn ptrs_offset_from(&self, ptrs: Self::Ptrs<'_>, origin: Self::Ptrs<'_>) -> isize;

    /// Collection of mutable pointers to each stored field.
    type MutPtrs<'a>: Clone;

    /// Restricts [mutable pointers](SoaRawContext::MutPtrs) to each stored field
    /// to be covariant over generic lifetime.
    fn mut_ptrs_upcast<'short, 'long: 'short>(from: Self::MutPtrs<'long>) -> Self::MutPtrs<'short>;

    /// Returns mutable dangling [pointers](SoaRawContext::MutPtrs) to each stored field.
    fn mut_ptrs_dangling(&self) -> Self::MutPtrs<'_> {
        let ptrs = self.nonnull_ptrs_dangling();
        self.nonnull_ptrs_as_mut_ptrs(ptrs)
    }

    /// Adds an unsigned offset to each [mutable pointer](SoaRawContext::MutPtrs) of each stored field.
    ///
    /// All the safety requirements resulting from applying [`pointer::add()`] method to each pointer
    /// should be satisfied to be safe to call this method.
    ///
    /// [`pointer::add()`]: https://doc.rust-lang.org/stable/core/primitive.pointer.html#method.add-1
    unsafe fn mut_ptrs_add<'a>(
        &'a self,
        ptrs: Self::MutPtrs<'a>,
        count: usize,
    ) -> Self::MutPtrs<'a> {
        let ptrs = self.ptrs_cast_const(ptrs);
        let ptrs = unsafe { self.ptrs_add(ptrs, count) };
        self.ptrs_cast_mut(ptrs)
    }

    /// Calculates the distance between two [mutable pointers](SoaRawContext::MutPtrs)
    /// to each stored field within the same allocation.
    ///
    /// All the safety requirements resulting from applying [`pointer::offset_from()`] method to each pointer
    /// should be satisfied to be safe to call this method.
    ///
    /// Note that resulting offsets should be the same for all the fields,
    /// or else this method could panic.
    ///
    /// [`pointer::offset_from()`]: https://doc.rust-lang.org/stable/core/primitive.pointer.html#method.offset_from-1
    unsafe fn mut_ptrs_offset_from(
        &self,
        ptrs: Self::MutPtrs<'_>,
        origin: Self::Ptrs<'_>,
    ) -> isize {
        let ptrs = Self::mut_ptrs_upcast(ptrs);
        let ptrs = self.ptrs_cast_const(ptrs);
        unsafe { self.ptrs_offset_from(ptrs, origin) }
    }

    /// Converts [pointers](SoaRawContext::Ptrs) of each stored field
    /// to the [mutable ones](SoaRawContext::MutPtrs).
    fn ptrs_cast_const<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::Ptrs<'a>;

    /// Converts [mutable pointers](SoaRawContext::MutPtrs) of each stored field
    /// to the [const ones](SoaRawContext::Ptrs).
    fn ptrs_cast_mut<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::MutPtrs<'a>;

    /// Swaps `count * size_of::<fields[0]>() + ...` bytes between the two [mutable regions](SoaRawContext::MutPtrs)
    /// of memory beginning at `x` and `y` for each stored field.
    ///
    /// The regions, as well as all the field pointers, must not overlap.
    ///
    /// Additionally, all the safety requirements resulting from applying
    /// [`ptr::swap_nonoverlapping()`](core::ptr::swap_nonoverlapping) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_swap_nonoverlapping(
        &self,
        x: Self::MutPtrs<'_>,
        y: Self::MutPtrs<'_>,
        count: usize,
    );

    /// Copies `count * size_of::<fields[0]>() + ...` bytes from [src](SoaRawContext::Ptrs)
    /// to [dst](SoaRawContext::MutPtrs) for each stored field.
    ///
    /// The source and destination, as well as all the field pointers, must not overlap.
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::copy_nonoverlapping()`](core::ptr::copy_nonoverlapping) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_copy_nonoverlapping(
        &self,
        src: Self::Ptrs<'_>,
        dst: Self::MutPtrs<'_>,
        count: usize,
    );

    /// Executes the destructors (if any) for the each stored field located at input [pointers](SoaRawContext::MutPtrs).
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::drop_in_place()`](core::ptr::drop_in_place) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_drop_in_place(&self, to_drop: Self::MutPtrs<'_>);

    /// Collection of non-null pointers to each stored field.
    type NonNullPtrs<'a>: Clone;

    /// Restricts [non-null pointers](SoaRawContext::NonNullPtrs) to each stored field
    /// to be covariant over generic lifetime.
    fn nonnull_ptrs_upcast<'short, 'long: 'short>(
        from: Self::NonNullPtrs<'long>,
    ) -> Self::NonNullPtrs<'short>;

    /// Returns dangling [non-null pointers](SoaRawContext::NonNullPtrs) to each stored field.
    fn nonnull_ptrs_dangling(&self) -> Self::NonNullPtrs<'_>;

    /// Creates [non-null pointers](SoaRawContext::NonNullPtrs) to each stored field
    /// from their [pointers](SoaRawContext::Ptrs).
    ///
    /// All the safety requirements resulting from applying
    /// [`NonNull::new_unchecked()`](core::ptr::NonNull::new_unchecked) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn nonnull_ptrs_from_ptrs<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::NonNullPtrs<'a> {
        let ptrs = self.ptrs_cast_mut(ptrs);
        unsafe { self.nonnull_ptrs_from_mut_ptrs(ptrs) }
    }

    /// Creates [non-null pointers](SoaRawContext::NonNullPtrs) to each stored field
    /// from their [mutable pointers](SoaRawContext::MutPtrs).
    ///
    /// All the safety requirements resulting from applying
    /// [`NonNull::new_unchecked()`](core::ptr::NonNull::new_unchecked) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn nonnull_ptrs_from_mut_ptrs<'a>(
        &'a self,
        ptrs: Self::MutPtrs<'a>,
    ) -> Self::NonNullPtrs<'a>;

    /// Adds an unsigned offset to each [non-null pointer](SoaRawContext::NonNullPtrs) of each stored field.
    ///
    /// All the safety requirements resulting from applying
    /// [`NonNull::add()`](core::ptr::NonNull::add) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn nonnull_ptrs_add<'a>(
        &'a self,
        ptrs: Self::NonNullPtrs<'a>,
        count: usize,
    ) -> Self::NonNullPtrs<'a> {
        let ptrs = self.nonnull_ptrs_as_mut_ptrs(ptrs);
        let ptrs = unsafe { self.mut_ptrs_add(ptrs, count) };
        unsafe { self.nonnull_ptrs_from_mut_ptrs(ptrs) }
    }

    /// Calculates the distance between two [non-null pointers](SoaRawContext::NonNullPtrs)
    /// to each stored field within the same allocation.
    ///
    /// All the safety requirements resulting from applying
    /// [`NonNull::offset_from()`](core::ptr::NonNull::offset_from) method to each pointer
    /// should be satisfied to be safe to call this method.
    ///
    /// Note that resulting offsets should be the same for all the fields,
    /// or else this method could panic.
    unsafe fn nonnull_ptrs_offset_from(
        &self,
        ptrs: Self::NonNullPtrs<'_>,
        origin: Self::NonNullPtrs<'_>,
    ) -> isize {
        let ptrs = Self::nonnull_ptrs_upcast(ptrs);
        let origin = Self::nonnull_ptrs_upcast(origin);

        let ptrs = self.nonnull_ptrs_as_mut_ptrs(ptrs);
        let origin = self.nonnull_ptrs_as_ptrs(origin);
        unsafe { self.mut_ptrs_offset_from(ptrs, origin) }
    }

    /// Acquires the underlying [pointers](SoaRawContext::Ptrs) from [non-null pointers](SoaRawContext::NonNullPtrs)
    /// to each stored field.
    fn nonnull_ptrs_as_ptrs<'a>(&'a self, ptrs: Self::NonNullPtrs<'a>) -> Self::Ptrs<'a> {
        let ptrs = self.nonnull_ptrs_as_mut_ptrs(ptrs);
        self.ptrs_cast_const(ptrs)
    }

    /// Acquires the underlying [mutable pointers](SoaRawContext::MutPtrs) from [non-null pointers](SoaRawContext::NonNullPtrs)
    /// to each stored field.
    fn nonnull_ptrs_as_mut_ptrs<'a>(&'a self, ptrs: Self::NonNullPtrs<'a>) -> Self::MutPtrs<'a>;

    /// Swaps `count * size_of::<fields[0]>() + ...` bytes between the two [non-null regions](SoaRawContext::NonNullPtrs)
    /// of memory beginning at `x` and `y` for each stored field.
    ///
    /// The regions, as well as all the field pointers, must not overlap.
    ///
    /// Additionally, all the safety requirements resulting from applying
    /// [`ptr::swap_nonoverlapping()`](core::ptr::swap_nonoverlapping) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn nonnull_ptrs_swap_nonoverlapping(
        &self,
        x: Self::NonNullPtrs<'_>,
        y: Self::NonNullPtrs<'_>,
        count: usize,
    ) {
        let x = Self::nonnull_ptrs_upcast(x);
        let y = Self::nonnull_ptrs_upcast(y);

        let x = self.nonnull_ptrs_as_mut_ptrs(x);
        let y = self.nonnull_ptrs_as_mut_ptrs(y);
        unsafe { self.ptrs_swap_nonoverlapping(x, y, count) }
    }

    /// Copies `count * size_of::<fields[0]>() + ...` bytes from [src](SoaRawContext::NonNullPtrs)
    /// to [dst](SoaRawContext::NonNullPtrs) for each stored field.
    ///
    /// The source and destination, as well as all the field pointers, must not overlap.
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::copy_nonoverlapping()`](core::ptr::copy_nonoverlapping) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn nonnull_ptrs_copy_nonoverlapping(
        &self,
        src: Self::NonNullPtrs<'_>,
        dst: Self::NonNullPtrs<'_>,
        count: usize,
    ) {
        let src = Self::nonnull_ptrs_upcast(src);
        let dst = Self::nonnull_ptrs_upcast(dst);

        let src = self.nonnull_ptrs_as_ptrs(src);
        let dst = self.nonnull_ptrs_as_mut_ptrs(dst);
        unsafe { self.ptrs_copy_nonoverlapping(src, dst, count) }
    }

    /// Executes the destructors (if any) for the each stored field located at input [pointers](SoaRawContext::NonNullPtrs).
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::drop_in_place()`](core::ptr::drop_in_place) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn nonnull_ptrs_drop_in_place(&self, to_drop: Self::NonNullPtrs<'_>) {
        let to_drop = Self::nonnull_ptrs_upcast(to_drop);
        let to_drop = self.nonnull_ptrs_as_mut_ptrs(to_drop);
        unsafe { self.ptrs_drop_in_place(to_drop) }
    }

    /// Collection of slice pointers to each stored field.
    type SlicePtrs<'a>: Clone;

    /// Restricts [slice pointers](SoaRawContext::SlicePtrs) to each stored field
    /// to be covariant over generic lifetime.
    fn slice_ptrs_upcast<'short, 'long: 'short>(
        from: Self::SlicePtrs<'long>,
    ) -> Self::SlicePtrs<'short>;

    /// Forms [slice pointers](SoaRawContext::SlicePtrs) to each stored field
    /// from [pointers](SoaRawContext::Ptrs) to each field and a length.
    ///
    /// The len argument is the number of elements, not the number of bytes.
    fn slice_ptrs_from_raw_parts<'a>(
        &'a self,
        data: Self::Ptrs<'a>,
        len: usize,
    ) -> Self::SlicePtrs<'a>;

    /// Returns the number of elements in slices to each [slice pointer](SoaRawContext::SlicePtrs) of stored fields,
    /// also referred to as their 'length'.
    ///
    /// Note that resulting lengths should be the same for all the slice pointers,
    /// or else this method could panic.
    fn slice_ptrs_len(&self, slices: &Self::SlicePtrs<'_>) -> usize;

    /// Returns [pointers](SoaRawContext::Ptrs) to the slice's buffer
    /// of each [slice pointer](SoaRawContext::SlicePtrs) of stored fields.
    fn slice_ptrs_as_ptrs<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::Ptrs<'a>;

    /// Collection of mutable slice pointers to each stored field.
    type SliceMutPtrs<'a>: Clone;

    /// Restricts [mutable slice pointers](SoaRawContext::SliceMutPtrs) to each stored field
    /// to be covariant over generic lifetime.
    fn mut_slice_ptrs_upcast<'short, 'long: 'short>(
        from: Self::SliceMutPtrs<'long>,
    ) -> Self::SliceMutPtrs<'short>;

    /// Forms [mutable slice pointers](SoaRawContext::SliceMutPtrs) to each stored field
    /// from [mutable pointers](SoaRawContext::MutPtrs) to each field and a length.
    ///
    /// The len argument is the number of elements, not the number of bytes.
    fn mut_slice_ptrs_from_raw_parts<'a>(
        &'a self,
        data: Self::MutPtrs<'a>,
        len: usize,
    ) -> Self::SliceMutPtrs<'a>;

    /// Returns the number of elements in slices to each [mutable slice pointer](SoaRawContext::SliceMutPtrs) of stored fields,
    /// also referred to as their 'length'.
    ///
    /// Note that resulting lengths should be the same for all the mutable slice pointers,
    /// or else this method could panic.
    fn mut_slice_ptrs_len(&self, slices: &Self::SliceMutPtrs<'_>) -> usize;

    /// Returns [pointers](SoaRawContext::Ptrs) to the slice's buffer
    /// of each [mutable slice pointer](SoaRawContext::SliceMutPtrs) of stored fields.
    fn mut_slice_ptrs_as_ptrs<'a>(&'a self, slices: Self::SliceMutPtrs<'a>) -> Self::Ptrs<'a> {
        let ptrs = self.mut_slice_ptrs_as_mut_ptrs(slices);
        self.ptrs_cast_const(ptrs)
    }

    /// Returns [mutable pointers](SoaRawContext::MutPtrs) to the slice's buffer
    /// of each [mutable slice pointer](SoaRawContext::SliceMutPtrs) of stored fields.
    fn mut_slice_ptrs_as_mut_ptrs<'a>(
        &'a self,
        slices: Self::SliceMutPtrs<'a>,
    ) -> Self::MutPtrs<'a>;

    /// Converts [slice pointers](SoaRawContext::SlicePtrs) of each field of stored fields
    /// to the [mutable ones](SoaRawContext::SliceMutPtrs).
    fn slice_ptrs_cast_const<'a>(&'a self, slices: Self::SliceMutPtrs<'a>) -> Self::SlicePtrs<'a> {
        let len = self.mut_slice_ptrs_len(&slices);
        let data = self.mut_slice_ptrs_as_ptrs(slices);
        self.slice_ptrs_from_raw_parts(data, len)
    }

    /// Converts [mutable slice pointers](SoaRawContext::SliceMutPtrs) of each field of stored fields
    /// to the [const ones](SoaRawContext::SlicePtrs).
    fn slice_ptrs_cast_mut<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::SliceMutPtrs<'a> {
        let len = self.slice_ptrs_len(&slices);
        let ptrs = self.slice_ptrs_as_ptrs(slices);
        let data = self.ptrs_cast_mut(ptrs);
        self.mut_slice_ptrs_from_raw_parts(data, len)
    }

    /// Executes the destructors (if any) for the each [slice](SoaRawContext::SliceMutPtrs) of stored fields.
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::drop_in_place()`](core::ptr::drop_in_place) method to each slice pointer
    /// should be satisfied to be safe to call this method.
    ///
    /// By default, this method just iterates by all the fields of slices and drops such fields one by one.
    unsafe fn slices_drop_in_place(&self, slices_to_drop: Self::SliceMutPtrs<'_>) {
        let slices = Self::mut_slice_ptrs_upcast(slices_to_drop);
        let len = self.mut_slice_ptrs_len(&slices);
        let ptrs = self.mut_slice_ptrs_as_mut_ptrs(slices);
        for index in 0..len {
            let to_drop = unsafe { self.mut_ptrs_add(ptrs.clone(), index) };
            unsafe { self.ptrs_drop_in_place(to_drop) }
        }
    }

    /// Collection of non-null slice pointers to each stored field.
    type SliceNonNullPtrs<'a>: Clone;

    /// Restricts [non-null slice pointers](SoaRawContext::SliceNonNullPtrs) to each stored field
    /// to be covariant over generic lifetime.
    fn nonnull_slice_ptrs_upcast<'short, 'long: 'short>(
        from: Self::SliceNonNullPtrs<'long>,
    ) -> Self::SliceNonNullPtrs<'short>;

    /// Forms [non-null slice pointers](SoaRawContext::SliceMutPtrs) to each stored field
    /// from [non-null pointers](SoaRawContext::MutPtrs) to each field and a length.
    ///
    /// The len argument is the number of elements, not the number of bytes.
    fn nonnull_slice_ptrs_from_raw_parts<'a>(
        &'a self,
        data: Self::NonNullPtrs<'a>,
        len: usize,
    ) -> Self::SliceNonNullPtrs<'a>;

    /// Returns the number of elements in slices to each [non-null slice pointer](SoaRawContext::SliceMutPtrs) of stored fields,
    /// also referred to as their 'length'.
    ///
    /// Note that resulting lengths should be the same for all the non-null slice pointers,
    /// or else this method could panic.
    fn nonnull_slice_ptrs_len(&self, slices: &Self::SliceNonNullPtrs<'_>) -> usize;
}

/// Alias for the [`Context`](SoaRaw::Context) associated type of a given [SoA](SoaRaw) type.
pub type Context<T> = <T as SoaRaw>::Context;

/// Alias for the [`Ptrs`](SoaRawContext::Ptrs) associated type
/// of the [`Context`](SoaRaw::Context) associated type of a given [SoA](SoaRaw) type.
pub type Ptrs<'a, T> = <Context<T> as SoaRawContext<T>>::Ptrs<'a>;

/// Alias for the [`MutPtrs`](SoaRawContext::MutPtrs) associated type
/// of the [`Context`](SoaRaw::Context) associated type of a given [SoA](SoaRaw) type.
pub type MutPtrs<'a, T> = <Context<T> as SoaRawContext<T>>::MutPtrs<'a>;

/// Alias for the [`NonNullPtrs`](SoaRawContext::NonNullPtrs) associated type
/// of the [`Context`](SoaRaw::Context) associated type of a given [SoA](SoaRaw) type.
pub type NonNullPtrs<'a, T> = <Context<T> as SoaRawContext<T>>::NonNullPtrs<'a>;

/// Alias for the [`SlicePtrs`](SoaRawContext::SlicePtrs) associated type
/// of the [`Context`](SoaRaw::Context) associated type of a given [SoA](SoaRaw) type.
pub type SlicePtrs<'a, T> = <Context<T> as SoaRawContext<T>>::SlicePtrs<'a>;

/// Alias for the [`SliceMutPtrs`](SoaRawContext::SliceMutPtrs) associated type
/// of the [`Context`](SoaRaw::Context) associated type of a given [SoA](SoaRaw) type.
pub type SliceMutPtrs<'a, T> = <Context<T> as SoaRawContext<T>>::SliceMutPtrs<'a>;

/// Alias for the [`SliceNonNullPtrs`](SoaRawContext::SliceNonNullPtrs) associated type
/// of the [`Context`](SoaRaw::Context) associated type of a given [SoA](SoaRaw) type.
pub type SliceNonNullPtrs<'a, T> = <Context<T> as SoaRawContext<T>>::SliceNonNullPtrs<'a>;

/// The main trait of the [crate] which defines behavior of this type
/// in the context of Structure of Arrays pattern, or SoA.
pub unsafe trait SoaRaw {
    /// Type of SoA [context](SoaRawContext).
    ///
    /// Most of the time, this should be zero-sized type.
    /// This is true for all the SoA types with stored fields' size and alignment known at compile-time.
    type Context: SoaRawContext<Self> + ?Sized;

    /// Special type containing all the fields which are stored inside of a buffer.
    ///
    /// This type is used to define implementations of [`Copy`], [`Send`], [`Sync`]
    /// and other traits for SoA containers.
    ///
    /// Most of the time, this should be just `Self`.
    /// This is true for such implementations which store all the fields of self.
    type Fields: ?Sized;
}

/// An extension of [SoA context](SoaRawContext) type which allows to perform copy-assignment of each stored field.
///
/// This trait is analogous to the unstable [`CloneToUninit`](core::clone::CloneToUninit) trait.
pub unsafe trait SoaCloneToUninitContext<T>: SoaRawContext<T>
where
    T: ?Sized,
{
    /// Performs copy-assignment of each stored field from [src](SoaRawContext::Ptrs) to [dst](SoaRawContext::MutPtrs).
    /// Before this function is called, src must point to initialized memory and dst may point to uninitialized memory.
    unsafe fn ptrs_clone_to_uninit(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>);

    /// Performs copy-assignment of each stored field from [src](SoaRawContext::NonNullPtrs) to [dst](SoaRawContext::NonNullPtrs).
    /// Before this function is called, src must point to initialized memory and dst may point to uninitialized memory.
    unsafe fn nonnull_ptrs_clone_to_uninit(
        &self,
        src: Self::NonNullPtrs<'_>,
        dst: Self::NonNullPtrs<'_>,
    ) {
        let src = Self::nonnull_ptrs_upcast(src);
        let dst = Self::nonnull_ptrs_upcast(dst);

        let src = self.nonnull_ptrs_as_ptrs(src);
        let dst = self.nonnull_ptrs_as_mut_ptrs(dst);
        unsafe { self.ptrs_clone_to_uninit(src, dst) }
    }
}

/// A generalization of [`Clone`] specifically for [SoA](SoaRaw) type.
///
/// This trait is analogous to the unstable [`CloneToUninit`](core::clone::CloneToUninit) trait.
pub unsafe trait SoaCloneToUninit: SoaRaw<Context: SoaCloneToUninitContext<Self>> {}

unsafe impl<T> SoaCloneToUninit for T
where
    T: SoaRaw + ?Sized,
    T::Context: SoaCloneToUninitContext<T>,
{
}

/// An extension of [SoA context](SoaRawContext) type which allows to read a value borrowed from self
/// from [pointers](SoaRawContext::Ptrs) to each stored field.
pub unsafe trait SoaReadContext<'a, T, R = T>: SoaRawContext<T>
where
    T: ?Sized,
{
    /// Constructs the value from reading each field to which [src](SoaRawContext::Ptrs) points without moving them.
    /// This leaves the memory in src unchanged.
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::read()`](core::ptr::read) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_read(&'a self, src: Self::Ptrs<'a>) -> R;

    /// Constructs the value from reading each field to which [src](SoaRawContext::MutPtrs) points without moving them.
    /// This leaves the memory in src unchanged.
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::read()`](core::ptr::read) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn mut_ptrs_read(&'a self, src: Self::MutPtrs<'a>) -> R {
        let src = self.ptrs_cast_const(src);
        unsafe { self.ptrs_read(src) }
    }

    /// Constructs the value from reading each field to which [src](SoaRawContext::NonNullPtrs) points without moving them.
    /// This leaves the memory in src unchanged.
    ///
    /// All the safety requirements resulting from applying
    /// [`NonNull::read()`](core::ptr::NonNull::read) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn nonnull_ptrs_read(&'a self, src: Self::NonNullPtrs<'a>) -> R {
        let src = self.nonnull_ptrs_as_mut_ptrs(src);
        unsafe { self.mut_ptrs_read(src) }
    }
}

/// An extension of [SoA](SoaRaw) type which allows to read a value borrowed from the context
/// from [pointers](SoaRawContext::Ptrs) to each stored field.
pub unsafe trait SoaRead<'a, R = Self>:
    SoaRaw<Context: SoaReadContext<'a, Self, R>>
{
}

unsafe impl<'a, T, R> SoaRead<'a, R> for T
where
    T: SoaRaw + ?Sized,
    T::Context: SoaReadContext<'a, T, R>,
{
}

/// An extension of [SoA](SoaRaw) type which allows to read a value of *any* lifetime
/// from [pointers](SoaRawContext::Ptrs) to each stored field.
pub unsafe trait SoaReadOwned<R = Self>: for<'a> SoaRead<'a, R> {}

unsafe impl<T, R> SoaReadOwned<R> for T where T: for<'a> SoaRead<'a, R> + ?Sized {}

/// An extension of [SoA context](SoaRawContext) type which allows to write a value
/// into [mutale pointers](SoaRawContext::MutPtrs) to each stored field.
pub unsafe trait SoaWriteContext<T, W = T>: SoaRawContext<T>
where
    T: ?Sized,
{
    /// Overwrites a memory [location](SoaRawContext::MutPtrs) of each stored field
    /// with the given value without reading or dropping the old value.
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::write()`](core::ptr::write) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_write(&self, dst: Self::MutPtrs<'_>, value: W);

    /// Overwrites a memory [location](SoaRawContext::NonNullPtrs) of each stored field
    /// with the given value without reading or dropping the old value.
    ///
    /// All the safety requirements resulting from applying
    /// [`NonNull::write()`](core::ptr::NonNull::write) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn nonnull_ptrs_write(&self, dst: Self::NonNullPtrs<'_>, value: W) {
        let dst = Self::nonnull_ptrs_upcast(dst);
        let dst = self.nonnull_ptrs_as_mut_ptrs(dst);
        unsafe { self.ptrs_write(dst, value) }
    }
}

/// An extension of [SoA](SoaRaw) type which allows to write given value
/// into [mutable pointers](SoaRawContext::Ptrs) to each stored field.
pub unsafe trait SoaWrite<W = Self>: SoaRaw<Context: SoaWriteContext<Self, W>> {}

unsafe impl<T, W> SoaWrite<W> for T
where
    T: SoaRaw + ?Sized,
    T::Context: SoaWriteContext<T, W>,
{
}

/// An extension of [SoA context](SoaRawContext) type which provides
/// reference and slice types of specific lifetime to each stored field.
pub unsafe trait SoaContext<'data, T>: SoaRawContext<T>
where
    T: ?Sized,
{
    /// Collection of references to each stored field.
    type Refs<'a>;

    /// Restricts [references](SoaContext::Refs) to each stored field
    /// to be covariant over generic lifetime.
    fn refs_upcast<'short, 'long: 'short>(from: Self::Refs<'long>) -> Self::Refs<'short>;

    /// Converts [pointers](SoaRawContext::Ptrs) to each stored field
    /// to their [references](SoaContext::Refs) by dereferencing each one of them.
    ///
    /// All the safety requirements resulting from dereferencing of each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn refs_from_ptrs<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::Refs<'a>;

    /// Converts [references](SoaContext::Refs) to each stored field
    /// to their [pointers](SoaRawContext::Ptrs) by taking the pointer of each one of them.
    fn refs_as_ptrs<'a>(&'a self, refs: Self::Refs<'a>) -> Self::Ptrs<'a>;

    /// Collection of mutable references to each stored field.
    type RefsMut<'a>;

    /// Restricts [mutable references](SoaContext::RefsMut) to each stored field
    /// to be covariant over generic lifetime.
    fn mut_refs_upcast<'short, 'long: 'short>(from: Self::RefsMut<'long>) -> Self::RefsMut<'short>;

    /// Converts [mutable pointers](SoaRawContext::MutPtrs) to each stored field
    /// to their [mutable references](SoaContext::RefsMut) by dereferencing each one of them.
    ///
    /// All the safety requirements resulting from dereferencing of each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn mut_refs_from_mut_ptrs<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::RefsMut<'a>;

    /// Converts [mutable references](SoaContext::RefsMut) to each stored field
    /// to their [pointers](SoaRawContext::Ptrs) by taking the pointer of each one of them.
    fn mut_refs_as_ptrs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::Ptrs<'a> {
        let ptrs = self.mut_refs_as_mut_ptrs(refs);
        self.ptrs_cast_const(ptrs)
    }

    /// Converts [mutable references](SoaContext::RefsMut) to each stored field
    /// to their [mutable pointers](SoaRawContext::MutPtrs) by taking the pointer of each one of them.
    fn mut_refs_as_mut_ptrs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::MutPtrs<'a>;

    /// Converts [mutable references](SoaContext::RefsMut) to each stored field
    /// to their [references](SoaContext::Refs) by explicitly converting each one of them via `&*` operator combination.
    fn mut_refs_as_refs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::Refs<'a> {
        let ptrs = self.mut_refs_as_ptrs(refs);
        unsafe { self.refs_from_ptrs(ptrs) }
    }

    /// Collection of slices of each stored field.
    type Slices<'a>;

    /// Restricts [slices](SoaContext::Slices) to each stored field
    /// to be covariant over generic lifetime.
    fn slices_upcast<'short, 'long: 'short>(from: Self::Slices<'long>) -> Self::Slices<'short>;

    /// Converts [slice pointers](SoaRawContext::SlicePtrs) to each stored field
    /// to their [slices](SoaContext::Slices) by dereferencing each one of them.
    ///
    /// All the safety requirements resulting from dereferencing of each slice pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn slices_from_slice_ptrs<'a>(&'a self, slices: Self::SlicePtrs<'a>)
    -> Self::Slices<'a>;

    /// Forms [slices](SoaContext::Slices) to each stored field
    /// from [pointers](SoaRawContext::Ptrs) to each field and a length.
    ///
    /// The len argument is the number of elements, not the number of bytes.
    ///
    /// All the safety requirements resulting from applying
    /// [`slice::from_raw_parts()`](core::slice::from_raw_parts) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn slices_from_raw_parts<'a>(
        &'a self,
        data: Self::Ptrs<'a>,
        len: usize,
    ) -> Self::Slices<'a> {
        let slices = self.slice_ptrs_from_raw_parts(data, len);
        unsafe { self.slices_from_slice_ptrs(slices) }
    }

    /// Converts [slices](SoaContext::Slices) to each stored field
    /// to their [slice pointers](SoaRawContext::SlicePtrs) by taking the pointer of each one of them.
    fn slices_as_slice_ptrs<'a>(&'a self, slices: Self::Slices<'a>) -> Self::SlicePtrs<'a>;

    /// Converts [slices](SoaContext::Slices) to each stored field
    /// to their [pointers](SoaRawContext::Ptrs) by taking the pointer of each one of them.
    fn slices_as_ptrs<'a>(&'a self, slices: Self::Slices<'a>) -> Self::Ptrs<'a> {
        let slices = self.slices_as_slice_ptrs(slices);
        self.slice_ptrs_as_ptrs(slices)
    }

    /// Returns the number of elements in [slices](SoaContext::Slices) to each stored field,
    /// also referred to as their 'length'.
    ///
    /// Note that resulting lengths should be the same for all the slices,
    /// or else this method could panic.
    fn slices_len(&self, slices: &Self::Slices<'_>) -> usize;

    /// Collection of mutable slices of each stored field.
    type SlicesMut<'a>;

    /// Restricts [mutable slices](SoaContext::SlicesMut) to each stored field
    /// to be covariant over generic lifetime.
    fn mut_slices_upcast<'short, 'long: 'short>(
        from: Self::SlicesMut<'long>,
    ) -> Self::SlicesMut<'short>;

    /// Converts [mutable slice pointers](SoaRawContext::SliceMutPtrs) to each stored field
    /// to their [mutable slices](SoaContext::SlicesMut) by dereferencing each one of them.
    ///
    /// All the safety requirements resulting from dereferencing of each mutable slice pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn mut_slices_from_mut_slice_ptrs<'a>(
        &'a self,
        slices: Self::SliceMutPtrs<'a>,
    ) -> Self::SlicesMut<'a>;

    /// Forms [mutable slices](SoaContext::SlicesMut) to each stored field
    /// from [mutable pointers](SoaRawContext::MutPtrs) to each field and a length.
    ///
    /// The len argument is the number of elements, not the number of bytes.
    ///
    /// All the safety requirements resulting from applying
    /// [`slice::from_raw_parts_mut()`](core::slice::from_raw_parts_mut) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn mut_slices_from_raw_parts<'a>(
        &'a self,
        data: Self::MutPtrs<'a>,
        len: usize,
    ) -> Self::SlicesMut<'a> {
        let slices = self.mut_slice_ptrs_from_raw_parts(data, len);
        unsafe { self.mut_slices_from_mut_slice_ptrs(slices) }
    }

    /// Converts [mutable slices](SoaContext::SlicesMut) to each stored field
    /// to their [slice pointers](SoaRawContext::SlicePtrs) by taking the pointer of each one of them.
    fn mut_slices_as_slice_ptrs<'a>(&'a self, slices: Self::SlicesMut<'a>) -> Self::SlicePtrs<'a> {
        let slices = self.mut_slices_as_mut_slice_ptrs(slices);
        self.slice_ptrs_cast_const(slices)
    }

    /// Converts [mutable slices](SoaContext::SlicesMut) to each stored field
    /// to their [mutable slice pointers](SoaRawContext::SliceMutPtrs) by taking the pointer of each one of them.
    fn mut_slices_as_mut_slice_ptrs<'a>(
        &'a self,
        slices: Self::SlicesMut<'a>,
    ) -> Self::SliceMutPtrs<'a>;

    /// Converts [mutable slices](SoaContext::SlicesMut) to each stored field
    /// to their [pointers](SoaRawContext::Ptrs) by taking the pointer of each one of them.
    fn mut_slices_as_ptrs<'a>(&'a self, slices: Self::SlicesMut<'a>) -> Self::Ptrs<'a> {
        let slices = self.mut_slices_as_slice_ptrs(slices);
        self.slice_ptrs_as_ptrs(slices)
    }

    /// Converts [mutable slices](SoaContext::SlicesMut) to each stored field
    /// to their [mutable pointers](SoaRawContext::MutPtrs) by taking the pointer of each one of them.
    fn mut_slices_as_mut_ptrs<'a>(&'a self, slices: Self::SlicesMut<'a>) -> Self::MutPtrs<'a> {
        let slices = self.mut_slices_as_mut_slice_ptrs(slices);
        self.mut_slice_ptrs_as_mut_ptrs(slices)
    }

    /// Returns the number of elements in [mutable slices](SoaContext::SlicesMut) to each stored field,
    /// also referred to as their 'length'.
    ///
    /// Note that resulting lengths should be the same for all the mutable slices,
    /// or else this method could panic.
    fn mut_slices_len(&self, slices: &Self::SlicesMut<'_>) -> usize;

    /// Converts [mutable slices](SoaContext::SlicesMut) to each stored field
    /// to their [slices](SoaContext::Slices) by explicitly converting each one of them via `&*` operator combination.
    fn mut_slices_as_slices<'a>(&'a self, slices: Self::SlicesMut<'a>) -> Self::Slices<'a> {
        let slices = self.mut_slices_as_slice_ptrs(slices);
        unsafe { self.slices_from_slice_ptrs(slices) }
    }
}

/// Alias for the [`Refs`](SoaContext::Refs) associated type
/// of the [`Context`](SoaRaw::Context) associated type of a given [SoA](Soa) type.
pub type Refs<'a, 'data, T> = <Context<T> as SoaContext<'data, T>>::Refs<'a>;

/// Alias for the [`RefsMut`](SoaContext::RefsMut) associated type
/// of the [`Context`](SoaRaw::Context) associated type of a given [SoA](Soa) type.
pub type RefsMut<'a, 'data, T> = <Context<T> as SoaContext<'data, T>>::RefsMut<'a>;

/// Alias for the [`Slices`](SoaContext::Slices) associated type
/// of the [`Context`](SoaRaw::Context) associated type of a given [SoA](Soa) type.
pub type Slices<'a, 'data, T> = <Context<T> as SoaContext<'data, T>>::Slices<'a>;

/// Alias for the [`SlicesMut`](SoaContext::SlicesMut) associated type
/// of the [`Context`](SoaRaw::Context) associated type of a given [SoA](Soa) type.
pub type SlicesMut<'a, 'data, T> = <Context<T> as SoaContext<'data, T>>::SlicesMut<'a>;

/// An extension of [SoA](SoaRaw) type which allows to access
/// each stored field by their reference types of specific lifetime.
pub unsafe trait Soa<'a>: SoaRaw<Context: SoaContext<'a, Self>> {}

unsafe impl<'a, T> Soa<'a> for T
where
    T: SoaRaw + ?Sized,
    T::Context: SoaContext<'a, T>,
{
}

/// An extension of [SoA](SoaRaw) type which allows to access
/// each stored field by their reference types of **any** lifetime.
pub trait SoaOwned: for<'a> Soa<'a> {}

impl<T> SoaOwned for T where T: for<'a> Soa<'a> + ?Sized {}
