//! デコードスレッドから PCM をストリーミングする rodio `Source` アダプタ

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crossbeam_channel::{Receiver, RecvTimeoutError};
use rodio::Source;

pub(super) struct DirectAudioSource {
    receiver: Receiver<Vec<f32>>,
    sample_rate: u32,
    channels: u16,
    current_data: Vec<f32>,
    position: usize,
    buffer_underrun_count: usize,
    is_finished: Arc<AtomicBool>,
    total_samples_played: u64,
}

impl DirectAudioSource {
    pub(super) fn new(
        receiver: Receiver<Vec<f32>>,
        sample_rate: u32,
        channels: u16,
        is_finished: Arc<AtomicBool>,
    ) -> Self {
        Self {
            receiver,
            sample_rate,
            channels,
            current_data: Vec::new(),
            position: 0,
            buffer_underrun_count: 0,
            is_finished,
            total_samples_played: 0,
        }
    }
}

impl Source for DirectAudioSource {
    fn current_frame_len(&self) -> Option<usize> {
        Some(8192)
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl Iterator for DirectAudioSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= self.current_data.len() {
            match self.receiver.recv_timeout(Duration::from_millis(500)) {
                Ok(data) => {
                    self.current_data = data;
                    self.position = 0;
                    self.buffer_underrun_count = 0;
                }
                Err(RecvTimeoutError::Timeout) => {
                    if self.is_finished.load(Ordering::Relaxed) {
                        if let Ok(data) = self.receiver.try_recv() {
                            self.current_data = data;
                            self.position = 0;
                        } else {
                            println!(
                                "DirectAudioSource: Stream ended, played {:.1}s",
                                self.total_samples_played as f64
                                    / (self.sample_rate as f64 * self.channels as f64)
                            );
                            return None;
                        }
                    } else {
                        self.buffer_underrun_count += 1;
                        if self.buffer_underrun_count.is_multiple_of(200) {
                            let played_seconds = self.total_samples_played as f64
                                / (self.sample_rate as f64 * self.channels as f64);
                            println!(
                                "Audio underrun at {:.1}s, waiting for more data...",
                                played_seconds
                            );
                        }
                        return Some(0.0);
                    }
                }
                Err(RecvTimeoutError::Disconnected) => {
                    println!(
                        "DirectAudioSource: Disconnected after {:.1}s",
                        self.total_samples_played as f64
                            / (self.sample_rate as f64 * self.channels as f64)
                    );
                    return None;
                }
            }
        }

        if self.position < self.current_data.len() {
            let sample = self.current_data[self.position];
            self.position += 1;
            self.total_samples_played += 1;
            Some(sample)
        } else {
            Some(0.0)
        }
    }
}
