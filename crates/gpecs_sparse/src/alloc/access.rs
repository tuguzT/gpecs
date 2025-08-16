use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
};

use crate::soa::traits::{MutPtrs, RefsMut, Soa, SoaWrite};

pub enum TryInsertAccess<'context, 'a, T>
where
    T: Soa + ?Sized + 'a,
{
    ReadWrite(RefsMut<'context, 'a, T>),
    WriteOnly(MutPtrs<'context, T>),
}

impl<'context, T> TryInsertAccess<'context, '_, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    pub fn into_ptrs(self, context: &'context T::Context) -> MutPtrs<'context, T> {
        match self {
            Self::ReadWrite(refs) => MutPtrs::new(T::refs_mut_as_ptrs(context, refs.into_inner())),
            Self::WriteOnly(ptrs) => ptrs,
        }
    }
}

impl<'context, 'a, T> From<RefsMut<'context, 'a, T>> for TryInsertAccess<'context, 'a, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    fn from(refs: RefsMut<'context, 'a, T>) -> Self {
        Self::ReadWrite(refs)
    }
}

impl<'context, T> From<MutPtrs<'context, T>> for TryInsertAccess<'context, '_, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    fn from(ptrs: MutPtrs<'context, T>) -> Self {
        Self::WriteOnly(ptrs)
    }
}

impl<'context, 'a, T> Debug for TryInsertAccess<'context, 'a, T>
where
    T: Soa + ?Sized,
    RefsMut<'context, 'a, T>: Debug,
    MutPtrs<'context, T>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadWrite(refs) => f.debug_tuple("ReadWrite").field(refs).finish(),
            Self::WriteOnly(ptrs) => f.debug_tuple("WriteOnly").field(ptrs).finish(),
        }
    }
}

impl<'context, 'a, T> PartialEq for TryInsertAccess<'context, 'a, T>
where
    T: Soa + ?Sized,
    RefsMut<'context, 'a, T>: PartialEq,
    MutPtrs<'context, T>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ReadWrite(refs), Self::ReadWrite(other_refs)) => refs == other_refs,
            (Self::WriteOnly(ptrs), Self::WriteOnly(other_ptrs)) => ptrs == other_ptrs,
            _ => false,
        }
    }
}

impl<'context, 'a, T> Eq for TryInsertAccess<'context, 'a, T>
where
    T: Soa + ?Sized,
    RefsMut<'context, 'a, T>: Eq,
    MutPtrs<'context, T>: Eq,
{
}

impl<'context, 'a, T> PartialOrd for TryInsertAccess<'context, 'a, T>
where
    T: Soa + ?Sized + 'a,
    RefsMut<'context, 'a, T>: PartialOrd,
    MutPtrs<'context, T>: PartialOrd,
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

impl<'context, 'a, T> Ord for TryInsertAccess<'context, 'a, T>
where
    T: Soa + ?Sized + 'a,
    RefsMut<'context, 'a, T>: Ord,
    MutPtrs<'context, T>: Ord,
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

impl<'context, 'a, T> Hash for TryInsertAccess<'context, 'a, T>
where
    T: Soa + ?Sized,
    RefsMut<'context, 'a, T>: Hash,
    MutPtrs<'context, T>: Hash,
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
    V: SoaWrite,
{
    let dst = match dst {
        Some(TryInsertAccess::ReadWrite(dst)) => {
            let dst = V::refs_mut_as_ptrs(context, dst.into_inner());
            unsafe { V::ptrs_drop_in_place(context, dst.clone()) }
            dst
        }
        Some(TryInsertAccess::WriteOnly(dst)) => dst.into_inner(),
        None => return,
    };
    unsafe { V::write(context, dst, value) }
}
