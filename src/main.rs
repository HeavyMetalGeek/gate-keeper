use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy::window::PrimaryWindow;
use rand::prelude::*;
use bevy::app::AppExit;

pub const PLAYER_SPEED: f32 = 500.0;
pub const PLAYER_HEIGHT: f32 = 50.0;
pub const PLAYER_WIDTH: f32 = 50.0;
pub const BULLET_SPEED: f32 = 200.0;
pub const BULLET_HEIGHT: f32 = 20.0;
pub const BULLET_WIDTH: f32 = 50.0;
pub const ENEMY_COUNT: usize = 5;
pub const ENEMY_SPEED: f32 = 100.0;
pub const ENEMY_WIDTH: f32 = 50.0;
pub const ENEMY_HEIGHT: f32 = 50.0;

pub const SHIP_PATH: &'static str = "sprites/prime.png";
//pub const SHIP_PATH: &'static str = "sprites/ship.png";
pub const BULLET_PATH: &'static str = "sprites/rust.png";
//pub const BULLET_PATH: &'static str = "sprites/bullet.png";
pub const ENEMY_PATH: &'static str = "sprites/typescript.png";
//pub const ENEMY_PATH: &'static str = "sprites/enemy.png";

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<BulletSpawnConfig>()
        .init_resource::<Score>()
        .add_event::<GameOver>()
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_player)
        .add_startup_system(spawn_enemies)
        .add_system(player_movement)
        .add_system(confine_player)
        .add_system(enemy_movement)
        .add_system(enemy_cross_gate)
        .add_system(spawn_bullet)
        .add_system(bullet_movement)
        .add_system(bullet_hit_enemy)
        .add_system(respawn_enemies)
        .add_system(handle_game_over)
        .add_system(exit_game)
        .run();
}

#[derive(Component)]
pub struct Player {}

#[derive(Component)]
pub struct Enemy {
    direction: Vec3,
}

#[derive(Component)]
pub struct Bullet {
    direction: Vec3,
}

#[derive(Resource)]
pub struct BulletSpawnConfig {
    timer: Timer,
}

#[derive(Resource)]
pub struct Score {
    value: u32,
}

impl Default for Score {
    fn default() -> Self {
        Self { value: 0 }
    }
}

impl Default for BulletSpawnConfig {
    fn default() -> Self {
        Self {
            timer: Timer::new(std::time::Duration::from_millis(100), TimerMode::Once),
        }
    }
}

pub fn exit_game(
    keyboard_input: Res<Input<KeyCode>>,
    mut app_exit_event_writer: EventWriter<AppExit>,
){
    if keyboard_input.just_pressed(KeyCode::Escape) {
        app_exit_event_writer.send(AppExit);
    }
}

pub fn handle_game_over(mut game_over_event_reader: EventReader<GameOver>) {
    for event in game_over_event_reader.iter() {
        println!("Game Over.  Your score is {}", event.score);
    }
}

pub struct GameOver {
    score: u32,
}

pub fn spawn_player(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window = window_query.get_single().unwrap();
    let winw = window.width();
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(winw / 2.0, PLAYER_HEIGHT, 0.0),
            texture: asset_server.load(SHIP_PATH),
            ..default()
        },
        Player {},
    ));
}

pub fn spawn_bullet(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut bullet_config: ResMut<BulletSpawnConfig>,
) {
    bullet_config.timer.tick(time.delta());
    if let Ok(player) = player_query.get_single() {
        if !keyboard_input.pressed(KeyCode::Space) {
            return;
        }
        if !bullet_config.timer.finished() {
            return;
        }
        commands.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(
                    player.translation.x,
                    player.translation.y + PLAYER_HEIGHT / 2.0,
                    0.0,
                ),
                texture: asset_server.load(BULLET_PATH),
                ..default()
            },
            Bullet {
                direction: Vec3::new(0.0, 1.0, 0.0),
            },
        ));
        bullet_config.timer.reset();
        audio.play(asset_server.load("audio/fire.ogg"));
    }
}

pub fn tick_bullet_timer(mut bullet_config: ResMut<BulletSpawnConfig>, time: Res<Time>) {
    bullet_config.timer.tick(time.delta());
}

pub fn bullet_movement(mut bullet_query: Query<(&mut Transform, &Bullet)>, time: Res<Time>) {
    for (mut xform, bullet) in bullet_query.iter_mut() {
        xform.translation += bullet.direction * BULLET_SPEED * time.delta_seconds();
    }
}

pub fn enemy_has_collision(x: f32, enemymap: &HashMap<usize, [f32; 2]>) -> bool {
    let buffer = ENEMY_WIDTH / 2.0;
    let left = x - buffer;
    let right = x + buffer;
    for (idx, pos) in enemymap.iter() {
        let fully_left = right < pos[0] - buffer;
        let fully_right = left > pos[0] + buffer;
        if !(fully_left || fully_right) {
            return true;
        }
    }
    return false;
}

pub fn spawn_enemies(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window = window_query.get_single().unwrap();
    let winw = window.width();
    let winh = window.height();
    let mut enemymap: HashMap<usize, [f32; 2]> = HashMap::new();
    for i in 0..ENEMY_COUNT {
        let mut x = 0.0;
        let mut collision = true;
        while collision == true {
            x = rand::random::<f32>() * (winw - ENEMY_WIDTH) + ENEMY_WIDTH / 2.0;
            collision = enemy_has_collision(x, &enemymap);
        }
        let pos = [x, winh - ENEMY_HEIGHT];
        enemymap.insert(i, pos);
        commands.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(pos[0], pos[1], 0.0),
                texture: asset_server.load(ENEMY_PATH),
                ..default()
            },
            Enemy {
                direction: Vec3::new(0.0, -1.0, 0.0),
            },
        ));
    }
}

pub fn respawn_enemies(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    enemy_query: Query<&Transform, With<Enemy>>,
    asset_server: Res<AssetServer>,
) {
    let mut enemy_count = enemy_query.iter().count();
    if enemy_count == ENEMY_COUNT {
        return;
    }
    let window = window_query.get_single().unwrap();
    let winw = window.width();
    let winh = window.height();
    let mut enemymap: HashMap<usize, [f32; 2]> = enemy_query
        .iter()
        .enumerate()
        .map(|(i, val)| (i, [val.translation.x, val.translation.y]))
        .collect();
    for _ in 0..(ENEMY_COUNT - enemy_count) {
        let mut x = 0.0;
        let mut collision = true;
        while collision == true {
            x = rand::random::<f32>() * (winw - ENEMY_WIDTH) + ENEMY_WIDTH / 2.0;
            collision = enemy_has_collision(x, &enemymap);
        }
        let pos = [x, winh - ENEMY_HEIGHT];
        enemymap.insert(enemymap.len(), pos);
        commands.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(pos[0], pos[1], 0.0),
                texture: asset_server.load(ENEMY_PATH),
                ..default()
            },
            Enemy {
                direction: Vec3::new(0.0, -1.0, 0.0),
            },
        ));
        enemy_count += 1;
    }
}

pub fn enemy_movement(mut enemy_query: Query<(&mut Transform, &Enemy)>, time: Res<Time>) {
    for (mut xform, enemy) in enemy_query.iter_mut() {
        xform.translation += enemy.direction * ENEMY_SPEED * time.delta_seconds();
    }
}

pub fn enemy_cross_gate(
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
    mut enemy_query: Query<&Transform, With<Enemy>>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    mut game_over_event_writer: EventWriter<GameOver>,
    score: Res<Score>,
) {
    if let Ok(player) = player_query.get_single() {
        for enemy_xform in enemy_query.iter_mut() {
            if enemy_xform.translation.y < 1.5 * PLAYER_HEIGHT {
                audio.play(asset_server.load("audio/you_lose.ogg"));
                commands.entity(player).despawn();
                game_over_event_writer.send(GameOver { score: score.value });
                return;
            }
        }
    };
}

pub fn bullet_hit_enemy(
    mut commands: Commands,
    mut enemy_query: Query<(&Transform, Entity), With<Enemy>>,
    mut bullet_query: Query<(&Transform, Entity), With<Bullet>>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    mut score: ResMut<Score>,
) {
    let enemy_xbuffer = ENEMY_WIDTH / 2.0;
    let enemy_ybuffer = ENEMY_HEIGHT / 2.0;
    let bullet_xbuffer = BULLET_WIDTH / 2.0;
    let bullet_ybuffer = BULLET_HEIGHT / 2.0;
    for (bullet_xform, bullet) in bullet_query.iter_mut() {
        for (enemy_xform, enemy) in enemy_query.iter_mut() {
            let xdist = (bullet_xform.translation.x - enemy_xform.translation.x).abs();
            let ydist = (bullet_xform.translation.y - enemy_xform.translation.y).abs();
            if xdist < enemy_xbuffer + bullet_xbuffer && ydist < enemy_ybuffer + bullet_ybuffer {
                audio.play(asset_server.load("audio/impact.ogg"));
                commands.entity(enemy).despawn();
                commands.entity(bullet).despawn();
                score.value += 1;
                return;
            }
        }
    }
}

pub fn spawn_camera(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
    let window = window_query.get_single().unwrap();
    let winh = window.height();
    let winw = window.width();
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(winw / 2.0, winh / 2.0, 0.0),
        ..default()
    });
}

pub fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    if let Ok(mut transform) = player_query.get_single_mut() {
        let mut direction = Vec3::ZERO;
        if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
            direction += Vec3::new(-1.0, 0.0, 0.0);
        }
        if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
            direction += Vec3::new(1.0, 0.0, 0.0);
        }

        if direction.length() > 0.0 {
            direction = direction.normalize();
            transform.translation += direction * PLAYER_SPEED * time.delta_seconds();
        }
    }
}

pub fn confine_player(
    mut player_query: Query<&mut Transform, With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(mut player_transform) = player_query.get_single_mut() {
        let window = window_query.get_single().unwrap();
        let winw = window.width();
        let xmin = PLAYER_WIDTH / 2.0;
        let xmax = winw - PLAYER_WIDTH / 2.0;

        let mut translation = player_transform.translation;

        if translation.x < xmin {
            translation.x = xmin;
        } else if translation.x > xmax {
            translation.x = xmax;
        }

        player_transform.translation = translation;
    }
}
