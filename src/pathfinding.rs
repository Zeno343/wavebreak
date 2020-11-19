use std::collections::HashMap;

use crate::{
    map::Map,
    util::Queue,
};

pub fn find_path(
    start: (usize, usize), 
    end: (usize, usize), 
    map: &Map
) -> Vec<(usize, usize)> {
    let mut frontier = Queue::new(usize::MAX);
    let mut reached = HashMap::new();
    
    frontier.push(start);
    reached.insert(start, None);

    while frontier.len() > 0 {
        let current_tile = frontier.pop();
        if current_tile == end {
            break;
        }

        for next_tile in map.neighbors(current_tile).into_iter() {
            if !reached.contains_key(&next_tile) {
                reached.insert(next_tile, Some(current_tile));
                frontier.push(next_tile);
            }
        }
    }

    let mut current_tile = end;
    let mut path = Vec::new();

    while current_tile != start {
        path.push(current_tile);

        current_tile = reached.get(&current_tile).unwrap().unwrap();
    }

    path
}
