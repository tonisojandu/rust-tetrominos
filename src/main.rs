use std::time::Duration;

use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::text::Text2dBounds;
use derive_more::Constructor;
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
        .add_event::<ReachedFloorEvent>()
        .add_event::<AreaClearedEvent>()
        .add_event::<NewPositionEvent>()
        .add_event::<NewPieceEvent>()
        .add_system(descend_piece)
        .add_system(clear_room.before(descend_piece))
        .add_system(spawn_on_clear)
        .add_system(rotate_piece)
        .add_system(move_sideways)
        .add_system(draw_piece)
        .add_system(draw_preview)
        .add_system(update_score)
        .add_system(bevy::window::close_on_esc)
        .run();
}

#[derive(Resource, Constructor)]
struct Preview {
    piece: Piece,
    angle: u8,
}

#[derive(Resource, Constructor)]
struct PiecePosition {
    piece: Piece,
    angle: u8,
    x: i32,
    y: i32,
    is_visible: bool,
}

#[derive(Component)]
struct PreviewSprite;

#[derive(Component)]
struct PieceSprite;

#[derive(Component)]
struct ScoreBoard;

#[derive(Component, Constructor)]
struct RockSprite {
    x: i32,
    y: i32,
    color: Piece,
}

#[derive(Resource, Constructor)]
struct GameState {
    level: i32,
    score: i32,
    descend_sleep: Duration,
}

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

#[derive(Default)]
struct NewPositionEvent;

#[derive(Default)]
struct NewPieceEvent;

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
        transform: calculate_transform(0., 0., 0.1, GAME_AREA.x, GAME_AREA.y),
        material: materials.add(ColorMaterial::from(Color::BLACK)),
        ..default()
    });

    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
        transform: calculate_transform(
            PREVIEW_CORNER.x,
            PREVIEW_CORNER.y,
            0.1,
            PREVIEW_AREA.x,
            PREVIEW_AREA.y,
        ),
        material: materials.add(ColorMaterial::from(Color::BLACK)),
        ..default()
    });

    commands.insert_resource(Preview::new(
        Piece::get_random(),
        thread_rng().gen::<u8>() % 4,
    ));
    commands.insert_resource(PiecePosition::new(Piece::O, 0, 0, 0, false));
    commands.insert_resource(LastDownPress(Duration::from_secs(0)));
    commands.insert_resource(LastSidePress(Duration::from_secs(0)));
    commands.insert_resource(LastUpPress(false));
    commands.insert_resource(LastSpacePress(false));
    commands.insert_resource(FirstSpawnDone(false));
    commands.insert_resource(GameOver(false));
    commands.insert_resource(GameState::new(
        1,
        0,
        Duration::from_millis(INITIAL_DESCEND_SLEEP),
    ));
}

fn first_spawn(
    commands: Commands,
    asset_server: Res<AssetServer>,
    position: ResMut<PiecePosition>,
    preview: ResMut<Preview>,
    mut first_spawn_done: ResMut<FirstSpawnDone>,
    new_piece_writer: EventWriter<NewPieceEvent>,
    game_over: Res<GameOver>,
) {
    if !first_spawn_done.0 {
        first_spawn_done.0 = true;
        spawn_new_piece(
            commands,
            asset_server,
            position,
            preview,
            game_over,
            new_piece_writer,
        );
    }
}

fn spawn_on_clear(
    commands: Commands,
    asset_server: Res<AssetServer>,
    position: ResMut<PiecePosition>,
    preview: ResMut<Preview>,
    area_cleared_reader: EventReader<AreaClearedEvent>,
    new_piece_writer: EventWriter<NewPieceEvent>,
    game_over: Res<GameOver>,
) {
    if !area_cleared_reader.is_empty() {
        area_cleared_reader.clear();
        spawn_new_piece(
            commands,
            asset_server,
            position,
            preview,
            game_over,
            new_piece_writer,
        );
    }
}

fn spawn_new_piece(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut position: ResMut<PiecePosition>,
    mut preview: ResMut<Preview>,
    game_over: Res<GameOver>,
    mut new_piece_writer: EventWriter<NewPieceEvent>,
) {
    if game_over.0 {
        return;
    }
    position.piece = preview.piece.clone();
    position.angle = preview.angle.clone();
    position.x = HORIZONTAL_TILES as i32 / 2 - 1;
    position.y = -5;
    position.is_visible = true;

    preview.piece = Piece::get_random();
    preview.angle = thread_rng().gen::<u8>() % 4;

    let mut new_tiles = position
        .piece
        .get_tiles(position.angle, position.x, position.y);

    loop {
        let offer_tiles = position
            .piece
            .get_tiles(position.angle, position.x, position.y + 1);
        let mut is_visible = false;
        for i in 0..new_tiles.len() {
            if offer_tiles[i].1 >= 0 {
                is_visible = true;
                break;
            }
        }
        if is_visible {
            break;
        }
        position.y = position.y + 1;
        new_tiles = offer_tiles;
    }

    place_piece(&mut commands, asset_server, position, new_tiles);

    new_piece_writer.send_default();
}

fn place_piece(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    position: ResMut<PiecePosition>,
    new_tiles: Vec<(i32, i32)>,
) {
    for i in 0..new_tiles.len() {
        let tile = new_tiles[i];
        commands.spawn((
            PieceSprite,
            SpriteBundle {
                texture: position.piece.get_image(&asset_server),
                visibility: Visibility {
                    is_visible: tile.1 >= 0,
                },
                transform: tile_transform(tile),
                ..default()
            },
        ));
    }
}

fn descend_piece(
    mut commands: Commands,
    mut position: ResMut<PiecePosition>,
    asset_server: Res<AssetServer>,
    game_state: Res<GameState>,
    mut game_over: ResMut<GameOver>,
    mut last_click: ResMut<LastDownPress>,
    keyboard_input: Res<Input<KeyCode>>,
    mut last_space: ResMut<LastSpacePress>,
    time: Res<Time>,
    rock_query: Query<(&RockSprite, Entity)>,
    mut reached_floor_writer: EventWriter<ReachedFloorEvent>,
    mut new_position_writer: EventWriter<NewPositionEvent>,
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
        || since_click >= game_state.descend_sleep
        || (keyboard_input.pressed(KeyCode::Down)
            && since_click >= Duration::from_millis(DOWN_MOVE_SLEEP))
    {
        last_click.0 = time.elapsed();

        let rocks_entities: Vec<(&RockSprite, Entity)> = rock_query.iter().collect();
        let rocks: Vec<&RockSprite> = rocks_entities.iter().map(|pair| pair.0).collect();
        loop {
            let new_y = position.y + 1;

            if collision(
                &position.piece,
                &position.angle,
                &position.x,
                &new_y,
                &rocks,
            ) == CollisionType::Floor
            {
                info!("Floor at x={} y={}", position.x, position.y);

                let mut rock_out_of_bounds = false;
                for (x, y) in position
                    .piece
                    .get_tiles(position.angle, position.x, position.y)
                {
                    if y < 0 {
                        rock_out_of_bounds = true;
                    }
                    spawn_rock(&mut commands, &asset_server, x, y, &position.piece);
                }

                if rock_out_of_bounds {
                    game_over.0 = true;
                    info!("Game Over!");
                }

                position.is_visible = false;
                reached_floor_writer.send_default();

                break;
            } else {
                position.y = new_y;
                new_position_writer.send_default();
                info!("New x={} y={}", position.x, new_y);
            }

            if !space_pressed {
                break;
            }
        }
    }
}

fn spawn_rock(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    x: i32,
    y: i32,
    color: &Piece,
) {
    commands.spawn((
        RockSprite::new(x, y, color.clone()),
        SpriteBundle {
            texture: color.get_image(&asset_server),
            transform: tile_transform((x, y)),
            visibility: Visibility { is_visible: y >= 0 },
            ..default()
        },
    ));
}

fn clear_room(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut game_state: ResMut<GameState>,
    mut rock_query: Query<(&RockSprite, Entity)>,
    reached_floor_reader: EventReader<ReachedFloorEvent>,
    mut area_cleared_writer: EventWriter<AreaClearedEvent>,
) {
    if !reached_floor_reader.is_empty() {
        reached_floor_reader.clear();

        let rocks = rock_query
            .iter_mut()
            .collect::<Vec<(&RockSprite, Entity)>>();

        let mut line_edits = vec![0; VERTICAL_TILES as usize];
        let mut line_counts = vec![0; VERTICAL_TILES as usize];
        let mut cleared = 0;

        for i in 0..rocks.len() {
            let rock = rocks[i].0;
            if rock.y >= 0 {
                let y = rock.y as usize;
                line_counts[y] += 1;
                if line_counts[y] == HORIZONTAL_TILES {
                    line_edits[y] = -1;
                    cleared += 1;
                }
            }
        }

        game_state.score += game_state.level
            * match cleared {
                0 => 0,
                1 => 100,
                2 => 300,
                3 => 500,
                4 => 800,
                _ => 0,
            };

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
            if rock.y >= 0 {
                let edit = line_edits[rock.y as usize];
                if edit == -1 {
                    commands.entity(entity).despawn();
                } else if edit > 0 {
                    commands.entity(entity).despawn();
                    let x = rock.x;
                    let y = rock.y + edit;
                    spawn_rock(&mut commands, &asset_server, x, y, &rock.color);
                }
            }
        }

        area_cleared_writer.send_default();
    }
}

fn collision(
    piece: &Piece,
    angle: &u8,
    x: &i32,
    y: &i32,
    rocks: &Vec<&RockSprite>,
) -> CollisionType {
    let new_coords = piece.get_tiles(*angle, *x, *y);

    for (new_x, new_y) in new_coords {
        if new_y >= VERTICAL_TILES as i32 {
            return CollisionType::Floor;
        }

        for rock in rocks {
            if new_x == rock.x && new_y == rock.y {
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
    mut commands: Commands,
    position: ResMut<PiecePosition>,
    sprite_query: Query<(&PieceSprite, Entity)>,
    new_position_reader: EventReader<NewPositionEvent>,
    asset_server: Res<AssetServer>,
) {
    if !new_position_reader.is_empty() {
        new_position_reader.clear();

        sprite_query.for_each(|(_, entity)| {
            commands.entity(entity).despawn();
        });

        if position.is_visible {
            let coords = position
                .piece
                .get_tiles(position.angle, position.x, position.y);
            place_piece(&mut commands, asset_server, position, coords);
        }
    }
}

fn draw_preview(
    mut commands: Commands,
    preview: Res<Preview>,
    sprite_query: Query<(&PreviewSprite, Entity)>,
    new_piece_reader: EventReader<NewPieceEvent>,
    asset_server: Res<AssetServer>,
) {
    if !new_piece_reader.is_empty() {
        new_piece_reader.clear();

        sprite_query.for_each(|(_, entity)| {
            commands.entity(entity).despawn();
        });

        let tiles = preview.piece.get_tiles(preview.angle, 0, 0);

        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;

        for i in 0..tiles.len() {
            max_x = max_x.max(tiles[i].0);
            max_y = max_y.max(tiles[i].1);
            min_x = min_x.min(tiles[i].0);
            min_y = min_y.min(tiles[i].1);
        }

        let d_left = MARGIN + TILE_SIZE * min_x as f32;
        let d_top = MARGIN + TILE_SIZE * min_y as f32;
        let d_right = MARGIN + TILE_SIZE * (PREVIEW_TILES - max_x - 1) as f32;
        let d_bottom = MARGIN + TILE_SIZE * (PREVIEW_TILES - max_y - 1) as f32;

        let horizontal_margin = (d_left + d_right) / 2.0;
        let vertical_margin = (d_top + d_bottom) / 2.0;

        for i in 0..tiles.len() {
            let tile = tiles[i];
            commands.spawn((
                PreviewSprite,
                SpriteBundle {
                    texture: preview.piece.get_image(&asset_server),
                    transform: preview_tile_translation(
                        tile,
                        horizontal_margin - d_left,
                        vertical_margin - d_top,
                    ),
                    ..default()
                },
            ));
        }
    }
}

fn rotate_piece(
    mut position: ResMut<PiecePosition>,
    keyboard_input: Res<Input<KeyCode>>,
    rock_query: Query<(&RockSprite, Entity)>,
    mut last_click: ResMut<LastUpPress>,
    mut new_position_writer: EventWriter<NewPositionEvent>,
) {
    if last_click.0 && !keyboard_input.pressed(KeyCode::Up) {
        last_click.0 = false;
    } else if !last_click.0 && keyboard_input.pressed(KeyCode::Up) {
        last_click.0 = true;
        let new_angle = (position.angle + 1) % 4;
        let mut new_x = position.x;

        let rocks: Vec<&RockSprite> = rock_query.iter().map(|pair| pair.0).collect();

        let mut collision_type = collision(
            &position.piece,
            &new_angle,
            &position.x,
            &position.y,
            &rocks,
        );

        let mut accepted = false;
        if collision_type == CollisionType::None {
            position.angle = new_angle;
            accepted = true;
        } else if collision_type == CollisionType::RightWall {
            new_x = position.x - 1;
            collision_type = collision(&position.piece, &new_angle, &new_x, &position.y, &rocks);
            if collision_type == CollisionType::None {
                accepted = true;
            } else {
                new_x = position.x - 2;
                collision_type =
                    collision(&position.piece, &new_angle, &new_x, &position.y, &rocks);
                if collision_type == CollisionType::None {
                    accepted = true;
                }
            }
        } else if collision_type == CollisionType::LeftWall {
            new_x = position.x + 1;
            collision_type = collision(&position.piece, &new_angle, &new_x, &position.y, &rocks);
            if collision_type == CollisionType::None {
                accepted = true;
            } else {
                new_x = position.x + 2;
                collision_type =
                    collision(&position.piece, &new_angle, &new_x, &position.y, &rocks);
                if collision_type == CollisionType::None {
                    accepted = true;
                }
            }
        }

        if accepted {
            position.angle = new_angle;
            position.x = new_x;
            new_position_writer.send_default();
        }
    }
}

fn move_sideways(
    mut position: ResMut<PiecePosition>,
    keyboard_input: Res<Input<KeyCode>>,
    rock_query: Query<(&RockSprite, Entity)>,
    time: Res<Time>,
    mut last_click: ResMut<LastSidePress>,
    mut new_position_writer: EventWriter<NewPositionEvent>,
) {
    let since_click = time.elapsed() - last_click.0;
    if since_click < Duration::from_millis(LEFT_RIGHT_MOVE_SLEEP) {
        return;
    }

    let mut delta_x = 0;
    if keyboard_input.pressed(KeyCode::Left) && !keyboard_input.pressed(KeyCode::Right) {
        delta_x = -1;
    } else if keyboard_input.pressed(KeyCode::Right) && !keyboard_input.pressed(KeyCode::Left) {
        delta_x = 1;
    }

    if delta_x != 0 {
        let new_x = position.x + delta_x;

        let rocks: Vec<&RockSprite> = rock_query.iter().map(|pair| pair.0).collect();

        if collision(
            &position.piece,
            &position.angle,
            &new_x,
            &position.y,
            &rocks,
        ) == CollisionType::None
        {
            position.x = new_x;
            last_click.0 = time.elapsed();
            new_position_writer.send_default();
        }
    }
}

fn update_score(
    mut commands: Commands,
    game_state: Res<GameState>,
    mut score_board_query: Query<(&ScoreBoard, Entity)>,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let text_style = TextStyle {
        font,
        font_size: SCORE_BOARD_HEIGHT,
        color: Color::WHITE,
    };
    score_board_query
        .iter_mut()
        .for_each(|(_, entity)| commands.entity(entity).despawn());
    commands.spawn((
        ScoreBoard,
        Text2dBundle {
            text: Text::from_section(game_state.score.to_string(), text_style)
                .with_alignment(TextAlignment::CENTER),
            text_2d_bounds: Text2dBounds {
                size: Vec2::new(SCORE_BOARD_WIDTH, SCORE_BOARD_HEIGHT),
            },
            transform: Transform {
                translation: calculate_translation(
                    PREVIEW_CORNER.x,
                    PREVIEW_CORNER.y + (1. + PREVIEW_TILES as f32) * TILE_SIZE + 2. * MARGIN,
                    2.,
                    SCORE_BOARD_WIDTH,
                    SCORE_BOARD_HEIGHT,
                ),
                ..default()
            },
            ..default()
        },
    ));
}
