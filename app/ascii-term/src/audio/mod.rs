//! オーディオ再生サブシステム
//!
//! - `source`: デコードスレッドから PCM を供給する rodio `Source` アダプタ
//! - `player`: 再生制御を担う `AudioPlayer`
//! - `decode_loop`: バックグラウンドのデコードループと診断

mod decode_loop;
mod player;
mod source;

pub use decode_loop::diagnose_audio_system;
pub use player::AudioPlayer;
