pub use self::pipeline::Pipeline;
pub use self::compression::Compression;

#[macro_use]
pub mod pipeline;
pub mod compression;

use Error;

/// A hook that sits between reading and writing packets.
pub trait Middleware
{
    /// Processes some data.
    fn encode_data(&mut self, data: Vec<u8>) -> Result<Vec<u8>, Error>;
    /// Un-processes some data.
    fn decode_data(&mut self, data: Vec<u8>) -> Result<Vec<u8>, Error>;
}
