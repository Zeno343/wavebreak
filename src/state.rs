use specs::prelude::*;
use crate::{
    CELL_WIDTH,
    CELL_HEIGHT,
    Color,
    components::*,
    FontCache,
    fov::compute_fov,
    map::{
        Map,
        TileType,
    },
    monster_ai::MonsterAi,
    Rect,
    View,
};

#[derive(Clone, Copy, PartialEq)]
pub enum RunState{
    Paused,
    Running,
}

pub struct State<'a> {
    pub world: World,
    pub run_state: RunState,
    pub font: FontCache<'a>,
}

impl<'a> State<'a> {
    pub fn tick(&mut self, view: &mut View) {
        if self.run_state == RunState::Running {
            reveal_map(&self.world);

            let mut monster_ai = MonsterAi { };
            monster_ai.run_now(&self.world);

            self.run_state = RunState::Paused;
        }

        let positions = self.world.read_storage::<Position>();
        let renderables = self.world.read_storage::<Renderable>();

        let map = self.world.fetch::<Map>(); 
        
        view.clear();
        draw_map(view, &mut self.font, &map).expect("Could not draw map");
        for (pos, render) in (&positions, &renderables).join() {
            if map[(pos.x, pos.y)].visible {
                draw_entity(view, &mut self.font, pos, render);
            }
        }

        view.present();
    }
}

pub fn draw_entity(view: &mut View, font: &mut FontCache, position: &Position, renderable: &Renderable) {
    view.draw_glyph(
        font, 
        renderable.glyph, 
        renderable.color, 
        Color::RGB(0, 0, 0),
        Rect::new((position.x as u32 * CELL_WIDTH) as i32, (position.y as u32 * CELL_HEIGHT) as i32, CELL_WIDTH, CELL_HEIGHT)
    )
        .expect("Could not render entity");
}

pub fn draw_map(view: &mut View, font: &mut FontCache, map: &Map) -> crossterm::Result<()> {
    for (idx, tile) in map.tiles.iter().enumerate() {
        let x = idx / map.height;
        let y = idx % map.height;
         
        let color: Color;
        let background = Color::RGB(0, 0, 0); 

        let visible = map[(x,y)].visible;

        if visible {
            color = Color::RGB(255, 255, 255);
        } else {
            color = Color::RGB(128, 128, 128);
        }

        if visible || map[(x,y)].revealed {
            match tile.tile_type {
               TileType::Wall => { 
                    view.draw_glyph(font, 
                        '\u{2593}', 
                        color,
                        background,
                        Rect::new((x as u32 * CELL_WIDTH) as i32, (y as u32 * CELL_HEIGHT) as i32, CELL_WIDTH, CELL_HEIGHT)
                    )
                        .expect("Could not render entity");
                },

                TileType::Floor => {
                    view.draw_glyph(font, 
                        '.', 
                        color,
                        background,
                        Rect::new((x as u32 * CELL_WIDTH) as i32, (y as u32 * CELL_HEIGHT) as i32, CELL_WIDTH, CELL_HEIGHT)
                    )
                        .expect("Could not render entity");
                },
            }
        }
    }

    Ok(())
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
