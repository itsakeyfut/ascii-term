use std::io::{BufReader, Read};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use anyhow::Result;
use crossbeam_channel::{Receiver, RecvTimeoutError, Sender, unbounded};
use rodio::{OutputStream, Sink, Source};

use media_core::MediaFile;

struct FFmpegAudioStream {
    process: std::process::Child,
    reader: BufReader<std::process::ChildStdout>,
    sample_rate: u32,
    channels: u16,
    bytes_per_sample: usize,
}

impl FFmpegAudioStream {
    fn new(file_path: &str, sample_rate: u32, channels: u16) -> Result<Self> {
        println!("Starting corrected FFmpeg audio stream for: {}", file_path);

        let mut cmd = Command::new("ffmpeg");

        cmd.args([
            "-i",
            file_path,
            "-vn", // Disable video
            "-f",
            "f32le",
            "-acodec",
            "pcm_f32le",
            "-ar",
            &sample_rate.to_string(),
            "-ac",
            &channels.to_string(),
            "-loglevel",
            "error",
            "-", // Output stdout
        ]);

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        println!("FFmpeg command: {:?}", cmd);

        let mut process = cmd
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to start FFmpeg: {}", e))?;

        let stdout = process
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to get FFmpeg stdout"))?;

        let reader = BufReader::new(stdout);

        println!("FFmpeg corrected audio stream started successfully");

        Ok(Self {
            process,
            reader,
            sample_rate,
            channels,
            bytes_per_sample: 4, // f32 = 4 bytes
        })
    }

    fn read_samples(&mut self, buffer: &mut [f32]) -> Result<usize> {
        let byte_buffer_size = buffer.len() * self.bytes_per_sample;
        let mut byte_buffer = vec![0u8; byte_buffer_size];

        let mut total_bytes_read = 0;

        // Loop until the required number of bytes are read
        while total_bytes_read < byte_buffer_size {
            match self
                .reader
                .get_mut()
                .read(&mut byte_buffer[total_bytes_read..])
            {
                Ok(0) => {
                    // EOF
                    break;
                }
                Ok(bytes_read) => {
                    total_bytes_read += bytes_read;
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        // Non-blocking IO, wait a bit
                        thread::sleep(Duration::from_millis(1));
                        continue;
                    } else {
                        return Err(anyhow::anyhow!("FFmpeg read error: {}", e));
                    }
                }
            }
        }

        if total_bytes_read == 0 {
            return Ok(0); // EOF
        }

        // Convert bytes to f32
        let samples_read = total_bytes_read / self.bytes_per_sample;

        for i in 0..samples_read {
            let byte_offset = i * self.bytes_per_sample;
            if byte_offset + self.bytes_per_sample <= total_bytes_read {
                let sample_bytes = &byte_buffer[byte_offset..byte_offset + self.bytes_per_sample];
                let sample = f32::from_le_bytes([
                    sample_bytes[0],
                    sample_bytes[1],
                    sample_bytes[2],
                    sample_bytes[3],
                ]);
                buffer[i] = if sample.is_finite() { sample } else { 0.0 };
            }
        }

        Ok(samples_read)
    }

    fn is_finished(&mut self) -> bool {
        match self.process.try_wait() {
            Ok(Some(status)) => {
                println!("FFmpeg process finished with status: {:?}", status);
                true
            }
            Ok(None) => false,
            Err(e) => {
                println!("FFmpeg process error: {}", e);
                true
            }
        }
    }
}

impl Drop for FFmpegAudioStream {
    fn drop(&mut self) {
        println!("Terminating FFmpeg process");
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

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
        // If current data is empty, get new data
        if self.position >= self.current_data.len() {
            match self.receiver.recv_timeout(Duration::from_millis(500)) {
                Ok(data) => {
                    self.current_data = data;
                    self.position = 0;
                    self.buffer_underrun_count = 0;
                }
                Err(RecvTimeoutError::Timeout) => {
                    if self.is_finished.load(Ordering::Relaxed) {
                        // Process all remaining data
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
                        // No audio is returned on timeout
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
        println!(
            "Initializing corrected FFmpeg audio player for: {}",
            file_path
        );

        // Get audio info from MediaFile
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

        // Initialize audio stream
        let (_stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| anyhow::anyhow!("Failed to initialize audio stream: {}", e))?;

        // Create sink
        let sink = Sink::try_new(&stream_handle)
            .map_err(|e| anyhow::anyhow!("Failed to create audio sink: {}", e))?;

        // Create channel and signal
        let (audio_sender, audio_receiver) = unbounded();
        let stop_signal = Arc::new(AtomicBool::new(false));
        let is_finished = Arc::new(AtomicBool::new(false));

        // Create audio source
        let audio_source =
            DirectAudioSource::new(audio_receiver, sample_rate, channels, is_finished.clone());

        // Add audio source to sink
        sink.append(audio_source);
        sink.set_volume(1.0);
        sink.pause();

        // Start decoder thread
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

        println!("FFmpeg audio player initialized successfully");

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
        println!("Starting FFmpeg audio playback at {} Hz", self.sample_rate);
        self.sink.play();
        Ok(())
    }

    pub fn pause(&mut self) -> Result<()> {
        println!("Pausing FFmpeg audio playback");
        self.sink.pause();
        Ok(())
    }

    pub fn resume(&mut self) -> Result<()> {
        println!("Resuming FFmpeg audio playback");
        self.sink.play();
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        println!("Stopping FFmpeg audio playback");
        self.stop_signal.store(true, Ordering::Relaxed);
        self.sink.stop();

        // Wait for decoder thread to finish
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
    println!("FFmpeg decode loop started");

    // Start FFmpeg stream
    let mut ffmpeg_stream = match FFmpegAudioStream::new(&file_path, sample_rate, channels) {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("Failed to create FFmpeg stream: {}", e);
            is_finished.store(true, Ordering::Relaxed);
            return;
        }
    };

    let mut total_samples_sent = 0;
    let start_time = std::time::Instant::now();
    let mut buffer = vec![0f32; 4096]; // 4096 samples buffer
    let mut read_count = 0;
    let mut consecutive_zero_reads = 0;

    let expected_duration_secs = expected_duration.map(|d| d.as_secs_f64()).unwrap_or(0.0);

    const MAX_CONSECUTIVE_ZERO_READS: u32 = 100;

    println!("Expected duration: {:.1}s", expected_duration_secs);

    while !stop_signal.load(Ordering::Relaxed) {
        // Manage buffer size
        let buffer_size = sender.len();
        if buffer_size > 15 {
            thread::sleep(Duration::from_millis(5));
            continue;
        }

        match ffmpeg_stream.read_samples(&mut buffer) {
            Ok(samples_read) => {
                if samples_read == 0 {
                    consecutive_zero_reads += 1;
                    if consecutive_zero_reads >= MAX_CONSECUTIVE_ZERO_READS {
                        println!("Too many consecutive zero reads, FFmpeg likely finished");
                        break;
                    }
                    thread::sleep(Duration::from_millis(10));
                    continue;
                } else {
                    consecutive_zero_reads = 0;
                }

                read_count += 1;

                let samples_to_send = buffer[..samples_read].to_vec();
                if sender.send(samples_to_send).is_ok() {
                    total_samples_sent += samples_read;
                } else {
                    println!("Corrected FFmpeg sender channel closed");
                    break;
                }

                if read_count <= 3 || read_count % 500 == 0 {
                    println!("FFmpeg read {}: {} samples", read_count, samples_read);
                }
            }
            Err(e) => {
                eprintln!("Corrected FFmpeg read error: {}", e);
                if ffmpeg_stream.is_finished() {
                    println!("FFmpeg process finished after error");
                    break;
                }
                thread::sleep(Duration::from_millis(50));
            }
        }
    }

    is_finished.store(true, Ordering::Relaxed);
    let final_elapsed = start_time.elapsed();
    let final_audio_time = total_samples_sent as f64 / (sample_rate as f64 * channels as f64);
    let coverage = if expected_duration_secs > 0.0 {
        (final_audio_time / expected_duration_secs) * 100.0
    } else {
        0.0
    };

    println!("=== FFmpeg Direct Audio Final Statistics ===");
    println!("Sample rate: {} Hz, channels: {}", sample_rate, channels);
    println!("Total samples: {}", total_samples_sent);
    println!("Audio duration: {:.1}s", final_audio_time);
    println!("Expected duration: {:.1}s", expected_duration_secs);
    println!("Coverage: {:.1}%", coverage);
    println!("Real time: {:.1}s", final_elapsed.as_secs_f64());
    println!("Read operations: {}", read_count);
    println!("Consecutive zero reads: {}", consecutive_zero_reads);

    if coverage >= 95.0 {
        println!("SUCCESS: FFmpeg audio decoded successfully");
    } else if coverage >= 80.0 {
        println!("PARTIAL: FFmpeg audio decoded partially");
    } else {
        println!("WARNING: FFmpeg audio coverage is low");
    }

    println!("=== End FFmpeg Statistics ===");
}

pub fn diagnose_audio_system() -> Result<()> {
    println!("=== Audio System Diagnostics ===");

    match Command::new("ffmpeg").arg("-version").output() {
        Ok(output) => {
            if output.status.success() {
                println!("✓ FFmpeg is available");
                let version = String::from_utf8_lossy(&output.stdout);
                if let Some(first_line) = version.lines().next() {
                    println!("  {}", first_line);
                }
            } else {
                println!("✗ FFmpeg command failed");
            }
        }
        Err(_) => {
            println!("✗ FFmpeg not found");
            return Err(anyhow::anyhow!("FFmpeg not available"));
        }
    }

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
