use crate::cell::*;
use crate::chain::*;
use std::ffi::c_uint;
use crate::log::*;
use std::collections::HashMap;

pub const NUM_ROWS: usize = 12;
pub const NUM_COLS: usize = 6;

extern "C" {
    fn get_rand(_: c_uint) -> c_uint;
}

pub struct Board {
    pub delta: f64, // scroll between 0. and 1. of a row
    // we are storing this in rows, so to get something you go x + y*NUM_COLS
    cells: [Cell; NUM_ROWS * NUM_COLS],
    just_touched: [bool; NUM_ROWS * NUM_COLS],
    pub chain: Chain,
    chains_valid: bool,

    next_garbage_id: u32,

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
            just_touched: [false; NUM_COLS * NUM_ROWS],
            chain: Default::default(),
            chains_valid: false,
            next_garbage_id: 0,
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

    fn end_frame(&mut self) {
        self.chains_valid = false;
        self.just_touched = [false; NUM_COLS * NUM_ROWS];
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
            Cell::Single(_, d) => {
                // if are at y == 1 and we have a cell under us then it will always be stopped
                if y == 1 {
                    false
                } else {
                    d.is_some()
                }
            }
            Cell::Garbage(_, d) => {
                if y == 1 {
                    false
                } else {
                    d.is_some()
                }
            }
            _ => false,
        }
    }

    pub fn swap_cells(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, user: bool) {
        let base_y0 = (self.bottom + y0) % NUM_ROWS;
        let base_y1 = (self.bottom + y1) % NUM_ROWS;

        let c1 = self.cells[base_y0 * NUM_COLS + x0];
        let c2 = self.cells[base_y1 * NUM_COLS + x1];

        if user && matches!(c1, Cell::Garbage { .. }) || matches!(c2, Cell::Garbage { .. }) {
            return;
        }

        if c1.get_fall_offset() != 0. || c2.get_fall_offset() != 0. {
            return;
        }

        self.cells[base_y0 * NUM_COLS + x0] = self.cells[base_y1 * NUM_COLS + x1];
        self.cells[base_y1 * NUM_COLS + x1] = c1;

        if user {
            self.just_touched[base_y0 * NUM_COLS + x0] = true;
            self.just_touched[base_y1 * NUM_COLS + x1] = true;
        }
    }

    fn check_matches(&mut self) {
        let mut state = [0_u32; NUM_ROWS * NUM_COLS];

        let mut made_match = None;
        let mut match_idx = 1;
        for x in 0..NUM_COLS {
            for y in 0..NUM_ROWS {
                let c = self.get_cell(x, y);
                if let Cell::Single(_, _) = c {
                    let mut xi = 0;
                    while x + xi + 1 < NUM_COLS && self.get_cell(x + xi + 1, y) == c {
                        xi += 1;
                    }
                    if xi >= 2 {
                        made_match = Some(true);
                        for xm in 0..xi + 1 {
                            let base_y = (self.bottom + y) % NUM_ROWS;
                            if self.just_touched[base_y * NUM_COLS + x + xm] {
                                made_match = Some(false);
                            }
                            state[base_y * NUM_COLS + x + xm] = match_idx;
                        }
                        match_idx += 1;
                    }
                    let mut yi = 0;
                    while y + yi + 1 < NUM_ROWS - 1 && self.get_cell(x, y + yi + 1) == c {
                        yi += 1;
                    }
                    if yi >= 2 {
                        if made_match.is_none() {
                            made_match = Some(true);
                        }
                        for ym in 0..yi + 1 {
                            let base_y = (self.bottom + y + ym) % NUM_ROWS;
                            if self.just_touched[base_y * NUM_COLS + x] {
                                made_match = Some(false);
                            }
                            state[base_y * NUM_COLS + x] = match_idx;
                        }
                        match_idx += 1;
                    }
                }
            }
        }

        if made_match.is_some() {
            if made_match.unwrap() == true {
                if self.chains_valid {
                    self.chain.inc();
                } else {
                    self.chain.clear();
                }
            } else {
                self.chain.clear();
            }
        }

        for x in 0..NUM_COLS {
            for y in 0..NUM_ROWS {
                let base_y = (self.bottom + y) % NUM_ROWS;
                if state[base_y * NUM_COLS + x] > 0 {
                    self.delete_block(x, y, state[base_y * NUM_COLS + x], self.chain);
                }
            }
        }
    }

    fn check_queued_deletes(&mut self, dt: f64) {
        let mut count = 1;
        let mut chains_valid = false;
        for x in 0..NUM_COLS {
            for y in 0..NUM_ROWS {
                let c = self.get_cell_mut(x, y);
                if let Cell::QueuedDelete(v, _, o, countdown, _) = c {
                    if *countdown > 0. {
                        chains_valid = true;
                        *countdown -= dt;
                    }
                    if *countdown <= 0. {
                        *c = Cell::DeathAnim(*v, *o, 0.1 * count as f64, 0.); // TODO tuning var
                        count += 1;
                    }
                }
            }
        }
        if chains_valid {
            self.chains_valid = true;
        }
    }

    fn check_death_anims(&mut self, dt: f64) {
        let mut chains_valid = false;
        for x in 0..NUM_COLS {
            for y in 0..NUM_ROWS {
                let c = self.get_cell_mut(x, y);
                if let Cell::DeathAnim(_, _, b, a) = c {
                    if *a > 0. {
                        chains_valid = true;
                        *a -= dt;
                        if *a <= 0. {
                            *c = Cell::Empty;
                        }
                    } else {
                        if *b > 0. {
                            chains_valid = true;
                            *b -= dt;
                        }
                        if *b <= 0. {
                            *b = 0.;
                            *a = 0.1;
                        }
                    }
                }
            }
        }
        if chains_valid {
            self.chains_valid = true;
        }
    }

    fn delete_block(&mut self, x: usize, y: usize, idx: u32, chain: Chain) {
        let cell = self.get_cell_mut(x, y);
        let val = cell.get_val();
        let offset = cell.get_fall_offset();
        *cell = Cell::QueuedDelete(val, idx, offset, 0.5, chain); // TODO tuning var
    }

    fn do_gravity(&mut self, dt: f64) {

       let mut garbage_map : HashMap<u32, Vec<(usize,usize,bool)>> = HashMap::new();

        for y in 1..NUM_ROWS {
            for x in 0..NUM_COLS {
                if self.below_is_empty(x, y) {
                    self.chains_valid = true;
                    let cell = self.get_cell_mut(x, y);
                    let mut swap = false;
                    if let Cell::Single(_v, o) = cell {
                        if o.is_none() {
                            *o = Some(0.);
                        }
                        let prev_o = o.unwrap();
                        let mut next_o = prev_o + dt * 4.;
                        if next_o >= 1. {
                            next_o = 0.;
                            swap = true;
                        }
                        *o = Some(next_o);
                    }
                    if let Cell::Garbage(id, _) = cell {
                       garbage_map.entry(*id).or_insert(vec![]).push((x,y,true));
                    }
                    if swap {
                        self.swap_cells(x, y, x, y - 1, false);
                    }
                } else {
                    let cell = self.get_cell_mut(x, y);
                    if let Cell::Single(_, o) = cell {
                        *o = None;
                    }
                    if let Cell::Garbage(id,_) = cell {
                        garbage_map.entry(*id).or_insert(vec![]).push((x,y,false));
                    }
                }
            }
        }

        for (_, cells) in garbage_map {
            let falling = cells.iter().fold(true, |acc,c| acc && c.2);
            cells.iter().for_each(|c| {
                let mut swap = false;
                let cell = self.get_cell_mut(c.0, c.1);
                if let Cell::Garbage(_, o) = cell {
                    if falling {
                        if o.is_none() {
                            *o = Some(0.);
                        }
                        let prev_o = o.unwrap();
                        let mut next_o = prev_o + dt * 4.;
                        if next_o >= 1. {
                            next_o = 0.;
                            swap = true;
                        }
                        *o = Some(next_o);
                    } else {
                        *o = None;
                    }
                }
                if swap {
                    self.swap_cells(c.0, c.1, c.0, c.1 - 1, false);
                }
            });
        }

    }

    pub fn swap_pieces_at_cursor(&mut self) {
        self.swap_cells(
            self.user_col,
            self.user_row,
            self.user_col + 1,
            self.user_row,
            true,
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
        *self.get_cell_mut(0, 0) = Cell::Single(unsafe { get_rand(6) }, None);
        *self.get_cell_mut(1, 0) = Cell::Single(unsafe { get_rand(6) }, None);
        *self.get_cell_mut(2, 0) = Cell::Single(unsafe { get_rand(6) }, None);
        *self.get_cell_mut(3, 0) = Cell::Single(unsafe { get_rand(6) }, None);
        *self.get_cell_mut(4, 0) = Cell::Single(unsafe { get_rand(6) }, None);
        *self.get_cell_mut(5, 0) = Cell::Single(unsafe { get_rand(6) }, None);
    }

    // TODO this can crash
    pub fn new_garbage(&mut self) {
        let id = self.next_garbage_id;
        self.next_garbage_id += 1;
        *self.get_cell_mut(1,11) = Cell::Garbage(id, None);
        *self.get_cell_mut(2,11) = Cell::Garbage(id, None);
        *self.get_cell_mut(3,11) = Cell::Garbage(id, None);
        *self.get_cell_mut(4,11) = Cell::Garbage(id, None);
        *self.get_cell_mut(5,11) = Cell::Garbage(id, None);
    }

    pub fn update(&mut self, dt: f64, boost: bool) -> Option<u32> {
        self.check_queued_deletes(dt);
        self.check_death_anims(dt);
        if self.delta >= 1. {
            self.delta = 0.;
            self.push_bottom_row_up();
            self.new_bottom_row();
            self.user_row += 1;
        }
        if boost {
            self.delta += dt * 5.;
        } else {
            self.delta += dt / 16.;
        }
        self.do_gravity(dt);
        self.check_matches();
        self.end_frame();
        let c = self.chain.update(dt);
        if c.is_some() {
            return Some(c.unwrap());
        }
        return None;
    }

    pub fn attack(&mut self) {
        self.new_garbage();
    }

}
