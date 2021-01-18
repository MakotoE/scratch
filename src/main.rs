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

#[derive(clap::Clap)]
#[clap(name = "scratch")]
struct Options {
    file_path: String,
}

fn main() {
    env_logger::Builder::new()
        .filter_module("scratch", log::LevelFilter::Trace)
        .init();

    let result = || -> Result<()> {
        use clap::Clap;
        let options = Options::parse();
        app::app(std::path::Path::new(&options.file_path))
    }();
    match result {
        Ok(_) => {}
        Err(e) => {
            log::error!("{:?}", e);
            std::process::exit(1);
        }
    }
}
