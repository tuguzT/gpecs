use core::ptr::NonNull;

pub mod gpu;

mod prim;

pub unsafe trait SliceItemPtrs<T> {
    type Const: ConstSliceItemPtr<Item = T, Ptrs = Self>;
    type Mut: MutSliceItemPtr<Item = T, Ptrs = Self>;
    type NonNull: NonNullSliceItemPtr<Item = T, Ptrs = Self>;
}

pub type CastConstPtr<T> =
    <<T as MutSliceItemPtr>::Ptrs as SliceItemPtrs<<T as SliceItemPtr>::Item>>::Const;

pub type CastMutPtr<T> =
    <<T as ConstSliceItemPtr>::Ptrs as SliceItemPtrs<<T as SliceItemPtr>::Item>>::Mut;

pub type NonNullAsPtr<T> =
    <<T as NonNullSliceItemPtr>::Ptrs as SliceItemPtrs<<T as SliceItemPtr>::Item>>::Mut;

pub unsafe trait SliceItemPtr: Copy {
    type Item;

    fn index(self) -> usize;

    #[must_use = "returns a new pointer rather than modifying its argument"]
    unsafe fn add(self, count: usize) -> Self;

    unsafe fn offset_from(self, origin: Self) -> isize;

    unsafe fn read(self) -> Self::Item;
}

pub unsafe trait ConstSliceItemPtr: SliceItemPtr {
    type Ptrs: SliceItemPtrs<Self::Item>;

    unsafe fn from_slice(slice: *const [Self::Item], index: usize) -> Self;

    fn slice(self) -> *const [Self::Item];

    unsafe fn as_ref<'a>(self) -> &'a Self::Item;

    fn cast_mut(self) -> CastMutPtr<Self> {
        let slice = self.slice().cast_mut();
        let index = self.index();
        unsafe { MutSliceItemPtr::from_slice(slice, index) }
    }
}

pub unsafe trait MutSliceItemPtr: SliceItemPtr {
    type Ptrs: SliceItemPtrs<Self::Item>;

    unsafe fn from_slice(slice: *mut [Self::Item], index: usize) -> Self;

    fn slice(self) -> *mut [Self::Item];

    unsafe fn as_mut<'a>(self) -> &'a mut Self::Item;

    unsafe fn write(self, value: Self::Item);

    unsafe fn swap(self, with: Self);

    unsafe fn copy_from(self, src: CastConstPtr<Self>, count: usize);

    unsafe fn copy_from_nonoverlapping(self, src: CastConstPtr<Self>, count: usize);

    fn cast_const(self) -> CastConstPtr<Self> {
        let slice = self.slice().cast_const();
        let index = self.index();
        unsafe { ConstSliceItemPtr::from_slice(slice, index) }
    }
}

pub unsafe trait NonNullSliceItemPtr: SliceItemPtr {
    type Ptrs: SliceItemPtrs<Self::Item>;

    unsafe fn from_slice(slice: NonNull<[Self::Item]>, index: usize) -> Self;

    fn slice(self) -> NonNull<[Self::Item]>;

    fn as_ptr(self) -> NonNullAsPtr<Self> {
        let slice = self.slice().as_ptr();
        let index = self.index();
        unsafe { MutSliceItemPtr::from_slice(slice, index) }
    }
}
