//! バックグラウンドのオーディオデコードループとオーディオシステム診断

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use anyhow::Result;
use crossbeam_channel::Sender;
use rodio::OutputStream;

use codec::audio::AudioDecoder;

pub(super) fn decode_audio_loop(
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
            Ok(Some(frame)) => match frame.samples_as_f32() {
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
            },
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
    let final_audio_time = total_samples_sent as f64 / (sample_rate as f64 * channels as f64);
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
