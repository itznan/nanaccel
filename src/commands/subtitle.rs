use std::process::Command;

pub fn run(
    input: &str,
    output: &str,
    operation: &str,
    sub_file: Option<&str>,
    shift_ms: Option<i32>,
    track_index: Option<u32>,
) {
    println!("Starting subtitle operation '{}' on {} ...", operation, input);
    match run_subtitle_operation(input, output, operation, sub_file, shift_ms, track_index) {
        Ok(_) => println!("\x1b[1m\x1b[32mSubtitle operation completed successfully!\x1b[0m"),
        Err(e) => eprintln!("\x1b[1m\x1b[31mSubtitle operation failed: {}\x1b[0m", e),
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
