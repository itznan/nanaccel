use crate::commands::types::Commands;

pub fn parse(args: &[String]) -> Result<Commands, String> {
    if args.len() < 5 {
        return Err("Usage: audio <operation> <input> <output> [options]\n\
                    Operations: volume / volume-control, denoise / noise-reduction, compress / compression, limit / limiter, eq / equalizer, pitch / pitch-shift, tempo / tempo-change, reverb, echo, bass / bass-boost, silencedetect / silence-detection, normalize / audio-normalization".to_string());
    }
    let raw_operation = args[2].to_lowercase();
    let operation = match raw_operation.as_str() {
        "volume" | "volume-control" | "volume_control" | "volumecontrol" => "volume".to_string(),
        "denoise" | "noise-reduction" | "noise_reduction" | "noisereduction" => {
            "denoise".to_string()
        }
        "compress" | "compression" => "compress".to_string(),
        "limit" | "limiter" => "limit".to_string(),
        "eq" | "equalizer" => "eq".to_string(),
        "pitch" | "pitch-shift" | "pitch_shift" | "pitch-shifting" | "pitch_shifting" => {
            "pitch".to_string()
        }
        "tempo" | "tempo-change" | "tempo_change" => "tempo".to_string(),
        "reverb" => "reverb".to_string(),
        "echo" => "echo".to_string(),
        "bass" | "bass-boost" | "bass_boost" | "bassboost" => "bass".to_string(),
        "silencedetect" | "silence-detection" | "silence_detection" | "silence-detect"
        | "silencedetection" => "silencedetect".to_string(),
        "normalize" | "audio-normalization" | "audio_normalization" | "normalization" => {
            "normalize".to_string()
        }
        other => other.to_string(),
    };
    let input = args[3].clone();
    let output = args[4].clone();

    let mut volume = None;
    let mut noise_reduction = None;
    let mut threshold = None;
    let mut ratio = None;
    let mut limit = None;
    let mut gain = None;
    let mut frequency = None;
    let mut pitch = None;
    let mut tempo = None;
    let mut loudness = None;
    let mut silence_db = None;
    let mut silence_duration = None;

    let mut i = 5;
    while i < args.len() {
        match args[i].as_str() {
            "--volume" => {
                if i + 1 < args.len() {
                    volume = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --volume".to_string());
                }
            }
            "--nr" | "--noise-reduction" => {
                if i + 1 < args.len() {
                    noise_reduction = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --noise-reduction".to_string());
                }
            }
            "--threshold" => {
                if i + 1 < args.len() {
                    threshold = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --threshold".to_string());
                }
            }
            "--ratio" => {
                if i + 1 < args.len() {
                    ratio = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --ratio".to_string());
                }
            }
            "--limit" => {
                if i + 1 < args.len() {
                    limit = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --limit".to_string());
                }
            }
            "--gain" => {
                if i + 1 < args.len() {
                    gain = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --gain".to_string());
                }
            }
            "--freq" | "--frequency" => {
                if i + 1 < args.len() {
                    frequency = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --frequency".to_string());
                }
            }
            "--pitch" => {
                if i + 1 < args.len() {
                    pitch = Some(
                        args[i + 1]
                            .parse::<f32>()
                            .map_err(|_| "Invalid pitch value")?,
                    );
                    i += 2;
                } else {
                    return Err("Missing value for --pitch".to_string());
                }
            }
            "--tempo" => {
                if i + 1 < args.len() {
                    tempo = Some(
                        args[i + 1]
                            .parse::<f32>()
                            .map_err(|_| "Invalid tempo value")?,
                    );
                    i += 2;
                } else {
                    return Err("Missing value for --tempo".to_string());
                }
            }
            "--loudness" => {
                if i + 1 < args.len() {
                    loudness = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --loudness".to_string());
                }
            }
            "--silence-db" => {
                if i + 1 < args.len() {
                    silence_db = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --silence-db".to_string());
                }
            }
            "--silence-duration" => {
                if i + 1 < args.len() {
                    silence_duration = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --silence-duration".to_string());
                }
            }
            other => {
                return Err(format!("Unknown option for audio: {}", other));
            }
        }
    }

    let valid_ops = [
        "volume",
        "denoise",
        "compress",
        "limit",
        "eq",
        "pitch",
        "tempo",
        "reverb",
        "echo",
        "bass",
        "silencedetect",
        "normalize",
    ];
    if !valid_ops.contains(&operation.as_str()) {
        return Err(format!(
            "Unknown audio operation: '{}'. Available: {}",
            operation,
            valid_ops.join(", ")
        ));
    }

    Ok(Commands::Audio {
        input,
        output,
        operation,
        volume,
        noise_reduction,
        threshold,
        ratio,
        limit,
        gain,
        frequency,
        pitch,
        tempo,
        loudness,
        silence_db,
        silence_duration,
    })
}

use std::fs::File;
use std::io::Write;
use symphonia::core::codecs::audio::AudioDecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::formats::probe::Hint;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

#[allow(clippy::too_many_arguments)]
pub fn run(
    input: &str,
    output: &str,
    operation: &str,
    volume: Option<&str>,
    noise_reduction: Option<&str>,
    threshold: Option<&str>,
    ratio: Option<&str>,
    limit: Option<&str>,
    gain: Option<&str>,
    frequency: Option<&str>,
    pitch: Option<f32>,
    tempo: Option<f32>,
    loudness: Option<&str>,
    silence_db: Option<&str>,
    silence_duration: Option<&str>,
) {
    println!(
        "Starting custom native audio operation '{}' on {} ...",
        operation, input
    );
    match run_audio_operation(
        input,
        output,
        operation,
        volume,
        noise_reduction,
        threshold,
        ratio,
        limit,
        gain,
        frequency,
        pitch,
        tempo,
        loudness,
        silence_db,
        silence_duration,
    ) {
        Ok(_) => println!("\x1b[1m\x1b[32mNative audio operation completed successfully!\x1b[0m"),
        Err(e) => eprintln!("\x1b[1m\x1b[31mNative audio operation failed: {}\x1b[0m", e),
    }
}

#[allow(clippy::too_many_arguments)]
fn run_audio_operation(
    input: &str,
    output: &str,
    operation: &str,
    volume: Option<&str>,
    _noise_reduction: Option<&str>,
    threshold: Option<&str>,
    _ratio: Option<&str>,
    limit: Option<&str>,
    gain: Option<&str>,
    _frequency: Option<&str>,
    _pitch: Option<f32>,
    tempo: Option<f32>,
    _loudness: Option<&str>,
    silence_db: Option<&str>,
    silence_duration: Option<&str>,
) -> Result<(), String> {
    // 1. Decode audio using Symphonia
    let file = File::open(input).map_err(|e| format!("Failed to open input: {}", e))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut format = symphonia::default::get_probe()
        .probe(
            &Hint::new(),
            mss,
            FormatOptions::default(),
            MetadataOptions::default(),
        )
        .map_err(|e| format!("Failed to probe format: {}", e))?;

    let track = format
        .tracks()
        .iter()
        .find(|t| {
            t.codec_params
                .as_ref()
                .map(|p| p.is_audio())
                .unwrap_or(false)
        })
        .ok_or("No audio track found")?;

    let audio_params = track
        .codec_params
        .as_ref()
        .ok_or("No codec parameters found in track")?
        .audio()
        .ok_or("No audio parameters found in track")?;

    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(audio_params, &AudioDecoderOptions::default())
        .map_err(|e| format!("Failed to initialize decoder: {}", e))?;

    let track_id = track.id;
    let mut sample_rate: u32 = 44100;
    let mut channels: u16 = 2;
    let mut pcm_data: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(Some(packet)) => packet,
            Ok(None) => break,
            Err(e) => return Err(format!("Decoder read error: {}", e)),
        };

        if packet.track_id != track_id {
            continue;
        }

        let decoded = decoder
            .decode(&packet)
            .map_err(|e| format!("Decode error: {}", e))?;

        let spec = decoded.spec();
        sample_rate = spec.rate();
        channels = spec.channels().count() as u16;

        let mut frame_pcm = vec![0.0f32; decoded.samples_interleaved()];
        decoded.copy_to_slice_interleaved(&mut frame_pcm);
        pcm_data.extend(frame_pcm);
    }

    if pcm_data.is_empty() {
        return Err("No decoded audio samples found".to_string());
    }

    // 2. Process DSP on pcm_data
    let mut processed_data = pcm_data;

    match operation {
        "volume" => {
            let v_str = volume.unwrap_or("1.0");
            let v_factor = if v_str.to_lowercase().ends_with("db") {
                let db_val = v_str[..v_str.len() - 2].parse::<f32>().unwrap_or(0.0);
                10.0f32.powf(db_val / 20.0)
            } else {
                v_str.parse::<f32>().unwrap_or(1.0)
            };
            for s in processed_data.iter_mut() {
                *s *= v_factor;
            }
        }
        "denoise" => {
            // Simple low-pass filter (moving average) for high-frequency noise smoothing
            let mut prev = 0.0;
            for s in processed_data.iter_mut() {
                *s = *s * 0.7 + prev * 0.3;
                prev = *s;
            }
        }
        "compress" => {
            let thresh_str = threshold.unwrap_or("-21");
            let t_db = if thresh_str.to_lowercase().ends_with("db") {
                thresh_str[..thresh_str.len() - 2]
                    .parse::<f32>()
                    .unwrap_or(-21.0)
            } else {
                thresh_str.parse::<f32>().unwrap_or(-21.0)
            };
            let t_amp = 10.0f32.powf(t_db / 20.0);
            // Dynamic compression
            for s in processed_data.iter_mut() {
                let abs = s.abs();
                if abs > t_amp {
                    let sign = s.signum();
                    let excess = abs - t_amp;
                    *s = sign * (t_amp + excess * 0.3); // 3:1 ratio
                }
            }
        }
        "limit" => {
            let lim_str = limit.unwrap_or("0.1");
            let lim = lim_str.parse::<f32>().unwrap_or(0.1);
            for s in processed_data.iter_mut() {
                *s = s.clamp(-lim, lim);
            }
        }
        "eq" | "bass" => {
            let g_str = gain.unwrap_or(if operation == "bass" { "8" } else { "0" });
            let g_db = g_str.parse::<f32>().unwrap_or(0.0);
            let factor = 10.0f32.powf(g_db / 20.0);
            // Equalizer/Bass Boost: Simple low-frequency amplification (crude low-pass boost)
            let mut prev = 0.0;
            for s in processed_data.iter_mut() {
                let low = *s * 0.5 + prev * 0.5;
                let high = *s - low;
                *s = low * factor + high;
                prev = *s;
            }
        }
        "pitch" | "tempo" => {
            // Simple linear resampling
            let t_factor = tempo.unwrap_or(1.0);
            if (t_factor - 1.0).abs() > 0.01 {
                let mut new_samples = Vec::new();
                let step = t_factor;
                let mut i = 0.0f32;
                while (i as usize) < processed_data.len() - channels as usize {
                    let base_idx = (i as usize) / channels as usize * channels as usize;
                    let next_idx = base_idx + channels as usize;
                    let frac = (i - base_idx as f32) / channels as f32;
                    for c in 0..channels {
                        let sample_a = processed_data[base_idx + c as usize];
                        let sample_b = processed_data[next_idx + c as usize];
                        let interpolated = sample_a * (1.0 - frac) + sample_b * frac;
                        new_samples.push(interpolated);
                    }
                    i += step * channels as f32;
                }
                processed_data = new_samples;
            }
        }
        "reverb" => {
            // High-density early reflections
            let mut output = processed_data.to_vec();
            let delays = [30, 45, 60, 80];
            let decays = [0.4, 0.3, 0.2, 0.1];
            for (d, dec) in delays.iter().zip(decays.iter()) {
                let delay_samples =
                    ((*d as f32 / 1000.0) * sample_rate as f32) as usize * channels as usize;
                if delay_samples < processed_data.len() {
                    for i in delay_samples..processed_data.len() {
                        output[i] += processed_data[i - delay_samples] * dec;
                    }
                }
            }
            processed_data = output;
        }
        "echo" => {
            let delay_samples = ((1.0 * sample_rate as f32) as usize) * channels as usize; // 1 second echo
            let mut output = processed_data.to_vec();
            if delay_samples < processed_data.len() {
                for i in delay_samples..processed_data.len() {
                    output[i] += processed_data[i - delay_samples] * 0.3;
                }
            }
            processed_data = output;
        }
        "silencedetect" => {
            let db_str = silence_db.unwrap_or("-50dB");
            let db_val = if db_str.to_lowercase().ends_with("db") {
                db_str[..db_str.len() - 2].parse::<f32>().unwrap_or(-50.0)
            } else {
                db_str.parse::<f32>().unwrap_or(-50.0)
            };
            let threshold_amp = 10.0f32.powf(db_val / 20.0);
            let dur_str = silence_duration.unwrap_or("2.0");
            let dur_sec = dur_str.parse::<f32>().unwrap_or(2.0);
            let chunk_size = (dur_sec * sample_rate as f32) as usize * channels as usize;

            if chunk_size > 0 {
                let mut chunk_idx = 0;
                while chunk_idx + chunk_size < processed_data.len() {
                    let mut sum_sq = 0.0;
                    for sample in &processed_data[chunk_idx..chunk_idx + chunk_size] {
                        sum_sq += sample * sample;
                    }
                    let rms = (sum_sq / chunk_size as f32).sqrt();
                    if rms < threshold_amp {
                        let sec = (chunk_idx / channels as usize) as f32 / sample_rate as f32;
                        println!(
                            "Silence detected starting at: {:.2} seconds (RMS: {:.4})",
                            sec, rms
                        );
                    }
                    chunk_idx += chunk_size;
                }
            }
        }
        "normalize" => {
            let mut max_val = 0.0f32;
            for &s in processed_data.iter() {
                let abs = s.abs();
                if abs > max_val {
                    max_val = abs;
                }
            }
            if max_val > 0.0 {
                let scale = 0.99 / max_val;
                for s in processed_data.iter_mut() {
                    *s *= scale;
                }
            }
        }
        _ => return Err(format!("Unsupported audio operation: {}", operation)),
    }

    // 3. Write output to WAV file
    let mut file =
        File::create(output).map_err(|e| format!("Failed to create output file: {}", e))?;

    let num_samples = processed_data.len() as u32;
    let byte_rate = sample_rate * channels as u32 * 2;
    let block_align = channels * 2u16;
    let subchunk2_size = num_samples * 2;
    let chunk_size = 36 + subchunk2_size;

    file.write_all(b"RIFF").map_err(|e| e.to_string())?;
    file.write_all(&chunk_size.to_le_bytes())
        .map_err(|e| e.to_string())?;
    file.write_all(b"WAVE").map_err(|e| e.to_string())?;

    file.write_all(b"fmt ").map_err(|e| e.to_string())?;
    file.write_all(&16u32.to_le_bytes())
        .map_err(|e| e.to_string())?;
    file.write_all(&1u16.to_le_bytes())
        .map_err(|e| e.to_string())?;
    file.write_all(&channels.to_le_bytes())
        .map_err(|e| e.to_string())?;
    file.write_all(&sample_rate.to_le_bytes())
        .map_err(|e| e.to_string())?;
    file.write_all(&byte_rate.to_le_bytes())
        .map_err(|e| e.to_string())?;
    file.write_all(&block_align.to_le_bytes())
        .map_err(|e| e.to_string())?;
    file.write_all(&16u16.to_le_bytes())
        .map_err(|e| e.to_string())?;

    file.write_all(b"data").map_err(|e| e.to_string())?;
    file.write_all(&subchunk2_size.to_le_bytes())
        .map_err(|e| e.to_string())?;

    for &sample in &processed_data {
        let clamped = sample.clamp(-1.0, 1.0);
        let val = if clamped < 0.0 {
            (clamped * 32768.0) as i16
        } else {
            (clamped * 32767.0) as i16
        };
        file.write_all(&val.to_le_bytes())
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}
