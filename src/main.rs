use std::{
    convert::TryInto,
    fs::{
        File,
        OpenOptions,
    },
    io::{
        stdout,
        Stdout,
        Write,
    },
    panic,
};

use crossterm::{
    cursor,
    event,
    ExecutableCommand,
    queue,
    QueueableCommand,
    style,
    terminal,
    terminal::ClearType,
};

use rand::{
    rngs::StdRng,
    SeedableRng,
    thread_rng,
};

mod fov;
use fov::compute_fov;

mod map;
use map::{
    Map,
    TileType,
};

use specs::prelude::*;
use specs_derive::Component;

const LOG_FILE: &str = "log";

fn log(message: &str) {
    let mut log = OpenOptions::new().append(true).create(true).open(LOG_FILE).expect("Could not open log file");
    log.write_all(message.as_bytes()).expect("could not write to log file");
    log.write(&['\n' as u8]).expect("could not write to log file");
}

pub struct View {
    stdout: Stdout,
    width: u16,
    height: u16,
}

impl View {
    pub fn init() -> crossterm::Result<View> {
        let (width, height) = terminal::size()?;

        let mut stdout = stdout();
        terminal::enable_raw_mode();
        stdout
            .execute(cursor::Hide)?
            .execute(event::EnableMouseCapture)?;

        Ok(View { 
            stdout,
            width,
            height,
        })
    }

    pub fn draw_entity(&mut self, position: &Position, renderable: &Renderable) -> crossterm::Result<()> {
        self.stdout
            .queue(cursor::MoveTo(position.x as u16, position.y as u16))?
            .queue(style::Print(renderable.glyph))?;

        Ok(())
    }

    pub fn draw_map(&mut self, map: &Map, world: &World) -> crossterm::Result<()> {
        let mut viewsheds = world.write_storage::<Viewshed>();
        let players = world.read_storage::<Player>();
        let positions = world.read_storage::<Position>();
        
        let (_, viewshed, position) = (&players, &mut viewsheds, &positions).join().nth(0).expect("No viewshed for player");

        viewshed.visible_tiles = compute_fov((position.x, position.y), map);

        for (idx, tile) in map.tiles.iter().enumerate() {
            let x = idx / map.height;
            let y = idx % map.height;
             
            if viewshed.contains(&(x, y)) {
                self.stdout.queue(cursor::MoveTo(x as u16, y as u16))?;

                if *tile == TileType::Wall {
                    self.stdout.queue(style::Print('\u{2592}'))?;
                } else {
                    self.stdout.queue(style::Print('.'))?;
                }
            }
        }

        Ok(())
    }

    pub fn begin_frame(&mut self) -> crossterm::Result<()> {
        self.stdout
            .execute(terminal::Clear(ClearType::All))?;

        Ok(())
    }

    pub fn end_frame(&mut self) {
        self.stdout.flush();
    }
}

impl Drop for View {
    fn drop(&mut self) {
        self.stdout
            .execute(terminal::Clear(ClearType::All)).unwrap()
            .execute(cursor::Show).unwrap()
            .execute(cursor::MoveTo(0,0)).unwrap()
            .execute(event::DisableMouseCapture).unwrap();

        terminal::disable_raw_mode();
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Position {
    x: usize,
    y: usize,
}

#[derive(Component)]
pub struct Renderable {
    glyph: char,
}

pub struct State {
    ecs: World,
}

#[derive(Component)]
pub struct Viewshed {
    visible_tiles: Vec<(usize, usize)>
}

impl Viewshed {
    pub fn contains(&self, tile: &(usize, usize)) -> bool {
        self.visible_tiles.contains(tile)
    }
}
        
fn try_move_player(d_x: i16, d_y: i16, world: &World) {
    let mut positions = world.write_storage::<Position>();
    let mut players = world.write_storage::<Player>();

    let map = world.fetch::<Map>();

    for (_, pos) in (&mut players, &mut positions).join() {
        let dest_x: Option<usize> = (pos.x as i16 + d_x).try_into().ok();
        let dest_y: Option<usize> = (pos.y as i16 + d_y).try_into().ok();
        
        log(&format!("Player attempting move from {},{} to {},{}", pos.x, pos.y, dest_x.unwrap(), dest_y.unwrap()));
        if dest_x.is_some() || dest_y.is_some() {
            let dest_x = dest_x.unwrap();
            let dest_y = dest_y.unwrap();

            if dest_x < map.width && dest_y < map.height {
                log(&format!("Player landed on {:?}", map[(dest_x, dest_y)]));
                if map[(dest_x, dest_y)] != TileType::Wall {
                    pos.x = dest_x;
                    pos.y = dest_y;
                }

            }
        }
    }
}

fn tick(state: &State, view: &mut View) {
    let positions = state.ecs.read_storage::<Position>();
    let renderables = state.ecs.read_storage::<Renderable>();

    let map = state.ecs.fetch::<Map>(); 

    view.begin_frame();
    view.draw_map(&map, &state.ecs);
    for (pos, render) in (&positions, &renderables).join() {
        view.draw_entity(pos, render);
    }
    view.end_frame();
}

fn main() -> crossterm::Result<()> {
    {
        let log = OpenOptions::new().write(true).truncate(true).open(LOG_FILE).expect("Could not open log file");
    }

    panic::set_hook(Box::new(|panic_info| {
        let mut log = OpenOptions::new().append(true).create(true).open(LOG_FILE).expect("Could not open log file");
        log.write_all(format!("panic occurred: {:?}", panic_info).as_bytes());
    }));

    let mut view = View::init().expect("Could not initialize view"); 

    let mut state = State { ecs: World::new() };
    
    let mut rng = StdRng::from_rng(thread_rng()).expect("could not seed rng");

    state.ecs.register::<Player>();
    state.ecs.register::<Position>();
    state.ecs.register::<Renderable>();
    state.ecs.register::<Viewshed>();
    
    let dimensions = terminal::size()?;
    let (map, rooms) = Map::random_rooms(dimensions.0 as usize, dimensions.1 as usize, 10, (5, 10), &mut rng);

    state.ecs.insert(map);  
    state.ecs
        .create_entity()
        .with(Player)
        .with(Position { x: rooms[0].center().0, y: rooms[0].center().1 })
        .with(Renderable {
            glyph: '@',
        })
        .with(Viewshed { visible_tiles: Vec::new() })
        .build();

    loop {
        tick(&state, &mut view);

        match event::read()? {
            event::Event::Key(event::KeyEvent { code, .. }) => 
                match code {
                    event::KeyCode::Esc => break,

                    event::KeyCode::Left => { try_move_player(-1, 0, &state.ecs) },
                    event::KeyCode::Right => { try_move_player(1, 0, &state.ecs) },
                    event::KeyCode::Up => { try_move_player(0, -1, &state.ecs) },
                    event::KeyCode::Down => { try_move_player(0, 1, &state.ecs) },
                    
                    _ => {},
                }

            _ => {}
        }
    }

    Ok(())
}
