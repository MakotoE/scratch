#![feature(async_closure)]
#![feature(try_trait)]
#![feature(str_split_once)]
#![recursion_limit = "512"]

#[macro_use]
mod error;
mod app;
mod blocks;
mod coordinate;
mod fileinput;
mod fileviewer;
mod interface;
mod pen;
mod runtime;
mod savefile;
mod sprite;
mod sprite_runtime;
mod thread;
mod vm;

use error::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use tokio::sync::RwLock;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen(start)]
pub fn start() -> Result<()> {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    yew::App::<app::App>::new().mount_to_body();
    Ok(())
}
