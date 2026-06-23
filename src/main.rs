use std::process::{Command, Stdio};

mod gpu_pipeline;
mod mux;

#[derive(Debug)]
#[allow(dead_code)]
enum Commands {
    Play {
        input: String,
        decoder: Option<String>,
        no_audio: bool,
        loop_video: bool,
    },
    Transcode {
        input: String,
        output: String,
        codec: String,
        preset: String,
        bitrate: Option<String>,
        scale: Option<String>,
        transcode_audio: bool,
        audio_codec: Option<String>,
    },
    Screenshot {
        input: String,
        output: String,
        time_ms: u32,
    },
    Subtitle {
        input: String,
        output: String,
        operation: String,
        sub_file: Option<String>,
        shift_ms: Option<i32>,
        track_index: Option<u32>,
    },
    Edit {
        input: String,
        output: String,
        operation: String,
        start_time: Option<String>,
        end_time: Option<String>,
        duration: Option<String>,
        crop: Option<String>,
        rotate: Option<String>,
        flip: Option<String>,
        scale: Option<String>,
        loop_count: Option<i32>,
        fade_in: Option<String>,
        fade_out: Option<String>,
        overlay_file: Option<String>,
        watermark_text: Option<String>,
        position: Option<String>,
        additional_inputs: Vec<String>,
    },
    Color {
        input: String,
        output: String,
        operation: String,
        lut_file: Option<String>,
        gamma: Option<String>,
        shadows: Option<String>,
        midtones: Option<String>,
        highlights: Option<String>,
        colorspace: Option<String>,
        temperature: Option<f32>,
        brightness: Option<f32>,
        contrast: Option<f32>,
        saturation: Option<f32>,
        tonemap: Option<String>,
    },
    Info,
}

struct StaticGpuInfo {
    name: String,
    driver_version: String,
    memory_total: u64, // in MB
}

struct DynamicGpuStats {
    gpu_util: u32,
    mem_util: u32,
    dec_util: u32,
    enc_util: u32,
    mem_used: u64, // in MB
    temp: u32,
    power: f32, // in Watts
}

fn query_static_gpu_info() -> Result<StaticGpuInfo, String> {
    let output = Command::new("nvidia-smi")
        .args(&[
            "--query-gpu=name,driver_version,memory.total",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .map_err(|e| format!("Failed to run nvidia-smi: {}", e))?;

    if !output.status.success() {
        return Err("nvidia-smi exited with error status".to_string());
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let line = text.lines().next().ok_or("Empty nvidia-smi output")?;
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() < 3 {
        return Err("Unexpected nvidia-smi static output format".to_string());
    }

    let memory_total = parts[parts.len() - 1]
        .trim()
        .parse::<u64>()
        .map_err(|e| format!("Failed to parse memory: {}", e))?;

    let driver_version = parts[parts.len() - 2].trim().to_string();
    let name = parts[..parts.len() - 2].join(",").trim().to_string();

    Ok(StaticGpuInfo {
        name,
        driver_version,
        memory_total,
    })
}

fn query_dynamic_gpu_stats() -> Result<DynamicGpuStats, String> {
    let output = Command::new("nvidia-smi")
        .args(&[
            "--query-gpu=utilization.gpu,utilization.memory,utilization.decoder,utilization.encoder,memory.used,temperature.gpu,power.draw",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .map_err(|e| format!("Failed to run nvidia-smi: {}", e))?;

    if !output.status.success() {
        return Err("nvidia-smi exited with error".to_string());
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let line = text
        .lines()
        .next()
        .ok_or("Empty dynamic nvidia-smi output")?;
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() < 7 {
        return Err("Unexpected nvidia-smi dynamic output format".to_string());
    }

    let gpu_util = parts[0].trim().parse::<u32>().unwrap_or(0);
    let mem_util = parts[1].trim().parse::<u32>().unwrap_or(0);
    let dec_util = parts[2].trim().parse::<u32>().unwrap_or(0);
    let enc_util = parts[3].trim().parse::<u32>().unwrap_or(0);
    let mem_used = parts[4].trim().parse::<u64>().unwrap_or(0);
    let temp = parts[5].trim().parse::<u32>().unwrap_or(0);
    let power = parts[6].trim().parse::<f32>().unwrap_or(0.0);

    Ok(DynamicGpuStats {
        gpu_util,
        mem_util,
        dec_util,
        enc_util,
        mem_used,
        temp,
        power,
    })
}

fn print_help() {
    println!("\x1b[1m\x1b[36mNVIDIA Hardware Accelerated Video CLI Tool (nann)\x1b[0m");
    println!("Written in Rust with zero compile-time dependencies to avoid application controls.");
    println!("\n\x1b[1mUsage:\x1b[0m");
    println!("  nann <subcommand> [options]");
    println!("\n\x1b[1mSubcommands:\x1b[0m");
    println!(
        "  \x1b[32minfo\x1b[0m                              Print NVIDIA GPU capabilities & live status"
    );
    println!(
        "  \x1b[32mplay <input>\x1b[0m                      Play a video file using hardware-accelerated NVDEC"
    );
    println!("       [-d, --decoder <decoder>]    Specify decoder (e.g., h264_cuvid, hevc_cuvid)");
    println!("       [--no-audio]                 Disable audio");
    println!("       [--loop]                     Loop playback infinitely");
    println!(
        "  \x1b[32mtranscode <input> <output>\x1b[0m        Transcode video using NVDEC -> CUDA -> NVENC"
    );
    println!("       [-c, --codec <codec>]        Target video codec: h264, hevc, av1, vp8, vp9, mpeg1, mpeg2, mpeg4, mjpeg, prores, dnxhd, cineform, huffyuv, ffv1, theora, dirac, vc1, wmv, xvid, divx (default: h264)");
    println!("       [-ac, --audio-codec <codec>] Target audio codec: aac, mp3, flac, opus, vorbis, pcm, alac, ac3, e-ac3, dts, amr, speex, wma, gsm, truehd, atmos, dts-hd");
    println!(
        "       [-p, --preset <preset>]      NVENC preset: p1 (fastest) to p7 (slowest) (default: p4)"
    );
    println!("       [-b, --bitrate <bitrate>]    Output video bitrate (e.g., 5M, 800k)");
    println!("       [--scale <width>x<height>]   Scale resolution on the GPU (e.g., 1280x720)");
    println!("       [--transcode-audio]          Transcode audio to AAC (default: copy stream)");
    println!(
        "  \x1b[32mscreenshot <input> <output>\x1b[0m       Extract a single frame from the video at a timestamp"
    );
    println!("       [-t, --time <ms>]            Timestamp in milliseconds (default: 0)");
    println!(
        "  \x1b[32msubtitle <operation> <input> <output>\x1b[0m Subtitle processing utility"
    );
    println!("       Operations:                  extract, convert, burn, sync, merge, remove");
    println!("       Format Support:              SRT, ASS, SSA, VTT, PGS, DVB, DVD subtitles, Teletext");
    println!("       [-s, --sub-file <file>]      Specify subtitle file for burn / merge operations");
    println!("       [-t, --track <idx>]          Specify track index for subtitle extraction (default: 0)");
    println!("       [--shift <ms>]               Specify timestamp shift in milliseconds for sync operation");
    println!(
        "  \x1b[32medit <operation> <input> <output>\x1b[0m  Video editing utility"
    );
    println!("       Operations:                  trim, cut, split, join, concat, crop, rotate, flip, scale,");
    println!("                                    stabilize, denoise, sharpen, deblock, deinterlace, reverse,");
    println!("                                    loop, fade, crossfade, overlay, watermark");
    println!("       [-ss, --start <time>]        Specify start time for trim / cut / fade");
    println!("       [-to, --end <time>]          Specify end time for trim / cut");
    println!("       [-t, --duration <time/sec>]  Specify duration for trim / cut / split / fade");
    println!("       [--crop <w:h:x:y>]           Crop window parameter");
    println!("       [--rotate <angle>]           Rotate angle: 90, 180, 270");
    println!("       [--flip <h|v|both>]          Flip direction");
    println!("       [--scale <w>x<h>]            Target output resolution");
    println!("       [--loop-count <N>]           Specify number of loops");
    println!("       [--fade-in / --fade-out]     Specify fade options (e.g. st=0:d=2)");
    println!("       [-f, --file / --overlay]     Specify secondary overlay / crossfade video file");
    println!("       [--watermark-text <text>]    Specify drawtext watermark text");
    println!("       [--position <x:y>]           Overlay/drawtext positioning");
    println!(
        "  \x1b[32mcolor <operation> <input> <output>\x1b[0m Color processing utility"
    );
    println!("       Operations:                  hdr2sdr, sdr2hdr, lut, gamma, grading, colorspace,");
    println!("                                    whitebalance, adjust, tonemap");
    println!("       [-l, --lut <file>]           Specify LUT file for lut operation");
    println!("       [--gamma <val>]              Specify gamma value (default: 1.0)");
    println!("       [--shadows / --midtones / --highlights <r:g:b>] Lift/Gamma/Gain color balance controls");
    println!("       [--colorspace <space>]       Specify colorspace conversion (e.g. bt709, bt2020)");
    println!("       [--temp <K>]                 Specify color temperature in Kelvin for white balance");
    println!("       [--brightness / --contrast / --saturation <val>] Adjust controls");
    println!("       [--tonemap <algo>]           Tone mapping algorithm");
    println!();
}

fn parse_args() -> Result<Commands, String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        return Err("No subcommand specified".to_string());
    }

    match args[1].as_str() {
        "help" | "-h" | "--help" => {
            print_help();
            std::process::exit(0);
        }
        "info" => Ok(Commands::Info),
        "play" => {
            if args.len() < 3 {
                return Err("Missing input file for play command".to_string());
            }
            let input = args[2].clone();
            let mut decoder = None;
            let mut no_audio = false;
            let mut loop_video = false;

            let mut i = 3;
            while i < args.len() {
                match args[i].as_str() {
                    "-d" | "--decoder" => {
                        if i + 1 < args.len() {
                            decoder = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            return Err("Missing value for --decoder".to_string());
                        }
                    }
                    "--no-audio" => {
                        no_audio = true;
                        i += 1;
                    }
                    "--loop" => {
                        loop_video = true;
                        i += 1;
                    }
                    other => {
                        return Err(format!("Unknown option for play: {}", other));
                    }
                }
            }
            Ok(Commands::Play {
                input,
                decoder,
                no_audio,
                loop_video,
            })
        }
        "transcode" => {
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
                            if target_codec == "h264"
                                || target_codec == "h.264"
                                || target_codec == "avc"
                            {
                                codec = "h264".to_string();
                            } else if target_codec == "hevc"
                                || target_codec == "h265"
                                || target_codec == "h.265"
                            {
                                codec = "hevc".to_string();
                            } else if target_codec == "av1" {
                                codec = "av1".to_string();
                            } else if ["vp8", "vp9", "mpeg1", "mpeg-1", "mpeg2", "mpeg-2", "mpeg4", "mpeg-4",
                                       "mjpeg", "prores", "dnxhd", "cineform", "cfhd", "huffyuv", "ffv1",
                                       "theora", "dirac", "vc-1", "vc1", "wmv", "xvid", "divx"].contains(&target_codec.as_str()) {
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
                            if ["aac", "mp3", "flac", "opus", "vorbis", "pcm", "alac", "ac3", "e-ac3", "eac3",
                                "dts", "amr", "speex", "wma", "gsm", "truehd", "dolby atmos", "atmos", "dts-hd"].contains(&target_audio.as_str()) {
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
        "screenshot" => {
            if args.len() < 4 {
                return Err("Usage: screenshot <input> <output> [options]".to_string());
            }
            let input = args[2].clone();
            let output = args[3].clone();
            let mut time_ms = 0;

            let mut i = 4;
            while i < args.len() {
                match args[i].as_str() {
                    "-t" | "--time" => {
                        if i + 1 < args.len() {
                            time_ms = args[i + 1].parse::<u32>().map_err(|_| {
                                "Invalid value for --time: must be a positive integer".to_string()
                            })?;
                            i += 2;
                        } else {
                            return Err("Missing value for --time".to_string());
                        }
                    }
                    other => {
                        return Err(format!("Unknown option for screenshot: {}", other));
                    }
                }
            }
            Ok(Commands::Screenshot {
                input,
                output,
                time_ms,
            })
        }
        "subtitle" | "sub" => {
            if args.len() < 5 {
                return Err("Usage: subtitle <operation> <input> <output> [options]\n\
                            Operations: extract, convert, burn, sync, merge, remove".to_string());
            }
            let operation = args[2].to_lowercase();
            let input = args[3].clone();
            let output = args[4].clone();

            let mut sub_file = None;
            let mut shift_ms = None;
            let mut track_index = None;

            let mut i = 5;
            while i < args.len() {
                match args[i].as_str() {
                    "-s" | "--sub-file" => {
                        if i + 1 < args.len() {
                            sub_file = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            return Err("Missing value for --sub-file".to_string());
                        }
                    }
                    "-t" | "--track" => {
                        if i + 1 < args.len() {
                            track_index = Some(args[i + 1].parse::<u32>().map_err(|_| "Invalid track index")?);
                            i += 2;
                        } else {
                            return Err("Missing value for --track".to_string());
                        }
                    }
                    "--shift" => {
                        if i + 1 < args.len() {
                            shift_ms = Some(args[i + 1].parse::<i32>().map_err(|_| "Invalid shift milliseconds value")?);
                            i += 2;
                        } else {
                            return Err("Missing value for --shift".to_string());
                        }
                    }
                    other => {
                        return Err(format!("Unknown option for subtitle: {}", other));
                    }
                }
            }

            if !["extract", "convert", "burn", "sync", "merge", "remove"].contains(&operation.as_str()) {
                return Err(format!("Unknown subtitle operation: '{}'. Available: extract, convert, burn, sync, merge, remove", operation));
            }

            Ok(Commands::Subtitle {
                input,
                output,
                operation,
                sub_file,
                shift_ms,
                track_index,
            })
        }
        "edit" => {
            if args.len() < 5 {
                return Err("Usage: edit <operation> <input> <output> [options]\n\
                            Operations: trim, cut, split, join, concat, crop, rotate, flip, scale, stabilize, denoise, sharpen, deblock, deinterlace, reverse, loop, fade, crossfade, overlay, watermark".to_string());
            }
            let operation = args[2].to_lowercase();
            let input = args[3].clone();
            let output = args[4].clone();

            let mut start_time = None;
            let mut end_time = None;
            let mut duration = None;
            let mut crop = None;
            let mut rotate = None;
            let mut flip = None;
            let mut scale = None;
            let mut loop_count = None;
            let mut fade_in = None;
            let mut fade_out = None;
            let mut overlay_file = None;
            let mut watermark_text = None;
            let mut position = None;
            let mut additional_inputs = Vec::new();

            let mut i = 5;
            while i < args.len() {
                match args[i].as_str() {
                    "-ss" | "--start" => {
                        if i + 1 < args.len() {
                            start_time = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            return Err("Missing value for -ss".to_string());
                        }
                    }
                    "-to" | "--end" => {
                        if i + 1 < args.len() {
                            end_time = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            return Err("Missing value for -to".to_string());
                        }
                    }
                    "-t" | "--duration" => {
                        if i + 1 < args.len() {
                            duration = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            return Err("Missing value for -t".to_string());
                        }
                    }
                    "--crop" => {
                        if i + 1 < args.len() {
                            crop = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            return Err("Missing value for --crop".to_string());
                        }
                    }
                    "--rotate" => {
                        if i + 1 < args.len() {
                            rotate = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            return Err("Missing value for --rotate".to_string());
                        }
                    }
                    "--flip" => {
                        if i + 1 < args.len() {
                            flip = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            return Err("Missing value for --flip".to_string());
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
                    "--loop-count" => {
                        if i + 1 < args.len() {
                            loop_count = Some(args[i + 1].parse::<i32>().map_err(|_| "Invalid loop count")?);
                            i += 2;
                        } else {
                            return Err("Missing value for --loop-count".to_string());
                        }
                    }
                    "--fade-in" => {
                        if i + 1 < args.len() {
                            fade_in = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            return Err("Missing value for --fade-in".to_string());
                        }
                    }
                    "--fade-out" => {
                        if i + 1 < args.len() {
                            fade_out = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            return Err("Missing value for --fade-out".to_string());
                        }
                    }
                    "-f" | "--file" | "--overlay" => {
                        if i + 1 < args.len() {
                            overlay_file = Some(args[i + 1].clone());
                            additional_inputs.push(args[i + 1].clone());
                            i += 2;
                        } else {
                            return Err("Missing value for file option".to_string());
                        }
                    }
                    "--watermark-text" => {
                        if i + 1 < args.len() {
                            watermark_text = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            return Err("Missing value for --watermark-text".to_string());
                        }
                    }
                    "--position" => {
                        if i + 1 < args.len() {
                            position = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            return Err("Missing value for --position".to_string());
                        }
                    }
                    other => {
                        return Err(format!("Unknown option for edit: {}", other));
                    }
                }
            }

            let valid_ops = [
                "trim", "cut", "split", "join", "concat", "crop", "rotate", "flip",
                "scale", "stabilize", "denoise", "sharpen", "deblock", "deinterlace",
                "reverse", "loop", "fade", "crossfade", "overlay", "watermark"
            ];
            if !valid_ops.contains(&operation.as_str()) {
                return Err(format!("Unknown edit operation: '{}'. Available: {}", operation, valid_ops.join(", ")));
            }

            Ok(Commands::Edit {
                input,
                output,
                operation,
                start_time,
                end_time,
                duration,
                crop,
                rotate,
                flip,
                scale,
                loop_count,
                fade_in,
                fade_out,
                overlay_file,
                watermark_text,
                position,
                additional_inputs,
            })
        }
        "color" => {
            if args.len() < 5 {
                return Err("Usage: color <operation> <input> <output> [options]\n\
                            Operations: hdr2sdr, sdr2hdr, lut, gamma, grading, colorspace, whitebalance, adjust, tonemap".to_string());
            }
            let operation = args[2].to_lowercase();
            let input = args[3].clone();
            let output = args[4].clone();

            let mut lut_file = None;
            let mut gamma = None;
            let mut shadows = None;
            let mut midtones = None;
            let mut highlights = None;
            let mut colorspace = None;
            let mut temperature = None;
            let mut brightness = None;
            let mut contrast = None;
            let mut saturation = None;
            let mut tonemap = None;

            let mut i = 5;
            while i < args.len() {
                match args[i].as_str() {
                    "-l" | "--lut" | "--lut-file" => {
                        if i + 1 < args.len() {
                            lut_file = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            return Err("Missing value for --lut-file".to_string());
                        }
                    }
                    "--gamma" => {
                        if i + 1 < args.len() {
                            gamma = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            return Err("Missing value for --gamma".to_string());
                        }
                    }
                    "--shadows" => {
                        if i + 1 < args.len() {
                            shadows = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            return Err("Missing value for --shadows".to_string());
                        }
                    }
                    "--midtones" => {
                        if i + 1 < args.len() {
                            midtones = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            return Err("Missing value for --midtones".to_string());
                        }
                    }
                    "--highlights" => {
                        if i + 1 < args.len() {
                            highlights = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            return Err("Missing value for --highlights".to_string());
                        }
                    }
                    "--colorspace" | "--space" => {
                        if i + 1 < args.len() {
                            colorspace = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            return Err("Missing value for --colorspace".to_string());
                        }
                    }
                    "--temp" | "--temperature" => {
                        if i + 1 < args.len() {
                            temperature = Some(args[i + 1].parse::<f32>().map_err(|_| "Invalid temperature value")?);
                            i += 2;
                        } else {
                            return Err("Missing value for --temp".to_string());
                        }
                    }
                    "--brightness" => {
                        if i + 1 < args.len() {
                            brightness = Some(args[i + 1].parse::<f32>().map_err(|_| "Invalid brightness value")?);
                            i += 2;
                        } else {
                            return Err("Missing value for --brightness".to_string());
                        }
                    }
                    "--contrast" => {
                        if i + 1 < args.len() {
                            contrast = Some(args[i + 1].parse::<f32>().map_err(|_| "Invalid contrast value")?);
                            i += 2;
                        } else {
                            return Err("Missing value for --contrast".to_string());
                        }
                    }
                    "--saturation" => {
                        if i + 1 < args.len() {
                            saturation = Some(args[i + 1].parse::<f32>().map_err(|_| "Invalid saturation value")?);
                            i += 2;
                        } else {
                            return Err("Missing value for --saturation".to_string());
                        }
                    }
                    "--tonemap" => {
                        if i + 1 < args.len() {
                            tonemap = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            return Err("Missing value for --tonemap".to_string());
                        }
                    }
                    other => {
                        return Err(format!("Unknown option for color: {}", other));
                    }
                }
            }

            let valid_ops = ["hdr2sdr", "sdr2hdr", "lut", "gamma", "grading", "colorspace", "whitebalance", "adjust", "tonemap"];
            if !valid_ops.contains(&operation.as_str()) {
                return Err(format!("Unknown color operation: '{}'. Available: {}", operation, valid_ops.join(", ")));
            }

            Ok(Commands::Color {
                input,
                output,
                operation,
                lut_file,
                gamma,
                shadows,
                midtones,
                highlights,
                colorspace,
                temperature,
                brightness,
                contrast,
                saturation,
                tonemap,
            })
        }
        sub => Err(format!("Unknown subcommand: {}", sub)),
    }
}

fn check_nvidia_gpu() -> bool {
    #[cfg(windows)]
    {
        if unsafe { libloading::Library::new("nvcuda.dll") }.is_ok() {
            return true;
        }
    }

    let status = Command::new("nvidia-smi")
        .arg("-L")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    if let Ok(s) = status {
        if s.success() {
            return true;
        }
    }

    false
}

fn main() {
    if !check_nvidia_gpu() {
        println!("gpu not detected");
        std::process::exit(1);
    }
    let args = match parse_args() {
        Ok(cmd) => cmd,
        Err(err) => {
            eprintln!("\x1b[31mError: {}\x1b[0m", err);
            print_help();
            std::process::exit(1);
        }
    };

    match args {
        Commands::Play {
            input,
            decoder: _,
            no_audio,
            loop_video,
        } => {
            println!("Initializing GPU Playback for: {} ...", input);
            let play_result = gpu_pipeline::play_gpu(&input, no_audio, loop_video);
            match play_result {
                Ok(_) => println!("Playback finished."),
                Err(e) => {
                    println!(
                        "\x1b[33mWarning: Native GPU playback failed ({}). Falling back to ffplay...\x1b[0m",
                        e
                    );
                    if let Err(fe) = run_ffplay(&input, no_audio, loop_video) {
                        eprintln!("\x1b[31mPlayback failed: {}\x1b[0m", fe);
                    } else {
                        println!("Playback finished.");
                    }
                }
            }
        }

        Commands::Transcode {
            input,
            output,
            codec,
            preset,
            bitrate,
            scale,
            transcode_audio,
            audio_codec,
        } => {
            let use_native_gpu = (codec == "h264" || codec == "hevc") && !transcode_audio;

            if use_native_gpu {
                let ext = std::path::Path::new(&output)
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
                let transcode_result = gpu_pipeline::transcode_gpu(
                    &input,
                    &output,
                    &codec,
                    &preset,
                    bitrate.as_deref(),
                    scale.as_deref(),
                );

                match transcode_result {
                    Ok(_) => {
                        println!("\x1b[1m\x1b[32mTranscoding completed successfully via GPU!\x1b[0m");
                    }
                    Err(e) => {
                        println!(
                            "\x1b[33mWarning: Native GPU transcode failed ({}). Falling back to FFmpeg transcode...\x1b[0m",
                            e
                        );
                        if let Err(fe) = run_ffmpeg_transcode(&input, &output, &codec, &preset, bitrate.as_deref(), scale.as_deref(), transcode_audio, audio_codec.as_deref()) {
                            println!("\x1b[1m\x1b[31mTranscoding failed: {}\x1b[0m", fe);
                        } else {
                            println!("\x1b[1m\x1b[32mTranscoding completed successfully via FFmpeg!\x1b[0m");
                        }
                    }
                }
            } else {
                println!("Non-native format or audio transcode requested. Delegating transcode to FFmpeg...");
                if let Err(fe) = run_ffmpeg_transcode(&input, &output, &codec, &preset, bitrate.as_deref(), scale.as_deref(), transcode_audio, audio_codec.as_deref()) {
                    println!("\x1b[1m\x1b[31mTranscoding failed: {}\x1b[0m", fe);
                } else {
                    println!("\x1b[1m\x1b[32mTranscoding completed successfully via FFmpeg!\x1b[0m");
                }
            }
        }

        Commands::Screenshot {
            input,
            output,
            time_ms,
        } => {
            println!(
                "Extracting GPU screenshot from {} to {} at time {} ms...",
                input, output, time_ms
            );
            let screenshot_result = gpu_pipeline::screenshot_gpu(&input, &output, time_ms);
            match screenshot_result {
                Ok(_) => {
                    println!(
                        "\x1b[1m\x1b[32mScreenshot extracted and saved successfully via GPU/WIC!\x1b[0m"
                    );
                }
                Err(e) => {
                    println!(
                        "\x1b[33mWarning: Native GPU screenshot failed ({}). Falling back to FFmpeg frame extraction...\x1b[0m",
                        e
                    );
                    if let Err(fe) = run_ffmpeg_screenshot(&input, &output, time_ms) {
                        println!("\x1b[1m\x1b[31mScreenshot failed: {}\x1b[0m", fe);
                    } else {
                        println!(
                            "\x1b[1m\x1b[32mScreenshot extracted and saved successfully via FFmpeg!\x1b[0m"
                        );
                    }
                }
            }
        }

        Commands::Subtitle {
            input,
            output,
            operation,
            sub_file,
            shift_ms,
            track_index,
        } => {
            println!("Starting subtitle operation '{}' on {} ...", operation, input);
            match run_subtitle_operation(&input, &output, &operation, sub_file.as_deref(), shift_ms, track_index) {
                Ok(_) => println!("\x1b[1m\x1b[32mSubtitle operation completed successfully!\x1b[0m"),
                Err(e) => eprintln!("\x1b[1m\x1b[31mSubtitle operation failed: {}\x1b[0m", e),
            }
        }

        Commands::Edit {
            input,
            output,
            operation,
            start_time,
            end_time,
            duration,
            crop,
            rotate,
            flip,
            scale,
            loop_count,
            fade_in,
            fade_out,
            overlay_file,
            watermark_text,
            position,
            additional_inputs,
        } => {
            println!("Starting video edit operation '{}' on {} ...", operation, input);
            match run_video_edit_operation(
                &input,
                &output,
                &operation,
                start_time.as_deref(),
                end_time.as_deref(),
                duration.as_deref(),
                crop.as_deref(),
                rotate.as_deref(),
                flip.as_deref(),
                scale.as_deref(),
                loop_count,
                fade_in.as_deref(),
                fade_out.as_deref(),
                overlay_file.as_deref(),
                watermark_text.as_deref(),
                position.as_deref(),
                &additional_inputs,
            ) {
                Ok(_) => println!("\x1b[1m\x1b[32mVideo edit operation completed successfully!\x1b[0m"),
                Err(e) => eprintln!("\x1b[1m\x1b[31mVideo edit operation failed: {}\x1b[0m", e),
            }
        }

        Commands::Color {
            input,
            output,
            operation,
            lut_file,
            gamma,
            shadows,
            midtones,
            highlights,
            colorspace,
            temperature,
            brightness,
            contrast,
            saturation,
            tonemap,
        } => {
            println!("Starting color processing operation '{}' on {} ...", operation, input);
            match run_color_operation(
                &input,
                &output,
                &operation,
                lut_file.as_deref(),
                gamma.as_deref(),
                shadows.as_deref(),
                midtones.as_deref(),
                highlights.as_deref(),
                colorspace.as_deref(),
                temperature,
                brightness,
                contrast,
                saturation,
                tonemap.as_deref(),
            ) {
                Ok(_) => println!("\x1b[1m\x1b[32mColor processing operation completed successfully!\x1b[0m"),
                Err(e) => eprintln!("\x1b[1m\x1b[31mColor processing operation failed: {}\x1b[0m", e),
            }
        }

        Commands::Info => {
            // Get Static GPU info
            let static_info = match query_static_gpu_info() {
                Ok(info) => info,
                Err(_) => {
                    println!("gpu not detected");
                    std::process::exit(1);
                }
            };

            println!("\x1b[1m\x1b[32m--- NVIDIA GPU Status ---\x1b[0m");
            println!("{:<20}: {}", "GPU Model", static_info.name);
            println!("{:<20}: {}", "Driver Version", static_info.driver_version);
            println!("{:<20}: {} MB", "Total VRAM", static_info.memory_total);

            if let Ok(dynamic) = query_dynamic_gpu_stats() {
                println!("{:<20}: {} W", "Power Draw", dynamic.power);
                println!("{:<20}: {} °C", "Temperature", dynamic.temp);
                println!(
                    "{:<20}: {} MB ({}%)",
                    "VRAM Usage",
                    dynamic.mem_used,
                    (dynamic.mem_used * 100) / static_info.memory_total
                );
                println!("{:<20}: {}%", "Core Utilization", dynamic.gpu_util);
                println!("{:<20}: {}%", "Memory Bus Load", dynamic.mem_util);
                println!("{:<20}: {}%", "Video Decoder Load", dynamic.dec_util);
                println!("{:<20}: {}%", "Video Encoder Load", dynamic.enc_util);
            }

            println!("\n\x1b[1m\x1b[32m--- FFmpeg Hardware Video Codecs (NVIDIA) ---\x1b[0m");
            println!("Querying NVDEC/NVENC hardware codecs in your FFmpeg installation...");

            println!("\n[Supported NVIDIA Decoders (NVDEC)]:");
            let decoders = run_ffmpeg_filter("-decoders", "cuvid");
            for dec in decoders {
                println!("  - {}", dec);
            }

            println!("\n[Supported NVIDIA Encoders (NVENC)]:");
            let encoders = run_ffmpeg_filter("-encoders", "nvenc");
            for enc in encoders {
                println!("  - {}", enc);
            }
        }
    }
}

fn run_ffmpeg_filter(arg: &str, filter: &str) -> Vec<String> {
    let output = Command::new("ffmpeg").arg(arg).output();

    let mut list = Vec::new();
    if let Ok(out) = output {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            if line.contains(filter) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    for &p in &parts {
                        if p.contains(filter) {
                            let desc = line.split(p).nth(1).unwrap_or("").trim();
                            list.push(format!("{:<15} : {}", p, desc));
                            break;
                        }
                    }
                }
            }
        }
    }
    list
}

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
            println!("\x1b[33mWarning: GPU-accelerated encoder '{}' failed or is unsupported. Falling back to software encoder '{}'...\x1b[0m", vcodec, sw_codec);
            
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

fn run_ffplay(input: &str, no_audio: bool, loop_video: bool) -> Result<(), String> {
    let mut args = vec![input.to_string()];
    if no_audio {
        args.push("-an".to_string());
    }
    if loop_video {
        args.push("-loop".to_string());
        args.push("0".to_string());
    }
    println!("Running ffplay: ffplay {}", args.join(" "));
    let status = Command::new("ffplay")
        .args(&args)
        .status()
        .map_err(|e| format!("Failed to start ffplay: {}", e))?;
    if status.success() {
        Ok(())
    } else {
        Err("ffplay exited with an error".to_string())
    }
}

fn run_ffmpeg_screenshot(input: &str, output: &str, time_ms: u32) -> Result<(), String> {
    let seconds = time_ms as f64 / 1000.0;
    let args = &[
        "-ss", &seconds.to_string(),
        "-i", input,
        "-vframes", "1",
        "-q:v", "2",
        output,
        "-y"
    ];
    println!("Running ffmpeg screenshot: ffmpeg {}", args.join(" "));
    let status = Command::new("ffmpeg")
        .args(args)
        .status()
        .map_err(|e| format!("Failed to run ffmpeg screenshot: {}", e))?;
    if status.success() {
        Ok(())
    } else {
        Err("ffmpeg screenshot exited with an error".to_string())
    }
}

fn run_subtitle_operation(
    input: &str,
    output: &str,
    operation: &str,
    sub_file: Option<&str>,
    shift_ms: Option<i32>,
    track_index: Option<u32>,
) -> Result<(), String> {
    let mut args = vec![];

    match operation {
        "extract" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            
            let track_idx = track_index.unwrap_or(0);
            args.push("-map".to_string());
            args.push(format!("0:s:{}", track_idx));
            
            args.push(output.to_string());
        }
        "convert" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push(output.to_string());
        }
        "burn" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            
            let sub_path = if let Some(sf) = sub_file {
                sf.to_string()
            } else {
                input.to_string()
            };
            
            let escaped_sub_path = sub_path
                .replace("\\", "/")
                .replace(":", "\\:");
                
            args.push("-vf".to_string());
            args.push(format!("subtitles='{}'", escaped_sub_path));
            
            args.push("-c:v".to_string());
            args.push("h264_nvenc".to_string());
            
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "sync" => {
            let offset_secs = (shift_ms.unwrap_or(0) as f64) / 1000.0;
            
            args.push("-y".to_string());
            args.push("-itsoffset".to_string());
            args.push(offset_secs.to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-c".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "merge" => {
            let sf = sub_file.ok_or_else(|| "For 'merge' operation, please specify a subtitle file with -s/--sub-file option.".to_string())?;
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-i".to_string());
            args.push(sf.to_string());
            
            args.push("-c:v".to_string());
            args.push("copy".to_string());
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            
            let out_ext = std::path::Path::new(output)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();
            if out_ext == "mp4" || out_ext == "m4v" {
                args.push("-c:s".to_string());
                args.push("mov_text".to_string());
            } else {
                args.push("-c:s".to_string());
                args.push("copy".to_string());
            }
            
            args.push("-map".to_string());
            args.push("0".to_string());
            args.push("-map".to_string());
            args.push("1".to_string());
            args.push(output.to_string());
        }
        "remove" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-c".to_string());
            args.push("copy".to_string());
            args.push("-sn".to_string());
            args.push(output.to_string());
        }
        _ => return Err(format!("Unsupported subtitle operation: {}", operation)),
    }

    println!("Running subtitle command: ffmpeg {}", args.join(" "));

    let status = Command::new("ffmpeg")
        .args(&args)
        .status()
        .map_err(|e| format!("Failed to execute ffmpeg for subtitle operation: {}", e))?;

    if status.success() {
        return Ok(());
    }

    if operation == "burn" {
        println!("\x1b[33mWarning: GPU-accelerated burn-in failed. Falling back to software encoder (libx264)...\x1b[0m");
        let mut sw_args = vec![];
        sw_args.push("-y".to_string());
        sw_args.push("-i".to_string());
        sw_args.push(input.to_string());
        
        let sub_path = if let Some(sf) = sub_file {
            sf.to_string()
        } else {
            input.to_string()
        };
        let escaped_sub_path = sub_path
            .replace("\\", "/")
            .replace(":", "\\:");
            
        sw_args.push("-vf".to_string());
        sw_args.push(format!("subtitles='{}'", escaped_sub_path));
        
        sw_args.push("-c:v".to_string());
        sw_args.push("libx264".to_string());
        sw_args.push("-preset".to_string());
        sw_args.push("medium".to_string());
        
        sw_args.push("-c:a".to_string());
        sw_args.push("copy".to_string());
        sw_args.push(output.to_string());
        
        let sw_status = Command::new("ffmpeg")
            .args(&sw_args)
            .status()
            .map_err(|e| format!("Failed to execute software fallback ffmpeg for subtitle operation: {}", e))?;
            
        if sw_status.success() {
            return Ok(());
        }
    }

    Err("ffmpeg subtitle operation failed".to_string())
}

fn run_video_edit_operation(
    input: &str,
    output: &str,
    operation: &str,
    start_time: Option<&str>,
    end_time: Option<&str>,
    duration: Option<&str>,
    crop: Option<&str>,
    rotate: Option<&str>,
    flip: Option<&str>,
    scale: Option<&str>,
    loop_count: Option<i32>,
    fade_in: Option<&str>,
    fade_out: Option<&str>,
    overlay_file: Option<&str>,
    watermark_text: Option<&str>,
    position: Option<&str>,
    additional_inputs: &[String],
) -> Result<(), String> {
    let mut args = vec![];

    match operation {
        "trim" | "cut" => {
            args.push("-y".to_string());
            if let Some(ss) = start_time {
                args.push("-ss".to_string());
                args.push(ss.to_string());
            }
            args.push("-i".to_string());
            args.push(input.to_string());
            if let Some(to) = end_time {
                args.push("-to".to_string());
                args.push(to.to_string());
            }
            if let Some(t) = duration {
                args.push("-t".to_string());
                args.push(t.to_string());
            }
            args.push("-c".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "split" => {
            let dur = duration.unwrap_or("60");
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-f".to_string());
            args.push("segment".to_string());
            args.push("-segment_time".to_string());
            args.push(dur.to_string());
            args.push("-c".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "join" | "concat" => {
            let mut files_list = vec![input.to_string()];
            files_list.extend(additional_inputs.iter().cloned());

            let list_path = format!("{}_concat_list.txt", output);
            let mut list_content = String::new();
            for f in &files_list {
                let escaped = f.replace("\\", "/").replace("'", "'\\''");
                list_content.push_str(&format!("file '{}'\n", escaped));
            }
            std::fs::write(&list_path, list_content)
                .map_err(|e| format!("Failed to write temporary concat list file: {}", e))?;

            args.push("-y".to_string());
            args.push("-f".to_string());
            args.push("concat".to_string());
            args.push("-safe".to_string());
            args.push("0".to_string());
            args.push("-i".to_string());
            args.push(list_path.clone());
            args.push("-c".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());

            println!("Running join command: ffmpeg {}", args.join(" "));
            let status = Command::new("ffmpeg")
                .args(&args)
                .status()
                .map_err(|e| {
                    let _ = std::fs::remove_file(&list_path);
                    format!("Failed to execute concat: {}", e)
                })?;

            let _ = std::fs::remove_file(&list_path);
            if status.success() {
                return Ok(());
            } else {
                return Err("ffmpeg concat failed".to_string());
            }
        }
        "crop" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            args.push(format!("crop={}", crop.unwrap_or("in_w:in_h:0:0")));
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "rotate" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            let trans = match rotate.unwrap_or("90") {
                "90" | "clock" => "transpose=1".to_string(),
                "180" => "hflip,vflip".to_string(),
                "270" | "cclock" => "transpose=2".to_string(),
                other => other.to_string(),
            };
            args.push(trans);
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "flip" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            let fl = match flip.unwrap_or("h") {
                "h" | "horizontal" => "hflip".to_string(),
                "v" | "vertical" => "vflip".to_string(),
                "both" => "hflip,vflip".to_string(),
                other => other.to_string(),
            };
            args.push(fl);
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "scale" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            let sc = scale.unwrap_or("1280x720");
            let parts: Vec<&str> = sc.split('x').collect();
            if parts.len() == 2 {
                args.push(format!("scale={}:{}", parts[0], parts[1]));
            } else {
                args.push(format!("scale={}", sc));
            }
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "stabilize" => {
            println!("Starting Pass 1 for video stabilization (detecting shakiness)...");
            let status1 = Command::new("ffmpeg")
                .args(&[
                    "-y",
                    "-i", input,
                    "-vf", "vidstabdetect=shakiness=10:accuracy=15:result=transforms.trf",
                    "-f", "null",
                    "-"
                ])
                .status()
                .map_err(|e| format!("Failed to execute stabilization pass 1: {}", e))?;

            if !status1.success() {
                let _ = std::fs::remove_file("transforms.trf");
                return Err("Stabilization pass 1 failed".to_string());
            }

            println!("Starting Pass 2 for video stabilization (applying transforms)...");
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            args.push("vidstabtransform=input=transforms.trf:zoom=2:smoothing=30".to_string());
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());

            let status2 = Command::new("ffmpeg")
                .args(&args)
                .status()
                .map_err(|e| {
                    let _ = std::fs::remove_file("transforms.trf");
                    format!("Failed to execute stabilization pass 2: {}", e)
                })?;

            let _ = std::fs::remove_file("transforms.trf");
            if status2.success() {
                return Ok(());
            } else {
                return Err("Stabilization pass 2 failed".to_string());
            }
        }
        "denoise" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            args.push("hqdn3d".to_string());
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "sharpen" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            args.push("unsharp=5:5:1.0:5:5:0.0".to_string());
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "deblock" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            args.push("deblock".to_string());
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "deinterlace" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            args.push("yadif".to_string());
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "reverse" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            args.push("reverse".to_string());
            args.push("-af".to_string());
            args.push("areverse".to_string());
            args.push(output.to_string());
        }
        "loop" => {
            let count = loop_count.unwrap_or(3);
            args.push("-y".to_string());
            args.push("-stream_loop".to_string());
            args.push(count.to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-c".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "fade" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            
            let mut vfilters = vec![];
            let mut afilters = vec![];
            
            if let Some(fi) = fade_in {
                vfilters.push(format!("fade=in:{}", fi));
                afilters.push(format!("afade=in:{}", fi));
            }
            if let Some(fo) = fade_out {
                vfilters.push(format!("fade=out:{}", fo));
                afilters.push(format!("afade=out:{}", fo));
            }
            
            if !vfilters.is_empty() {
                args.push("-vf".to_string());
                args.push(vfilters.join(","));
            }
            if !afilters.is_empty() {
                args.push("-af".to_string());
                args.push(afilters.join(","));
            }
            args.push(output.to_string());
        }
        "crossfade" => {
            let f2 = overlay_file.ok_or_else(|| "For 'crossfade' operation, please specify a second video file with -f/--file option.".to_string())?;
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-i".to_string());
            args.push(f2.to_string());
            args.push("-filter_complex".to_string());
            args.push("xfade=transition=fade:duration=1:offset=5".to_string());
            args.push(output.to_string());
        }
        "overlay" => {
            let f2 = overlay_file.ok_or_else(|| "For 'overlay' operation, please specify an overlay file with -f/--file option.".to_string())?;
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-i".to_string());
            args.push(f2.to_string());
            args.push("-filter_complex".to_string());
            let pos = position.unwrap_or("10:10");
            args.push(format!("overlay={}", pos));
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "watermark" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            
            if let Some(txt) = watermark_text {
                args.push("-vf".to_string());
                let pos = position.unwrap_or("w-tw-10:h-th-10");
                args.push(format!("drawtext=text='{}':x={}:y={}:fontsize=24:fontcolor=white", txt, pos.split(':').next().unwrap_or("w-tw-10"), pos.split(':').nth(1).unwrap_or("h-th-10")));
                args.push("-c:a".to_string());
                args.push("copy".to_string());
            } else if let Some(img) = overlay_file {
                args.push("-i".to_string());
                args.push(img.to_string());
                args.push("-filter_complex".to_string());
                let pos = position.unwrap_or("W-w-10:H-h-10");
                args.push(format!("overlay={}", pos));
                args.push("-c:a".to_string());
                args.push("copy".to_string());
            } else {
                return Err("For 'watermark' operation, please specify either --watermark-text or an image with -f/--file option.".to_string());
            }
            args.push(output.to_string());
        }
        _ => return Err(format!("Unsupported edit operation: {}", operation)),
    }

    println!("Running edit command: ffmpeg {}", args.join(" "));

    let status = Command::new("ffmpeg")
        .args(&args)
        .status()
        .map_err(|e| format!("Failed to execute ffmpeg for video edit operation: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err("ffmpeg video edit operation failed".to_string())
    }
}

fn run_color_operation(
    input: &str,
    output: &str,
    operation: &str,
    lut_file: Option<&str>,
    gamma: Option<&str>,
    shadows: Option<&str>,
    midtones: Option<&str>,
    highlights: Option<&str>,
    colorspace: Option<&str>,
    temperature: Option<f32>,
    brightness: Option<f32>,
    contrast: Option<f32>,
    saturation: Option<f32>,
    tonemap: Option<&str>,
) -> Result<(), String> {
    let mut args = vec![];

    match operation {
        "hdr2sdr" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            
            let tm = tonemap.unwrap_or("mobius");
            args.push("-vf".to_string());
            args.push(format!("zscale=t=linear:npl=100,format=gbrpf32le,tonemap=tonemap={}:desat=2,zscale=p=bt709:t=bt709:m=bt709,format=yuv420p", tm));
            
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "sdr2hdr" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            
            args.push("-vf".to_string());
            args.push("zscale=p=bt2020:t=arib-std-b67:m=bt2020nc,format=yuv420p10le".to_string());
            
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "lut" => {
            let lf = lut_file.ok_or_else(|| "For 'lut' operation, please specify a LUT file with -l/--lut option.".to_string())?;
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            
            let escaped_lut_path = lf.replace("\\", "/").replace(":", "\\:");
            args.push("-vf".to_string());
            args.push(format!("lut3d=file='{}'", escaped_lut_path));
            
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "gamma" => {
            let g = gamma.unwrap_or("1.0");
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            args.push(format!("eq=gamma={}", g));
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "grading" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            
            let mut balance = vec![];
            if let Some(s) = shadows {
                let parts: Vec<&str> = s.split(':').collect();
                if parts.len() == 3 {
                    balance.push(format!("rs={}:gs={}:bs={}", parts[0], parts[1], parts[2]));
                }
            }
            if let Some(m) = midtones {
                let parts: Vec<&str> = m.split(':').collect();
                if parts.len() == 3 {
                    balance.push(format!("rm={}:gm={}:bm={}", parts[0], parts[1], parts[2]));
                }
            }
            if let Some(h) = highlights {
                let parts: Vec<&str> = h.split(':').collect();
                if parts.len() == 3 {
                    balance.push(format!("rh={}:gh={}:bh={}", parts[0], parts[1], parts[2]));
                }
            }
            
            args.push("-vf".to_string());
            if balance.is_empty() {
                args.push("copy".to_string());
            } else {
                args.push(format!("colorbalance={}", balance.join(":")));
            }
            
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "colorspace" => {
            let cs = colorspace.unwrap_or("bt709");
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            args.push(format!("colorspace=all={}", cs));
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "whitebalance" => {
            let temp = temperature.unwrap_or(6500.0);
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            args.push(format!("colortemperature=temperature={}", temp));
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "adjust" => {
            let b = brightness.unwrap_or(0.0);
            let c = contrast.unwrap_or(1.0);
            let s = saturation.unwrap_or(1.0);
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            args.push(format!("eq=brightness={}:contrast={}:saturation={}", b, c, s));
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "tonemap" => {
            let tm = tonemap.unwrap_or("mobius");
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            args.push(format!("tonemap=tonemap={}", tm));
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        _ => return Err(format!("Unsupported color operation: {}", operation)),
    }

    println!("Running color command: ffmpeg {}", args.join(" "));

    let status = Command::new("ffmpeg")
        .args(&args)
        .status()
        .map_err(|e| format!("Failed to execute ffmpeg for color operation: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err("ffmpeg color operation failed".to_string())
    }
}
