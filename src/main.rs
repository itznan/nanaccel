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
    },
    Screenshot {
        input: String,
        output: String,
        time_ms: u32,
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
    println!("       [-c, --codec <codec>]        Target codec: h264, hevc, av1 (default: h264)");
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

            let mut i = 4;
            while i < args.len() {
                match args[i].as_str() {
                    "-c" | "--codec" => {
                        if i + 1 < args.len() {
                            codec = args[i + 1].clone();
                            i += 2;
                        } else {
                            return Err("Missing value for --codec".to_string());
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
                Err(e) => eprintln!("Playback failed: {}", e),
            }
        }

        Commands::Transcode {
            input,
            output,
            codec,
            preset,
            bitrate,
            scale,
            transcode_audio: _,
        } => {
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
                    println!("\x1b[1m\x1b[31mTranscoding failed: {}\x1b[0m", e);
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
                    println!("\x1b[1m\x1b[31mScreenshot failed: {}\x1b[0m", e);
                }
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
