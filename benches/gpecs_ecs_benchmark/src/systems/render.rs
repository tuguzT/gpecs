use crate::{
    components::{Position, Sprite},
    framebuffer::Framebuffer,
};

pub fn render_sprite<B>(position: &Position, sprite: &Sprite, framebuffer: &mut Framebuffer<B>)
where
    B: AsMut<[u32]>,
{
    framebuffer.draw(position.x as i32, position.y as i32, sprite.character);
}
