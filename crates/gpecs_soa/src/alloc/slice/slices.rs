use crate::{
    alloc::set_len_on_drop::SetLenOnDrop,
    slice::{SoaSlices, SoaSlicesMut},
    traits::{AllocSoa, CloneToUninitSoaContext, RawSoaContext, SoaCloneToUninit},
    vec::SoaVec,
};

pub trait ToSoaVec {
    type Soa: AllocSoa<Context: Clone> + SoaCloneToUninit + ?Sized;

    fn to_vec(&self) -> SoaVec<Self::Soa>;
}

impl<T> ToSoaVec for SoaSlices<'_, '_, T>
where
    T: AllocSoa + SoaCloneToUninit + ?Sized,
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

            let (context, dst, _) = set_len_on_drop.vec.mut_slices().into_parts();
            for (index, src) in self.raw_iter().enumerate() {
                set_len_on_drop.local_len = index;

                let dst = unsafe { context.ptrs_add_mut(dst.clone(), index) };
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

impl<T> ToSoaVec for SoaSlicesMut<'_, '_, T>
where
    T: AllocSoa + SoaCloneToUninit + ?Sized,
    T::Context: Clone,
{
    type Soa = T;

    #[inline]
    fn to_vec(&self) -> SoaVec<T> {
        self.slices().to_vec()
    }
}
