use std::process::Command;

pub fn run(input: &str, output: &str, time_ms: u32) {
    println!(
        "Extracting GPU screenshot from {} to {} at time {} ms...",
        input, output, time_ms
    );
    let screenshot_result = crate::gpu_pipeline::screenshot_gpu(input, output, time_ms);
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
            if let Err(fe) = run_ffmpeg_screenshot(input, output, time_ms) {
                println!("\x1b[1m\x1b[31mScreenshot failed: {}\x1b[0m", fe);
            } else {
                println!(
                    "\x1b[1m\x1b[32mScreenshot extracted and saved successfully via FFmpeg!\x1b[0m"
                );
            }
        }
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
