use bytemuck::{Pod, Zeroable};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
#[repr(C)]
pub struct FramebufferDesc {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Framebuffer<B> {
    desc: FramebufferDesc,
    buffer: B,
}

impl<B> Framebuffer<B> {
    pub fn desc(&self) -> FramebufferDesc {
        let Self { desc, .. } = *self;
        desc
    }

    pub fn buffer(&self) -> &B {
        let Self { buffer, .. } = self;
        buffer
    }

    pub fn buffer_mut(&mut self) -> &mut B {
        let Self { buffer, .. } = self;
        buffer
    }
}

impl<B> Framebuffer<B>
where
    B: AsRef<[u32]>,
{
    pub fn new(width: u32, height: u32, buffer: B) -> Self {
        assert!(
            buffer.as_ref().len() <= (width * height) as usize,
            "buffer is too small for the given width {width} and height {height}",
        );
        let desc = FramebufferDesc { width, height };
        Self { desc, buffer }
    }
}

impl<B> Framebuffer<B>
where
    B: AsMut<[u32]>,
{
    pub fn draw(&mut self, x: i32, y: i32, char: u32) {
        let (Ok(x), Ok(y)) = (u32::try_from(x), u32::try_from(y)) else {
            return;
        };

        let Self {
            desc,
            ref mut buffer,
        } = *self;

        let buffer = buffer.as_mut();
        let FramebufferDesc { width, height } = desc;
        if y < height && x < width {
            buffer[(x + y * width) as usize] = char;
        }
    }
}
