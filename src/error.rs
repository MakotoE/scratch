use super::*;

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
        WrappedError(error: Box<Error>, file: &'static str, line: u32) {
            description("wrapped error")
            display("{}\n{}:{}", error, file, line)
        }

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

        File(error: Box<Error>, file: String) {
            description("file error")
            display("{}: {}", file, error.to_string())
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

macro_rules! wrap_err {
    ($e:expr) => {
        Error::from_kind(ErrorKind::WrappedError(
            Box::new($e.into()),
            file! {},
            line! {},
        ));
    };
}
