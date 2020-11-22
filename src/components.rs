use specs::prelude::*;
use specs_derive::Component;

use crate::Color;

#[derive(Component)]
pub struct BlocksTile;

#[derive(Component, Debug)]
pub struct CombatStats {
    pub max_hp : i32,
    pub hp : i32,
    pub defense : i32,
    pub power : i32
}

#[derive(Component)]
pub struct Monster;

#[derive(Component)]
pub struct Name {
    pub name: String,
}

#[derive(Component)]
pub struct Player;

#[derive(Component, Copy, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: char,
    pub color: Color,
}

#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<(usize, usize)>,
    pub range: usize,
    pub dirty: bool
}

