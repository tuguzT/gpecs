use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
};

use crate::soa::{
    traits::{MutPtrs, RawSoa, RawSoaContext, RefsMut, Soa, SoaContext, SoaWrite, WriteSoaContext},
    wrapper,
};

#[repr(transparent)]
pub struct ReadWriteAccess<'ctx, 'a, T>
where
    T: RawSoa + ?Sized,
{
    ptrs: wrapper::MutPtrs<'ctx, T>,
    phantom: PhantomData<fn(&'a ()) -> &'a ()>,
}

impl<'ctx, T> ReadWriteAccess<'ctx, '_, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    pub unsafe fn from_ptrs(ptrs: MutPtrs<'ctx, T>) -> Self {
        Self {
            ptrs: wrapper::MutPtrs::new(ptrs),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn as_ptrs(&self) -> &MutPtrs<'_, T> {
        let Self { ptrs, .. } = self;
        ptrs.as_inner()
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> &mut MutPtrs<'_, T> {
        let Self { ptrs, .. } = self;
        ptrs.as_inner_mut()
    }

    #[inline]
    pub fn into_ptrs(self) -> MutPtrs<'ctx, T> {
        let Self { ptrs, .. } = self;
        ptrs.into_inner()
    }
}

impl<'ctx, 'a, T> ReadWriteAccess<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
{
    #[inline]
    pub fn from_refs(context: &'ctx T::Context, refs: RefsMut<'ctx, 'a, T>) -> Self {
        let ptrs = context.mut_refs_as_mut_ptrs(refs);
        unsafe { Self::from_ptrs(ptrs) }
    }

    #[inline]
    pub fn into_refs(self, context: &'ctx T::Context) -> RefsMut<'ctx, 'a, T> {
        let ptrs = self.into_ptrs();
        unsafe { context.mut_ptrs_to_mut_refs(ptrs) }
    }
}

impl<T> Debug for ReadWriteAccess<'_, '_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> MutPtrs<'ctx, T>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { ptrs, .. } = self;
        f.debug_tuple("ReadWriteAccess").field(ptrs).finish()
    }
}

impl<T> PartialEq for ReadWriteAccess<'_, '_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> MutPtrs<'ctx, T>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { ptrs, phantom } = self;
        *ptrs == other.ptrs && *phantom == other.phantom
    }
}

impl<T> Eq for ReadWriteAccess<'_, '_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> MutPtrs<'ctx, T>: Eq,
{
}

impl<T> PartialOrd for ReadWriteAccess<'_, '_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> MutPtrs<'ctx, T>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { ptrs, phantom } = self;

        match ptrs.partial_cmp(&other.ptrs) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        phantom.partial_cmp(&other.phantom)
    }
}

impl<T> Ord for ReadWriteAccess<'_, '_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> MutPtrs<'ctx, T>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { ptrs, phantom } = self;

        match ptrs.cmp(&other.ptrs) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        phantom.cmp(&other.phantom)
    }
}

impl<T> Hash for ReadWriteAccess<'_, '_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> MutPtrs<'ctx, T>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { ptrs, phantom } = self;
        ptrs.hash(state);
        phantom.hash(state);
    }
}

pub enum TryInsertAccess<'ctx, 'a, T>
where
    T: RawSoa + ?Sized,
{
    ReadWrite(ReadWriteAccess<'ctx, 'a, T>),
    WriteOnly(wrapper::MutPtrs<'ctx, T>),
}

impl<'ctx, T> TryInsertAccess<'ctx, '_, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    pub fn write_only(ptrs: MutPtrs<'ctx, T>) -> Self {
        let ptrs = wrapper::MutPtrs::new(ptrs);
        Self::WriteOnly(ptrs)
    }

    #[inline]
    pub unsafe fn read_write_unchecked(ptrs: MutPtrs<'ctx, T>) -> Self {
        let refs = unsafe { ReadWriteAccess::from_ptrs(ptrs) };
        Self::ReadWrite(refs)
    }

    #[inline]
    pub fn as_ptrs(&self) -> &MutPtrs<'_, T> {
        match self {
            Self::ReadWrite(refs) => refs.as_ptrs(),
            Self::WriteOnly(ptrs) => ptrs.as_inner(),
        }
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> &mut MutPtrs<'_, T> {
        match self {
            Self::ReadWrite(refs) => refs.as_mut_ptrs(),
            Self::WriteOnly(ptrs) => ptrs.as_inner_mut(),
        }
    }

    #[inline]
    pub fn into_ptrs(self) -> MutPtrs<'ctx, T> {
        match self {
            Self::ReadWrite(refs) => refs.into_ptrs(),
            Self::WriteOnly(ptrs) => ptrs.into_inner(),
        }
    }
}

impl<'ctx, 'a, T> TryInsertAccess<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
{
    #[inline]
    pub fn read_write(context: &'ctx T::Context, refs: RefsMut<'ctx, 'a, T>) -> Self {
        let refs = ReadWriteAccess::from_refs(context, refs);
        Self::ReadWrite(refs)
    }

    #[inline]
    pub fn into_refs(self, context: &'ctx T::Context) -> Option<RefsMut<'ctx, 'a, T>> {
        match self {
            Self::ReadWrite(refs) => Some(refs.into_refs(context)),
            Self::WriteOnly(_) => None,
        }
    }
}

impl<T> Debug for TryInsertAccess<'_, '_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> MutPtrs<'ctx, T>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadWrite(refs) => f.debug_tuple("ReadWrite").field(refs.as_ptrs()).finish(),
            Self::WriteOnly(ptrs) => f.debug_tuple("WriteOnly").field(ptrs).finish(),
        }
    }
}

impl<T> PartialEq for TryInsertAccess<'_, '_, T>
where
    T: RawSoa + ?Sized,
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
    T: RawSoa + ?Sized,
    for<'ctx> MutPtrs<'ctx, T>: Eq,
{
}

impl<T> PartialOrd for TryInsertAccess<'_, '_, T>
where
    T: RawSoa + ?Sized,
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
    T: RawSoa + ?Sized,
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
    T: RawSoa + ?Sized,
    for<'ctx> MutPtrs<'ctx, T>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::ReadWrite(refs) => refs.hash(state),
            Self::WriteOnly(ptrs) => ptrs.hash(state),
        }
    }
}

pub unsafe fn drop_old_then_write<V, W>(context: &V::Context, dst: TryInsertAccess<V>, value: W)
where
    V: SoaWrite<W> + ?Sized,
{
    let dst = match dst {
        TryInsertAccess::ReadWrite(refs) => {
            let dst = refs.into_ptrs();
            unsafe { context.ptrs_drop_in_place(dst.clone()) }
            dst
        }
        TryInsertAccess::WriteOnly(ptrs) => ptrs.into_inner(),
    };
    unsafe { context.write(dst, value) }
}
