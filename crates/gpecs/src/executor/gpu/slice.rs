use wgpu::{Buffer, BufferAddress, BufferSize, BufferSlice};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NonEmptyBufferSlice<'a> {
    slice: BufferSlice<'a>,
}

impl<'a> NonEmptyBufferSlice<'a> {
    #[inline]
    pub fn new(slice: BufferSlice<'a>) -> Option<Self> {
        if slice.size() == 0 {
            return None;
        }

        let me = Self { slice };
        Some(me)
    }

    #[inline]
    pub fn buffer(&self) -> &'a Buffer {
        let Self { slice } = self;
        slice.buffer()
    }

    #[inline]
    pub fn offset(&self) -> BufferAddress {
        let Self { slice } = self;
        slice.offset()
    }

    #[inline]
    pub fn size(&self) -> BufferSize {
        let Self { slice } = self;
        slice
            .size()
            .try_into()
            .expect("slice size should be non-zero")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonEmptyContents<'a> {
    contents: &'a [u8],
}

impl<'a> NonEmptyContents<'a> {
    #[inline]
    pub fn new(contents: &'a [u8]) -> Option<Self> {
        if contents.is_empty() {
            return None;
        }

        let me = Self { contents };
        Some(me)
    }

    #[inline]
    pub fn as_slice(&self) -> &'a [u8] {
        let Self { contents } = self;
        contents
    }
}
