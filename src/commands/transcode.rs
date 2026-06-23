use std::process::Command;

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
    let use_native_gpu = (codec == "h264" || codec == "hevc") && !transcode_audio;

    if use_native_gpu {
        let ext = std::path::Path::new(output)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        if ext != "mp4" && ext != "mov" && ext != "m4v" && ext != "3gp" && !ext.is_empty() {
            println!(
                "\x1b[33mWarning: nann natively encodes and muxes to standard ISO-MP4/MOV formats. \
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
                println!(
                    "\x1b[33mWarning: Native GPU transcode failed ({}). Falling back to FFmpeg transcode...\x1b[0m",
                    e
                );
                if let Err(fe) = run_ffmpeg_transcode(
                    input,
                    output,
                    codec,
                    preset,
                    bitrate,
                    scale,
                    transcode_audio,
                    audio_codec,
                ) {
                    println!("\x1b[1m\x1b[31mTranscoding failed: {}\x1b[0m", fe);
                } else {
                    println!(
                        "\x1b[1m\x1b[32mTranscoding completed successfully via FFmpeg!\x1b[0m"
                    );
                }
            }
        }
    } else {
        println!(
            "Non-native format or audio transcode requested. Delegating transcode to FFmpeg..."
        );
        if let Err(fe) = run_ffmpeg_transcode(
            input,
            output,
            codec,
            preset,
            bitrate,
            scale,
            transcode_audio,
            audio_codec,
        ) {
            println!("\x1b[1m\x1b[31mTranscoding failed: {}\x1b[0m", fe);
        } else {
            println!("\x1b[1m\x1b[32mTranscoding completed successfully via FFmpeg!\x1b[0m");
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn run_ffmpeg_transcode(
    input: &str,
    output: &str,
    codec: &str,
    preset: &str,
    bitrate: Option<&str>,
    scale: Option<&str>,
    transcode_audio: bool,
    audio_codec: Option<&str>,
) -> Result<(), String> {
    let mut args = vec!["-y".to_string(), "-i".to_string(), input.to_string()];

    let vcodec = match codec.to_lowercase().as_str() {
        "h264" | "h.264" | "avc" => "h264_nvenc".to_string(),
        "hevc" | "h265" | "h.265" => "hevc_nvenc".to_string(),
        "av1" => "av1_nvenc".to_string(),
        "vp8" => "vp8".to_string(),
        "vp9" => "vp9".to_string(),
        "mpeg1" | "mpeg-1" => "mpeg1video".to_string(),
        "mpeg2" | "mpeg-2" => "mpeg2video".to_string(),
        "mpeg4" | "mpeg-4" => "mpeg4".to_string(),
        "mjpeg" => "mjpeg".to_string(),
        "prores" => "prores".to_string(),
        "dnxhd" => "dnxhd".to_string(),
        "cineform" | "cfhd" => "cfhd".to_string(),
        "huffyuv" => "huffyuv".to_string(),
        "ffv1" => "ffv1".to_string(),
        "theora" => "libtheora".to_string(),
        "dirac" => "dirac".to_string(),
        "vc-1" | "vc1" => "vc1".to_string(),
        "wmv" => "wmv2".to_string(),
        "xvid" => "libxvid".to_string(),
        "divx" => "mpeg4".to_string(),
        other => other.to_string(),
    };

    args.push("-c:v".to_string());
    args.push(vcodec.clone());

    if vcodec.contains("nvenc") {
        args.push("-preset".to_string());
        args.push(preset.to_string());
    } else {
        let sw_preset = match preset.to_lowercase().as_str() {
            "p1" => "ultrafast",
            "p2" => "superfast",
            "p3" => "veryfast",
            "p4" => "medium",
            "p5" => "slow",
            "p6" => "slower",
            "p7" => "veryslow",
            _ => "medium",
        };
        args.push("-preset".to_string());
        args.push(sw_preset.to_string());
    }

    if let Some(br) = bitrate {
        args.push("-b:v".to_string());
        args.push(br.to_string());
    }

    if let Some(scale_str) = scale {
        let parts: Vec<&str> = scale_str.split('x').collect();
        if parts.len() == 2 {
            args.push("-vf".to_string());
            args.push(format!("scale={}:{}", parts[0], parts[1]));
        }
    }

    if transcode_audio || audio_codec.is_some() {
        let acodec = match audio_codec.unwrap_or("aac") {
            "aac" => "aac".to_string(),
            "mp3" => "libmp3lame".to_string(),
            "flac" => "flac".to_string(),
            "opus" => "libopus".to_string(),
            "vorbis" => "libvorbis".to_string(),
            "pcm" => "pcm_s16le".to_string(),
            "alac" => "alac".to_string(),
            "ac3" => "ac3".to_string(),
            "e-ac3" | "eac3" => "eac3".to_string(),
            "dts" => "dts".to_string(),
            "amr" => "libopencore_amrnb".to_string(),
            "speex" => "libspeex".to_string(),
            "wma" => "wmav2".to_string(),
            "gsm" => "libgsm".to_string(),
            "truehd" => "truehd".to_string(),
            "dolby atmos" | "atmos" => "truehd".to_string(),
            "dts-hd" => "dts".to_string(),
            other => other.to_string(),
        };
        args.push("-c:a".to_string());
        args.push(acodec);
    } else {
        args.push("-c:a".to_string());
        args.push("copy".to_string());
    }

    args.push(output.to_string());

    println!("Running ffmpeg transcode: ffmpeg {}", args.join(" "));

    let status = Command::new("ffmpeg")
        .args(&args)
        .status()
        .map_err(|e| format!("Failed to execute ffmpeg subprocess: {}", e))?;

    if status.success() {
        return Ok(());
    }

    if vcodec.contains("nvenc") {
        let fallback_codec = match codec.to_lowercase().as_str() {
            "h264" | "h.264" | "avc" => Some("libx264"),
            "hevc" | "h265" | "h.265" => Some("libx265"),
            "av1" => Some("libsvtav1"),
            _ => None,
        };

        if let Some(sw_codec) = fallback_codec {
            println!(
                "\x1b[33mWarning: GPU-accelerated encoder '{}' failed or is unsupported. Falling back to software encoder '{}'...\x1b[0m",
                vcodec, sw_codec
            );

            let mut sw_args = vec!["-y".to_string(), "-i".to_string(), input.to_string()];
            sw_args.push("-c:v".to_string());
            sw_args.push(sw_codec.to_string());

            let sw_preset = match preset.to_lowercase().as_str() {
                "p1" => "ultrafast",
                "p2" => "superfast",
                "p3" => "veryfast",
                "p4" => "medium",
                "p5" => "slow",
                "p6" => "slower",
                "p7" => "veryslow",
                _ => "medium",
            };
            sw_args.push("-preset".to_string());
            sw_args.push(sw_preset.to_string());

            if let Some(br) = bitrate {
                sw_args.push("-b:v".to_string());
                sw_args.push(br.to_string());
            }

            if let Some(scale_str) = scale {
                let parts: Vec<&str> = scale_str.split('x').collect();
                if parts.len() == 2 {
                    sw_args.push("-vf".to_string());
                    sw_args.push(format!("scale={}:{}", parts[0], parts[1]));
                }
            }

            if transcode_audio || audio_codec.is_some() {
                let acodec = match audio_codec.unwrap_or("aac") {
                    "aac" => "aac".to_string(),
                    "mp3" => "libmp3lame".to_string(),
                    "flac" => "flac".to_string(),
                    "opus" => "libopus".to_string(),
                    "vorbis" => "libvorbis".to_string(),
                    "pcm" => "pcm_s16le".to_string(),
                    "alac" => "alac".to_string(),
                    "ac3" => "ac3".to_string(),
                    "e-ac3" | "eac3" => "eac3".to_string(),
                    "dts" => "dts".to_string(),
                    "amr" => "libopencore_amrnb".to_string(),
                    "speex" => "libspeex".to_string(),
                    "wma" => "wmav2".to_string(),
                    "gsm" => "libgsm".to_string(),
                    "truehd" => "truehd".to_string(),
                    "dolby atmos" | "atmos" => "truehd".to_string(),
                    "dts-hd" => "dts".to_string(),
                    other => other.to_string(),
                };
                sw_args.push("-c:a".to_string());
                sw_args.push(acodec);
            } else {
                sw_args.push("-c:a".to_string());
                sw_args.push("copy".to_string());
            }

            sw_args.push(output.to_string());

            let sw_status = Command::new("ffmpeg")
                .args(&sw_args)
                .status()
                .map_err(|e| format!("Failed to execute software ffmpeg subprocess: {}", e))?;

            if sw_status.success() {
                return Ok(());
            }
        }
    }

    Err("ffmpeg subprocess exited with an error status".to_string())
}
