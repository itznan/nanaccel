use std::process::Command;

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
        "Starting audio editing operation '{}' on {} ...",
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
        Ok(_) => println!("\x1b[1m\x1b[32mAudio editing operation completed successfully!\x1b[0m"),
        Err(e) => eprintln!(
            "\x1b[1m\x1b[31mAudio editing operation failed: {}\x1b[0m",
            e
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn run_audio_operation(
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
) -> Result<(), String> {
    let mut args = vec!["-y".to_string(), "-i".to_string(), input.to_string()];
    let filter_str = match operation {
        "volume" => {
            let v = volume.unwrap_or("1.0");
            format!("volume={}", v)
        }
        "denoise" => {
            let nr = noise_reduction.unwrap_or("12");
            format!("afftdn=nr={}", nr)
        }
        "compress" => {
            let mut thresh = threshold.unwrap_or("-21").to_string();
            if !thresh.to_lowercase().ends_with("db") {
                thresh.push_str("dB");
            }
            let rat = ratio.unwrap_or("4");
            format!("acompressor=threshold={}:ratio={}", thresh, rat)
        }
        "limit" => {
            let lim = limit.unwrap_or("0.1");
            format!("alimiter=limit={}", lim)
        }
        "eq" => {
            let freq = frequency.unwrap_or("1000");
            let g = gain.unwrap_or("0");
            format!("equalizer=f={}:width_type=h:width=200:g={}", freq, g)
        }
        "pitch" => {
            let p = pitch.unwrap_or(1.0);
            format!("rubberband=pitch={}", p)
        }
        "tempo" => {
            let t = tempo.unwrap_or(1.0);
            format!("atempo={}", t)
        }
        "reverb" => {
            // Richer room simulation using a multi-tap delay line
            "aecho=0.8:0.9:30|45|60|80:0.4|0.3|0.2|0.1".to_string()
        }
        "echo" => "aecho=0.8:0.9:1000:0.3".to_string(),
        "bass" => {
            let g = gain.unwrap_or("8");
            format!("equalizer=f=100:width_type=h:width=100:g={}", g)
        }
        "silencedetect" => {
            let db = silence_db.unwrap_or("-50dB");
            let dur = silence_duration.unwrap_or("2.0");
            format!("silencedetect=noise={}:d={}", db, dur)
        }
        "normalize" => {
            let l = loudness.unwrap_or("-16");
            format!("loudnorm=I={}", l)
        }
        _ => return Err(format!("Unsupported audio operation: {}", operation)),
    };

    args.push("-af".to_string());
    args.push(filter_str);

    // Copy video track if present, to keep editing audio fast and preserve video quality
    args.push("-c:v".to_string());
    args.push("copy".to_string());

    args.push(output.to_string());

    println!("Running audio command: ffmpeg {}", args.join(" "));

    let status = Command::new("ffmpeg")
        .args(&args)
        .status()
        .map_err(|e| format!("Failed to execute ffmpeg for audio operation: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err("ffmpeg audio operation failed".to_string())
    }
}
