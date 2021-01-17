#[macro_use]
extern crate conrod_core;

mod app;
mod blocks;
mod broadcaster;
mod canvas;
mod coordinate;
mod error;
mod event_sender;
mod file;
mod interface;
mod pen;
mod runtime;
mod sprite;
mod sprite_runtime;
mod thread;
mod traced_rwlock;
mod vm;

use anyhow::Result;
use error::*;

fn main() -> Result<()> {
    app::app()
}
