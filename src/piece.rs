use derive_more::Constructor;
use rand::prelude::thread_rng;
use rand::Rng;

pub enum Piece {
    I, L, J, O, S, Z
}

#[derive(Constructor)]
pub struct Shape {
    max_size: i32,
    coords: Vec<(i32, i32)>
}

impl Piece {

    pub fn get_tiles(&self, angle: u8, piece_x: i32, piece_y: i32) -> Vec<(i32, i32)> {
        let original_shape = self.get_shape();
        return match angle % 4 {
            0 => original_shape.coords.iter()
                .map(|(x, y)| (
                    *x + piece_x,
                    *y + piece_y,
                ))
                .collect(),
            1 => original_shape.coords.iter()
                .map(|(x, y)| (
                    original_shape.max_size - 1 - y + piece_x,
                    *x + piece_y,
                ))
                .collect(),
            2 => original_shape.coords.iter()
                .map(|(x, y)| (
                    original_shape.max_size - 1 - x + piece_x,
                    original_shape.max_size - 1 - y + piece_y,
                ))
                .collect(),
            3 => original_shape.coords.iter()
                .map(|(x, y)| (
                    *y + piece_x,
                    original_shape.max_size - 1 - x + piece_y,
                )).collect(),
            u => panic!("Wrong angle: {}", u)
        }
    }

    pub fn get_image(&self) -> String {
        match self {
            Piece::I => "img/red.png",
            Piece::L => "img/purple.png",
            Piece::J => "img/blue.png",
            Piece::O => "img/yellow.png",
            Piece::S => "img/cyan.png",
            Piece::Z => "img/green.png",
        }.to_string()
    }

    pub fn get_random() -> Piece {
        let random: u8 = thread_rng().gen::<u8>() % 6;
        match random {
            0 => Piece::I,
            1 => Piece::L,
            2 => Piece::J,
            3 => Piece::O,
            4 => Piece::S,
            5 => Piece::Z,
            _ => panic!("Wrong random: {}", random)
        }
    }

    fn get_shape(&self) -> Shape {
        match self {
            Piece::I => Shape::new(4, vec![(0, 2), (1, 2), (2, 2), (3, 2)]),
            Piece::L => Shape::new(3, vec![(1, 0), (1, 1), (1, 2), (2, 2)]),
            Piece::J => Shape::new(3, vec![(1, 0), (1, 1), (1, 2), (0, 2)]),
            Piece::O => Shape::new(2, vec![(0, 0), (1, 0), (0, 1), (1, 1)]),
            Piece::S => Shape::new(3, vec![(0, 2), (1, 2), (1, 1), (2, 1)]),
            Piece::Z => Shape::new(3, vec![(0, 1), (1, 1), (1, 2), (2, 2)]),
        }
    }
}
