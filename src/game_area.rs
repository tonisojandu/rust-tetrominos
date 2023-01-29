use bevy::math::{Vec2, Vec3};
use bevy::prelude::Transform;

pub const HORIZONTAL_TILES: u32 = 10;
pub const VERTICAL_TILES: u32 = 20;
pub const TILE_SIZE: f32 = 30.0;
pub const GAME_AREA: Vec2 = Vec2::new(
    HORIZONTAL_TILES as f32 * TILE_SIZE,
    VERTICAL_TILES as f32 * TILE_SIZE,
);
pub const PREVIEW_TILES: i32 = 4;
pub const PREVIEW_AREA: Vec2 = Vec2::new(
    PREVIEW_TILES as f32 * TILE_SIZE + 2. * MARGIN,
    PREVIEW_TILES as f32 * TILE_SIZE + 2. * MARGIN,
);
pub const PREVIEW_CORNER: Vec2 = Vec2::new(GAME_AREA.x + TILE_SIZE, TILE_SIZE);
pub const MARGIN: f32 = 10.0;

pub const LEFT_RIGHT_MOVE_SLEEP: u64 = 100;
pub const DOWN_MOVE_SLEEP: u64 = 100;
pub const INITIAL_DESCEND_SLEEP: u64 = 1000;

pub const SCORE_BOARD_WIDTH: f32 = 200.0;
pub const SCORE_BOARD_HEIGHT: f32 = 40.0;

pub const BOUNDS: Vec2 = Vec2::new(
    MARGIN + GAME_AREA.x + TILE_SIZE + SCORE_BOARD_WIDTH + MARGIN,
    GAME_AREA.y + 2. * MARGIN,
);

pub fn calculate_translation(x: f32, y: f32, z: f32, width: f32, height: f32) -> Vec3 {
    Vec3::new(
        (BOUNDS.x - width) / -2. + MARGIN + x,
        (BOUNDS.y - height) / 2. - MARGIN - y,
        z,
    )
}

pub fn calculate_transform(x: f32, y: f32, z: f32, width: f32, height: f32) -> Transform {
    return Transform {
        translation: calculate_translation(x, y, z, width, height),
        scale: Vec3::new(width, height, 0.),
        ..Default::default()
    };
}

pub fn tile_transform(coords: (i32, i32)) -> Transform {
    return Transform {
        translation: calculate_translation(
            coords.0 as f32 * TILE_SIZE,
            coords.1 as f32 * TILE_SIZE,
            1.,
            TILE_SIZE,
            TILE_SIZE,
        ),
        ..Default::default()
    };
}

pub fn preview_tile_translation(coords: (i32, i32), x_adjust: f32, y_adjust: f32) -> Transform {
    return Transform {
        translation: calculate_translation(
            MARGIN + PREVIEW_CORNER.x + (coords.0 as f32 * TILE_SIZE) + x_adjust,
            MARGIN + PREVIEW_CORNER.y + (coords.1 as f32 * TILE_SIZE) + y_adjust,
            1.,
            TILE_SIZE,
            TILE_SIZE,
        ),
        ..Default::default()
    };
}
