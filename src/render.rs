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

    pub fn draw_board(board: &Board) {
        unsafe {draw_block(0,100,100,100,100)};
    }

    pub unsafe fn draw(&mut self, game_state: GameState, game: &Game, dt: f64) {
        clear_screen();

        match game_state {
            GameState::Intro(_) => {
                draw_intro();
            }
            GameState::Playing => {
                RenderData::draw_board(&game.board);
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
