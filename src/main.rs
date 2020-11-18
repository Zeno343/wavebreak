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

use specs::prelude::*;
use specs_derive::Component;

mod fov;
use fov::compute_fov;

mod map;
use map::{
    Map,
    TileType,
};

mod util;

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
            .queue(style::PrintStyledContent(
                    style::style(renderable.glyph)
                    .with(renderable.foreground)
                    .on(renderable.background)
                )
            )?;

        Ok(())
    }

    pub fn draw_map(&mut self, map: &Map, world: &World) -> crossterm::Result<()> {
        for (idx, tile) in map.tiles.iter().enumerate() {
            let x = idx / map.height;
            let y = idx % map.height;
             
            if map[(x, y)].visible {
                self.stdout.queue(cursor::MoveTo(x as u16, y as u16))?;
                
                if tile.tile_type == TileType::Wall {
                    self.stdout.queue(
                        style::PrintStyledContent(
                            style::style('\u{2592}')
                            .with(style::Color::White)
                            .on(style::Color::Black)
                        )
                    )?;

                } else {
                    self.stdout.queue(
                        style::PrintStyledContent(
                            style::style('.')
                            .with(style::Color::White)
                            .on(style::Color::Black)
                        )
                    )?;
                }
            } else if map[(x, y)].revealed {
                self.stdout.queue(cursor::MoveTo(x as u16, y as u16))?;
                
                if tile.tile_type == TileType::Wall {
                    self.stdout.queue(
                        style::PrintStyledContent(
                            style::style('\u{2592}')
                            .with(style::Color::Rgb{r: 128, g: 128, b: 128})
                            .on(style::Color::Black)
                        )
                    )?;
                } else {
                    self.stdout.queue(
                        style::PrintStyledContent(
                            style::style('.')
                            .with(style::Color::Rgb{r: 128, g: 128, b: 128})
                            .on(style::Color::Black)
                        )
                    )?;
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

    pub fn end_frame(&mut self) -> crossterm::Result<()> {
        self.stdout.flush();

        Ok(())
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
    foreground: style::Color,
    background: style::Color,
}

pub struct State {
    ecs: World,
}

#[derive(Component)]
pub struct Viewshed {
    visible_tiles: Vec<(usize, usize)>,
    range: usize,
    dirty: bool
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
                    pos.x = dest_x;
                    pos.y = dest_y;

                    viewshed.dirty = true;
                }

            }
        }
    }
}

fn tick(state: &State, view: &mut View) {
    reveal_map(&state.ecs);

    let positions = state.ecs.read_storage::<Position>();
    let renderables = state.ecs.read_storage::<Renderable>();
    let viewsheds = state.ecs.read_storage::<Viewshed>();
    let player = state.ecs.read_storage::<Player>();

    let map = state.ecs.fetch::<Map>(); 

    view.begin_frame();
    view.draw_map(&map, &state.ecs);
    for (pos, render) in (&positions, &renderables).join() {
        if map[(pos.x, pos.y)].visible {
            view.draw_entity(pos, render);
        }
    }
    view.end_frame();
}

fn main() -> crossterm::Result<()> {
    let _ = OpenOptions::new().write(true).truncate(true).open(LOG_FILE).expect("Could not open log file");

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
    let map = Map::random_rooms(dimensions.0 as usize, dimensions.1 as usize, 10, (5, 10), &mut rng);
    state.ecs
        .create_entity()
        .with(Player)
        .with(Position { x: map.rooms[0].center().0, y: map.rooms[0].center().1 })
        .with(Renderable {
            glyph: '@',
            foreground: style::Color::Rgb{ r: 0, b: 0, g: 255 },
            background: style::Color::Black,
        })
        .with(Viewshed { visible_tiles: Vec::new(), range: 10, dirty: true })
        .build();

    for room in map.rooms.iter().skip(1) {
        state.ecs
            .create_entity()
            .with(Position { x: room.center().0, y: room.center().1 })
            .with(Renderable {
                glyph: 'g',
                foreground: style::Color::Rgb{ r: 255, b: 0, g: 0 },
                background: style::Color::Black,
            })
            .with(Viewshed { visible_tiles: Vec::new(), range: 8, dirty: true })
            .build();
    }

    state.ecs.insert(map);  

    loop {
        tick(&state, &mut view);

        if event::poll(std::time::Duration::from_millis(30))? {
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
    }

    Ok(())
}
