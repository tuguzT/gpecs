use crate::traits::Soa;

/// Version of [`core::mem::replace()`] but for [SoA][`Soa`] references.
pub fn replace<T>(context: &T::Context, dest: T::RefsMut<'_>, src: T) -> T
where
    T: Soa,
{
    let dest = T::mut_refs_as_ptrs(context, dest);

    // SAFETY: We read from `dest` but directly write `src` into it afterwards,
    // such that the old value is not duplicated. Nothing is dropped and
    // nothing here can panic.
    unsafe {
        let result = T::ptrs_read(context, T::ptrs_cast_const(context, dest));
        T::ptrs_write(context, dest, src);
        result
    }
}

/// Version of [`core::mem::swap()`] but for [SoA][`Soa`] references.
pub fn swap<T>(context: &T::Context, x: T::RefsMut<'_>, y: T::RefsMut<'_>)
where
    T: Soa,
{
    let x = T::mut_refs_as_ptrs(context, x);
    let y = T::mut_refs_as_ptrs(context, y);

    // SAFETY: `&mut` guarantees these are typed readable and writable
    // as well as non-overlapping.
    unsafe { T::ptrs_swap(context, x, y) }
}
