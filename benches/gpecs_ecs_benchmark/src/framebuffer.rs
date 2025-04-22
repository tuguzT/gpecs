#[derive(Debug)]
#[repr(C, align(16))]
pub struct Framebuffer<B> {
    width: u32,
    height: u32,
    buffer: B,
}

impl<B> Framebuffer<B> {
    pub fn width(&self) -> u32 {
        let Self { width, .. } = *self;
        width
    }

    pub fn height(&self) -> u32 {
        let Self { height, .. } = *self;
        height
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
        Self {
            width,
            height,
            buffer,
        }
    }

    pub fn buffer(&self) -> &[u32] {
        let Self { buffer, .. } = self;
        buffer.as_ref()
    }
}

impl<B> Framebuffer<B>
where
    B: AsMut<[u32]>,
{
    pub fn draw(&mut self, x: i32, y: i32, char: u32) {
        let Self {
            width,
            height,
            ref mut buffer,
        } = *self;

        if y >= 0 && y < height as i32 {
            if x >= 0 && x < width as i32 {
                buffer.as_mut()[(x + y * width as i32) as usize] = char;
            }
        }
    }
}
