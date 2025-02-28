use std::os::raw::{c_char, c_int,c_uint};
use std::sync::mpsc::Sender;
use crate::input::Input;
use crate::log::*;
use std::cmp::{min,max};
use std::ffi::CString;

extern "C" {

	fn check_high_score();

	fn wait_high_score() -> c_int;

	fn wait_outro_complete() -> c_int;

	fn new_session();

	fn handle_game_over();

//	fn update_local_score(_: c_int);

	fn get_rand(_: c_uint) -> c_uint;

	fn console_log_int(_: c_int);
	fn console_log_uint(_: c_uint);
}

#[derive(Clone,Copy,PartialEq)]
pub enum GameState {
	Intro(f64),
	Playing,
	Death(f64),
	CheckHighScore,
	WaitHighScore,
	ShowHighScore(bool,bool,bool),
	GameOver(f64),
}

pub enum GameEvent {
// 	ScoreChanged(i32),
}

// 12 x 6 is the size of the board

#[derive(Clone,Copy,PartialEq,Debug)]
pub enum Cell {
    Empty,
    Single(u32,f64),
    Block(u32,usize,usize), // TODO figure out how to represent this    
}

impl Cell {
    pub fn new() -> Self {
        Cell::Empty
    }
}

pub const NUM_ROWS : usize = 12;
pub const NUM_COLS : usize = 6;
type Row = [Cell;NUM_COLS];

pub struct Board {
    pub delta: f64, // scroll between 0. and 1. of a row
    pub rows: [Row;NUM_ROWS],
 	/// user cursor
	 pub user_row: usize,
	 pub user_col: usize, 
    bottom: usize, // this is the bottom row; it moves backwards through the indices
}

impl Board {
    pub fn new() -> Self {
        Board { delta: 0., rows: [[Cell::new(); NUM_COLS]; NUM_ROWS], user_row: 0, user_col: 2, bottom: 0 }
    }

	pub fn init_with_state(&mut self, level: usize) {

		for i in 0..level-1 {
			self.new_bottom_row();
			self.push_bottom_row_up();
		}
		self.new_bottom_row();
		// we don't push the last row up because otherwise we end up with a gap at the bottom
		self.user_row = level / 2;
	}

	fn clamp_to_board_width(n: i32) -> usize {
		if n < 0 {
			0
		} else if n > NUM_COLS as i32 -1 {
			NUM_COLS - 1
		} else {
			n as usize
		}
	}

	fn clamp_to_board_height(n: i32) -> usize {
		if n < 0 {
			0
		} else if n > NUM_ROWS as i32 -1 {
			NUM_ROWS - 1
		} else {
			n as usize
		}
	}

	// fn check_matches(&mut self, x: usize, y: usize) -> Vec<(usize,usize)> {
	// 	for y in 1..NUM_ROWS {
	// 		let row = self.get_row_mut(y);
	// 		for (x,cell) in row.iter_mut().enumerate() {
	// 		}
	// 	}
	// 	vec![]
	// }

	// TODO this doesn't work on the right side
	// TODO when we match 4 vertically it removes more
	fn check_matches(&mut self, x: usize, y: usize) -> Vec<(usize,usize)> {

		let mut ret = vec![];
		let c = self.rows[y][x];

		let mut last = None;
		let mut count = 1;

		let firstx = Board::clamp_to_board_width(x as i32 -2);
		let lastx = Board::clamp_to_board_width(x as i32+3);
		for xi in firstx..lastx {
			let row = self.get_row(y);
			if Some(row[xi]) == last && last != Some(Cell::Empty) {
				count += 1;
			} else {
				if count >= 3 {
					log(&format!("matched at x {} with count {}", xi, count));
					for i in 0..count {
						ret.push((xi-1-i as usize,y));
					}
				}
				count = 1;
			}
			last = Some(row[xi]);
		}
		if count >= 3 {
			log(&format!("matched at x {} with count {}", lastx, count));
			for i in 0..count {
				ret.push((lastx-1-i as usize,y));
			}
		}

		count = 1;
		last = None;
		let firsty = Board::clamp_to_board_height(y as i32 -2);
		let lasty = Board::clamp_to_board_height(y as i32 + 3);
		for yi in firsty..lasty {
			let row = self.get_row(yi);
			if Some(row[x]) == last && last != Some(Cell::Empty) {
				count += 1;
			} else {
				if count >= 3 {
					log(&format!("matched at y {} with count {}", yi, count));
					for i in 0..count {
						ret.push((x,yi-1-i as usize));
					}
				} else {
					count = 1;
				}
			}
			last = Some(row[x]);
		}
		if count >= 3 {
			log(&format!("matched at y {} with count {}", lasty, count));
			for i in 0..count {
				ret.push((x,lasty-1-i as usize));
			}
		}

		ret
	}

	fn delete_blocks(&mut self, blocks: Vec<(usize,usize)>) {
		//log(&format!("deleting blocks {:?}", blocks));
		let mut fall = vec![];
		for block in blocks {
			let row = self.get_row_mut(block.1);
			row[block.0] = Cell::Empty;
			fall.push((block.0,block.1));
		}
	}

	fn do_gravity(&mut self, dt: f64) {
		let mut queued_drop = vec![];
		for y in 1..NUM_ROWS {
			let below = self.get_row_cloned(y-1);
			let row = self.get_row_mut(y);
			for (x,cell) in row.iter_mut().enumerate() {
				//log(&format!("Below[{}][{}] = {:?}", x, y-1, below[x]));
				if below[x] == Cell::Empty {
					if let Cell::Single(v,o) = cell {
						*o += dt;
						// TODO switching up the Row format
						// if dt >= 1. {
						// 	queued_drop.push((x,y,v));
						// }
//						log(&format!("dropping {} {} {}", x,y,o));
					}
				}
			}
		}

		// TODO switching up the Row format
		// we need to increase y for every one of these
		// for q in queued_drop {
		// 	let row = self.get_row_mut(q.1 + 1);
		// 	row[q.0] = Cell::Single(*q.2,0.);
		// }
	}

	fn swap_pieces_at_cursor(&mut self) {
		let col = self.user_col;
		let row = self.get_row_mut(self.user_row);
		let tmp = row[col];
		row[col] = row[col+1];
		row[col+1] = tmp;

		let mut matches = self.check_matches(col, self.user_row);
		let mut m = self.check_matches(col+1, self.user_row);
		matches.append(&mut m);
		matches.sort();
		matches.dedup();
		self.delete_blocks(matches);
	}

    pub fn get_row(&self, index: usize) -> &Row {
        &self.rows[(self.bottom + index) % NUM_ROWS]
    }

	// copies out for inspection
	pub fn get_row_cloned(&self, index: usize) -> Row {
        self.rows[(self.bottom + index) % NUM_ROWS]
	}

	pub fn get_row_mut(&mut self, index: usize) -> &mut Row {
        // TODO scrolling will need to be taken into account here
        &mut self.rows[(self.bottom + index) % NUM_ROWS]
    }

	fn push_bottom_row_up(&mut self) {
		if self.bottom == 0 {
			self.bottom = NUM_ROWS-1;
		} else {
			self.bottom -= 1;
		}
	}

	pub fn new_bottom_row(&mut self) {
		log(&format!("bottom row at {}", self.bottom));
		// rows appear randomly not at the bottom
		let r = self.get_row_mut(0);
		r[0] = Cell::Single(unsafe{get_rand(6)},0.);
		r[1] = Cell::Single(unsafe{get_rand(6)},0.);
		r[2] = Cell::Single(unsafe{get_rand(6)},0.);
		r[3] = Cell::Single(unsafe{get_rand(6)},0.);
		r[4] = Cell::Single(unsafe{get_rand(6)},0.);
		r[5] = Cell::Single(unsafe{get_rand(6)},0.);
	}

    pub fn update(&mut self, dt: f64) {
        if self.delta >= 1. {
            self.delta = 0.;
			self.push_bottom_row_up();
			self.new_bottom_row();
			self.user_row+=1;
        }
        self.delta += dt / 16.;
		self.do_gravity(dt);
    }
}

/// The data structure that contains the state of the game
pub struct Game {
    pub board: Board,
    /// The current score of the player
    pub score: i32,
	/// state of the game
	pub game_state: GameState,

	last_left: (bool,f64),
	last_right: (bool,f64),
	last_up: (bool,f64),
	last_down: (bool,f64),
	action_pressed: bool,

	/// name picker
	pub letter_index: i32,
	pub cur_letter: i32,

	/// Events to other parts of the system
	sender: Sender<GameEvent>,
}

impl Game {
    /// Returns a new `Game` containing a `World` of the given `Size`
    pub fn new(tx: Sender<GameEvent>) -> Game {
        Game {
            board: Board::new(),
            score: 0,
			game_state: GameState::Intro(0.5),
			last_left: (false,0.),
			last_right: (false,0.),
			last_up: (false,0.),
			last_down: (false,0.),
			action_pressed: false,
			letter_index: 0,
			cur_letter: 0,
			sender: tx,
        }
    }

	fn swap_pieces(&mut self) {
		self.board.swap_pieces_at_cursor();
	}

	pub fn send_game_event(&mut self, event: GameEvent) {
		self.sender.send(event).expect("Wasn't able to send event");
	}

	fn do_input(cur: bool, dir: &mut (bool,f64), ) -> bool {
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
			if self.board.user_col < NUM_COLS-2 {
				self.board.user_col += 1;
			}
		}
		if Game::do_input(input.up, &mut self.last_up) {
			if self.board.user_row < NUM_ROWS-2 {
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
			},
			GameState::Playing => {
				self.handle_input(dt, input);
                self.board.update(dt);

			},
			GameState::Death(ref mut timer) => {
				*timer -= dt;
				if *timer < 0. {
					self.game_state = GameState::Playing;
				}
			},
			GameState::CheckHighScore => {
				check_high_score();
				self.game_state = GameState::WaitHighScore;
			},
			GameState::WaitHighScore => {
				let ret = wait_high_score();
				match ret {
					1 => {
//						self.reset(ResetType::New);
						self.game_state = GameState::Intro(0.5);
					},
					2 => self.game_state = GameState::ShowHighScore(false,false,false),
					_ => {},
				}
			},
			GameState::ShowHighScore(ref mut action,ref mut left,ref mut right) => {
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
			},
			GameState::GameOver(ref mut timer) => {
				if *timer >= 0. {
					*timer -= dt;
				} else {
					let ret = wait_outro_complete();
					if ret == 1 {
						self.game_state = GameState::CheckHighScore;
					}
				}
			},			
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