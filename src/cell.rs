use crate::chain::*;

type FallOffset = f64;
type DeleteCountdown = f64;
type DeathAnimFuture = f64;
type DeathAnimCountdown = f64;

#[derive(Clone, Copy, Debug)]
pub enum Cell {
    Empty,
    Single(u32, Option<FallOffset>),
    QueuedDelete(u32, u32, FallOffset, DeleteCountdown, Chain),
    DeathAnim(u32, FallOffset, DeathAnimFuture, DeathAnimCountdown),
    _Block(u32, usize, usize), // TODO figure out how to represent this
}

impl Cell {
    pub fn new() -> Self {
        Cell::Empty
    }

    pub fn get_val(&self) -> u32 {
        match self {
            Cell::Empty => 0,
            Cell::Single(v, _) => *v,
            Cell::QueuedDelete(v, _, _, _, _) => *v,
            Cell::DeathAnim(v, _, _, _) => *v,
            Cell::_Block(v, _, _) => *v,
        }
    }

    pub fn get_fall_offset(&self) -> f64 {
        match self {
            Cell::Single(_, o) => {
                if o.is_some() {
                    o.unwrap()
                } else {
                    0.
                }
            }
            Cell::QueuedDelete(_, _, o, _, _) => *o,
            Cell::DeathAnim(_, o, _, _) => *o,
            _ => 0.,
        }
    }
}

impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
        if !matches!(self, _other) {
            return false;
        }

        match self {
            Cell::Empty => false,
            Cell::Single(v, f) => {
                if let Cell::Single(v2, f2) = other {
                    v == v2 && f.is_none() && f2.is_none() // we don't want to ever match falling blocks
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}
