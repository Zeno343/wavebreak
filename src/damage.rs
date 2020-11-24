use specs::prelude::*;
use super::{CombatStats, IncomingDamage};

pub struct Damage {}

impl<'a> System<'a> for Damage {
    type SystemData = ( WriteStorage<'a, CombatStats>,
                        WriteStorage<'a, IncomingDamage> );

    fn run(&mut self, data : Self::SystemData) {
        let (mut stats, mut damage) = data;

        for (mut stats, damage) in (&mut stats, &damage).join() {
            stats.hp = stats.hp.saturating_sub(damage.damage.iter().sum::<usize>());
        }

        damage.clear();
    }
}
