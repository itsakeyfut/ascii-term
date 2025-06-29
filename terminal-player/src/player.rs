use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use anyhow::Result;
use crossbeam_channel::{unbounded, Receiver, Sender};
use tokio::time;

use media_core::{MediaFile, MediaType, video::VideoDecoder};
use crate::renderer::{AsciiRenderer, RenderConfig, RenderedFrame};

/// プレイヤー設定
pub struct PlayerConfig {
    pub fps: Option<f64>,
    pub loop_playback: bool,
    pub char_map_index: u8,
    pub grayscale: bool,
    pub width_modifier: u32,
    pub allow_frame_skip: bool,
    pub add_newlines: bool,
    pub enable_audio: bool,
}
