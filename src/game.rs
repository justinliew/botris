/*
GAMEPLAY TODOs:

- verify that falling blocks don't match as they fall. I thought I fixed this but maybe not?
- try to not have the new row cause matches.
- you can swap out and back really quickly once, we should avoid this I think.
- don't clear the current chain if we make a non-chain match. Start a new one?
- sometimes triggering the garbage doesn't work
*/

use crate::board::*;
use crate::enemy::*;
use crate::input::Input;
use crate::log::*;
use std::os::raw::{c_int, c_uint};
use std::sync::mpsc::Sender;

extern "C" {

    fn check_high_score();

    fn wait_high_score() -> c_int;

    fn wait_outro_complete() -> c_int;

    fn new_session();

    fn handle_game_over();

    //	fn update_local_score(_: c_int);

    fn get_rand(_: c_uint) -> c_uint;

    fn _console_log_int(_: c_int);
    fn _console_log_uint(_: c_uint);
}

#[derive(Clone, Copy, PartialEq)]
pub enum GameState {
    Intro(f64),
    Playing,
    _Death(f64),
    CheckHighScore,
    WaitHighScore,
    ShowHighScore(bool, bool, bool),
    _GameOver(f64),
}

pub enum GameEvent {
    // 	ScoreChanged(i32),
}

/// The data structure that contains the state of the game
pub struct Game {
    pub board: Board,
    pub enemy: Enemy,
    /// state of the game
    pub game_state: GameState,

    last_left: (bool, f64),
    last_right: (bool, f64),
    last_up: (bool, f64),
    last_down: (bool, f64),
    action_pressed: bool,
    boost: bool,

    /// name picker
    pub letter_index: i32,
    pub cur_letter: i32,

    /// Events to other parts of the system
    _sender: Sender<GameEvent>,
}

impl Game {
    /// Returns a new `Game` containing a `World` of the given `Size`
    pub fn new(tx: Sender<GameEvent>) -> Game {
        Game {
            board: Board::new(),
            enemy: Enemy::new(),
            game_state: GameState::Intro(0.5),
            last_left: (false, 0.),
            last_right: (false, 0.),
            last_up: (false, 0.),
            last_down: (false, 0.),
            action_pressed: false,
            boost: false,
            letter_index: 0,
            cur_letter: 0,
            _sender: tx,
        }
    }

    fn swap_pieces(&mut self) {
        self.board.swap_pieces_at_cursor();
    }

    pub fn _send_game_event(&mut self, event: GameEvent) {
        self._sender.send(event).expect("Wasn't able to send event");
    }

    fn do_input(cur: bool, dir: &mut (bool, f64)) -> bool {
        if cur {
            if !dir.0 {
                dir.0 = true;
                dir.1 = 0.;
                return true;
            }
            if dir.0 && dir.1 < 0.5 {
                return false;
            } else if dir.0 && dir.1 > 0.5 {
                return true;
            }
        } else {
            dir.0 = false;
            dir.1 = 0.;
        }
        return false;
    }

    fn is_over_captcha(&self) -> bool {
        self.board.is_over_captcha()
    }

    pub fn handle_input(&mut self, dt: f64, input: &Input) {

        // if we are over a captcha, it eats the input until it's done
        let is_captcha = self.is_over_captcha();

        if Game::do_input(input.left, &mut self.last_left) {
            if is_captcha {
                self.board.do_captcha_input(CaptchaInput::LEFT);
            } else {
                if self.board.user_col > 0 {
                    self.board.user_col -= 1;
                }    
            }
        }
        if Game::do_input(input.right, &mut self.last_right) {
            if is_captcha {
                self.board.do_captcha_input(CaptchaInput::RIGHT);
            } else {
                if self.board.user_col < NUM_COLS - 2 {
                    self.board.user_col += 1;
                }    
            }
        }
        if Game::do_input(input.up, &mut self.last_up) {
            if is_captcha {
                self.board.do_captcha_input(CaptchaInput::UP);
            } else {
                if self.board.user_row < NUM_ROWS - 2 {
                    self.board.user_row += 1;
                }    
            }
        }
        if Game::do_input(input.down, &mut self.last_down) {
            if is_captcha {
                self.board.do_captcha_input(CaptchaInput::DOWN);
            } else {
                if self.board.user_row > 0 {
                    self.board.user_row -= 1;
                }    
            }
        }
        if input.action {
            if !self.action_pressed {
                self.action_pressed = true;
                self.swap_pieces();
            }
        } else {
            self.action_pressed = false;
        }

        if input.alt {
            self.boost = true;
        } else {
            self.boost = false;
        }

        if self.last_left.0 {
            self.last_left.1 += dt;
        }
        if self.last_right.0 {
            self.last_right.1 += dt;
        }
        if self.last_up.0 {
            self.last_up.1 += dt;
        }
        if self.last_down.0 {
            self.last_down.1 += dt;
        }
    }

    pub unsafe fn update(&mut self, input: &Input, dt: f64) {
        match self.game_state {
            GameState::Intro(ref mut timer) => {
                if *timer >= 0. {
                    *timer -= dt;
                } else {
                    if input.any {
                        new_session();
                        self.game_state = GameState::Playing;
                    }
                }
            }
            GameState::Playing => {
                self.handle_input(dt, input);
                if let Some(a) = self.board.update(dt, self.boost) {
                    self.enemy.attack(a);
                }
                if self.enemy.update(dt) {
                    self.board.attack();
                }
            }
            GameState::_Death(ref mut timer) => {
                *timer -= dt;
                if *timer < 0. {
                    self.game_state = GameState::Playing;
                }
            }
            GameState::CheckHighScore => {
                check_high_score();
                self.game_state = GameState::WaitHighScore;
            }
            GameState::WaitHighScore => {
                let ret = wait_high_score();
                match ret {
                    1 => {
                        //						self.reset(ResetType::New);
                        self.game_state = GameState::Intro(0.5);
                    }
                    2 => self.game_state = GameState::ShowHighScore(false, false, false),
                    _ => {}
                }
            }
            GameState::ShowHighScore(ref mut action, ref mut left, ref mut right) => {
                let mut advance = false;
                if !input.action && *action {
                    if self.letter_index == 2 {
                        advance = true;
                    } else {
                        self.letter_index += 1;
                        self.cur_letter = 0;
                    }
                }
                if !input.left && *left {
                    self.cur_letter -= 1;
                    if self.cur_letter < 0 {
                        self.cur_letter = 25;
                    }
                }
                if !input.right && *right {
                    self.cur_letter += 1;
                    if self.cur_letter > 25 {
                        self.cur_letter = 0;
                    }
                }
                *action = input.action;
                *left = input.left;
                *right = input.right;
                if advance {
                    handle_game_over();
                    //					self.reset(ResetType::New);
                    self.game_state = GameState::Intro(0.5);
                }
            }
            GameState::_GameOver(ref mut timer) => {
                if *timer >= 0. {
                    *timer -= dt;
                } else {
                    let ret = wait_outro_complete();
                    if ret == 1 {
                        self.game_state = GameState::CheckHighScore;
                    }
                }
            }
        }
    }
}

pub struct GameData {
    pub game: Game,
    pub input: Input,
}

impl GameData {
    pub fn new(tx: Sender<GameEvent>) -> GameData {
        let game = Game::new(tx);
        GameData {
            game: game,
            input: Input::default(),
        }
    }
}
