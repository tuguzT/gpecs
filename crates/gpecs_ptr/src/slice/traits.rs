use core::ptr::{self, NonNull};

pub type ConstPtr<T> = <T as SliceItemPtrs>::Const;
pub type MutPtr<T> = <T as SliceItemPtrs>::Mut;
pub type NonNullPtr<T> = <T as SliceItemPtrs>::NonNull;
pub type PtrsItem<T> = <T as SliceItemPtrs>::Item;

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

    unsafe fn from_parts(slice: *const [Self::Item], index: usize) -> Self;

    fn slice(self) -> *const [Self::Item];

    fn into_parts(self) -> (*const [Self::Item], usize) {
        (self.slice(), self.index())
    }

    fn dangling() -> Self {
        let slice = ptr::slice_from_raw_parts(ptr::dangling(), 0);
        unsafe { Self::from_parts(slice, 0) }
    }

    fn as_ptr(self) -> *const Self::Item {
        let (slice, index) = self.into_parts();
        unsafe { slice.cast::<Self::Item>().add(index) }
    }

    unsafe fn as_ref_unchecked<'a>(self) -> &'a Self::Item {
        unsafe { self.as_ptr().as_ref_unchecked() }
    }

    fn cast_mut(self) -> MutPtr<Self::Ptrs> {
        let (slice, index) = self.into_parts();
        let slice = slice.cast_mut();
        unsafe { MutSliceItemPtr::from_parts(slice, index) }
    }

    fn cast_nonnull(self) -> Option<NonNullPtr<Self::Ptrs>> {
        NonNullSliceItemPtr::from_ptr(self)
    }

    unsafe fn cast_nonnull_unchecked(self) -> NonNullPtr<Self::Ptrs> {
        unsafe { NonNullSliceItemPtr::from_ptr_unchecked(self) }
    }
}

pub unsafe trait MutSliceItemPtr: SliceItemPtr {
    type Ptrs: SliceItemPtrs<Item = Self::Item, Mut = Self>;

    unsafe fn from_parts(slice: *mut [Self::Item], index: usize) -> Self;

    fn slice(self) -> *mut [Self::Item];

    fn into_parts(self) -> (*mut [Self::Item], usize) {
        (self.slice(), self.index())
    }

    fn dangling() -> Self {
        let slice = ptr::slice_from_raw_parts_mut(ptr::dangling_mut(), 0);
        unsafe { Self::from_parts(slice, 0) }
    }

    fn as_ptr(self) -> *const Self::Item {
        self.as_mut_ptr().cast_const()
    }

    fn as_mut_ptr(self) -> *mut Self::Item {
        let (slice, index) = self.into_parts();
        unsafe { slice.cast::<Self::Item>().add(index) }
    }

    unsafe fn as_ref_unchecked<'a>(self) -> &'a Self::Item {
        unsafe { self.as_ptr().as_ref_unchecked() }
    }

    unsafe fn as_mut_unchecked<'a>(self) -> &'a mut Self::Item {
        unsafe { self.as_mut_ptr().as_mut_unchecked() }
    }

    fn cast_const(self) -> ConstPtr<Self::Ptrs> {
        let (slice, index) = self.into_parts();
        let slice = slice.cast_const();
        unsafe { ConstSliceItemPtr::from_parts(slice, index) }
    }

    fn cast_nonnull(self) -> Option<NonNullPtr<Self::Ptrs>> {
        NonNullSliceItemPtr::from_mut_ptr(self)
    }

    unsafe fn cast_nonnull_unchecked(self) -> NonNullPtr<Self::Ptrs> {
        unsafe { NonNullSliceItemPtr::from_mut_ptr_unchecked(self) }
    }

    unsafe fn drop_in_place(self) {
        unsafe { self.as_mut_ptr().drop_in_place() }
    }

    unsafe fn write(self, value: Self::Item);

    unsafe fn swap(self, with: Self);

    unsafe fn swap_nonoverlapping(self, with: Self, count: usize);

    unsafe fn copy_from(self, src: ConstPtr<Self::Ptrs>, count: usize);

    unsafe fn copy_from_nonoverlapping(self, src: ConstPtr<Self::Ptrs>, count: usize);
}

pub unsafe trait NonNullSliceItemPtr: SliceItemPtr {
    type Ptrs: SliceItemPtrs<Item = Self::Item, NonNull = Self>;

    unsafe fn from_parts(slice: NonNull<[Self::Item]>, index: usize) -> Self;

    fn slice(self) -> NonNull<[Self::Item]>;

    fn into_parts(self) -> (NonNull<[Self::Item]>, usize) {
        (self.slice(), self.index())
    }

    fn dangling() -> Self {
        let slice = NonNull::slice_from_raw_parts(NonNull::dangling(), 0);
        unsafe { Self::from_parts(slice, 0) }
    }

    fn from_ptr(ptr: ConstPtr<Self::Ptrs>) -> Option<Self> {
        let (slice, index) = ptr.into_parts();
        let slice = NonNull::new(slice.cast_mut())?;
        let me = unsafe { Self::from_parts(slice, index) };
        Some(me)
    }

    unsafe fn from_ptr_unchecked(ptr: ConstPtr<Self::Ptrs>) -> Self {
        let (slice, index) = ptr.into_parts();
        let slice = unsafe { NonNull::new_unchecked(slice.cast_mut()) };
        unsafe { Self::from_parts(slice, index) }
    }

    fn from_mut_ptr(ptr: MutPtr<Self::Ptrs>) -> Option<Self> {
        let (slice, index) = ptr.into_parts();
        let slice = NonNull::new(slice)?;
        let me = unsafe { Self::from_parts(slice, index) };
        Some(me)
    }

    unsafe fn from_mut_ptr_unchecked(ptr: MutPtr<Self::Ptrs>) -> Self {
        let (slice, index) = ptr.into_parts();
        let slice = unsafe { NonNull::new_unchecked(slice) };
        unsafe { Self::from_parts(slice, index) }
    }

    fn as_ptr(self) -> NonNull<Self::Item> {
        let (slice, index) = self.into_parts();
        unsafe { slice.cast::<Self::Item>().add(index) }
    }

    fn cast_const(self) -> ConstPtr<Self::Ptrs> {
        let (slice, index) = self.into_parts();
        let slice = slice.as_ptr().cast_const();
        unsafe { ConstSliceItemPtr::from_parts(slice, index) }
    }

    fn cast_mut(self) -> MutPtr<Self::Ptrs> {
        let (slice, index) = self.into_parts();
        let slice = slice.as_ptr();
        unsafe { MutSliceItemPtr::from_parts(slice, index) }
    }

    unsafe fn as_ref<'a>(self) -> &'a Self::Item {
        unsafe { self.as_ptr().as_ref() }
    }

    unsafe fn as_mut<'a>(self) -> &'a mut Self::Item {
        unsafe { self.as_ptr().as_mut() }
    }

    unsafe fn drop_in_place(self) {
        unsafe { self.cast_mut().drop_in_place() }
    }

    unsafe fn write(self, value: Self::Item) {
        unsafe { self.cast_mut().write(value) }
    }

    unsafe fn swap(self, with: Self) {
        unsafe { self.cast_mut().swap(with.cast_mut()) }
    }

    unsafe fn swap_nonoverlapping(self, with: Self, count: usize) {
        let with = with.cast_mut();
        unsafe { self.cast_mut().swap_nonoverlapping(with, count) }
    }

    unsafe fn copy_from(self, src: Self, count: usize) {
        unsafe { self.cast_mut().copy_from(src.cast_const(), count) }
    }

    unsafe fn copy_from_nonoverlapping(self, src: Self, count: usize) {
        let src = src.cast_const();
        unsafe { self.cast_mut().copy_from_nonoverlapping(src, count) }
    }
}
