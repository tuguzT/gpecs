use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
};

use crate::soa::{
    traits::{MutPtrs, RawSoaContext, Soa, SoaWrite},
    wrapper,
};

pub enum TryInsertAccess<'ctx, 'a, T>
where
    T: Soa + ?Sized + 'a,
{
    ReadWrite(wrapper::RefsMut<'ctx, 'a, T>),
    WriteOnly(wrapper::MutPtrs<'ctx, T>),
}

impl<'ctx, 'a, T> TryInsertAccess<'ctx, 'a, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    pub fn read_write(refs: T::RefsMut<'ctx, 'a>) -> Self {
        let refs = wrapper::RefsMut::new(refs);
        Self::ReadWrite(refs)
    }

    #[inline]
    pub fn write_only(ptrs: MutPtrs<'ctx, T>) -> Self {
        let ptrs = wrapper::MutPtrs::new(ptrs);
        Self::WriteOnly(ptrs)
    }

    #[inline]
    pub fn into_ptrs(self, context: &'ctx T::Context) -> MutPtrs<'ctx, T> {
        match self {
            Self::ReadWrite(refs) => T::refs_mut_as_ptrs(context, refs.into_inner()),
            Self::WriteOnly(ptrs) => ptrs.into_inner(),
        }
    }
}

impl<T> Debug for TryInsertAccess<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'ctx, 'a> T::RefsMut<'ctx, 'a>: Debug,
    for<'ctx> MutPtrs<'ctx, T>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadWrite(refs) => f.debug_tuple("ReadWrite").field(refs).finish(),
            Self::WriteOnly(ptrs) => f.debug_tuple("WriteOnly").field(ptrs).finish(),
        }
    }
}

impl<T> PartialEq for TryInsertAccess<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'ctx, 'a> T::RefsMut<'ctx, 'a>: PartialEq,
    for<'ctx> MutPtrs<'ctx, T>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ReadWrite(refs), Self::ReadWrite(other_refs)) => refs == other_refs,
            (Self::WriteOnly(ptrs), Self::WriteOnly(other_ptrs)) => ptrs == other_ptrs,
            _ => false,
        }
    }
}

impl<T> Eq for TryInsertAccess<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'ctx, 'a> T::RefsMut<'ctx, 'a>: Eq,
    for<'ctx> MutPtrs<'ctx, T>: Eq,
{
}

impl<T> PartialOrd for TryInsertAccess<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'ctx, 'a> T::RefsMut<'ctx, 'a>: PartialOrd,
    for<'ctx> MutPtrs<'ctx, T>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match (self, other) {
            (Self::ReadWrite(refs), Self::ReadWrite(other_refs)) => refs.partial_cmp(other_refs),
            (Self::WriteOnly(ptrs), Self::WriteOnly(other_ptrs)) => ptrs.partial_cmp(other_ptrs),
            (Self::ReadWrite(_), Self::WriteOnly(_)) => Some(cmp::Ordering::Less),
            (Self::WriteOnly(_), Self::ReadWrite(_)) => Some(cmp::Ordering::Greater),
        }
    }
}

impl<T> Ord for TryInsertAccess<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'ctx, 'a> T::RefsMut<'ctx, 'a>: Ord,
    for<'ctx> MutPtrs<'ctx, T>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match (self, other) {
            (Self::ReadWrite(refs), Self::ReadWrite(other_refs)) => refs.cmp(other_refs),
            (Self::WriteOnly(ptrs), Self::WriteOnly(other_ptrs)) => ptrs.cmp(other_ptrs),
            (Self::ReadWrite(_), Self::WriteOnly(_)) => cmp::Ordering::Less,
            (Self::WriteOnly(_), Self::ReadWrite(_)) => cmp::Ordering::Greater,
        }
    }
}

impl<T> Hash for TryInsertAccess<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'ctx, 'a> T::RefsMut<'ctx, 'a>: Hash,
    for<'ctx> MutPtrs<'ctx, T>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::ReadWrite(refs) => refs.hash(state),
            Self::WriteOnly(ptrs) => ptrs.hash(state),
        }
    }
}

pub unsafe fn drop_old_then_write<V>(
    context: &V::Context,
    dst: Option<TryInsertAccess<V>>,
    value: V,
) where
    V: Soa + SoaWrite,
{
    let dst = match dst {
        Some(TryInsertAccess::ReadWrite(dst)) => {
            let dst = V::refs_mut_as_ptrs(context, dst.into_inner());
            unsafe { context.ptrs_drop_in_place(dst.clone()) }
            dst
        }
        Some(TryInsertAccess::WriteOnly(dst)) => dst.into_inner(),
        None => return,
    };
    unsafe { V::write(context, dst, value) }
}
