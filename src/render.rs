use crate::game::*;
use crate::log::*;
use std::ffi::{c_int, c_uint};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

extern "C" {
    fn clear_screen();
    fn draw_intro();
    fn draw_block(_: c_uint, _: c_uint, _: c_uint, _: c_uint, _: c_uint);
    fn draw_cursor_blocks(_: c_uint, _: c_uint, _: c_uint, _: c_uint);
    fn draw_name_picker(_: c_int, _: c_int);
    fn _update_local_score(_: c_int);

}

pub struct RenderData {
    screen_width: u32,
    screen_height: u32,
    _receiver: Receiver<GameEvent>,
    pub sender: Sender<GameEvent>,
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
        let dim = (self.screen_height - 100) / 12;
        let delta = board.delta * dim as f64;
        const NUM_ROWS_MIN_1: usize = NUM_ROWS - 1;

        for y in 0..NUM_ROWS {
            for x in 0..NUM_COLS {
                match board.get_cell(x, y) {
                    Cell::Single(id, offset) => {
                        let xb = dim + x as u32 * dim;
                        let yb = (NUM_ROWS - y) as u32 * dim - delta as u32
                            + (dim as f64 * offset) as u32;

                        match y {
                            0 => unsafe { draw_block(*id, xb, yb, dim, delta as u32) },
                            NUM_ROWS_MIN_1 => unsafe {
                                draw_block(*id, xb, yb + delta as u32, dim, dim - delta as u32)
                            },
                            _ => unsafe { draw_block(*id, xb, yb, dim, dim) },
                        };
                    }
                    Cell::QueuedDelete(id, offset, countdown) => {
                        let xb = dim + x as u32 * dim;
                        let yb = (NUM_ROWS - y) as u32 * dim - delta as u32
                            + (dim as f64 * offset) as u32;

                        match y {
                            0 => {
                                unsafe { draw_block(*id, xb, yb, dim, delta as u32) };
                                if (countdown * 100.) as u32 % 2 == 0 {
                                    unsafe { draw_block(99, xb, yb, dim, delta as u32) };
                                }
                            }
                            NUM_ROWS_MIN_1 => {
                                unsafe {
                                    draw_block(*id, xb, yb + delta as u32, dim, dim - delta as u32)
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
                                unsafe { draw_block(*id, xb, yb, dim, dim) };
                                if (countdown * 100.) as u32 % 2 == 0 {
                                    unsafe { draw_block(99, xb, yb, dim, dim) };
                                }
                            }
                        };
                    }

                    _ => {}
                }
            }
        }
    }

    pub fn draw_cursor(&self, board: &Board) {
        let dim = (self.screen_height - 100) / 12;
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

    pub unsafe fn draw(&self, game_state: GameState, game: &Game, _dt: f64) {
        clear_screen();

        match game_state {
            GameState::Intro(_) => {
                draw_intro();
            }
            GameState::Playing => {
                self.draw_board(&game.board);
                self.draw_cursor(&game.board);
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
