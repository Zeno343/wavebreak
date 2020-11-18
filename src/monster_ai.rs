use specs::prelude::*;
use crate::{
    components::*,
    fov::compute_fov,
    Map, 
    util::Queue,
};

pub struct MonsterAi;

impl<'a> System<'a> for MonsterAi {
    type SystemData = ( WriteExpect<'a, Queue<String>>, 
                        ReadExpect<'a, Position>,
                        ReadExpect<'a, Map>,
                        WriteStorage<'a, Viewshed>, 
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, Monster>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut messages, player_position, map, mut viewsheds, positions, monsters) = data;
        
        for (mut viewshed, position, _) in (&mut viewsheds, &positions, &monsters).join() {
            if viewshed.dirty {
                viewshed.visible_tiles = compute_fov((position.x, position.y), &map, viewshed.range);
                viewshed.dirty = false;
            }

            if viewshed.visible_tiles.contains(&(player_position.x, player_position.y)) {
                messages.push("Goblin hurls insults at you!".to_string());
            }
        }
    }
}

