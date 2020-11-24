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

use specs::Entity;

use crate::{
    log,
};

use wavebreaker_util::{
    algorithms::{
        fov::FovNode,
        simple_rng::SimpleRng,
    },
    data_structures::Graph,
};

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
        self.x1 <= other.x2 && self.x2 >= other.x1 
            && self.y1 <= other.y2 && self.y2 >= other.y1
    }

    pub fn center(&self) -> (usize, usize) {
        ((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }
}

#[derive(Clone, Debug)]
pub struct Tile {
    pub tile_type: TileType,
    pub revealed: bool,
    pub visible: bool,
    pub blocked: bool,
    pub entities: Vec<Entity>
}

impl FovNode for Tile {
    fn blocks_view(&self) -> bool {
        self.tile_type == TileType::Wall
    }
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
                    visible: false,
                    blocked: false,
                    entities: Vec::new(),
                }; 
                width * height
            ],
            width: width,
            height: height,
            rooms: Vec::new(),
        }
    }

    pub fn random_rooms(
        width: usize, 
        height: usize, 
        max_rooms: usize, 
        (min_side_length, max_side_length): (usize, usize), 
        rng: &mut SimpleRng
    ) -> Map {
        let mut map = Map::new(width, height);
        log("Filling map with rooms"); 

        for _ in 0..max_rooms {
            let x1 = rng.roll(0, width);
            let x2 = x1 + rng.roll(min_side_length, max_side_length);

            let y1 = rng.roll(0, height);
            let y2 = y1 + rng.roll(min_side_length, max_side_length);

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
                    if rng.roll(0,2) == 1 {
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

    pub fn populate_blocked(&mut self)  {
        for mut tile in self.tiles.iter_mut() {
            if tile.tile_type == TileType::Wall {
                tile.blocked = true;
            } else {
                tile.blocked = false;
            }
        }
    }
    
    pub fn clear_entities(&mut self) {
        for mut tile in self.tiles.iter_mut() {
            tile.entities.clear();
        }
    }

}

impl Graph<(usize, usize), Tile> for Map {
    fn neighbors(&self, tile: &(usize, usize)) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::new();
        for x in -1..=1 {
            for y in -1..=1 {
                let potential_neighbor = (
                    (tile.0 as i64 + x) as usize, 
                    (tile.1 as i64 + y) as usize
                );
                if !self[potential_neighbor].blocked {
                    neighbors.push(potential_neighbor);
                }
            }
        }

        neighbors
    }

    fn cost(&self, start: &(usize, usize), end: &(usize, usize)) -> f64 {
        let x_distance;
        let y_distance;
        
        if start.0 > end.0 {
            x_distance = start.0 - end.0
        } else {
            x_distance = end.0 - start.0
        }

        if start.1 > end.1 {
            y_distance = start.1 - end.1
        } else {
            y_distance = end.1 - start.1
        }
        
        (x_distance as f64 + y_distance as f64).sqrt()
    }

    fn contains(&self, point: &(usize, usize)) -> bool {
        point.0 < self.width && point.1 < self.height
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
