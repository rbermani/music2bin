mod bin_decoder;
mod bin_encoder;
mod bin_to_ir;
mod ir_to_bin;

pub use bin_encoder::{MusicEncoder, MUSIC_ELEMENT_LENGTH};
pub use bin_to_ir::bin_to_ir;
pub use ir_to_bin::ir_to_bin;
