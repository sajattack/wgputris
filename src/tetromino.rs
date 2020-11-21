use crate::gameboard::Gameboard;
use crate::Vertex;
use crate::BLOCK_SIZE;
use crate::GAMEBOARD_OFFSET;

use rand::prelude::*;

#[derive(Debug, Copy, Clone)]
pub struct Tetromino {
    x: i32,
    y: i32,
    color: [f32; 4],
    block_locs: [(i32, i32); 4],
}

struct Block {
    pub x: f32,
    pub y: f32,
}

impl Tetromino {
    /// Creates a new O-shaped Tetromino.
    pub fn new_o() -> Self {
        Self {
            x: 0,
            y: 0,
            color: [1.0, 1.0, 0.0, 1.0],
            block_locs: [(0, 1), (1, 1), (0, 0), (1, 0)],
        }
    }

    /// Creates a new I-shaped Tetromino.
    pub fn new_i() -> Self {
        Self {
            x: 0,
            y: 0,
            color: [0.0, 1.0, 1.0, 1.0],
            block_locs: [(0, 0), (0, 1), (0, 2), (0, -1)],
        }
    }

    /// Creates a new S-shaped Tetromino.
    pub fn new_s() -> Self {
        Self {
            x: 0,
            y: 0,
            color: [1.0, 0.0, 0.0, 1.0],
            block_locs: [(0, 1), (-1, 1), (0, 0), (1, 0)],
        }
    }

    /// Creates a new Z-shaped Tetromino.
    pub fn new_z() -> Self {
        Self {
            x: 0,
            y: 0,
            color: [0.0, 1.0, 0.0, 1.0],
            block_locs: [(0, 0), (0, 1), (-1, 0), (1, 1)],
        }
    }

    /// Creates a new L-shaped Tetromino.
    pub fn new_l() -> Self {
        Self {
            x: 0,
            y: 0,
            color: [1.0, 0.55, 0.0, 1.0],
            block_locs: [(0, 1), (0, 0), (0, -1), (-1, -1)],
        }
    }

    /// Creates a new J-shaped Tetromino.
    pub fn new_j() -> Self {
        Self {
            x: 0,
            y: 0,
            color: [1.0, 0.0, 1.0, 1.0],
            block_locs: [(0, 1), (0, 0), (0, -1), (1, -1)],
        }
    }

    /// Creates a new T-shaped Tetromino.
    pub fn new_t() -> Self {
        Self {
            x: 0,
            y: 0,
            color: [0.0, 0.0, 1.0, 1.0],
            block_locs: [(1, 0), (0, 0), (-1, 0), (0, -1)],
        }
    }

    /// Creates a new Tetromino with a random shape
    ///
    /// # Parameters
    ///
    /// - `rng`: An initialized `ChaChaRng` random number generator from the
    /// `rand_chacha` crate.
    pub fn new_random(rng: &mut ThreadRng) -> Self {
        let rand_num = rng.gen_range(0, 7);
        match rand_num {
            1 => Tetromino::new_o(),
            2 => Tetromino::new_i(),
            3 => Tetromino::new_s(),
            4 => Tetromino::new_z(),
            5 => Tetromino::new_l(),
            6 => Tetromino::new_j(),
            _ => Tetromino::new_t(),
        }
    }

    fn as_blocks(&self) -> [Block; 4] {
        [
            Block {
                x: (self.block_locs[0].0 + self.x) as f32 * BLOCK_SIZE as f32,
                y: (self.block_locs[0].1 + self.y) as f32 * BLOCK_SIZE as f32,
            },
            Block {
                x: (self.block_locs[1].0 + self.x) as f32 * BLOCK_SIZE as f32,
                y: (self.block_locs[1].1 + self.y) as f32 * BLOCK_SIZE as f32,
            },
            Block {
                x: (self.block_locs[2].0 + self.x) as f32 * BLOCK_SIZE as f32,
                y: (self.block_locs[2].1 + self.y) as f32 * BLOCK_SIZE as f32,
            },
            Block {
                x: (self.block_locs[3].0 + self.x) as f32 * BLOCK_SIZE as f32,
                y: (self.block_locs[3].1 + self.y) as f32 * BLOCK_SIZE as f32,
            },
        ]
    }

    pub fn as_vertices(&self, buf: &mut [Vertex]) {
        self.as_blocks()
            .iter()
            .flat_map(|b| {
                Some(Vertex {
                    position: [b.x, b.y, 0.0],
                    tex_coords: [0.0, 0.0],
                    color: self.color,
                })
                .into_iter()
                .chain(Some(Vertex {
                    position: [b.x + BLOCK_SIZE as f32, b.y, 0.0],
                    tex_coords: [1.0, 0.0],
                    color: self.color,
                }))
                .into_iter()
                .chain(Some(Vertex {
                    position: [b.x + BLOCK_SIZE as f32, b.y + BLOCK_SIZE as f32, 0.0],
                    tex_coords: [1.0, 1.0],
                    color: self.color,
                }))
                .into_iter()
                .chain(Some(Vertex {
                    position: [b.x + BLOCK_SIZE as f32, b.y + BLOCK_SIZE as f32, 0.0],
                    tex_coords: [1.0, 1.0],
                    color: self.color,
                }))
                .into_iter()
                .chain(Some(Vertex {
                    position: [b.x, b.y + BLOCK_SIZE as f32, 0.0],
                    tex_coords: [0.0, 1.0],
                    color: self.color,
                }))
                .into_iter()
                .chain(Some(Vertex {
                    position: [b.x, b.y, 0.0],
                    tex_coords: [0.0, 0.0],
                    color: self.color,
                }))
            })
            .zip(buf.iter_mut())
            .for_each(|(v, dst)| *dst = v);
    }

    /// Sets the position of a `Tetromino`.
    /// Position is in block units, not screen units, i.e. screen units divided by
    /// `BLOCK_SIZE`.
    ///
    /// # Parameters
    ///
    /// - `x`: Position on the horizontal axis.
    /// - `y`: Position on the vertical axis.
    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    /// Rotates a `Tetromino` counter-clockwise.
    pub fn rotate_ccw(&mut self) {
        for i in 0..4 {
            self.block_locs[i] = (self.block_locs[i].1, 0 - self.block_locs[i].0);
        }
    }

    /// Rotates a `Tetromino` clockwise.
    pub fn rotate_cw(&mut self) {
        for i in 0..4 {
            self.block_locs[i] = (0 - self.block_locs[i].1, self.block_locs[i].0);
        }
    }

    /// Locks a `Tetromino` in place to a `Gameboard`
    ///
    /// # Parameters
    ///
    /// - `gameboard`: Mutable reference to a `Gameboard`.
    pub fn lock_to_gameboard(&self, gameboard: &mut Gameboard) {
        for block_loc in self.block_locs.iter() {
            gameboard
                .set_content(
                    (block_loc.0 + self.x - GAMEBOARD_OFFSET.0 as i32) as usize,
                    (block_loc.1 + self.y - GAMEBOARD_OFFSET.1 as i32) as usize,
                    Some(self.color),
                )
                .unwrap();
        }
    }

    /// Adds an x and y coordinate to the current position.
    ///
    /// # Parameters
    ///
    /// - `x`: Horizontal position to add to the current position
    /// - `y`: Vertical position to add to the current position
    pub fn add_pos(&mut self, x: i32, y: i32) {
        self.x += x;
        self.y += y;
    }

    /// Returns the position of each block subtracted from `GAMEBOARD_OFFSET`.
    /// Effectively, the position within a Gameboard.
    pub fn get_mapped_locs(&self) -> [(usize, usize); 4] {
        [
            (
                (self.block_locs[0].0 + self.x) as usize - GAMEBOARD_OFFSET.0,
                (self.block_locs[0].1 + self.y) as usize - GAMEBOARD_OFFSET.1,
            ),
            (
                (self.block_locs[1].0 + self.x) as usize - GAMEBOARD_OFFSET.0,
                (self.block_locs[1].1 + self.y) as usize - GAMEBOARD_OFFSET.1,
            ),
            (
                (self.block_locs[2].0 + self.x) as usize - GAMEBOARD_OFFSET.0,
                (self.block_locs[2].1 + self.y) as usize - GAMEBOARD_OFFSET.1,
            ),
            (
                (self.block_locs[3].0 + self.x) as usize - GAMEBOARD_OFFSET.0,
                (self.block_locs[3].1 + self.y) as usize - GAMEBOARD_OFFSET.1,
            ),
        ]
    }
}
