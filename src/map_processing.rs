use specs::prelude::*;
use super::{Map, Position, BlocksTile};

pub struct MapProcessing { }

impl<'a> System<'a> for MapProcessing {
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, BlocksTile>,
                        Entities<'a>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, position, blockers, entities) = data;

        map.populate_blocked();
        map.clear_entities();
        for (entity, position) in (&entities, &position).join() {
            if let Some(_) = blockers.get(entity) {
                map[(position.x, position.y)].blocked = true;
            }

            map[(position.x, position.y)].entities.push(entity);  
        }
    }
}
