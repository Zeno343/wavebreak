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
                        ReadStorage<'a, Monster>,
                        ReadStorage<'a, Name>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut messages, player_position, map, mut viewsheds, positions, monsters, names) = data;
        
        for (mut viewshed, position, _, name) in (&mut viewsheds, &positions, &monsters, &names).join() {
            if viewshed.dirty {
                viewshed.visible_tiles = compute_fov((position.x, position.y), &map, viewshed.range);
                viewshed.dirty = false;
            }

            if viewshed.visible_tiles.contains(&(player_position.x, player_position.y)) {
                messages.push(format!("{} hurls insults at you!", name.name));
            }
        }
    }
}

