use std::clone::Clone;
use std::time::Duration;

use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::time::FixedTimestep;
use rand::prelude::thread_rng;
use rand::Rng;

use game_area::*;
use piece::*;

mod game_area;
mod piece;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(first_spawn)
        .add_system_set(SystemSet::new()
            .with_run_criteria(FixedTimestep::step(0.01))
            .with_system(draw_piece)
        )
        .add_event::<ReachedFloorEvent>()
        .add_event::<AreaClearedEvent>()
        .add_system(descend_piece)
        .add_system(spawn_on_clear)
        .add_system(clear_room)
        .add_system(rotate_piece)
        .add_system(move_sideways)
        .run();
}

#[derive(Resource)]
struct PiecePosition(Piece, u8, i32, i32);

#[derive(Component)]
struct PieceSprite;

#[derive(Component)]
struct RockSprite(i32, i32);

#[derive(Resource)]
struct LastDownPress(Duration);

#[derive(Resource)]
struct LastSidePress(Duration);

#[derive(Resource)]
struct LastUpPress(bool);

#[derive(Resource)]
struct LastSpacePress(bool);

#[derive(Resource)]
struct FirstSpawnDone(bool);

#[derive(Default)]
struct ReachedFloorEvent;

#[derive(Default)]
struct AreaClearedEvent;

#[derive(Resource)]
struct GameOver(bool);

#[derive(PartialEq)]
enum CollisionType {
    LeftWall,
    RightWall,
    Floor,
    None,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
        transform: Transform::default().with_scale(Vec3::from((BOUNDS, 0.))),
        material: materials.add(ColorMaterial::from(Color::rgb_u8(51, 53, 66))),
        ..default()
    });

    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
        transform: Transform::from_translation(find_translation(0., 0., 0.1, GAME_AREA.x, GAME_AREA.y))
            .with_scale(Vec3::from((GAME_AREA, 0.))),
        material: materials.add(ColorMaterial::from(Color::BLACK)),
        ..default()
    });

    commands.insert_resource(PiecePosition(Piece::O, 0, 0, 0));
    commands.insert_resource(LastDownPress(Duration::from_secs(0)));
    commands.insert_resource(LastSidePress(Duration::from_secs(0)));
    commands.insert_resource(LastUpPress(false));
    commands.insert_resource(LastSpacePress(false));
    commands.insert_resource(FirstSpawnDone(false));
    commands.insert_resource(GameOver(false));
}

fn first_spawn(
    commands: Commands,
    asset_server: Res<AssetServer>,
    position: ResMut<PiecePosition>,
    rock_query: Query<&RockSprite>,
    mut first_spawn_done: ResMut<FirstSpawnDone>,
    game_over: ResMut<GameOver>,
) {
    if !first_spawn_done.0 {
        spawn_new_piece(commands, asset_server, position, rock_query, game_over);
        first_spawn_done.0 = true
    }
}

fn spawn_on_clear(
    commands: Commands,
    asset_server: Res<AssetServer>,
    position: ResMut<PiecePosition>,
    rock_query: Query<&RockSprite>,
    collision_events: EventReader<AreaClearedEvent>,
    game_over: ResMut<GameOver>,
) {
    if !collision_events.is_empty() {
        collision_events.clear();
        spawn_new_piece(commands, asset_server, position, rock_query, game_over);
    }
}

fn spawn_new_piece(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut position: ResMut<PiecePosition>,
    rock_query: Query<&RockSprite>,
    mut game_over: ResMut<GameOver>,
) {
    let piece = Piece::get_random();
    let color = asset_server.load(piece.get_image());

    position.0 = piece;
    position.1 = thread_rng().gen::<u8>() % 4;
    position.2 = HORIZONTAL_TILES as i32 / 2 - 1;
    position.3 = -5;

    let mut new_tiles = position.0.get_tiles(position.1, position.2, position.3);

    loop {
        let offer_tiles = position.0.get_tiles(position.1, position.2, position.3 + 1);
        let mut is_visible = false;
        for i in 0..new_tiles.len() {
            if offer_tiles[i].1 >= 0 {
                is_visible = true;
                break
            }
        }
        if is_visible {
            break;
        }
        position.3 = position.3 + 1;
        new_tiles = offer_tiles;
    }

    let rocks: Vec<&RockSprite> = rock_query.iter().collect();
    for i in 0..new_tiles.len() {
        let tile = new_tiles[i];
        if collision(&position.0, &position.1, &tile.0, &tile.1, &rocks) == CollisionType::Floor {
            info!("Game Over");
            game_over.0 = true;
            return;
        }
    }

    for tile in new_tiles {
        commands.spawn((PieceSprite, SpriteBundle {
            texture: color.clone(),
            visibility: Visibility {
                is_visible: tile.1 >= 0,
            },
            transform: Transform::from_translation(find_translation(
                tile.0 as f32,
                tile.1 as f32,
                0.1,
                TILE_SIZE,
                TILE_SIZE,
            )),
            ..default()
        }));
    }
}

fn descend_piece(
    mut commands: Commands,
    mut position: ResMut<PiecePosition>,
    asset_server: Res<AssetServer>,
    mut piece_query: Query<(&PieceSprite, Entity)>,
    rock_query: Query<(&RockSprite, Entity)>,
    time: Res<Time>,
    mut last_click: ResMut<LastDownPress>,
    keyboard_input: Res<Input<KeyCode>>,
    mut last_space: ResMut<LastSpacePress>,
    mut reached_floor_writer: EventWriter<ReachedFloorEvent>,
    game_over: ResMut<GameOver>,
) {
    if game_over.0 {
        return;
    }

    let mut space_pressed = false;
    if last_space.0 && !keyboard_input.pressed(KeyCode::Space) {
        last_space.0 = false;
    } else if !last_space.0 && keyboard_input.pressed(KeyCode::Space) {
        space_pressed = true;
        last_space.0 = true;
    }

    let since_click = time.elapsed() - last_click.0;
    if space_pressed
        || since_click >= Duration::from_millis(1000)
        || (keyboard_input.pressed(KeyCode::Down) && since_click >= Duration::from_millis(100)) {

        last_click.0 = time.elapsed();

        let rocks_entities: Vec<(&RockSprite, Entity)> = rock_query.iter().collect();
        let rocks: Vec<&RockSprite> = rocks_entities.iter().map(|pair| pair.0).collect();
        loop {
            let new_y = position.3 + 1;

            if collision(&position.0, &position.1, &position.2, &new_y, &rocks) == CollisionType::Floor {
                for (_, entity) in piece_query.iter_mut() {
                    commands.entity(entity).despawn();
                }

                for (x, y) in  position.0.get_tiles(position.1, position.2, position.3) {
                    spawn_rock(&mut commands, &asset_server, x, y);
                }

                reached_floor_writer.send_default();

                info!("Floor at x={} y={}", position.2, position.3);
                break;
            } else {
                position.3 = new_y;
                info!("New x={} y={}", position.2, new_y);
            }

            if !space_pressed {
                break;
            }
        }
    }
}

fn spawn_rock(commands: &mut Commands, asset_server: &Res<AssetServer>, x: i32, y: i32) {
    commands.spawn((RockSprite(x, y), SpriteBundle {
        texture: asset_server.load("img/grey.png"),
        transform: Transform::from_translation(tile_translation((x, y))),
        ..default()
    }));
}

fn clear_room(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    reached_floor_events: EventReader<ReachedFloorEvent>,
    mut area_cleared_events: EventWriter<AreaClearedEvent>,
    mut rock_query: Query<(&RockSprite, Entity)>,
) {
    if !reached_floor_events.is_empty() {
        reached_floor_events.clear();

        let rocks = rock_query.iter_mut().collect::<Vec<(&RockSprite, Entity)>>();

        let mut line_edits = vec![0; VERTICAL_TILES as usize];
        let mut line_counts = vec![0; VERTICAL_TILES as usize];

        for i in 0..rocks.len() {
            let rock = rocks[i].0;
            if rock.1 >= 0 {
                let y = rock.1 as usize;
                line_counts[y] += 1;
                if line_counts[y] == HORIZONTAL_TILES {
                    line_edits[y] = -1;
                }
            }
        }

        let mut remove = 0;
        for i in (0..VERTICAL_TILES as usize).rev() {
            if line_edits[i] == -1 {
                remove += 1;
            } else {
                line_edits[i] = remove;
            }
        }

        for i in 0..rocks.len() {
            let rock = rocks[i].0;
            let entity = rocks[i].1;
            if rock.1 >= 0 {
                let edit = line_edits[rock.1 as usize];
                if edit == -1 {
                    commands.entity(entity).despawn();
                } else if edit > 0 {
                    commands.entity(entity).despawn();
                    let x = rock.0;
                    let y = rock.1 + edit;
                    commands.spawn((RockSprite(x, y), SpriteBundle {
                        texture: asset_server.load("img/grey.png"),
                        transform: Transform::from_translation(tile_translation((x, y))),
                        ..default()
                    }));
                }
            }
        }

        area_cleared_events.send_default();
    }
}

fn collision(
    piece: &Piece,
    angle: &u8,
    x: &i32,
    y: &i32,
    rocks: &Vec<&RockSprite>
) -> CollisionType {
    let new_coords = piece.get_tiles(*angle, *x, *y);

    for (new_x, new_y) in new_coords {
        if new_y >= VERTICAL_TILES as i32 {
            return CollisionType::Floor;
        }

        for rock in rocks {
            if new_x == rock.0 && new_y == rock.1 {
                return CollisionType::Floor;
            }
        }

        if new_x < 0 {
            return CollisionType::LeftWall;
        }

        if new_x >= HORIZONTAL_TILES as i32 {
            return CollisionType::RightWall;
        }
    }

    return CollisionType::None;
}

fn draw_piece(
    position: ResMut<PiecePosition>,
    mut sprite_query: Query<(&PieceSprite, &mut Transform, &mut Visibility)>,
) {
    let mut i = 0;
    let coords = position.0.get_tiles(position.1, position.2, position.3);
    for (_, mut transform, mut visibility) in sprite_query.iter_mut() {
        visibility.is_visible = coords[i].1 >= 0;
        transform.translation = tile_translation(coords[i]);
        i += 1;
    }
}

fn rotate_piece(
    mut position: ResMut<PiecePosition>,
    keyboard_input: Res<Input<KeyCode>>,
    rock_query: Query<(&RockSprite, Entity)>,
    mut last_click: ResMut<LastUpPress>,
) {
    if last_click.0 && !keyboard_input.pressed(KeyCode::Up) {
        last_click.0 = false;
    } else if !last_click.0 && keyboard_input.pressed(KeyCode::Up) {
        last_click.0 = true;
        let new_angle = (position.1 + 1) % 4;

        let rocks: Vec<&RockSprite> = rock_query.iter().map(|pair| pair.0).collect();

        let mut collision_type = collision(&position.0, &new_angle, &position.2, &position.3, &rocks);

        if collision_type == CollisionType::None {
            position.1 = new_angle;
            return;
        }

        if collision_type == CollisionType::RightWall {
            let new_x = position.2 - 1;
            collision_type = collision(&position.0, &new_angle, &new_x, &position.3, &rocks);
            if collision_type == CollisionType::None {
                position.1 = new_angle;
                position.2 = new_x;
                return;
            }
            let new_x = position.2 - 2;
            collision_type = collision(&position.0, &new_angle, &new_x, &position.3, &rocks);
            if collision_type == CollisionType::None {
                position.1 = new_angle;
                position.2 = new_x;
                return;
            }
        } else if collision_type == CollisionType::LeftWall {
            let new_x = position.2 + 1;
            collision_type = collision(&position.0, &new_angle, &new_x, &position.3, &rocks);
            if collision_type == CollisionType::None {
                position.1 = new_angle;
                position.2 = new_x;
                return;
            }
            let new_x = position.2 + 2;
            collision_type = collision(&position.0, &new_angle, &new_x, &position.3, &rocks);
            if collision_type == CollisionType::None {
                position.1 = new_angle;
                position.2 = new_x;
                return;
            }
        }
    }
}

fn move_sideways(
    mut position: ResMut<PiecePosition>,
    keyboard_input: Res<Input<KeyCode>>,
    rock_query: Query<(&RockSprite, Entity)>,
    time: Res<Time>,
    mut last_click: ResMut<LastSidePress>,
) {
    let since_click = time.elapsed() - last_click.0;
    if since_click < Duration::from_millis(100) {
        return;
    }

    let mut delta_x = 0;
    if keyboard_input.pressed(KeyCode::Left) && !keyboard_input.pressed(KeyCode::Right) {
        delta_x = -1;
    } else if keyboard_input.pressed(KeyCode::Right) && !keyboard_input.pressed(KeyCode::Left) {
        delta_x = 1;
    }

    if delta_x != 0 {
        let new_x = position.2 + delta_x;

        let rocks: Vec<&RockSprite> = rock_query.iter().map(|pair| pair.0).collect();

        if collision(&position.0, &position.1, &new_x, &position.3, &rocks) == CollisionType::None {
            position.2 = new_x;
            last_click.0 = time.elapsed();
        }
    }

}
