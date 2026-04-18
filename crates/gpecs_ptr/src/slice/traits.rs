use core::ptr::NonNull;

pub type ConstPtr<T> = <T as SliceItemPtrs>::Const;
pub type MutPtr<T> = <T as SliceItemPtrs>::Mut;
pub type NonNullPtr<T> = <T as SliceItemPtrs>::NonNull;
pub type PtrsItem<T> = <T as SliceItemPtrs>::Item;

pub type CastConst<T> = <<T as MutSliceItemPtr>::Ptrs as SliceItemPtrs>::Const;
pub type CastMut<T> = <<T as ConstSliceItemPtr>::Ptrs as SliceItemPtrs>::Mut;
pub type NonNullAsPtr<T> = <<T as NonNullSliceItemPtr>::Ptrs as SliceItemPtrs>::Mut;

pub unsafe trait SliceItemPtrs {
    type Item;

    type Const: ConstSliceItemPtr<Item = Self::Item, Ptrs = Self>;
    type Mut: MutSliceItemPtr<Item = Self::Item, Ptrs = Self>;
    type NonNull: NonNullSliceItemPtr<Item = Self::Item, Ptrs = Self>;
}

pub unsafe trait SliceItemPtr: Copy {
    type Item;

    fn index(self) -> usize;

    #[must_use = "returns a new pointer rather than modifying its argument"]
    unsafe fn add(self, count: usize) -> Self;

    unsafe fn offset_from(self, origin: Self) -> isize;

    unsafe fn read(self) -> Self::Item;
}

pub unsafe trait ConstSliceItemPtr: SliceItemPtr {
    type Ptrs: SliceItemPtrs<Item = Self::Item, Const = Self>;

    unsafe fn from_slice(slice: *const [Self::Item], index: usize) -> Self;

    fn slice(self) -> *const [Self::Item];

    fn as_item_ptr(self) -> *const Self::Item {
        let count = self.index();
        unsafe { self.slice().cast::<Self::Item>().add(count) }
    }

    unsafe fn as_item<'a>(self) -> &'a Self::Item {
        unsafe { self.as_item_ptr().as_ref_unchecked() }
    }

    fn cast_mut(self) -> CastMut<Self> {
        let slice = self.slice().cast_mut();
        let index = self.index();
        unsafe { MutSliceItemPtr::from_slice(slice, index) }
    }
}

pub unsafe trait MutSliceItemPtr: SliceItemPtr {
    type Ptrs: SliceItemPtrs<Item = Self::Item, Mut = Self>;

    unsafe fn from_slice(slice: *mut [Self::Item], index: usize) -> Self;

    fn slice(self) -> *mut [Self::Item];

    fn as_mut_item_ptr(self) -> *mut Self::Item {
        let count = self.index();
        unsafe { self.slice().cast::<Self::Item>().add(count) }
    }

    unsafe fn as_mut_item<'a>(self) -> &'a mut Self::Item {
        unsafe { self.as_mut_item_ptr().as_mut_unchecked() }
    }

    fn cast_const(self) -> CastConst<Self> {
        let slice = self.slice().cast_const();
        let index = self.index();
        unsafe { ConstSliceItemPtr::from_slice(slice, index) }
    }

    unsafe fn write(self, value: Self::Item);

    unsafe fn swap(self, with: Self);

    unsafe fn copy_from(self, src: CastConst<Self>, count: usize);

    unsafe fn copy_from_nonoverlapping(self, src: CastConst<Self>, count: usize);
}

pub unsafe trait NonNullSliceItemPtr: SliceItemPtr {
    type Ptrs: SliceItemPtrs<Item = Self::Item, NonNull = Self>;

    unsafe fn from_slice(slice: NonNull<[Self::Item]>, index: usize) -> Self;

    fn slice(self) -> NonNull<[Self::Item]>;

    fn as_item_ptr(self) -> NonNull<Self::Item> {
        let count = self.index();
        unsafe { self.slice().cast::<Self::Item>().add(count) }
    }

    fn as_ptr(self) -> NonNullAsPtr<Self> {
        let slice = self.slice().as_ptr();
        let index = self.index();
        unsafe { MutSliceItemPtr::from_slice(slice, index) }
    }
}
