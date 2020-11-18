use std::{
    cmp::{
        max,
        min,
    },
    ops::{
        Index,
        IndexMut,
    },
};

use rand::{
    rngs::StdRng,
    Rng,
};

use crate::log;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum TileType {
    Floor,
    Wall,
}

#[derive(Clone, Copy, Debug)]
pub struct Rectangle {
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
}

impl Rectangle {
    pub fn intersects(&self, other: &Rectangle) -> bool {
        self.x1 <= other.x2 && self.x2 >= other.x1 && self.y1 <= other.y2 && self.y2 >= other.y1
    }

    pub fn center(&self) -> (usize, usize) {
        ((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Tile {
    pub tile_type: TileType,
    pub revealed: bool,
    pub visible: bool,
}

pub struct Map {
    pub tiles: Vec<Tile>,
    pub rooms: Vec<Rectangle>,
    pub width: usize,
    pub height: usize,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Map {
        log(&format!("Created new map with dimensions {}x{}", width - 1, height - 1));
        Map {
            tiles: vec![
                Tile { 
                    tile_type: TileType::Wall, 
                    revealed: false, 
                    visible: false 
                }; 
                width * height
            ],
            width: width,
            height: height,
            rooms: Vec::new(),
        }
    }

    pub fn random_noise(width: usize, height: usize, density: f64, rng: &mut StdRng) -> Map {
        let mut map = Map::new(width, height);

        for x in 0 .. map.width {
            for y in 0 .. map.height {
                if x == map.width - 1 || x == 0 || y == map.height - 1|| y == 0 {
                    map[(x, y)].tile_type = TileType::Wall;
                    
                } else if rng.gen_range(0.0, 1.0) <= density {
                    map[(x,y)].tile_type = TileType::Floor;
                }
            }
        }

        map
    }

    pub fn random_rooms(
        width: usize, 
        height: usize, 
        max_rooms: usize, 
        (min_side_length, max_side_length): (usize, usize), 
        rng: &mut StdRng
    ) -> Map {
        let mut map = Map::new(width, height);
        log("Filling map with rooms"); 

        for _ in 0..max_rooms {
            let x1 = rng.gen_range(0, width);
            let x2 = x1 + rng.gen_range(min_side_length, max_side_length);

            let y1 = rng.gen_range(0, height);
            let y2 = y1 + rng.gen_range(min_side_length, max_side_length);

            let new_room = Rectangle { x1, x2, y1, y2 };
            
            log(&format!("Generated room: {:?}", new_room));
            let mut valid = true;
            
            if new_room.x2 >= map.width || new_room.y2 >= map.height { valid = false }

            for room in map.rooms.iter() {
                if new_room.intersects(room) { valid = false }
            }

            if valid {
                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len()-1].center();
                    if rng.gen_range(0,2) == 1 {
                        map.add_horizontal_corridor(prev_x, new_x, prev_y);
                        map.add_vertical_corridor(prev_y, new_y, new_x);
                    } else {
                        map.add_vertical_corridor(prev_y, new_y, prev_x);
                        map.add_horizontal_corridor(prev_x, new_x, new_y);
                    }
                }

                map.add_room(new_room);
            }

        }

        map
    }

    fn add_room(&mut self, room: Rectangle) {
        for x in room.x1 + 1 .. room.x2 {
            for y in room.y1 + 1 .. room.y2 {
                self[(x, y)].tile_type = TileType::Floor;
            }
        }

        self.rooms.push(room);
    }

    fn add_horizontal_corridor(&mut self, x1: usize, x2: usize, y: usize) {
        if y > 0 && y <= self.height as usize {
            for x in min(x1,x2) ..= max(x1,x2) {
                if x > 0 && x <= self.width as usize{
                    self[(x, y)].tile_type = TileType::Floor;
                }
            }
        }
    }

    fn add_vertical_corridor(&mut self, y1: usize, y2: usize, x: usize) {
        if x > 0 && x <= self.width as usize {
            for y in min(y1,y2) ..= max(y1,y2) {
                if y > 0 && y <= self.height as usize {
                    self[(x, y)].tile_type = TileType::Floor;
                }
            }
        }
    }
}

impl Index<(usize, usize)> for Map {
    type Output = Tile;
    
    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.tiles[(x * self.height) + y]
    }
}

impl IndexMut<(usize, usize)> for Map {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        &mut self.tiles[(x * self.height) + y]
    }
}
