use super::*;
use crate::file::BlockID;

#[derive(Debug, thiserror::Error)]
pub enum ScratchError {
    #[error("option is None")]
    Option,

    #[error("initialization error (block id \"{id}\", category {category}): {error}")]
    BlockInitialization {
        id: BlockID,
        category: String,
        error: Error,
    },

    #[error("input error (block id \"{block_id}\", input \"{input_id}\"): {error}")]
    BlockInput {
        block_id: BlockID,
        input_id: String,
        error: Error,
    },

    #[error("field error (block id \"{block_id}\", field \"{field_id}\"): {error}")]
    BlockField {
        block_id: BlockID,
        field_id: String,
        error: Error,
    },

    #[error("block \"{id}\" of type {name} returned error during execution: {error}")]
    Block {
        id: BlockID,
        name: &'static str,
        error: Error,
    },
}
