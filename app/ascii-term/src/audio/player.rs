//! オーディオ再生の制御

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use anyhow::Result;
use crossbeam_channel::{Sender, unbounded};
use rodio::{OutputStream, Sink};

use codec::MediaFile;

use super::decode_loop::decode_audio_loop;
use super::source::DirectAudioSource;

pub struct AudioPlayer {
    _stream: OutputStream,
    sink: Sink,
    is_muted: Arc<AtomicBool>,
    original_volume: f32,
    _audio_sender: Option<Sender<Vec<f32>>>,
    decoder_thread: Option<thread::JoinHandle<()>>,
    stop_signal: Arc<AtomicBool>,
    _is_finished: Arc<AtomicBool>,
    sample_rate: u32,
}

impl AudioPlayer {
    pub fn new(file_path: &str) -> Result<Self> {
        println!("Initializing audio player for: {}", file_path);

        let media_file = MediaFile::open(file_path)?;
        if !media_file.info.has_audio {
            return Err(anyhow::anyhow!("Media file has no audio stream"));
        }

        let sample_rate = media_file.info.sample_rate.unwrap_or(44100);
        let channels = media_file.info.channels.unwrap_or(2);

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
            _audio_sender: Some(audio_sender),
            decoder_thread: Some(decoder_thread),
            stop_signal,
            _is_finished: is_finished,
            sample_rate,
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
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        self.stop_signal.store(true, Ordering::Relaxed);
        if let Some(thread) = self.decoder_thread.take() {
            let _ = thread.join();
        }
    }
}
