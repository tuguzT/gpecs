use crate::components::{Health, StatusEffect};

pub fn update_health(health: &mut Health) {
    if health.hp <= 0 && health.status != StatusEffect::Dead {
        health.hp = 0;
        health.status = StatusEffect::Dead;
    } else if health.status == StatusEffect::Dead && health.hp == 0 {
        health.hp = health.max_hp;
        health.status = StatusEffect::Spawn;
    } else if health.hp >= health.max_hp && health.status != StatusEffect::Alive {
        health.hp = health.max_hp;
        health.status = StatusEffect::Alive;
    } else {
        health.status = StatusEffect::Alive;
    }
}
