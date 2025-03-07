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

// 12 x 6 is the size of the board

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Cell {
    Empty,
    Single(u32, f64),
    _Block(u32, usize, usize), // TODO figure out how to represent this
}

impl Cell {
    pub fn new() -> Self {
        Cell::Empty
    }
}

pub const NUM_ROWS: usize = 12;
pub const NUM_COLS: usize = 6;

pub struct Board {
    pub delta: f64, // scroll between 0. and 1. of a row
    // we are storing this in rows, so to get something you go x + y*NUM_COLS
    cells: [Cell; NUM_ROWS * NUM_COLS],

    /// user cursor
    pub user_row: usize,
    pub user_col: usize,
    bottom: usize, // this is the bottom row; it moves backwards through the indices
}

impl Board {
    pub fn new() -> Self {
        Board {
            delta: 0.,
            cells: [Cell::new(); NUM_COLS * NUM_ROWS],
            user_row: 0,
            user_col: 2,
            bottom: 0,
        }
    }

    pub fn init_with_state(&mut self, level: usize) {
        for _i in 0..level - 1 {
            self.new_bottom_row();
            self.push_bottom_row_up();
        }
        self.new_bottom_row();
        // we don't push the last row up because otherwise we end up with a gap at the bottom
        self.user_row = level / 2;
    }

    pub fn get_cell(&self, x: usize, y: usize) -> &Cell {
        let base_y = (self.bottom + y) % NUM_ROWS;
        &self.cells[base_y * NUM_COLS + x]
    }

    pub fn get_cell_mut(&mut self, x: usize, y: usize) -> &mut Cell {
        let base_y = (self.bottom + y) % NUM_ROWS;
        &mut self.cells[base_y * NUM_COLS + x]
    }

    pub fn below_is_empty(&self, x: usize, y: usize) -> bool {
        // bottom row is on a solid foundation :)
        if y == 0 {
            return false;
        }
        let base_y = (self.bottom + y - 1) % NUM_ROWS;
        match self.cells[base_y * NUM_COLS + x] {
            Cell::Empty => true,
            Cell::Single(_, d) => d > 0.,
            _ => false,
        }
    }

    pub fn swap_cells(&mut self, x0: usize, y0: usize, x1: usize, y1: usize) {
        let base_y0 = (self.bottom + y0) % NUM_ROWS;
        let base_y1 = (self.bottom + y1) % NUM_ROWS;
        let tmp = self.cells[base_y0 * NUM_COLS + x0];
        self.cells[base_y0 * NUM_COLS + x0] = self.cells[base_y1 * NUM_COLS + x1];
        self.cells[base_y1 * NUM_COLS + x1] = tmp;
    }

    fn check_matches(&mut self) {
        let mut state = [0_u32; NUM_ROWS * NUM_COLS];

        for x in 0..NUM_COLS {
            for y in 0..NUM_ROWS {
                let c = self.get_cell(x, y);
                if let Cell::Single(_, _) = c {
                    let mut xi = 0;
                    while x + xi + 1 < NUM_COLS - 1 && self.get_cell(x + xi + 1, y) == c {
                        xi += 1;
                    }
                    if xi >= 2 {
                        for xm in 0..xi + 1 {
                            let base_y = (self.bottom + y) % NUM_ROWS;
                            state[base_y * NUM_COLS + x + xm] = 1;
                        }
                    }
                    let mut yi = 0;
                    while y + yi + 1 < NUM_ROWS - 1 && self.get_cell(x, y + yi + 1) == c {
                        yi += 1;
                    }
                    if yi >= 2 {
                        for ym in 0..yi + 1 {
                            let base_y = (self.bottom + y + ym) % NUM_ROWS;
                            state[base_y * NUM_COLS + x] = 1;
                        }
                    }
                }
            }
        }

        for x in 0..NUM_COLS {
            for y in 0..NUM_ROWS {
                let base_y = (self.bottom + y) % NUM_ROWS;
                if state[base_y * NUM_COLS + x] == 1 {
                    self.delete_block(x, y);
                }
            }
        }
    }

    fn delete_block(&mut self, x: usize, y: usize) {
        let cell = self.get_cell_mut(x, y);
        *cell = Cell::Empty;
    }

    fn do_gravity(&mut self, dt: f64) {
        for y in 1..NUM_ROWS {
            for x in 0..NUM_COLS {
                if self.below_is_empty(x, y) {
                    let cell = self.get_cell_mut(x, y);
                    if let Cell::Single(_v, o) = cell {
                        *o += dt * 4.;
                        if *o >= 1. {
                            *o = 0.;
                            self.swap_cells(x, y, x, y - 1);
                        }
                    }
                }
            }
        }
    }

    fn swap_pieces_at_cursor(&mut self) {
        self.swap_cells(
            self.user_col,
            self.user_row,
            self.user_col + 1,
            self.user_row,
        );
    }

    fn push_bottom_row_up(&mut self) {
        if self.bottom == 0 {
            self.bottom = NUM_ROWS - 1;
        } else {
            self.bottom -= 1;
        }
    }

    pub fn new_bottom_row(&mut self) {
        *self.get_cell_mut(0, 0) = Cell::Single(unsafe { get_rand(6) }, 0.);
        *self.get_cell_mut(1, 0) = Cell::Single(unsafe { get_rand(6) }, 0.);
        *self.get_cell_mut(2, 0) = Cell::Single(unsafe { get_rand(6) }, 0.);
        *self.get_cell_mut(3, 0) = Cell::Single(unsafe { get_rand(6) }, 0.);
        *self.get_cell_mut(4, 0) = Cell::Single(unsafe { get_rand(6) }, 0.);
        *self.get_cell_mut(5, 0) = Cell::Single(unsafe { get_rand(6) }, 0.);
    }

    pub fn update(&mut self, dt: f64) {
        self.check_matches();
        if self.delta >= 1. {
            self.delta = 0.;
            self.push_bottom_row_up();
            self.new_bottom_row();
            self.user_row += 1;
        }
        self.delta += dt / 16.;
        self.do_gravity(dt);
    }
}

/// The data structure that contains the state of the game
pub struct Game {
    pub board: Board,
    /// The current score of the player
    pub _score: i32,
    /// state of the game
    pub game_state: GameState,

    last_left: (bool, f64),
    last_right: (bool, f64),
    last_up: (bool, f64),
    last_down: (bool, f64),
    action_pressed: bool,

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
            _score: 0,
            game_state: GameState::Intro(0.5),
            last_left: (false, 0.),
            last_right: (false, 0.),
            last_up: (false, 0.),
            last_down: (false, 0.),
            action_pressed: false,
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

    pub fn handle_input(&mut self, dt: f64, input: &Input) {
        if Game::do_input(input.left, &mut self.last_left) {
            if self.board.user_col > 0 {
                self.board.user_col -= 1;
            }
        }
        if Game::do_input(input.right, &mut self.last_right) {
            if self.board.user_col < NUM_COLS - 2 {
                self.board.user_col += 1;
            }
        }
        if Game::do_input(input.up, &mut self.last_up) {
            if self.board.user_row < NUM_ROWS - 2 {
                self.board.user_row += 1;
            }
        }
        if Game::do_input(input.down, &mut self.last_down) {
            if self.board.user_row > 0 {
                self.board.user_row -= 1;
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
                self.board.update(dt);
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
