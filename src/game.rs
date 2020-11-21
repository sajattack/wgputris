use crate::Vertex;
use crate::gameboard::Gameboard;
use crate::tetromino::Tetromino;
use crate::{BLOCK_SIZE, GAMEBOARD_OFFSET, GAMEBOARD_WIDTH, GAMEBOARD_HEIGHT};
use winit::event::{KeyboardInput, VirtualKeyCode, ElementState};

use rand::prelude::*;
use std::time::Instant;

/// Stores the state of our entire game
pub struct Game {
    score: usize,
    board: Gameboard,
    next_shape: Tetromino,
    current_shape: Tetromino,
    next_shape_offset: (usize, usize),
    seconds_per_tick: f64,
    seconds_since_tick: f64,
    shape_placed: bool,
    rng: ThreadRng,
    last_loop_end: Instant,
    pub game_over: bool,
}

impl Game {
    /// Creates a new `Game`
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();

        let gameboard = Gameboard::new();

        let mut next_shape = Tetromino::new_random(&mut rng);
        next_shape.set_pos(30, 7);

        let mut current_shape = Tetromino::new_random(&mut rng);
        let spawn_loc = gameboard.get_spawn_loc(); 
        current_shape.set_pos(spawn_loc.0 as i32, spawn_loc.1 as i32);

        Self {
            score: 0,
            board: gameboard,
            next_shape,
            current_shape,
            next_shape_offset: (30, 7),
            seconds_per_tick: 0.25,
            seconds_since_tick: 0.0,
            shape_placed: false,
            rng,
            last_loop_end: Instant::now(),
            game_over: false,
        }
    }

    /// Handles user input
    pub fn process_input(&mut self, input: KeyboardInput) -> bool {
        match (input.virtual_keycode, input.state) {
            (None, _) => {
                return false;
            }
            (Some(key), ElementState::Pressed) => {
                match key {
                    VirtualKeyCode::Left => {
                        self.attempt_move(-1, 0);
                        true
                    },
                    VirtualKeyCode::Right => {
                        self.attempt_move(1, 0);
                        true
                    },
                    VirtualKeyCode::Down => {
                        self.drop();
                        self.current_shape.lock_to_gameboard(&mut self.board);
                        self.shape_placed = true;
                        true
                    },
                    VirtualKeyCode::Z => {
                        self.attempt_rotate_ccw();
                        true
                    },
                    VirtualKeyCode::X => {
                        self.attempt_rotate_cw();
                        true
                    }
                    _ => {false}, 
                }
            }
            _ => {false},
        }
    }

    /// Called once per loop of the game, does all the biz.
    ///
    pub fn process_game_loop(&mut self) {
        let loop_start = Instant::now();
        self.seconds_since_tick += (loop_start - self.last_loop_end).as_secs_f64();
        if self.seconds_since_tick > self.seconds_per_tick {
            self.tick();
            self.seconds_since_tick -= self.seconds_per_tick;
        }
        if self.shape_placed {
            if !self.spawn_next_shape() {
                self.game_over = true;
            } else {
                self.pick_next_shape();
                let rows_complete = self.board.remove_completed_rows();
                self.set_score(self.score + 400 * rows_complete);
            }
            self.shape_placed = false;
        }
        self.last_loop_end = Instant::now();
    }

    /// Moves `current_shape` down 1 unit and locks to board if it collides.
    pub fn tick(&mut self) {
        if !self.attempt_move(0, 1) {
            self.current_shape.lock_to_gameboard(&mut self.board);
            self.shape_placed = true;
        }
    }

    /// Setter for `score`
    ///
    /// # Parameters
    /// 
    /// - `score`: Score to set.
    pub fn set_score(&mut self, score: usize) {
        self.score = score;
    }

    /// Getter for `score`
    ///
    /// # Return Value
    ///
    /// Current game score
    pub fn get_score(&mut self) -> usize {
        self.score
    }

    /// Moves the `next_shape` into the `current_shape` and sets position accordingly.
    pub fn spawn_next_shape(&mut self) -> bool {
        self.current_shape = self.next_shape;
        let spawn_loc = self.board.get_spawn_loc();
        self.current_shape.set_pos(spawn_loc.0 as i32, spawn_loc.1 as i32);
        self.is_position_legal(&self.current_shape)
    }

    /// Picks the next Tetromino, sets it's position on the screen to be in the 
    /// "Next Shape:" section
    pub fn pick_next_shape(&mut self) {
        self.next_shape = Tetromino::new_random(&mut self.rng);
        self.next_shape.set_pos(self.next_shape_offset.0 as i32, self.next_shape_offset.1 as i32);
    }

    /// Attempts to add to the `current_shape` position, returns true if successful.
    ///
    /// # Parameters
    ///
    /// - `x`: horizontal position to add
    /// - `y`: vertical position to add
    ///
    /// # Return Value
    ///
    /// `true` if successful
    pub fn attempt_move(&mut self, x: i32, y: i32) -> bool {
        let mut temp: Tetromino = self.current_shape.clone();
        temp.add_pos(x, y);
        if self.is_position_legal(&temp) {
            self.current_shape.add_pos(x, y);
            return true;
        }
        false
    }

    /// Attempts to rotate `current_shape` clockwise, returns true if successful.
    ///
    /// # Return Value
    ///
    /// `true` if successful
    pub fn attempt_rotate_cw(&mut self) -> bool {
        let mut temp: Tetromino = self.current_shape.clone();
        temp.rotate_cw();
        if self.is_position_legal(&temp) {
            self.current_shape.rotate_cw();
            return true;
        }
        false
    }

    /// Attempts to rotate `current_shape` counterclockwise, returns true if successful.
    ///
    /// # Return Value
    ///
    /// `true` if successful
    pub fn attempt_rotate_ccw(&mut self) -> bool {
        let mut temp: Tetromino = self.current_shape.clone();
        temp.rotate_ccw();
        if self.is_position_legal(&temp) {
            self.current_shape.rotate_ccw();
            return true;
        }
        false
    }

    /// Checks if the position of the given tetromino is within boundaries and does 
    /// not collide.
    /// 
    /// # Parameters
    ///
    /// - `shape`: `Tetromino` to check
    ///
    /// # Return Value
    ///
    /// `true` if position is in bounds and does not collide
    pub fn is_position_legal(&self, shape: &Tetromino) -> bool {
        self.is_shape_within_borders(shape) 
        && !self.does_shape_intersect_locked_blocks(shape)
    }

    /// Checks if the position of the given tetromino is within boundaries of the
    /// gameboard
    ///
    /// # Parameters
    ///
    /// - `shape`: `Tetromino` to check
    ///
    /// # Return Value
    ///
    /// `true` if within boundaries of `board`
    pub fn is_shape_within_borders(&self, shape: &Tetromino) -> bool {
        let mapped_locs = shape.get_mapped_locs();
        for p in mapped_locs.iter() {
            if !(p.0 < GAMEBOARD_WIDTH 
            && p.1 < GAMEBOARD_HEIGHT) {
                return false
            }
        }
        true
    }

    /// Checks if the given tetromino's position collides with a block in the gameboard
    ///
    /// # Parameters
    ///
    /// `shape`: `Tetromino` to check
    /// 
    /// # Return Value
    ///
    /// `true` if shape collides
    pub fn does_shape_intersect_locked_blocks(&self, shape: &Tetromino) -> bool {
        let mapped_locs = shape.get_mapped_locs();
        !self.board.are_locs_empty(mapped_locs.to_vec())
    }

    /// Hard drop function
    pub fn drop(&mut self) {
        while self.attempt_move(0, 1) {}
    }

    fn render_background(&self, buf: &mut [Vertex]) {
        buf[0] = Vertex {
            position: [
                BLOCK_SIZE as f32 * GAMEBOARD_OFFSET.0 as f32,
                BLOCK_SIZE as f32 * GAMEBOARD_OFFSET.1 as f32,
                -1.0,
            ],
            tex_coords: [0.0, 0.0],
            color: [0.20, 0.20, 0.20, 0.5],
        };
        buf[1] = Vertex {
            position: [
                BLOCK_SIZE as f32 * GAMEBOARD_OFFSET.0 as f32 + BLOCK_SIZE as f32 * GAMEBOARD_WIDTH as f32,
                BLOCK_SIZE as f32 * GAMEBOARD_OFFSET.1 as f32,
                -1.0,
            ],
            tex_coords: [GAMEBOARD_WIDTH as f32, 0.0],
            color: [0.20, 0.20, 0.20, 0.5],
        };
        buf[2] = Vertex {
            position: [
                BLOCK_SIZE as f32 * GAMEBOARD_OFFSET.0 as f32 + BLOCK_SIZE as f32 * GAMEBOARD_WIDTH as f32,
                BLOCK_SIZE as f32 * GAMEBOARD_OFFSET.1 as f32 + BLOCK_SIZE as f32 * GAMEBOARD_HEIGHT as f32,
                -1.0,
            ],
            tex_coords: [GAMEBOARD_WIDTH as f32, GAMEBOARD_HEIGHT as f32],
            color: [0.20, 0.20, 0.20, 0.5],
        };
        buf[3] = Vertex {
            position: [
                BLOCK_SIZE as f32 * GAMEBOARD_OFFSET.0 as f32 + BLOCK_SIZE as f32 * GAMEBOARD_WIDTH as f32,
                BLOCK_SIZE as f32 * GAMEBOARD_OFFSET.1 as f32 + BLOCK_SIZE as f32 * GAMEBOARD_HEIGHT as f32,
                -1.0,
            ],
            tex_coords: [GAMEBOARD_WIDTH as f32, GAMEBOARD_HEIGHT as f32],
            color: [0.20, 0.20, 0.20, 0.5],
        };
        buf[4] = Vertex {
            position: [
                BLOCK_SIZE as f32 * GAMEBOARD_OFFSET.0 as f32,
                BLOCK_SIZE as f32 * GAMEBOARD_OFFSET.1 as f32 + BLOCK_SIZE as f32 * GAMEBOARD_HEIGHT as f32,
                -1.0,
            ],
            tex_coords: [0.0, GAMEBOARD_HEIGHT as f32],
            color: [0.20, 0.20, 0.20, 0.5],
        };
        buf[5] = Vertex {
            position: [
                BLOCK_SIZE as f32 * GAMEBOARD_OFFSET.0 as f32,
                BLOCK_SIZE as f32 * GAMEBOARD_OFFSET.1 as f32,
                -1.0,
            ],
            tex_coords: [0.0, 0.0],
            color: [0.20, 0.20, 0.20, 0.5],
        };
    }

    /// Returns renderable vertices to the main graphics api
    pub fn render(
        &self,
        buf: &mut [Vertex],
    ) {
            self.render_background(&mut buf[0..6]);
            self.board.as_vertices(&mut buf[6..1206]);
            self.current_shape.as_vertices(&mut buf[1206..1230]);
            self.next_shape.as_vertices(&mut buf[1230..1254]);
    }
}
