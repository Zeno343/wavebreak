use specs::prelude::*;
use super::{Map, Position, BlocksTile};

pub struct MapProcessing { }

impl<'a> System<'a> for MapProcessing {
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, BlocksTile>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, position, blockers) = data;

        map.populate_blocked();
        for (position, _blocks) in (&position, &blockers).join() {
            map[(position.x, position.y)].blocked = true;
        }
    }
}
