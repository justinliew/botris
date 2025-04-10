use std::ffi::{c_char, c_double, c_int, c_uint, CString};
use std::sync::Mutex;

mod game;
mod board;
mod cell;
mod chain;
mod input;
mod log;
mod render;

use game::*;
use render::*;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref RENDER: Mutex<RenderData> = Mutex::new(RenderData::new());
    static ref GAME: Mutex<GameData> =
        Mutex::new(GameData::new(RENDER.lock().unwrap().sender.clone()));
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let data = &mut GAME.lock().unwrap();
    data.game.board.init_with_state(5); // TODO level
    let _render = &mut RENDER.lock().unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn resize(width: c_uint, height: c_uint) {
    let render = &mut RENDER.lock().unwrap();
    render.resize(width, height);
}

#[no_mangle]
pub unsafe extern "C" fn update(dt: c_double) {
    let data: &mut GameData = &mut GAME.lock().unwrap();

    data.game.update(&data.input, dt);
}

#[no_mangle]
pub unsafe extern "C" fn draw(dt: c_double) {
    let render = &mut RENDER.lock().unwrap();
    let data = &mut GAME.lock().unwrap();
    let game = &data.game;

    render.draw(game.game_state, game, dt);
}

fn int_to_bool(i: c_int) -> bool {
    i != 0
}

#[no_mangle]
pub extern "C" fn key_pressed(_: c_char, b: c_int) {
    let data = &mut GAME.lock().unwrap();
    data.input.any = int_to_bool(b);
}

#[no_mangle]
pub extern "C" fn toggle_left(b: c_int) {
    let data = &mut GAME.lock().unwrap();
    data.input.left = int_to_bool(b);
}

#[no_mangle]
pub extern "C" fn toggle_right(b: c_int) {
    let data = &mut GAME.lock().unwrap();
    data.input.right = int_to_bool(b);
}

#[no_mangle]
pub extern "C" fn toggle_up(b: c_int) {
    let data = &mut GAME.lock().unwrap();
    data.input.up = int_to_bool(b);
}

#[no_mangle]
pub extern "C" fn toggle_down(b: c_int) {
    let data = &mut GAME.lock().unwrap();
    data.input.down = int_to_bool(b);
}

#[no_mangle]
pub extern "C" fn toggle_action(b: c_int) {
    let data = &mut GAME.lock().unwrap();
    data.input.action = int_to_bool(b);
}

#[no_mangle]
pub extern "C" fn toggle_alt(b: c_int) {
    let data = &mut GAME.lock().unwrap();
    data.input.alt = int_to_bool(b);
}

#[no_mangle]
pub extern "C" fn dealloc_str(ptr: *mut c_char) {
    let _ = unsafe { CString::from_raw(ptr) };
}
