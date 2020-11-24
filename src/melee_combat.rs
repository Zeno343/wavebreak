use specs::prelude::*;
use super::{CombatStats, MeleeAttack, IncomingDamage};

pub struct MeleeCombat {}

impl<'a> System<'a> for MeleeCombat {
    type SystemData = ( Entities<'a>,
                        WriteStorage<'a, MeleeAttack>,
                        ReadStorage<'a, CombatStats>,
                        WriteStorage<'a, IncomingDamage>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (
            entities, 
            mut melee_attacks,
            combat_stats, 
            mut inflict_damage
        ) = data;

        for (_entity, attack, stats) in 
            (&entities, &melee_attacks, &combat_stats).join() 
        {
            if stats.hp > 0 {
                let target_stats = combat_stats.get(attack.target).unwrap();
                if target_stats.hp > 0 {
                    let damage = stats.power.saturating_sub(target_stats.defense);

                    if damage != 0 {
                        IncomingDamage::add_damage(
                            &mut inflict_damage, 
                            attack.target, 
                            damage
                        );
                    }
                }
            }
        }

        melee_attacks.clear();
    }
}
