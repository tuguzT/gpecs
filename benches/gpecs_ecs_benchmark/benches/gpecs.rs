use std::{cell::RefCell, fs::File, io::Write, rc::Rc};

use gpecs::prelude::*;
use gpecs_ecs_benchmark::{
    components::{Damage, Data, Health, Player, PlayerType, Position, Sprite, Velocity},
    framebuffer::Framebuffer,
    systems::{
        render_sprite, update_components, update_damage, update_data, update_health,
        update_position, update_sprite, NONE_SPRITE, SPAWN_SPRITE,
    },
    utils::{RandomXoshiro128, TimeDelta},
};

const FRAMEBUFFER_WIDTH: usize = 320;
const FRAMEBUFFER_HEIGHT: usize = 240;
const FRAMEBUFFER_SIZE: usize = FRAMEBUFFER_WIDTH * FRAMEBUFFER_HEIGHT;
const SPAWN_AREA_MARGIN: u32 = 100;

fn main() {
    let mut rng = rand::rng();
    let mut context = Context::new();

    for _ in 0..100 {
        create_player(&mut context, &mut rng, None);
    }

    let time_delta = TimeDelta(0.0);
    let framebuffer = Framebuffer::new(
        FRAMEBUFFER_WIDTH as u32,
        FRAMEBUFFER_HEIGHT as u32,
        vec![NONE_SPRITE; FRAMEBUFFER_SIZE],
    );

    let mut executor = CpuExecutor::new(&mut context);

    let time_delta = Rc::new(RefCell::new(time_delta));
    let framebuffer = Rc::new(RefCell::new(framebuffer));
    register_systems(&mut executor, time_delta, framebuffer.clone());

    executor.execute();

    save_framebuffer_to_file(&*framebuffer.borrow());
}

fn register_systems<B>(
    executor: &mut CpuExecutor,
    time_delta: Rc<RefCell<TimeDelta>>,
    framebuffer: Rc<RefCell<Framebuffer<B>>>,
) where
    B: AsMut<[u32]> + 'static,
{
    let time_delta_clone = Rc::clone(&time_delta);
    let system = executor.register_system(move |bundles: BundlesMut<(Position, Velocity)>| {
        for (_, (position, velocity)) in bundles {
            update_position(position, velocity, *time_delta_clone.borrow());
        }
    });
    executor.add_system(system);

    let system = executor.register_system(move |bundles: BundlesMut<(Data,)>| {
        for (_, (data,)) in bundles {
            update_data(data, *time_delta.clone().borrow());
        }
    });
    executor.add_system(system);

    let system = executor.register_system(|bundles: BundlesMut<(Position, Velocity, Data)>| {
        for (_, (position, velocity, data)) in bundles {
            update_components(position, velocity, data);
        }
    });
    executor.add_system(system);

    let system = executor.register_system(|bundles: BundlesMut<(Health,)>| {
        for (_, (health,)) in bundles {
            update_health(health);
        }
    });
    executor.add_system(system);

    let system = executor.register_system(|bundles: BundlesMut<(Health, Damage)>| {
        for (_, (health, damage)) in bundles {
            update_damage(health, damage);
        }
    });
    executor.add_system(system);

    let system = executor.register_system(|bundles: BundlesMut<(Sprite, Player, Health)>| {
        for (_, (sprite, player, health)) in bundles {
            update_sprite(sprite, player, health);
        }
    });
    executor.add_system(system);

    let system = executor.register_system(move |bundles: BundlesMut<(Position, Sprite)>| {
        for (_, (position, sprite)) in bundles {
            render_sprite(position, sprite, &mut *framebuffer.borrow_mut());
        }
    });
    executor.add_system(system);
}

fn create_player(context: &mut Context, rng: &mut impl rand::Rng, player_type: Option<PlayerType>) {
    let entity = context.spawn();

    let mut rng = RandomXoshiro128::new(rng.next_u32());
    let r#type = player_type.unwrap_or_else(|| {
        let rate = rng.range(1..100);
        match rate {
            ..=3 => PlayerType::NPC,
            ..=30 => PlayerType::Hero,
            _ => PlayerType::Monster,
        }
    });
    let mut player = Player { rng, r#type };

    let health = Health {
        hp: 0,
        max_hp: match player.r#type {
            PlayerType::Hero => player.rng.range(5..15) as i32,
            PlayerType::Monster => player.rng.range(4..12) as i32,
            PlayerType::NPC => player.rng.range(6..12) as i32,
        },
        status: Default::default(),
    };

    let damage = Damage {
        attack: match player.r#type {
            PlayerType::Hero => player.rng.range(4..10) as i32,
            PlayerType::Monster => player.rng.range(3..9) as i32,
            PlayerType::NPC => 0,
        },
        defense: match player.r#type {
            PlayerType::Hero => player.rng.range(2..6) as i32,
            PlayerType::Monster => player.rng.range(2..8) as i32,
            PlayerType::NPC => player.rng.range(3..8) as i32,
        },
    };

    let sprite = Sprite {
        character: SPAWN_SPRITE,
    };

    let position = Position {
        x: player
            .rng
            .range(0..FRAMEBUFFER_WIDTH as u32 + SPAWN_AREA_MARGIN) as f32
            - SPAWN_AREA_MARGIN as f32,
        y: player
            .rng
            .range(0..FRAMEBUFFER_HEIGHT as u32 + SPAWN_AREA_MARGIN) as f32
            - SPAWN_AREA_MARGIN as f32,
    };

    // let velocity = Velocity::default();

    let components = (player, health, damage, sprite, position);
    context
        .insert_bundle(entity, components)
        .expect("entity should present & archetype should be valid");
}

fn save_framebuffer_to_file<B>(framebuffer: &Framebuffer<B>)
where
    B: AsRef<[u32]>,
{
    let mut framebuffer_file =
        File::create("framebuffer.txt").expect("failed to create framebuffer file");
    for column in framebuffer.buffer().chunks_exact(FRAMEBUFFER_WIDTH) {
        for &pixel in column {
            let pixel = char::from_u32(pixel).expect("failed to convert pixel to char");
            framebuffer_file
                .write_all(&[pixel as u8])
                .expect("failed to write pixel to file");
        }
        framebuffer_file
            .write_all(b"\n")
            .expect("failed to write newline to file");
    }
}
