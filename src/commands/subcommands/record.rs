use crate::commands::types::Commands;

pub fn parse(args: &[String]) -> Result<Commands, String> {
    if args.len() < 3 {
        return Err("Missing output file for record command".to_string());
    }
    let output = args[2].clone();
    let mut fps = None;
    let mut bitrate = None;
    let mut duration_sec = None;

    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--fps" => {
                if i + 1 < args.len() {
                    fps = Some(
                        args[i + 1]
                            .parse::<u32>()
                            .map_err(|_| "Invalid value for --fps")?,
                    );
                    i += 2;
                } else {
                    return Err("Missing value for --fps".to_string());
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
            "-d" | "--duration" => {
                if i + 1 < args.len() {
                    duration_sec = Some(
                        args[i + 1]
                            .parse::<u32>()
                            .map_err(|_| "Invalid value for --duration")?,
                    );
                    i += 2;
                } else {
                    return Err("Missing value for --duration".to_string());
                }
            }
            other => {
                return Err(format!("Unknown option for record: {}", other));
            }
        }
    }
    Ok(Commands::Record {
        output,
        fps,
        bitrate,
        duration_sec,
    })
}

pub fn run(output: &str, fps: Option<u32>, bitrate: Option<&str>, duration: Option<u32>) {
    let target_fps = fps.unwrap_or(60);
    let target_bitrate = bitrate.unwrap_or("8M");
    let target_duration = duration.unwrap_or(5);

    println!("\x1b[1m\x1b[32mInitializing NanAccel DXGI Screen Recording Pipeline...\x1b[0m");
    println!("  Output File      : {}", output);
    println!("  Target Frame Rate: {} FPS", target_fps);
    println!("  Target Bitrate   : {}", target_bitrate);
    println!("  Duration Limit   : {} seconds", target_duration);

    let record_result = crate::gpu_pipeline::record_gpu(
        output,
        Some(target_fps),
        Some(target_bitrate),
        Some(target_duration),
    );

    match record_result {
        Ok(_) => {
            println!(
                "\x1b[1m\x1b[32mRecording completed successfully! Saved to: {}\x1b[0m",
                output
            );
        }
        Err(e) => {
            eprintln!("\x1b[1m\x1b[31mRecording failed: {}\x1b[0m", e);
        }
    }
}
