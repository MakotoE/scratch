#![feature(async_closure)]
#![feature(str_split_once)]
#![feature(maybe_uninit_uninit_array)]

#[macro_use]
extern crate conrod_core;

mod app;
mod blocks;
mod broadcaster;
mod coordinate;
mod error;
mod event_sender;
mod file;
mod fileviewer;
mod interface;
mod pen;
mod runtime;
mod sprite;
mod sprite_map;
mod sprite_runtime;
mod thread;
mod vm;

use anyhow::{Error, Result};
use async_lock::RwLock;
use error::*;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use tokio::spawn;
use tokio::task::JoinHandle;

#[derive(clap::Clap)]
#[clap(name = "scratch")]
struct Options {
    command: Command,
    file_path: String,
}

#[derive(strum::EnumString)]
#[strum(serialize_all = "snake_case")]
enum Command {
    Vm,
    Viewer,
}

#[tokio::main]
async fn main() {
    use clap::Clap;

    env_logger::init();

    let options = Options::parse();
    let path = std::path::Path::new(&options.file_path);

    match options.command {
        Command::Vm => app::app(path).await,
        Command::Viewer => fileviewer::fileviewer(path).await,
    }
    .unwrap_or_else(|e| {
        log::error!("fatal: {:?}", e);
        std::process::exit(1);
    });
}
