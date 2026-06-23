use crate::gpu_utils::{query_dynamic_gpu_stats, query_static_gpu_info};
use std::process::Command;

pub fn run() {
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
