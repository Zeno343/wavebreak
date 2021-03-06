use specs::prelude::*;
use crate::{
    log,
    components::*,
    map::Tile,
    Map, 
};

use wavebreaker_util::{
    algorithms::{
        fov::compute_fov,
        pathfinding::find_path,
    },
    data_structures::Queue,
};

pub struct MonsterAi;

impl<'a> System<'a> for MonsterAi {
    type SystemData = (WriteExpect<'a, Queue<String>>, 
                       ReadExpect<'a, Position>,
                       ReadExpect<'a, Map>,
                       WriteStorage<'a, Viewshed>, 
                       WriteStorage<'a, Position>,
                       ReadStorage<'a, Monster>,
                       ReadStorage<'a, Name>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut messages, 
             player_position, 
             map, 
             mut viewsheds, 
             mut positions, 
             monsters, 
             names) = data;
        
        for (mut viewshed, mut position, _, name) 
            in (&mut viewsheds, &mut positions, &monsters, &names).join() 
        {
            if viewshed.dirty {
                viewshed.visible_tiles = compute_fov(
                    (position.x, position.y), 
                    &*map, 
                    viewshed.range
                );

                viewshed.dirty = false;
            }

            if viewshed.visible_tiles.contains(&(player_position.x, player_position.y)) {
                messages.push(format!("{} hurls insults at you!", name.name));
                let mut path = find_path::<Tile>(
                    (position.x, position.y), 
                    (player_position.x, player_position.y), 
                    &*map
                );

                if path.len() > 1 {
                    let next_tile = path.pop().unwrap();
                    *position = Position { x: next_tile.0, y: next_tile.1 };
                    viewshed.dirty = true;
                }
            }
        }
    }
}

