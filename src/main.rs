#[macro_use]
extern crate conrod_core;

mod app;
mod error;
mod interface;

use anyhow::Result;
use error::*;

fn main() -> Result<()> {
    app::app()
}
