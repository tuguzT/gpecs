use gpecs::prelude::*;

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(C)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Mass {
    pub value: u32,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Name {
    pub value: String,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Tag;

impl Component for Position {}
impl Component for Mass {}
impl Component for Name {}
impl Component for Tag {}
