#![feature(async_closure)]
#![feature(maybe_uninit_uninit_array)]
#![feature(str_split_once)]

#[macro_use]
extern crate conrod_core;

mod app;
mod blocks;
mod broadcaster;
mod coordinate;
mod error;
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
use file::{BlockID, Image, Monitor, ScratchFile, Target};
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use tokio::spawn;
use tokio::task::JoinHandle;

#[cfg(test)]
use rstest::rstest;

pub type HashMap<K, V> = std::collections::HashMap<K, V, fnv::FnvBuildHasher>;
pub type HashSet<V> = std::collections::HashSet<V, fnv::FnvBuildHasher>;

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

fn main() {
    use clap::Clap;

    env_logger::init();

    let options = Options::parse();
    let path = std::path::Path::new(&options.file_path);

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let result = match options.command {
                Command::Vm => app::app(path).await,
                Command::Viewer => fileviewer::fileviewer(path).await,
            };
            let exit_code = match result {
                Ok(_) => 0,
                Err(e) => {
                    log::error!("{:?}", e);
                    1
                }
            };
            std::process::exit(exit_code);
        });
}
