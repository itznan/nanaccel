use crate::commands::types::Commands;

pub fn parse(args: &[String]) -> Result<Commands, String> {
    if args.len() < 4 {
        return Err("Usage: transcode <input> <output> [options]".to_string());
    }
    let input = args[2].clone();
    let output = args[3].clone();
    let mut codec = "h264".to_string();
    let mut preset = "p4".to_string();
    let mut bitrate = None;
    let mut scale = None;
    let mut transcode_audio = false;
    let mut audio_codec = None;

    let mut i = 4;
    while i < args.len() {
        match args[i].as_str() {
            "-c" | "--codec" => {
                if i + 1 < args.len() {
                    let target_codec = args[i + 1].to_lowercase();
                    if target_codec == "h264" || target_codec == "h.264" || target_codec == "avc" {
                        codec = "h264".to_string();
                    } else if target_codec == "hevc"
                        || target_codec == "h265"
                        || target_codec == "h.265"
                    {
                        codec = "hevc".to_string();
                    } else if target_codec == "av1" {
                        codec = "av1".to_string();
                    } else if [
                        "vp8", "vp9", "mpeg1", "mpeg-1", "mpeg2", "mpeg-2", "mpeg4", "mpeg-4",
                        "mjpeg", "prores", "dnxhd", "cineform", "cfhd", "huffyuv", "ffv1",
                        "theora", "dirac", "vc-1", "vc1", "wmv", "xvid", "divx",
                    ]
                    .contains(&target_codec.as_str())
                    {
                        codec = target_codec;
                    } else {
                        return Err(format!("Unsupported video codec: '{}'.", args[i + 1]));
                    }
                    i += 2;
                } else {
                    return Err("Missing value for --codec".to_string());
                }
            }
            "-ac" | "--audio-codec" => {
                if i + 1 < args.len() {
                    let target_audio = args[i + 1].to_lowercase();
                    if [
                        "aac",
                        "mp3",
                        "flac",
                        "opus",
                        "vorbis",
                        "pcm",
                        "alac",
                        "ac3",
                        "e-ac3",
                        "eac3",
                        "dts",
                        "amr",
                        "speex",
                        "wma",
                        "gsm",
                        "truehd",
                        "dolby atmos",
                        "atmos",
                        "dts-hd",
                    ]
                    .contains(&target_audio.as_str())
                    {
                        audio_codec = Some(target_audio);
                        transcode_audio = true;
                    } else {
                        return Err(format!("Unsupported audio codec: '{}'.", args[i + 1]));
                    }
                    i += 2;
                } else {
                    return Err("Missing value for --audio-codec".to_string());
                }
            }
            "-p" | "--preset" => {
                if i + 1 < args.len() {
                    preset = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("Missing value for --preset".to_string());
                }
            }
            "-b" | "--bitrate" => {
                if i + 1 < args.len() {
                    bitrate = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --bitrate".to_string());
                }
            }
            "--scale" => {
                if i + 1 < args.len() {
                    scale = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --scale".to_string());
                }
            }
            "--transcode-audio" => {
                transcode_audio = true;
                i += 1;
            }
            other => {
                return Err(format!("Unknown option for transcode: {}", other));
            }
        }
    }
    Ok(Commands::Transcode {
        input,
        output,
        codec,
        preset,
        bitrate,
        scale,
        transcode_audio,
        audio_codec,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn run(
    input: &str,
    output: &str,
    codec: &str,
    preset: &str,
    bitrate: Option<&str>,
    scale: Option<&str>,
    transcode_audio: bool,
    audio_codec: Option<&str>,
) {
    let use_native_gpu =
        (codec == "h264" || codec == "hevc") && !transcode_audio && audio_codec.is_none();

    if use_native_gpu {
        let ext = std::path::Path::new(output)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        if ext != "mp4" && ext != "mov" && ext != "m4v" && ext != "3gp" && !ext.is_empty() {
            println!(
                "\x1b[33mWarning: NanAccel natively encodes and muxes to standard ISO-MP4/MOV formats. \
                The output stream will be written as a valid MP4/MOV container structure inside the requested '.{}' file.\x1b[0m",
                ext
            );
        }
        println!("Starting GPU transcode: {} -> {} ...", input, output);
        let transcode_result =
            crate::gpu_pipeline::transcode_gpu(input, output, codec, preset, bitrate, scale);

        match transcode_result {
            Ok(_) => {
                println!("\x1b[1m\x1b[32mTranscoding completed successfully via GPU!\x1b[0m");
            }
            Err(e) => {
                eprintln!("\x1b[1m\x1b[31mTranscoding failed: {}\x1b[0m", e);
            }
        }
    } else {
        eprintln!(
            "\x1b[1m\x1b[31mTranscoding error: NanAccel's native GPU pipeline only supports H.264 and HEVC video stream transcoding without audio transcoding.\x1b[0m"
        );
    }
}
