use crate::traits::{MutPtrs, Ptrs, Refs, RefsMut, Soa, SoaContext};

#[inline]
pub fn upcast<'short, 'long: 'short, 'data, T>(
    from: Refs<'long, 'data, T>,
) -> Refs<'short, 'data, T>
where
    T: Soa<'data> + ?Sized,
{
    T::Context::refs_upcast(from)
}

#[inline]
pub unsafe fn from_ptrs<'a, 'data, T>(
    context: &'a T::Context,
    ptrs: Ptrs<'a, T>,
) -> Refs<'a, 'data, T>
where
    T: Soa<'data> + ?Sized,
{
    unsafe { context.refs_from_ptrs(ptrs) }
}

#[inline]
pub fn as_ptrs<'a, 'data, T>(context: &'a T::Context, refs: Refs<'a, 'data, T>) -> Ptrs<'a, T>
where
    T: Soa<'data> + ?Sized,
{
    context.refs_as_ptrs(refs)
}

#[inline]
pub fn upcast_mut<'short, 'long: 'short, 'data, T>(
    from: RefsMut<'long, 'data, T>,
) -> RefsMut<'short, 'data, T>
where
    T: Soa<'data> + ?Sized,
{
    T::Context::mut_refs_upcast(from)
}

#[inline]
pub unsafe fn from_mut_ptrs<'a, 'data, T>(
    context: &'a T::Context,
    ptrs: MutPtrs<'a, T>,
) -> RefsMut<'a, 'data, T>
where
    T: Soa<'data> + ?Sized,
{
    unsafe { context.mut_refs_from_mut_ptrs(ptrs) }
}

#[inline]
pub fn as_mut_ptrs<'a, 'data, T>(
    context: &'a T::Context,
    refs: RefsMut<'a, 'data, T>,
) -> MutPtrs<'a, T>
where
    T: Soa<'data> + ?Sized,
{
    context.mut_refs_as_mut_ptrs(refs)
}

#[inline]
pub fn as_refs<'a, 'data, T>(
    context: &'a T::Context,
    refs: RefsMut<'a, 'data, T>,
) -> Refs<'a, 'data, T>
where
    T: Soa<'data> + ?Sized,
{
    context.mut_refs_as_refs(refs)
}
