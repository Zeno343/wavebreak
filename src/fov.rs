use std::ops::Mul;

use crate::{
    map::{
        Map,
        TileType,
    },
    log,
};

pub fn compute_fov((col, row): (usize, usize), map: &Map) -> Vec<(usize, usize)> {
    static DIRECTIONS: [Direction; 4] = [Direction::North, Direction::East, Direction::South, Direction::West];
    let mut revealed_tiles = Vec::new();

    for &direction in DIRECTIONS.iter() {
        let scanner = Scanner { 
            direction, 
            origin: (col, row),
            start_slope: -1.0,
            end_slope: 1.0,
        };

        revealed_tiles.extend(scanner.scan(map));
    }

    revealed_tiles
}


fn slope((o_col, o_row): (usize, usize), (col, row): (usize, usize)) -> f64 {
    (o_row as f64 - row as f64) / (o_col as f64 - col as f64)
}

#[derive(Copy, Clone, Debug)]
enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Debug)]
struct Scanner {
    direction: Direction,
    origin: (usize, usize),
    start_slope: f64,
    end_slope: f64,
}

impl Scanner {
    fn scan(&self, map: &Map) -> Vec<(usize, usize)> {
        log(&format!("Scanning with parameters: {:?}", self));

        let mut revealed_tiles = Vec::new();
        let mut previous_tile: Option<(usize, usize)> = None;
        
        match self.direction {
            North => {
                for tile in self.row(self.origin.1 + 1) {
                    if tile.1 > map.width || tile.0 > map.height { 
                        log(&format!("Invalid tile at {:?}", tile));
                        continue; 
                    }

                    if map[tile] == TileType::Wall || map[tile] == TileType::Floor {
                        log(&format!("{:?}: {:?}", tile, map[tile]));
                        revealed_tiles.push(tile);
                    }

                    if let Some(p_tile) = previous_tile {
                        log(&format!("Examining previous tile: {:?}", p_tile));

                        if map[p_tile] == TileType::Wall && map[tile] == TileType::Floor {
                            let scanner = Scanner { 
                                direction: self.direction, 
                                origin: (tile.0, tile.1 + 1),
                                start_slope: slope(self.origin, tile),
                                end_slope: self.start_slope,
                            };

                            revealed_tiles.extend(scanner.scan(map));
                        } else if map[p_tile] == TileType::Floor && map[tile] == TileType::Wall {
                            let scanner = Scanner { 
                                direction: self.direction, 
                                origin: (tile.0, tile.1 + 1),
                                start_slope: self.start_slope,
                                end_slope: slope(self.origin, tile)
                            };

                            revealed_tiles.extend(scanner.scan(map));
                        } else {
                            log("No need to recurse");
                        }
                    } else {
                        log("No previous tile to examine");
                    }

                    previous_tile = Some(tile);
                }

                if let Some(p_tile) = previous_tile {
                    log(&format!("Last tile in previous row: {:?}", p_tile));

                    if map[p_tile] == TileType::Floor {
                        let scanner = Scanner { 
                            direction: self.direction, 
                            origin: (self.origin.0, self.origin.1 + 1),
                            start_slope: self.start_slope,
                            end_slope: self.end_slope,
                        };
                        scanner.scan(map); 
                        revealed_tiles.extend(scanner.scan(map));
                    }
                } else {
                    log("No tile left from previous row");
                }
            }
        }

        revealed_tiles
    }

    fn row(&self, row: usize) -> Vec<(usize, usize)> {
        let depth = row - self.origin.1;
        let min_col = self.origin.0 - f64::floor((depth as f64 * self.start_slope)) as usize - 1;
        let max_col = self.origin.0 + f64::ceil((depth as f64 * self.end_slope)) as usize;

        log(&format!("Sweeping row {} from {} to {}", row, min_col, max_col));

        (min_col ..= max_col)
            .map(|col| (row, col))
            .collect()
    }

    fn col(&self, col: usize) -> Vec<(usize, usize)> {
        let depth = col - self.origin.0;
        let min_row = self.origin.1 - f64::floor((depth as f64 * self.start_slope)) as usize - 1;
        let max_row = self.origin.1 + f64::ceil((depth as f64 * self.end_slope)) as usize;

        log(&format!("Sweeping col {} from {} to {}", col, min_row, max_row));

        (min_row ..= max_row)
            .map(|row| (row, col))
            .collect()
    }
}
