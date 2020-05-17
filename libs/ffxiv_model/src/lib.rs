mod character;
mod character_part;
mod constants;
mod model_reader;
mod shader_holder;
mod type_adapter;

pub use character::Character;
pub use constants::{BodyId, ModelPart};
pub use shader_holder::ShaderHolder; // TODO move this to internal state manager
