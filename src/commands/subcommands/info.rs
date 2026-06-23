use crate::commands::types::Commands;
use crate::gpu_utils::{query_dynamic_gpu_stats, query_static_gpu_info};

pub fn parse(_args: &[String]) -> Result<Commands, String> {
    Ok(Commands::Info)
}

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
}
