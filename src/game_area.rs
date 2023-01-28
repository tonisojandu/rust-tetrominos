use bevy::math::{Vec2, Vec3};

pub const HORIZONTAL_TILES: u32 = 10;
pub const VERTICAL_TILES: u32 = 20;
pub const TILE_SIZE: f32 = 30.0;
pub const GAME_AREA: Vec2 = Vec2::new(HORIZONTAL_TILES as f32 * TILE_SIZE, VERTICAL_TILES as f32 * TILE_SIZE);
pub const MARGIN: f32 = 10.0;
pub const SIDE_PANEL_WIDTH: f32 = 200.0;

pub const BOUNDS: Vec2 = Vec2::new(3. * MARGIN + GAME_AREA.x + SIDE_PANEL_WIDTH, GAME_AREA.y + 2. * MARGIN);

pub fn find_translation(x: f32, y: f32, z: f32, width: f32, height: f32) -> Vec3 {
    return Vec3::new(
        (BOUNDS.x - width) / -2. + MARGIN + x,
        (BOUNDS.y - height) / 2. - MARGIN - y,
        z,
    );
}

pub fn tile_translation(coords: (i32, i32)) -> Vec3 {
    return find_translation(
        coords.0 as f32 * TILE_SIZE,
        coords.1 as f32 * TILE_SIZE,
        1.,
        TILE_SIZE,
        TILE_SIZE,
    );
}
