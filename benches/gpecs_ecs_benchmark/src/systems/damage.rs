use crate::components::{Damage, Health};

pub fn update_damage(health: &mut Health, damage: &Damage) {
    let total_damage = damage.attack - damage.defense;
    if health.hp > 0 && total_damage > 0 {
        health.hp = Ord::max(health.hp - total_damage, 0);
    }
}
