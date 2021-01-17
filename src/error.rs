use super::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScratchError {
    #[error("option is None")]
    Option,
}
