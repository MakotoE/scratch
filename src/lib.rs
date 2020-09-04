#![feature(async_closure)]
#![feature(try_trait)]
#![feature(str_split_once)]
#![recursion_limit = "512"]

mod blocks;
mod controller;
mod page;
mod pen;
mod runtime;
mod savefile;
mod sprite;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::RwLock;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen(start)]
pub fn start() -> Result<()> {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    yew::App::<page::Page>::new().mount_to_body();
    Ok(())
}

error_chain::error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        Zip(zip::result::ZipError);
        JSON(serde_json::error::Error);
        IO(std::io::Error);
        ParseFloatError(std::num::ParseFloatError);
        ParseIntError(std::num::ParseIntError);
    }

    errors {
        Initialization(error: Box<Error>) {
            description("initialization error")
            display(
                "error during initialization: {}",
                error.to_string(),
            )
        }

        Block(block_name: &'static str, block_id: String, error: Box<Error>) {
            description("block error")
            display(
                r#"block "{}" of type {} returned error during execution: {}"#,
                block_id,
                block_name,
                error.to_string(),
            )
        }
    }
}

impl std::convert::From<wasm_bindgen::JsValue> for Error {
    fn from(v: JsValue) -> Self {
        let mut s = format!("{:?}", v);
        if let Some(stripped_prefix) = s.strip_prefix("JsValue(") {
            s = stripped_prefix.strip_suffix(")").unwrap_or(&s).to_string();
        }
        s.into()
    }
}

impl<T> std::convert::From<std::sync::PoisonError<T>> for Error {
    fn from(e: std::sync::PoisonError<T>) -> Self {
        e.to_string().into()
    }
}

impl std::convert::Into<wasm_bindgen::JsValue> for Error {
    fn into(self) -> JsValue {
        self.to_string().into()
    }
}
