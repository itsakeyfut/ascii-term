use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use anyhow::Result;
use crossbeam_channel::{Receiver, RecvTimeoutError, Sender, unbounded};
use rodio::{OutputStream, Sink, Source};

use codec::MediaFile;
use codec::audio::AudioDecoder;

struct DirectAudioSource {
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
    fn new(
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
                        loop {
                            match self.receiver.try_recv() {
                                Ok(data) => {
                                    self.current_data = data;
                                    self.position = 0;
                                    break;
                                }
                                Err(_) => {
                                    println!(
                                        "DirectAudioSource: Stream ended, played {:.1}s",
                                        self.total_samples_played as f64
                                            / (self.sample_rate as f64 * self.channels as f64)
                                    );
                                    return None;
                                }
                            }
                        }
                    } else {
                        self.buffer_underrun_count += 1;
                        if self.buffer_underrun_count % 200 == 0 {
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

pub struct AudioPlayer {
    _stream: OutputStream,
    sink: Sink,
    is_muted: Arc<AtomicBool>,
    original_volume: f32,
    audio_sender: Option<Sender<Vec<f32>>>,
    decoder_thread: Option<thread::JoinHandle<()>>,
    stop_signal: Arc<AtomicBool>,
    is_finished: Arc<AtomicBool>,
    sample_rate: u32,
    channels: u16,
}

impl AudioPlayer {
    pub fn new(file_path: &str) -> Result<Self> {
        println!("Initializing audio player for: {}", file_path);

        let media_file = MediaFile::open(file_path)?;
        if !media_file.info.has_audio {
            return Err(anyhow::anyhow!("Media file has no audio stream"));
        }

        let sample_rate = media_file.info.sample_rate.unwrap_or(44100);
        let channels = media_file.info.channels.unwrap_or(2) as u16;

        println!(
            "Media file info: {} Hz, {} channels, duration: {:?}",
            sample_rate, channels, media_file.info.duration
        );

        let (_stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| anyhow::anyhow!("Failed to initialize audio stream: {}", e))?;

        let sink = Sink::try_new(&stream_handle)
            .map_err(|e| anyhow::anyhow!("Failed to create audio sink: {}", e))?;

        let (audio_sender, audio_receiver) = unbounded();
        let stop_signal = Arc::new(AtomicBool::new(false));
        let is_finished = Arc::new(AtomicBool::new(false));

        let audio_source =
            DirectAudioSource::new(audio_receiver, sample_rate, channels, is_finished.clone());

        sink.append(audio_source);
        sink.set_volume(1.0);
        sink.pause();

        let file_path_clone = file_path.to_string();
        let decoder_stop_signal = stop_signal.clone();
        let decoder_sender = audio_sender.clone();
        let decoder_is_finished = is_finished.clone();
        let expected_duration = media_file.info.duration;

        let decoder_thread = thread::spawn(move || {
            decode_audio_loop(
                file_path_clone,
                sample_rate,
                channels,
                decoder_sender,
                decoder_stop_signal,
                decoder_is_finished,
                expected_duration,
            );
        });

        println!("Audio player initialized successfully");

        Ok(Self {
            _stream,
            sink,
            is_muted: Arc::new(AtomicBool::new(false)),
            original_volume: 1.0,
            audio_sender: Some(audio_sender),
            decoder_thread: Some(decoder_thread),
            stop_signal,
            is_finished,
            sample_rate,
            channels,
        })
    }

    pub fn play(&mut self) -> Result<()> {
        println!("Starting audio playback at {} Hz", self.sample_rate);
        self.sink.play();
        Ok(())
    }

    pub fn pause(&mut self) -> Result<()> {
        println!("Pausing audio playback");
        self.sink.pause();
        Ok(())
    }

    pub fn resume(&mut self) -> Result<()> {
        println!("Resuming audio playback");
        self.sink.play();
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        println!("Stopping audio playback");
        self.stop_signal.store(true, Ordering::Relaxed);
        self.sink.stop();

        if let Some(thread) = self.decoder_thread.take() {
            let _ = thread.join();
        }

        Ok(())
    }

    pub fn set_volume(&mut self, volume: f32) -> Result<()> {
        let clamped_volume = volume.clamp(0.0, 1.0);
        self.original_volume = clamped_volume;

        if !self.is_muted.load(Ordering::Relaxed) {
            self.sink.set_volume(clamped_volume);
        }

        println!("Volume set to: {:.2}", clamped_volume);
        Ok(())
    }

    pub fn volume(&self) -> f32 {
        if self.is_muted.load(Ordering::Relaxed) {
            0.0
        } else {
            self.original_volume
        }
    }

    pub fn mute(&mut self) -> Result<()> {
        println!("Muting audio");
        self.is_muted.store(true, Ordering::Relaxed);
        self.sink.set_volume(0.0);
        Ok(())
    }

    pub fn unmute(&mut self) -> Result<()> {
        println!("Unmuting audio");
        self.is_muted.store(false, Ordering::Relaxed);
        self.sink.set_volume(self.original_volume);
        Ok(())
    }

    pub fn toggle_mute(&mut self) -> Result<()> {
        if self.is_muted.load(Ordering::Relaxed) {
            self.unmute()
        } else {
            self.mute()
        }
    }

    pub fn is_playing(&self) -> bool {
        !self.sink.is_paused()
    }

    pub fn is_muted(&self) -> bool {
        self.is_muted.load(Ordering::Relaxed)
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channels(&self) -> u16 {
        self.channels
    }
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        self.stop_signal.store(true, Ordering::Relaxed);
        if let Some(thread) = self.decoder_thread.take() {
            let _ = thread.join();
        }
    }
}

fn decode_audio_loop(
    file_path: String,
    sample_rate: u32,
    channels: u16,
    sender: Sender<Vec<f32>>,
    stop_signal: Arc<AtomicBool>,
    is_finished: Arc<AtomicBool>,
    expected_duration: Option<Duration>,
) {
    println!("Audio decode loop started");

    let mut decoder = match AudioDecoder::new(&file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to create audio decoder: {}", e);
            is_finished.store(true, Ordering::Relaxed);
            return;
        }
    };

    let mut total_samples_sent = 0u64;
    let start_time = std::time::Instant::now();
    let expected_duration_secs = expected_duration.map(|d| d.as_secs_f64()).unwrap_or(0.0);

    println!("Expected duration: {:.1}s", expected_duration_secs);

    while !stop_signal.load(Ordering::Relaxed) {
        if sender.len() > 15 {
            thread::sleep(Duration::from_millis(5));
            continue;
        }

        match decoder.decode_one() {
            Ok(Some(frame)) => {
                match frame.samples_as_f32() {
                    Ok(samples) if !samples.is_empty() => {
                        total_samples_sent += samples.len() as u64;
                        if sender.send(samples).is_err() {
                            break;
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Audio frame conversion error: {}", e);
                    }
                }
            }
            Ok(None) => {
                println!("Audio stream EOF");
                break;
            }
            Err(e) => {
                eprintln!("Audio decode error: {}", e);
                break;
            }
        }
    }

    is_finished.store(true, Ordering::Relaxed);

    let final_elapsed = start_time.elapsed();
    let final_audio_time =
        total_samples_sent as f64 / (sample_rate as f64 * channels as f64);
    let coverage = if expected_duration_secs > 0.0 {
        (final_audio_time / expected_duration_secs) * 100.0
    } else {
        0.0
    };

    println!("=== Audio Decode Statistics ===");
    println!("Sample rate: {} Hz, channels: {}", sample_rate, channels);
    println!("Audio duration: {:.1}s", final_audio_time);
    println!("Expected duration: {:.1}s", expected_duration_secs);
    println!("Coverage: {:.1}%", coverage);
    println!("Real time: {:.1}s", final_elapsed.as_secs_f64());
    println!("=== End Audio Statistics ===");
}

pub fn diagnose_audio_system() -> Result<()> {
    println!("=== Audio System Diagnostics ===");

    match OutputStream::try_default() {
        Ok((_stream, _handle)) => {
            println!("✓ Default audio device is available");
        }
        Err(e) => {
            println!("✗ Default audio device failed: {}", e);
            return Err(anyhow::anyhow!("Audio system not available"));
        }
    }

    println!("=== End Diagnostics ===");
    Ok(())
}
