use std::{
    convert::TryInto,
    fs::OpenOptions,
    io::{
        Write,
    },
    panic,
};

use crossterm::{
    event,
    style,
};

use rand::{
    rngs::StdRng,
    SeedableRng,
    thread_rng,
};

use specs::prelude::*;

mod components;
use components::*;

mod fov;

mod map;
use map::{
    Map,
    TileType,
};

mod monster_ai;

mod state;
use state::{
    RunState,
    State,
};

mod util;
use util::Queue;

mod view;
use view::View;

const LOG_FILE: &str = "log";

fn log(message: &str) {
    let mut log = OpenOptions::new().append(true).create(true).open(LOG_FILE).expect("Could not open log file");
    log.write_all(message.as_bytes()).expect("could not write to log file");
    log.write(&['\n' as u8]).expect("could not write to log file");
}



    
fn try_move_player(d_x: i16, d_y: i16, world: &World) {
    let mut positions = world.write_storage::<Position>();
    let players = world.write_storage::<Player>();
    let mut viewsheds = world.write_storage::<Viewshed>();

    let map = world.fetch::<Map>();
    
    for (_, pos, viewshed) in (&players, &mut positions, &mut viewsheds).join() {
        let dest_x: Option<usize> = (pos.x as i16 + d_x).try_into().ok();
        let dest_y: Option<usize> = (pos.y as i16 + d_y).try_into().ok();
        
        log(&format!("Player attempting move from {},{} to {},{}", pos.x, pos.y, dest_x.unwrap(), dest_y.unwrap()));
        if dest_x.is_some() || dest_y.is_some() {
            let dest_x = dest_x.unwrap();
            let dest_y = dest_y.unwrap();

            if dest_x < map.width && dest_y < map.height {
                log(&format!("Player landed on {:?}", map[(dest_x, dest_y)]));
                if map[(dest_x, dest_y)].tile_type != TileType::Wall {
                    //set player's position component
                    pos.x = dest_x;
                    pos.y = dest_y;

                    //update player position resource
                    let mut player_pos = world.write_resource::<Position>();
                    player_pos.x = pos.x;
                    player_pos.y = pos.y;

                    viewshed.dirty = true;
                }

            }
        }
    }
}

fn main() -> crossterm::Result<()> {
    let _ = OpenOptions::new().write(true).truncate(true).open(LOG_FILE).expect("Could not open log file");

    panic::set_hook(Box::new(|panic_info| {
        let mut log = OpenOptions::new().append(true).create(true).open(LOG_FILE).expect("Could not open log file");
        log.write_all(format!("panic occurred: {:?}", panic_info).as_bytes()).expect("Error writing to log file");
    }));

    let mut view = View::init().expect("Could not initialize view"); 

    let mut state = State { 
        world: World::new(),
        run_state: RunState::Running,
    };
    
    let mut rng = StdRng::from_rng(thread_rng()).expect("could not seed rng");

    state.world.register::<Player>();
    state.world.register::<Position>();
    state.world.register::<Renderable>();
    state.world.register::<Viewshed>();
    state.world.register::<Monster>();
    state.world.register::<Name>();
    
    let map = Map::random_rooms(view.width as usize, view.height as usize - 3, 10, (5, 10), &mut rng);
    let player_position = Position { x: map.rooms[0].center().0, y: map.rooms[0].center().1 };
    
    state.world
        .create_entity()
        .with(Player)
        .with(Name { name: "Player".to_string() })
        .with(player_position)
        .with(Renderable {
            glyph: '@',
            foreground: style::Color::Rgb{ r: 0, b: 0, g: 255 },
            background: style::Color::Black,
        })
        .with(Viewshed { visible_tiles: Vec::new(), range: 10, dirty: true })
        .build();

    for (idx, room) in map.rooms.iter().skip(1).enumerate() {
        state.world
            .create_entity()
            .with(Name { name: format!("Goblin #{}", idx + 1) })
            .with(Position { x: room.center().0, y: room.center().1 })
            .with(Renderable {
                glyph: 'g',
                foreground: style::Color::Rgb{ r: 255, b: 0, g: 0 },
                background: style::Color::Black,
            })
            .with(Viewshed { visible_tiles: Vec::new(), range: 8, dirty: true })
            .with(Monster)
            .build();
    }

    state.world.insert(map);  
    state.world.insert(player_position);

    let messages = Queue::<String>::new(3);
    state.world.insert(messages);

    loop {
        state.tick(&mut view);

        if event::poll(std::time::Duration::from_millis(30))? {
            match event::read()? {
                event::Event::Key(event::KeyEvent { code, .. }) => 
                    match code {
                        event::KeyCode::Esc => break,

                        event::KeyCode::Left => { try_move_player(-1, 0, &state.world) },
                        event::KeyCode::Right => { try_move_player(1, 0, &state.world) },
                        event::KeyCode::Up => { try_move_player(0, -1, &state.world) },
                        event::KeyCode::Down => { try_move_player(0, 1, &state.world) },
                        
                        _ => {},
                    }

                    
                _ => {}
            }

            state.run_state = RunState::Running;
        } else {
            state.run_state = RunState::Paused;
        }
    }

    Ok(())
}
