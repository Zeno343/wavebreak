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

#[derive(Debug)]
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

pub struct Map {
    pub tiles: Vec<TileType>,
    pub width: usize,
    pub height: usize,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Map {
        log(&format!("Created new map with dimensions {}x{}", width, height));
        Map {
            tiles: vec![TileType::Wall; width * height],
            width,
            height,
        }
    }

    pub fn random_noise(width: usize, height: usize, density: f64, rng: &mut StdRng) -> Map {
        let mut map = Map {
            tiles: vec![TileType::Wall; width * height],
            width,
            height,
        };

        for x in 0..width {
            for y in 0..height {
                if x == width - 1 || x == 0 || y == height - 1 || y == 0 {
                    map[(x, y)] = TileType::Wall;
                } else if rng.gen_range(0.0, 1.0) <= density {
                    map[(x,y)] = TileType::Wall;
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
    ) -> (Map, Vec<Rectangle>) {
        let mut map = Map::new(width, height);
        log("Filling map with rooms"); 

        let mut rooms: Vec<Rectangle> = Vec::new();

        for _ in 0..max_rooms {
            let x1 = rng.gen_range(0, width);
            let x2 = x1 + rng.gen_range(min_side_length, max_side_length);

            let y1 = rng.gen_range(0, height);
            let y2 = y1 + rng.gen_range(min_side_length, max_side_length);

            let new_room = Rectangle { x1, x2, y1, y2 };
            
            log(&format!("Generated room: {:?}", new_room));
            let mut valid = true;
            
            if new_room.x2 > width || new_room.y2 >= height { valid = false }

            for room in rooms.iter() {
                if new_room.intersects(room) { valid = false }
            }

            if valid {
                map.add_room(&new_room);
                if !rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = rooms[rooms.len()-1].center();
                    if rng.gen_range(0,2) == 1 {
                        map.add_horizontal_corridor(prev_x, new_x, prev_y);
                        map.add_vertical_corridor(prev_y, new_y, new_x);
                    } else {
                        map.add_vertical_corridor(prev_y, new_y, prev_x);
                        map.add_horizontal_corridor(prev_x, new_x, new_y);
                    }
                }

                rooms.push(new_room);
            }
        }

        (map, rooms)
    }

    fn add_room(&mut self, room: &Rectangle) {
        for x in room.x1 + 1 ..= room.x2 {
            for y in room.y1 + 1 ..= room.y2 {
                self[(x, y)] = TileType::Floor;
            }
        }
    }

    fn add_horizontal_corridor(&mut self, x1: usize, x2: usize, y: usize) {
        if y >= 0 && y <= self.height as usize {
            for x in min(x1,x2) ..= max(x1,x2) {
                if x >= 0 && x <= self.width as usize{
                    self[(x, y)] = TileType::Floor;
                }
            }
        }
    }

    fn add_vertical_corridor(&mut self, y1: usize, y2: usize, x: usize) {
        if x >= 0 && x <= self.width as usize {
            for y in min(y1,y2) ..= max(y1,y2) {
                if y >= 0 && y <= self.height as usize {
                    self[(x, y)] = TileType::Floor;
                }
            }
        }
    }
}

impl Index<(usize, usize)> for Map {
    type Output = TileType;
    
    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.tiles[(x * self.height) + y]
    }
}

impl IndexMut<(usize, usize)> for Map {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        &mut self.tiles[(x * self.height) + y]
    }
}
