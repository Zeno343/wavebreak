use specs::prelude::*;
use crate::{
    components::*,
    fov::compute_fov,
    map::Map,
    monster_ai::MonsterAi,
    util::Queue,
    View,
};

#[derive(Clone, Copy, PartialEq)]
pub enum RunState{
    Paused,
    Running,
}

pub struct State {
    pub world: World,
    pub run_state: RunState,
}

impl State {
    pub fn tick(&mut self, view: &mut View) {
        if self.run_state == RunState::Running {
            reveal_map(&self.world);

            let mut monster_ai = MonsterAi { };
            monster_ai.run_now(&self.world);

        }

        let positions = self.world.read_storage::<Position>();
        let renderables = self.world.read_storage::<Renderable>();

        let map = self.world.fetch::<Map>(); 
        let messages = self.world.fetch::<Queue<String>>();
        
        view.begin_frame().expect("Could not begin frame");
        view.draw_map(&map).expect("Could not draw map");
        view.draw_message_log(&messages).expect("Could not draw message log");
        for (pos, render) in (&positions, &renderables).join() {
            if map[(pos.x, pos.y)].visible {
                view.draw_entity(pos, render).expect("Could not draw entity");
            }
        }

        view.end_frame().expect("Could not end frame");
    }
}

fn reveal_map(world: &World) {
    let players = world.read_storage::<Player>();
    let mut viewsheds = world.write_storage::<Viewshed>();
    let positions = world.read_storage::<Position>();

    let mut map = world.fetch_mut::<Map>();

    let (_, viewshed, position) = (&players, &mut viewsheds, &positions).join().nth(0).expect("No viewshed for player");

    if viewshed.dirty {
        for mut tile in &mut map.tiles {
            tile.visible = false;
        }

        viewshed.visible_tiles = compute_fov((position.x, position.y), &map, viewshed.range);

        for &(x, y) in &viewshed.visible_tiles {
            map[(x, y)].visible = true;
            map[(x, y)].revealed = true;
        }
    }
}
