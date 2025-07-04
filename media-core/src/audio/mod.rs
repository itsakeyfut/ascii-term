pub mod frame;
pub mod decoder;
pub mod processor;

pub use decoder::AudioDecoder;
pub use frame::{AudioFrame, AudioFormat};
pub use processor::AudioProcessor;