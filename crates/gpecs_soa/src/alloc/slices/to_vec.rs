use crate::{
    alloc::set_len_on_drop::SetLenOnDrop,
    slices::{SoaView, SoaViewMut},
    traits::{SoaAlloc, SoaCloneToUninit, SoaCloneToUninitContext, SoaRawContext},
    vec::SoaVec,
};

pub trait ToSoaVec {
    type Soa: SoaAlloc<Context: Clone> + SoaCloneToUninit + ?Sized;

    fn to_vec(&self) -> SoaVec<Self::Soa>;
}

impl<T> ToSoaVec for SoaView<'_, '_, T>
where
    T: SoaAlloc + SoaCloneToUninit + ?Sized,
    T::Context: Clone,
{
    type Soa = T;

    #[inline]
    fn to_vec(&self) -> SoaVec<T> {
        let len = self.len();
        let context = self.context().clone();
        let mut vec = SoaVec::<T>::with_context_and_capacity(context, len);

        {
            let mut set_len_on_drop = SetLenOnDrop {
                vec: &mut vec,
                local_len: 0,
            };

            let (context, dst) = set_len_on_drop.vec.as_mut_ptrs_with_context();
            for (index, src) in self.iter_ptrs().enumerate() {
                set_len_on_drop.local_len = index;

                let dst = unsafe { context.mut_ptrs_add(dst.clone(), index) };
                unsafe { context.ptrs_clone_to_uninit(src, dst) }
            }
        }

        // SAFETY:
        // the vec was allocated and initialized above to at least this length.
        unsafe {
            vec.set_len(len);
        }
        vec
    }
}

impl<T> ToSoaVec for SoaViewMut<'_, '_, T>
where
    T: SoaAlloc + SoaCloneToUninit + ?Sized,
    T::Context: Clone,
{
    type Soa = T;

    #[inline]
    fn to_vec(&self) -> SoaVec<T> {
        self.as_view().to_vec()
    }
}
