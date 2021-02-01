#![feature(async_closure)]

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
mod vm;

use anyhow::{Error, Result};
use error::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use std::sync::Arc;
use tokio::spawn;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

#[derive(clap::Clap)]
#[clap(name = "scratch")]
struct Options {
    file_path: String,
}

#[tokio::main]
async fn main() {
    use clap::Clap;

    env_logger::Builder::new()
        .filter_module("scratch", log::LevelFilter::Trace)
        .init();

    let options = Options::parse();
    match app::app(std::path::Path::new(&options.file_path)).await {
        Ok(_) => {}
        Err(e) => {
            log::error!("{:?}", e);
            std::process::exit(1);
        }
    }
}
