use std::{
    convert::TryInto,
    fs::OpenOptions,
    io::{
        Write,
    },
    panic,
    time::{
        SystemTime,
        UNIX_EPOCH,
    },
};

use specs::prelude::*;

mod app;
use app::{
    App,
    State,
};

mod components;
use components::*;
mod map;
use map::{
    Map,
};

mod map_processing;
mod monster_ai;

pub use wavebreaker_sdl2::{
    font::{
        FontCache,
        FontManager,
    },
    view::{
        Color,
        Event,
        Keycode,
        Rect,
        View,
    }
};

pub use wavebreaker_util::{
    algorithms::{
        fov::compute_fov,
        pathfinding::find_path,
        simple_rng::SimpleRng,
    },
    data_structures::Queue,
};

const LOG_FILE: &str = "log";

const SCREEN_WIDTH: u32 = 1920;
const SCREEN_HEIGHT: u32 = 1024;
const CELL_WIDTH: u32 = 12;
const CELL_HEIGHT: u32 = 20;

fn log(message: &str) {
    let mut log = OpenOptions::new().append(true).create(true).open(LOG_FILE).expect("Could not open log file");
    log.write_all(message.as_bytes()).expect("could not write to log file");
    log.write(&['\n' as u8]).expect("could not write to log file");
}

fn try_move_player(d_x: i16, d_y: i16, world: &World) -> bool {
    let mut positions = world.write_storage::<Position>();
    let players = world.write_storage::<Player>();
    let mut viewsheds = world.write_storage::<Viewshed>();

    let map = world.fetch::<Map>();
    
    for (_, pos, viewshed) in (&players, &mut positions, &mut viewsheds).join() {
        let dest_x: Option<usize> = (pos.x as i16 + d_x).try_into().ok();
        let dest_y: Option<usize> = (pos.y as i16 + d_y).try_into().ok();
        
        log(
            &format!("Player attempting move from {},{} to {},{}", 
                pos.x, 
                pos.y, 
                dest_x.unwrap(), 
                dest_y.unwrap()
            )
        );

        if dest_x.is_some() && dest_y.is_some() {
            let dest_x = dest_x.unwrap();
            let dest_y = dest_y.unwrap();

            if dest_x < map.width && dest_y < map.height {
                log(&format!("Player landed on {:?}", map[(dest_x, dest_y)]));
                if !map[(dest_x, dest_y)].blocked {
                    //set player's position component
                    pos.x = dest_x;
                    pos.y = dest_y;

                    //update player position resource
                    let mut player_pos = world.write_resource::<Position>();
                    player_pos.x = pos.x;
                    player_pos.y = pos.y;

                    viewshed.dirty = true;

                    return true;
                }

            }
        }
    }

    false
}

fn main() -> Result<(), String> {
    let _ = OpenOptions::new().write(true).create(true).truncate(true).open(LOG_FILE).expect("Could not open log file");

    panic::set_hook(Box::new(|panic_info| {
        let mut log = OpenOptions::new().append(true).create(true).open(LOG_FILE).expect("Could not open log file");
        log.write_all(format!("panic occurred: {:?}", panic_info).as_bytes()).expect("Error writing to log file");
    }));

    let mut view = View::init("Wavebreaker", SCREEN_WIDTH, SCREEN_HEIGHT).expect("Could not initialize view"); 
    let font_manager = FontManager::init(view.canvas())?;
    let input_mono = font_manager.load("assets/InputMono-Regular.ttf")?;

    let mut state = App { 
        world: World::new(),
        run_state: State::Running,
        font: input_mono,
    };

    let now = SystemTime::now();
    let timestamp = now
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let mut rng = SimpleRng::new(timestamp as usize);

    state.world.register::<Player>();
    state.world.register::<Position>();
    state.world.register::<Renderable>();
    state.world.register::<Viewshed>();
    state.world.register::<Monster>();
    state.world.register::<Name>();
    state.world.register::<BlocksTile>();
    state.world.register::<CombatStats>();
    
    let map_width = SCREEN_WIDTH / CELL_WIDTH;
    let map_height = SCREEN_HEIGHT / CELL_HEIGHT;
    let map = Map::random_rooms(
        map_width as usize, 
        map_height as usize, 
        10, (5, 10), 
        &mut rng
    );
    let player_position = Position { 
        x: map.rooms[0].center().0, 
        y: map.rooms[0].center().1 
    };
    
    state.world
        .create_entity()
        .with(Player)
        .with(Name { name: "Player".to_string() })
        .with(player_position)
        .with(Renderable {
            glyph: '@',
            color: Color::RGB(0, 0, 255),
        })
        .with(Viewshed { visible_tiles: Vec::new(), range: 10, dirty: true })
        .with(CombatStats {max_hp: 30, hp: 30, defense: 2, power: 5 })
        .build();

    for (idx, room) in map.rooms.iter().skip(1).enumerate() {
        state.world
            .create_entity()
            .with(Name { name: format!("Goblin #{}", idx + 1) })
            .with(Position { x: room.center().0, y: room.center().1 })
            .with(Renderable {
                glyph: 'g',
                color: Color::RGB(255, 0, 0)
            })
            .with(Viewshed { visible_tiles: Vec::new(), range: 8, dirty: true })
            .with(Monster)
            .with(BlocksTile)
            .with(CombatStats{ max_hp: 16, hp: 16, defense: 1, power: 4 })
            .build();
    }

    state.world.insert(map);  
    state.world.insert(player_position);

    let messages = Queue::<String>::new(3);
    state.world.insert(messages);

    let mut event_pump = view.event_pump()?;

    let mut quit = false;
    while !quit {
        state.tick(&mut view);
        
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    quit = true;
                },

                Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                    if try_move_player(-1, 0, &state.world) {
                        state.run_state = State::Running;
                    }
                }

                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                    if try_move_player(1, 0, &state.world) {
                        state.run_state = State::Running;
                    }
                }

                Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                    if try_move_player(0, -1, &state.world) {
                        state.run_state = State::Running;
                    }
                }

                Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                    if try_move_player(0, 1, &state.world) {
                        state.run_state = State::Running;
                    }
                }

                _ => {}
            }
        }
    }

    Ok(())
}
