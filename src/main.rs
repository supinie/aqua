use bevy::{math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume}, prelude::*};

const FLOOR_COLOUR: Color = Color::rgb(0.8, 0.8, 0.8);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, setup)
        .add_systems(Update, (animate_sprite, (movement_input, apply_velocity, check_for_collisions).chain()))
        .run();
}

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component)]
struct Anguilla;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Collider;

#[derive(Event, Default)]
struct CollisionEvent;

#[derive(PartialEq, Eq, Copy, Clone)]
enum Collision {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Bundle)]
struct TerrainBundle {
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

enum TerrainType {
    Floor,
}

impl TerrainType {
    fn position(&self) -> Vec2 {
        let bottom = -300.0;
        match self {
            TerrainType::Floor => Vec2::new(0.0, bottom),
        }
    }

    fn size(&self) -> Vec2 {
        let width = 900.0;
        let thickness = 10.0;

        match self {
            TerrainType::Floor => Vec2::new(width + thickness, thickness),
        }
    }
}

impl TerrainBundle {
    fn new(terrain_type: TerrainType) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: terrain_type.position().extend(0.0),
                    scale: terrain_type.size().extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: FLOOR_COLOUR,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
        }
    }
}
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // camera
    commands.spawn(Camera2dBundle::default());

    // terrain
    commands.spawn(TerrainBundle::new(TerrainType::Floor));

    // anguilla
    let texture = asset_server.load("Anguilla/Anguilla_Neutral.png");
    let layout = TextureAtlasLayout::from_grid(Vec2::new(48.0, 32.0), 1, 2, None, None);
    // let texture = asset_server.load("Anguilla/Anguilla_Run.png");
    // let layout = TextureAtlasLayout::from_grid(Vec2::new(48.0, 32.0), 1, 8, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    // let animation_indices = AnimationIndices { first: 1, last: 7 };
    let animation_indices = AnimationIndices { first: 1, last: 2 };
    commands.spawn((
        SpriteSheetBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: Vec3::new(2.0, 2.0, 1.0),
                ..default()
            },
            texture,
            atlas: TextureAtlas {
                layout: texture_atlas_layout,
                index: animation_indices.first,
            },
            ..default()
        },
        Anguilla,
        Velocity(Vec2::new(0.0, 0.0)),
        Collider,
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
}

fn movement_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Velocity, With<Anguilla>>,
    time: Res<Time>,
) {
    let mut velocity = query.single_mut();
    let mut velocity_transform = Vec2::new(0.0, 0.0);

    if keyboard_input.pressed(KeyCode::KeyA) {
        velocity_transform.x -= 600.0 * time.delta_seconds();
    }

    if keyboard_input.pressed(KeyCode::KeyD) {
        velocity_transform.x += 600.0 * time.delta_seconds();
    } 

    velocity.0 = (velocity.0 + velocity_transform).clamp(Vec2::new(-1000.0, -10000.0), Vec2::new(1000.0, 10000.0));
}


fn apply_velocity(
    mut query: Query<(&mut Transform, &Velocity)>,
    time: Res<Time>,
) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.0.x * time.delta_seconds();
        transform.translation.y += velocity.0.y * time.delta_seconds();
    }
}

fn check_for_collisions(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut anguilla_query: Query<(&mut Velocity, &Transform), With<Anguilla>>,
    collider_query: Query<&Transform, With<Collider>>,
    // mut collision_events: EventWriter<CollisionEvent>,
    time: Res<Time>,
) {
    let (mut anguilla_velocity, anguilla_transform) = anguilla_query.single_mut();

    for collider_transform in &collider_query {
        let collision = anguilla_collision(
            Aabb2d::new(anguilla_transform.translation.truncate(), Vec2::new(32.0, 32.0)),
            Aabb2d::new(
                collider_transform.translation.truncate(),
                collider_transform.scale.truncate() / 2.0,
            ),
        );

        if let Some(collision) = collision {
            // collision_events.send_default();

            match collision {
                Collision::Top => {
                    anguilla_velocity.0.y = 0.0;

                    if keyboard_input.pressed(KeyCode::KeyW) {
                        anguilla_velocity.0.y += 300.0;
                    }

                    if !keyboard_input.pressed(KeyCode::KeyA) && anguilla_velocity.0.x < 0.0 {
                        anguilla_velocity.0.x = (anguilla_velocity.0.x + 800.0 * time.delta_seconds()).clamp(-500.0, 0.0);
                    }
                    if !keyboard_input.pressed(KeyCode::KeyD) && anguilla_velocity.0.x > 0.0 {
                        anguilla_velocity.0.x = (anguilla_velocity.0.x - 800.0 * time.delta_seconds()).clamp(0.0, 500.0);
                    }
                },
                _ => {},
            }
        } else {
            anguilla_velocity.0.y -= 300.0 * time.delta_seconds();
        }
    }
}

fn anguilla_collision(player: Aabb2d, bounding_box: Aabb2d) -> Option<Collision> {
    if !player.intersects(&bounding_box) {
        return None;
    }

    let closest = bounding_box.closest_point(player.center());
    let offset = player.center() - closest;
    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0.0 {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y > 0.0 {
        Collision::Top
    } else {
        Collision::Bottom
    };

    Some(side)
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut TextureAtlas)>,
) {
    for (indices, mut timer, mut atlas) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            atlas.index = if atlas.index == indices.last {
                indices.first
            } else {
                atlas.index + 1
            };
        }
    }
}
