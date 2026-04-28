use glam::IVec2;

use crate::{
    components::{Position, Sprite},
    framebuffer::Framebuffer,
};

pub fn render_sprite<B>(position: &Position, sprite: &Sprite, framebuffer: &mut Framebuffer<B>)
where
    B: AsMut<[u32]>,
{
    let IVec2 { x, y } = position.data.as_ivec2();
    framebuffer.draw(x, y, sprite.character);
}
