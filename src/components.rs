use specs::prelude::*;
use specs_derive::Component;

use crossterm::style;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Monster;

#[derive(Component, Copy, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: char,
    pub foreground: style::Color,
    pub background: style::Color,
}

#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<(usize, usize)>,
    pub range: usize,
    pub dirty: bool
}

