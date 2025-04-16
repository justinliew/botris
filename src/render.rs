use crate::board::*;
use crate::cell::*;
use crate::game::*;
use crate::enemy::*;
use crate::log::*;

use std::collections::HashMap;
use std::ffi::{c_int, c_uint};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

extern "C" {
    fn clear_screen();
    fn draw_intro();
    fn draw_borders(_: c_uint, _: c_uint, _: c_uint, _: c_uint);
    fn draw_block(_: c_uint, _: c_uint, _: c_uint, _: c_uint, _: c_uint);
    fn draw_multiplier(_: c_uint, _: c_uint, _: c_uint);
    fn draw_cursor_blocks(_: c_uint, _: c_uint, _: c_uint, _: c_uint);
    fn draw_name_picker(_: c_int, _: c_int);
    fn draw_multiplier_ui(_: c_uint);
    fn draw_enemy_ui(_: c_uint);
    fn _update_local_score(_: c_int);

    // sprite id, frame index, x, y
    fn draw_sprite(
        _: c_uint,
        _: c_uint,
        _: c_uint,
        _: c_uint,
        _: c_uint,
        _: c_uint,
        _: c_uint,
        _: c_uint,
    );
}

pub struct RenderData {
    screen_width: u32,
    screen_height: u32,
    _receiver: Receiver<GameEvent>,
    pub sender: Sender<GameEvent>,
}

struct Multiplier {
    x: u32,
    y: u32,
    value: u32,
}

impl RenderData {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        RenderData {
            screen_width: 1024,
            screen_height: 768,
            _receiver: rx,
            sender: tx,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.screen_width = width;
        self.screen_height = height;
    }

    unsafe fn _handle_game_event(&mut self, _event: GameEvent) {
        // match event {
        //     GameEvent::ScoreChanged(i) => {
        //         update_local_score(i);
        //     }
        // }
    }

    pub fn draw_board(&self, board: &Board) {
        let dim = self.screen_height / 12;
        let delta = board.delta * dim as f64;
        const NUM_ROWS_MIN_1: usize = NUM_ROWS - 1;

        let mut multipliers = HashMap::new();

        for y in 0..NUM_ROWS {
            for x in 0..NUM_COLS {
                match board.get_cell(x, y) {
                    Cell::Single(id, offset) => {
                        let offset_v = match offset {
                            Some(v) => *v,
                            None => 0.,
                        };
                        let xb = dim + x as u32 * dim;
                        let yb = (NUM_ROWS - y) as u32 * dim - delta as u32
                            + (dim as f64 * offset_v) as u32;

                        match y {
                            0 => unsafe { draw_sprite(*id, 0, xb, yb, 0, 0, dim, delta as u32) },
                            NUM_ROWS_MIN_1 => unsafe {
                                draw_sprite(
                                    *id,
                                    0,
                                    xb,
                                    yb,
                                    0,
                                    delta as u32,
                                    dim,
                                    dim - delta as u32,
                                )
                            },
                            _ => unsafe { draw_sprite(*id, 0, xb, yb, 0, 0, dim, dim) },
                        };
                    }
                    Cell::QueuedDelete(val, id, offset, countdown, chain) => {
                        let xb = dim + x as u32 * dim;
                        let yb = (NUM_ROWS - y) as u32 * dim - delta as u32
                            + (dim as f64 * offset) as u32;

                        match y {
                            0 => {
                                unsafe { draw_sprite(*val, 0, xb, yb, 0, 0, dim, delta as u32) };
                                if (countdown * 100.) as u32 % 2 == 0 {
                                    unsafe { draw_block(99, xb, yb, dim, delta as u32) };
                                }
                            }
                            NUM_ROWS_MIN_1 => {
                                unsafe {
                                    draw_sprite(
                                        *val,
                                        0,
                                        xb,
                                        yb + delta as u32,
                                        0,
                                        delta as u32,
                                        dim,
                                        dim - delta as u32,
                                    )
                                };
                                if (countdown * 100.) as u32 % 2 == 0 {
                                    unsafe {
                                        draw_block(
                                            99,
                                            xb,
                                            yb + delta as u32,
                                            dim,
                                            dim - delta as u32,
                                        )
                                    };
                                }
                            }
                            _ => {
                                unsafe { draw_sprite(*val, 0, xb, yb, 0, 0, dim, dim) };
                                if (countdown * 100.) as u32 % 2 == 0 {
                                    unsafe { draw_block(99, xb, yb, dim, dim) };
                                }
                            }
                        };
                        let c = chain.get_value();
                        if c.is_some() {
                            if !multipliers.contains_key(id) {
                                multipliers.insert(
                                    id,
                                    Multiplier {
                                        x: xb,
                                        y: yb,
                                        value: c.unwrap(),
                                    },
                                );
                            }
                        }
                    }
                    Cell::DeathAnim(id, offset, _, _) => {
                        let xb = dim + x as u32 * dim;
                        let yb = (NUM_ROWS - y) as u32 * dim - delta as u32
                            + (dim as f64 * offset) as u32;

                        match y {
                            0 => unsafe { draw_sprite(*id, 0, xb, yb, 0, 0, dim, delta as u32) },
                            NUM_ROWS_MIN_1 => unsafe {
                                draw_sprite(
                                    *id,
                                    0,
                                    xb,
                                    yb,
                                    0,
                                    delta as u32,
                                    dim,
                                    dim - delta as u32,
                                )
                            },
                            _ => unsafe { draw_sprite(*id, 0, xb, yb, 0, 0, dim, dim) },
                        };
                    }
                    Cell::Garbage(_id, offset) => {
                        let offset_v = match offset {
                            Some(v) => *v,
                            None => 0.,
                        };
                        let xb = dim + x as u32 * dim;
                        let yb = (NUM_ROWS - y) as u32 * dim - delta as u32
                            + (dim as f64 * offset_v) as u32;

                        match y {
                            0 => unsafe { draw_sprite(99, 0, xb, yb, 0, 0, dim, delta as u32) },
                            NUM_ROWS_MIN_1 => unsafe {
                                draw_sprite(
                                    99,
                                    0,
                                    xb,
                                    yb,
                                    0,
                                    delta as u32,
                                    dim,
                                    dim - delta as u32,
                                )
                            },
                            _ => unsafe { draw_sprite(99, 0, xb, yb, 0, 0, dim, dim) },
                        };
                    }                    
                    _ => {}
                }
            }
        }

        for (_, m) in &multipliers {
            unsafe { draw_multiplier(m.x, m.y, m.value as u32) };
        }
    }

    pub fn draw_cursor(&self, board: &Board) {
        let dim = (self.screen_height) / 12;
        let delta = board.delta * dim as f64;
        const NUM_ROWS_MIN_1: usize = NUM_ROWS - 1;
        let xb = dim + board.user_col as u32 * dim;
        let yb = (NUM_ROWS - board.user_row) as u32 * dim - delta as u32;

        match board.user_row {
            0 => unsafe { draw_cursor_blocks(xb, yb, dim, delta as u32) },
            NUM_ROWS_MIN_1 => unsafe {
                draw_cursor_blocks(xb, yb + delta as u32, dim, dim - delta as u32)
            },
            _ => unsafe { draw_cursor_blocks(xb, yb, dim, dim) },
        };
    }

    pub fn draw_ui(&self, board: &Board, enemy: &Enemy) {
        unsafe {draw_enemy_ui(enemy.countdown as u32)};
        let c = board.chain.get_value();
        if c.is_some() {
            unsafe { draw_multiplier_ui(c.unwrap() as u32) };
        }
    }

    pub unsafe fn draw(&self, game_state: GameState, game: &Game, _dt: f64) {
        clear_screen();

        let dim = self.screen_height / 12;
        unsafe { draw_borders(dim, dim, dim * (NUM_COLS + 1) as u32, dim * NUM_ROWS as u32) };

        match game_state {
            GameState::Intro(_) => {
                draw_intro();
            }
            GameState::Playing => {
                self.draw_board(&game.board);
                self.draw_cursor(&game.board);
                self.draw_ui(&game.board,&game.enemy);
            }
            GameState::_Death(_) => {}
            GameState::CheckHighScore => {}
            GameState::WaitHighScore => {}
            GameState::ShowHighScore(_, _, _) => {
                draw_name_picker(game.letter_index, game.cur_letter);
            }
            GameState::_GameOver(_) => {}
        }
    }
}
