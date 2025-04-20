use crate::components::{Damage, Health};

pub fn update_damage(health: &mut Health, damage: &Damage) {
    let total_damage = damage.attack - damage.defense;
    if health.hp > 0 && total_damage > 0 {
        health.hp = max(health.hp - total_damage, 0);
    }
}

// This exists because `rust_gpu` can't handle `Ord::max` somehow
fn max(a: i32, b: i32) -> i32 {
    if b < a {
        a
    } else {
        b
    }
}
