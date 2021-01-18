#[macro_use]
extern crate conrod_core;

mod app;
mod blocks;
mod broadcaster;
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

use anyhow::{Error, Result};
use error::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use tokio::sync::RwLock;

fn main() -> Result<()> {
    app::app()
}
