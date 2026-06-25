pub mod dds;
pub mod error;
pub mod spr;
pub mod swizzle;
pub mod tex;
mod c_api;

pub type PitaResult<T> = Result<T, error::PitaError>;
