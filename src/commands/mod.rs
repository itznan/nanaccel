pub mod play;
pub mod transcode;
pub mod screenshot;
pub mod subtitle;
pub mod edit;
pub mod color;
pub mod info;
pub mod audio;

#[derive(Debug)]
#[allow(dead_code)]
pub enum Commands {
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
    Audio {
        input: String,
        output: String,
        operation: String,
        volume: Option<String>,
        noise_reduction: Option<String>,
        threshold: Option<String>,
        ratio: Option<String>,
        limit: Option<String>,
        gain: Option<String>,
        frequency: Option<String>,
        pitch: Option<f32>,
        tempo: Option<f32>,
        loudness: Option<String>,
        silence_db: Option<String>,
        silence_duration: Option<String>,
    },
    Info,
}

pub fn print_help() {
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
    println!(
        "  \x1b[32maudio <operation> <input> <output>\x1b[0m Audio editing utility"
    );
    println!("       Operations:                  volume, denoise, compress, limit, eq, pitch, tempo,");
    println!("                                    reverb, echo, bass, silencedetect, normalize");
    println!("       [--volume <val>]             Volume scale multiplier or dB (default: 1.0)");
    println!("       [--nr <val>]                 Noise reduction level (default: 12)");
    println!("       [--threshold / --ratio <val>] Compression parameters (default: -21dB / 4)");
    println!("       [--limit <val>]              Lookahead limiter input/output limit (default: 0.1)");
    println!("       [--freq / --gain <val>]      Parametric equalizer band options");
    println!("       [--pitch <val>]              Pitch shift scale (default: 1.0)");
    println!("       [--tempo <val>]              Tempo speed scale (default: 1.0)");
    println!("       [--loudness <val>]           Target LUFS normalization loudness (default: -16)");
    println!("       [--silence-db / --silence-duration <val>] Silence detection parameters");
    println!();
}

pub fn parse_args() -> Result<Commands, String> {
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
        "audio" => {
            if args.len() < 5 {
                return Err("Usage: audio <operation> <input> <output> [options]\n\
                            Operations: volume, denoise, compress, limit, eq, pitch, tempo, reverb, echo, bass, silencedetect, normalize".to_string());
            }
            let operation = args[2].to_lowercase();
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
                            pitch = Some(args[i + 1].parse::<f32>().map_err(|_| "Invalid pitch value")?);
                            i += 2;
                        } else {
                            return Err("Missing value for --pitch".to_string());
                        }
                    }
                    "--tempo" => {
                        if i + 1 < args.len() {
                            tempo = Some(args[i + 1].parse::<f32>().map_err(|_| "Invalid tempo value")?);
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

            let valid_ops = ["volume", "denoise", "compress", "limit", "eq", "pitch", "tempo", "reverb", "echo", "bass", "silencedetect", "normalize"];
            if !valid_ops.contains(&operation.as_str()) {
                return Err(format!("Unknown audio operation: '{}'. Available: {}", operation, valid_ops.join(", ")));
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
        sub => Err(format!("Unknown subcommand: {}", sub)),
    }
}

pub fn execute(cmd: Commands) {
    match cmd {
        Commands::Play {
            input,
            decoder: _,
            no_audio,
            loop_video,
        } => {
            play::run(&input, no_audio, loop_video);
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
            transcode::run(
                &input,
                &output,
                &codec,
                &preset,
                bitrate.as_deref(),
                scale.as_deref(),
                transcode_audio,
                audio_codec.as_deref(),
            );
        }

        Commands::Screenshot {
            input,
            output,
            time_ms,
        } => {
            screenshot::run(&input, &output, time_ms);
        }

        Commands::Subtitle {
            input,
            output,
            operation,
            sub_file,
            shift_ms,
            track_index,
        } => {
            subtitle::run(
                &input,
                &output,
                &operation,
                sub_file.as_deref(),
                shift_ms,
                track_index,
            );
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
            edit::run(
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
            );
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
            color::run(
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
            );
        }

        Commands::Audio {
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
        } => {
            audio::run(
                &input,
                &output,
                &operation,
                volume.as_deref(),
                noise_reduction.as_deref(),
                threshold.as_deref(),
                ratio.as_deref(),
                limit.as_deref(),
                gain.as_deref(),
                frequency.as_deref(),
                pitch,
                tempo,
                loudness.as_deref(),
                silence_db.as_deref(),
                silence_duration.as_deref(),
            );
        }

        Commands::Info => {
            info::run();
        }
    }
}
