use std::ffi::{c_char, c_double, c_uint, c_int};
use std::sync::Mutex;

mod input;
mod game;
mod render;

use render::*;
use game::*;

extern "C" {
    fn clear_screen();
}

#[macro_use]
extern crate lazy_static;


lazy_static! {
	static ref RENDER: Mutex<RenderData> = Mutex::new(RenderData::new());
    static ref GAME: Mutex<GameData> = Mutex::new(GameData::new(RENDER.lock().unwrap().sender.clone()));
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let _data = &mut GAME.lock().unwrap();
	let _render = &mut RENDER.lock().unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn resize(width: c_uint, height: c_uint) {
    let render = &mut RENDER.lock().unwrap();
    render.resize(width,height);
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
pub extern "C" fn toggle_fire(b: c_int) {
    let data = &mut GAME.lock().unwrap();
    data.input.fire = int_to_bool(b);
}

#[no_mangle]
pub extern "C" fn toggle_alt(b: c_int) {
    let data = &mut GAME.lock().unwrap();
    data.input.alt = int_to_bool(b);
}
