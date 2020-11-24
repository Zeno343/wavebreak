use specs::prelude::*;
use specs_derive::Component;

use crate::Color;

#[derive(Component)]
pub struct BlocksTile;

#[derive(Component, Debug)]
pub struct CombatStats {
    pub max_hp: usize,
    pub hp: usize,
    pub defense: usize,
    pub power: usize 
}

#[derive(Component)]
pub struct IncomingDamage {
    pub damage: Vec<usize>,
}

impl IncomingDamage {
    pub fn add_damage(
        store: &mut WriteStorage<IncomingDamage>,
        target: Entity,
        damage: usize
    ) {
        if let Some(target) = store.get_mut(target) {
            target.damage.push(damage);
        } else {
            let incoming_damage = IncomingDamage { damage: vec![damage] };
            store.insert(target, incoming_damage).expect("Couldn't add damage");
        }
    }
}

#[derive(Component)]
pub struct Monster;

#[derive(Component)]
pub struct MeleeAttack {
    pub target: Entity,
}

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

