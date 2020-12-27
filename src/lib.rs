#![feature(async_closure)]
#![feature(try_trait)]
#![feature(str_split_once)]
#![recursion_limit = "512"]
#![allow(clippy::await_holding_refcell_ref)]

#[allow(unused_imports)]
use debug_macro::debug;
use error::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use tokio::sync::RwLock;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[macro_use]
mod error;
#[macro_use]
mod logger;
mod app;
mod blocks;
mod broadcaster;
mod canvas;
mod coordinate;
mod event_sender;
mod file;
mod fileinput;
mod fileviewer;
mod interface;
mod pen;
mod runtime;
mod sprite;
mod sprite_runtime;
mod thread;
mod vm;

#[wasm_bindgen(start)]
pub fn start() -> Result<()> {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    yew::App::<app::App>::new().mount_to_body();
    Ok(())
}
