use bevy::{math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume}, prelude::*};

const FLOOR_COLOUR: Color = Color::rgb(0.8, 0.8, 0.8);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .init_state::<AnguillaState>()
        .init_state::<AnguillaDirection>()
        .add_event::<CollisionEvent>()
        .add_event::<ProjectileEvent>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            animate_sprite, 
            count_cooldown,
            (
                movement_input, 
                apply_velocity, 
                check_for_collisions, 
                update_state,
                spawn_projectile,
                change_direction,
            ).chain()
        ))
        .add_systems(OnEnter(AnguillaState::Run), animation_run)
        .add_systems(OnEnter(AnguillaState::Jump), animation_jump)
        .add_systems(OnEnter(AnguillaState::Neutral), animation_neutral)
        .add_systems(OnEnter(AnguillaState::CleanBubble), animation_clean)
        .run();
}

#[derive(Component, Eq, PartialEq)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component, Deref, DerefMut, Debug)]
struct AttackLock(Timer);

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum AnguillaState {
    #[default]
    Neutral,
    Run,
    Jump,
    CleanBubble,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum AnguillaDirection {
    #[default]
    Right,
    Left,
}

#[derive(Component)]
struct Anguilla;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Projectile;

#[derive(PartialEq, Eq, Copy, Clone)]
enum ProjectileType {
    Clean,
}

#[derive(Event)]
struct ProjectileEvent(ProjectileType, Vec3);

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
    commands.spawn((
        SpriteSheetBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: Vec3::new(2.0, 2.0, 1.0),
                ..default()
            },
            texture: asset_server.load("Anguilla/Anguilla_Neutral.png"),
            atlas: TextureAtlas {
                layout: texture_atlas_layouts.add(
                            TextureAtlasLayout::from_grid(Vec2::new(48.0, 32.0), 1, 4, None, None)
                ),
                index: 0,
            },
            ..default()
        },
        Anguilla,
        Velocity(Vec2::new(0.0, 0.0)),
        Collider,
        AnimationIndices { first: 0, last: 1 },
        AnimationTimer(Timer::from_seconds(0.4, TimerMode::Repeating)),
        AttackLock(Timer::from_seconds(0.0, TimerMode::Once)),
    ));
}

fn change_direction(
    mut query: Query<&mut Sprite, With<Anguilla>>,
    mut dir_change: EventReader<StateTransitionEvent<AnguillaDirection>>,
    direction: Res<State<AnguillaDirection>>,
) {
    for _change in dir_change.read() {
        let mut sprite = query.single_mut();
        sprite.flip_x = match direction.get() {
            AnguillaDirection::Right => false,
            AnguillaDirection::Left => true,
        };
    }
}

fn animation_jump(
    mut query: Query<(&mut AnimationIndices, &mut AnimationTimer, &mut Handle<Image>, &mut TextureAtlas), With<Anguilla>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let (mut indices, mut animation_timer, mut texture, mut atlas) = query.single_mut();

    *indices = AnimationIndices { first: 0, last: 3 };
    *texture = asset_server.load("Anguilla/Anguilla_Jump.png");
    atlas.layout = texture_atlas_layouts.add(
        TextureAtlasLayout::from_grid(Vec2::new(48.0, 32.0), 1, 4, None, None)
    );
    atlas.index = 0;
    animation_timer.0 = Timer::from_seconds(0.1, TimerMode::Repeating);
}

fn animation_run(
    mut query: Query<(&mut AnimationIndices, &mut AnimationTimer, &mut Handle<Image>, &mut TextureAtlas), With<Anguilla>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let (mut indices, mut animation_timer, mut texture, mut atlas) = query.single_mut();

    *indices = AnimationIndices { first: 0, last: 7 };
    *texture = asset_server.load("Anguilla/Anguilla_Run.png");
    atlas.layout = texture_atlas_layouts.add(
        TextureAtlasLayout::from_grid(Vec2::new(48.0, 32.0), 1, 8, None, None)
    );
    atlas.index = 0;
    animation_timer.0 = Timer::from_seconds(0.1, TimerMode::Repeating);
}

fn animation_neutral(
    mut query: Query<(&mut AnimationIndices, &mut AnimationTimer, &mut Handle<Image>, &mut TextureAtlas), With<Anguilla>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let (mut indices, mut animation_timer, mut texture, mut atlas) = query.single_mut();
    *indices = AnimationIndices { first: 0, last: 1 };
    *texture = asset_server.load("Anguilla/Anguilla_Neutral.png");
    atlas.layout = texture_atlas_layouts.add(
        TextureAtlasLayout::from_grid(Vec2::new(48.0, 32.0), 1, 2, None, None)
    );
    atlas.index = 0;
    *indices = AnimationIndices { first: 0, last: 1 };
    animation_timer.0 = Timer::from_seconds(0.4, TimerMode::Repeating);
}

fn animation_clean(
    mut query: Query<(&mut AnimationIndices, &mut AnimationTimer, &mut Handle<Image>, &mut TextureAtlas), With<Anguilla>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let (mut indices, mut animation_timer, mut texture, mut atlas) = query.single_mut();
    *indices = AnimationIndices { first: 0, last: 1 };
    *texture = asset_server.load("Anguilla/Anguilla_Clean_Projectile.png");
    atlas.layout = texture_atlas_layouts.add(
        TextureAtlasLayout::from_grid(Vec2::new(48.0, 32.0), 1, 12, None, None)
    );
    atlas.index = 0;
    *indices = AnimationIndices { first: 0, last: 11 };
    animation_timer.0 = Timer::from_seconds(0.1, TimerMode::Repeating);
}

fn update_state(
    mut query: Query<(&Velocity, &mut AttackLock), With<Anguilla>>,
    state: Res<State<AnguillaState>>,
    direction: Res<State<AnguillaDirection>>,
    mut next_state: ResMut<NextState<AnguillaState>>,
    mut next_direction: ResMut<NextState<AnguillaDirection>>,
    mut new_proj: EventReader<ProjectileEvent>,
) {
    let (velocity, mut timer) = query.single_mut();
    for _proj in new_proj.read() {
        next_state.set(AnguillaState::CleanBubble);
        *timer = AttackLock(Timer::from_seconds(1.2, TimerMode::Once));
    }
    if timer.finished() {
        if velocity.0.y != 0.0 && state.get() != &AnguillaState::Jump {
            next_state.set(AnguillaState::Jump);
        } else if velocity.0.x != 0.0 && velocity.0.y == 0.0 && state.get() != &AnguillaState::Run {
            next_state.set(AnguillaState::Run);
        } else if velocity.0.y == 0.0 && velocity.0.x == 0.0 && state.get() != &AnguillaState::Neutral {
            next_state.set(AnguillaState::Neutral);
        }

        if velocity.0.x < 0.0 && direction.get() != &AnguillaDirection::Left {
            next_direction.set(AnguillaDirection::Left);
        } else if velocity.0.x > 0.0 && direction.get() != &AnguillaDirection::Right {
            next_direction.set(AnguillaDirection::Right);
        }
    }
}

fn movement_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Velocity, With<Anguilla>>,
    state: Res<State<AnguillaState>>,
    time: Res<Time>,
) {
    let mut velocity = query.single_mut();
    let mut velocity_transform = Vec2::new(0.0, 0.0);
    
    if state.get() != &AnguillaState::CleanBubble {
        if keyboard_input.pressed(KeyCode::KeyA) {
            velocity_transform.x -= 800.0 * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::KeyD) {
            velocity_transform.x += 800.0 * time.delta_seconds();
        } 

        velocity.0 = (velocity.0 + velocity_transform).clamp(Vec2::new(-1000.0, -10000.0), Vec2::new(1000.0, 10000.0));
    }
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
    mut collision_events: EventWriter<CollisionEvent>,
    mut projectile_events: EventWriter<ProjectileEvent>,
    state: Res<State<AnguillaState>>,
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
            collision_events.send_default();

            match collision {
                Collision::Top => {
                    anguilla_velocity.0.y = 0.0;

                    if state.get() != &AnguillaState::CleanBubble {
                        if keyboard_input.pressed(KeyCode::KeyW) {
                            anguilla_velocity.0.y += 500.0;
                        }

                        if keyboard_input.just_pressed(KeyCode::KeyJ) {
                            anguilla_velocity.0.x = anguilla_velocity.0.x * 0.5 * time.delta_seconds();
                            let origin = anguilla_transform.translation;
                            projectile_events.send(ProjectileEvent(ProjectileType::Clean, origin));
                        }
                    }

                    if !keyboard_input.pressed(KeyCode::KeyA) && anguilla_velocity.0.x < 0.0 {
                        anguilla_velocity.0.x = (anguilla_velocity.0.x + 1000.0 * time.delta_seconds()).clamp(-500.0, 0.0);
                    }
                    if !keyboard_input.pressed(KeyCode::KeyD) && anguilla_velocity.0.x > 0.0 {
                        anguilla_velocity.0.x = (anguilla_velocity.0.x - 1000.0 * time.delta_seconds()).clamp(0.0, 500.0);
                    }
                },
                _ => {},
            }
        } else {
            anguilla_velocity.0.y -= 500.0 * time.delta_seconds();
        }
    }
}

fn spawn_projectile(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut new_proj: EventReader<ProjectileEvent>,
    direction: Res<State<AnguillaDirection>>,
) {
    let dir = match direction.get() {
        AnguillaDirection::Right => 1.0,
        AnguillaDirection::Left => -1.0,
    };
    for event in new_proj.read() {
        let translation = event.1 + Vec3::new(dir * 32.0, 0.0, 0.0);
        commands.spawn((
            SpriteSheetBundle {
                transform: Transform {
                    translation,
                    scale: Vec3::new(2.0, 2.0, 1.0),
                    ..default()
                },
                texture: asset_server.load("Projectiles/Clean_Bubble.png"),
                atlas: TextureAtlas {
                    layout: texture_atlas_layouts.add(
                                TextureAtlasLayout::from_grid(
                                    Vec2::new(10.0, 10.0),
                                    1,
                                    8,
                                    None,
                                    None
                                )),
                    index: 0,
                },
                ..default()
            },
            Projectile,
            Velocity(Vec2::new(dir * 500.0, 0.0)),
            AnimationIndices { first: 0, last: 7 },
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        ));
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

fn count_cooldown(
    time: Res<Time>,
    mut query: Query<&mut AttackLock>,
) {
    for mut timer in &mut query {
        timer.tick(time.delta());
    }
}
