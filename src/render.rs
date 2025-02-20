use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::ffi::{c_int,c_uint};
use crate::game::*;

extern "C" {
    fn clear_screen();
    fn draw_intro();
    fn draw_block(_: c_int, _: c_uint, _: c_uint, _: c_uint, _: c_uint);
    fn draw_name_picker(_: c_int, _: c_int);
    fn update_local_score(_: c_int);

}

pub struct RenderData {
    screen_width: u32,
    screen_height: u32,
    receiver: Receiver<GameEvent>,
    pub sender: Sender<GameEvent>,
}

impl RenderData {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        RenderData {
            screen_width: 1024,
            screen_height: 768,
            receiver: rx,
            sender: tx,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.screen_width = width;
        self.screen_height = height;
    }

    unsafe fn handle_game_event(&mut self, event: GameEvent) {
        // match event {
        //     GameEvent::ScoreChanged(i) => {
        //         update_local_score(i);
        //     }
        // }
    }

    pub fn draw_board(&mut self, board: &Board) {

        let dim = (self.screen_height - 100) / 12;
        let delta = board.delta * dim as f64;
        const NUM_ROWS_MIN_1 : usize = NUM_ROWS - 1;

        for y in 0..NUM_ROWS {
            let row = board.get_row(y);
            for (x,cell) in row.iter().enumerate() {
                match cell {
                    Cell::Empty |  // temp
                    Cell::Single(_) => {
                        let xb = dim + x as u32 * dim;
                        let yb = (NUM_ROWS - y) as u32 * dim - delta as u32;
                        let c = (x % 2) as i32;

                        match y {
                            0 => unsafe {draw_block(c,xb,yb,dim,delta as u32)},
                            NUM_ROWS_MIN_1 => {
                                unsafe {draw_block(c,xb,yb + delta as u32,dim,dim - delta as u32)} 

                            }
                            _ => unsafe {draw_block(c,xb,yb,dim,dim)} 

                        };
                    }
                    _ => {}
                }
            }
        }
    }

    pub unsafe fn draw(&mut self, game_state: GameState, game: &Game, dt: f64) {
        clear_screen();

        match game_state {
            GameState::Intro(_) => {
                draw_intro();
            }
            GameState::Playing => {
                self.draw_board(&game.board);
            }
            GameState::Death(_) => {
            }
            GameState::CheckHighScore => {
            }
            GameState::WaitHighScore => {}
            GameState::ShowHighScore(_, _, _) => {
                draw_name_picker(game.letter_index, game.cur_letter);
            }
            GameState::GameOver(_) => {}
        }
    }
}
