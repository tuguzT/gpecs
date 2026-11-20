use core::{ptr::NonNull, slice};

pub type ConstPtr<T> = <T as SlicePtrsAtIndex>::Const;
pub type MutPtr<T> = <T as SlicePtrsAtIndex>::Mut;
pub type NonNullPtr<T> = <T as SlicePtrsAtIndex>::NonNull;
pub type PtrsItem<T> = <T as SlicePtrsAtIndex>::Item;

pub unsafe trait SlicePtrsAtIndex {
    type Item;

    type Const: ConstSlicePtrAtIndex<Item = Self::Item, Ptrs = Self>;
    type Mut: MutSlicePtrAtIndex<Item = Self::Item, Ptrs = Self>;
    type NonNull: NonNullSlicePtrAtIndex<Item = Self::Item, Ptrs = Self>;
}

pub unsafe trait SlicePtrAtIndex: Copy {
    type Item;

    fn dangling() -> Self;

    fn index(self) -> usize;

    #[must_use = "returns a new pointer rather than modifying its argument"]
    unsafe fn add(self, count: usize) -> Self;

    unsafe fn offset_from(self, origin: Self) -> isize;

    unsafe fn read(self) -> Self::Item;
}

pub unsafe trait ConstSlicePtrAtIndex: SlicePtrAtIndex {
    type Ptrs: SlicePtrsAtIndex<Item = Self::Item, Const = Self>;

    unsafe fn from_parts(slice: *const [Self::Item], index: usize) -> Self;

    fn slice(self) -> *const [Self::Item];

    fn into_parts(self) -> (*const [Self::Item], usize) {
        (self.slice(), self.index())
    }

    fn as_ptr(self) -> *const Self::Item {
        let (slice, index) = self.into_parts();
        unsafe { slice.cast::<Self::Item>().add(index) }
    }

    unsafe fn as_ref<'a>(self) -> Option<&'a Self::Item> {
        unsafe { self.as_ptr().as_ref() }
    }

    unsafe fn as_ref_unchecked<'a>(self) -> &'a Self::Item {
        unsafe { self.as_ptr().as_ref_unchecked() }
    }

    fn cast_mut(self) -> MutPtr<Self::Ptrs> {
        let (slice, index) = self.into_parts();
        let slice = slice.cast_mut();
        unsafe { MutSlicePtrAtIndex::from_parts(slice, index) }
    }

    fn cast_nonnull(self) -> Option<NonNullPtr<Self::Ptrs>> {
        NonNullSlicePtrAtIndex::from_ptr(self)
    }

    unsafe fn cast_nonnull_unchecked(self) -> NonNullPtr<Self::Ptrs> {
        unsafe { NonNullSlicePtrAtIndex::from_ptr_unchecked(self) }
    }
}

pub unsafe trait MutSlicePtrAtIndex: SlicePtrAtIndex {
    type Ptrs: SlicePtrsAtIndex<Item = Self::Item, Mut = Self>;

    unsafe fn from_parts(slice: *mut [Self::Item], index: usize) -> Self;

    fn slice(self) -> *mut [Self::Item];

    fn into_parts(self) -> (*mut [Self::Item], usize) {
        (self.slice(), self.index())
    }

    fn as_ptr(self) -> *const Self::Item {
        self.as_mut_ptr().cast_const()
    }

    fn as_mut_ptr(self) -> *mut Self::Item {
        let (slice, index) = self.into_parts();
        unsafe { slice.cast::<Self::Item>().add(index) }
    }

    unsafe fn as_ref<'a>(self) -> Option<&'a Self::Item> {
        unsafe { self.as_ptr().as_ref() }
    }

    unsafe fn as_ref_unchecked<'a>(self) -> &'a Self::Item {
        unsafe { self.as_ptr().as_ref_unchecked() }
    }

    unsafe fn as_mut<'a>(self) -> Option<&'a mut Self::Item> {
        unsafe { self.as_mut_ptr().as_mut() }
    }

    unsafe fn as_mut_unchecked<'a>(self) -> &'a mut Self::Item {
        unsafe { self.as_mut_ptr().as_mut_unchecked() }
    }

    fn cast_const(self) -> ConstPtr<Self::Ptrs> {
        let (slice, index) = self.into_parts();
        let slice = slice.cast_const();
        unsafe { ConstSlicePtrAtIndex::from_parts(slice, index) }
    }

    fn cast_nonnull(self) -> Option<NonNullPtr<Self::Ptrs>> {
        NonNullSlicePtrAtIndex::from_mut_ptr(self)
    }

    unsafe fn cast_nonnull_unchecked(self) -> NonNullPtr<Self::Ptrs> {
        unsafe { NonNullSlicePtrAtIndex::from_mut_ptr_unchecked(self) }
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

pub unsafe trait NonNullSlicePtrAtIndex: SlicePtrAtIndex {
    type Ptrs: SlicePtrsAtIndex<Item = Self::Item, NonNull = Self>;

    unsafe fn from_parts(slice: NonNull<[Self::Item]>, index: usize) -> Self;

    fn slice(self) -> NonNull<[Self::Item]>;

    fn into_parts(self) -> (NonNull<[Self::Item]>, usize) {
        (self.slice(), self.index())
    }

    fn from_ref(r: &Self::Item) -> Self {
        let slice = slice::from_ref(r);
        let slice = NonNull::from_ref(slice);
        unsafe { Self::from_parts(slice, 0) }
    }

    fn from_mut(r: &mut Self::Item) -> Self {
        let slice = slice::from_mut(r);
        let slice = NonNull::from_mut(slice);
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
        unsafe { ConstSlicePtrAtIndex::from_parts(slice, index) }
    }

    fn cast_mut(self) -> MutPtr<Self::Ptrs> {
        let (slice, index) = self.into_parts();
        let slice = slice.as_ptr();
        unsafe { MutSlicePtrAtIndex::from_parts(slice, index) }
    }

    unsafe fn as_ref<'a>(self) -> &'a Self::Item {
        let ptr = &self.as_ptr();
        unsafe { NonNull::as_ref(ptr) }
    }

    unsafe fn as_mut<'a>(self) -> &'a mut Self::Item {
        let ptr = &mut self.as_ptr();
        unsafe { NonNull::as_mut(ptr) }
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
