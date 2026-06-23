use std::process::Command;

pub fn run(input: &str, no_audio: bool, loop_video: bool) {
    println!("Initializing GPU Playback for: {} ...", input);
    let play_result = crate::gpu_pipeline::play_gpu(input, no_audio, loop_video);
    match play_result {
        Ok(_) => println!("Playback finished."),
        Err(e) => {
            println!(
                "\x1b[33mWarning: Native GPU playback failed ({}). Falling back to ffplay...\x1b[0m",
                e
            );
            if let Err(fe) = run_ffplay(input, no_audio, loop_video) {
                eprintln!("\x1b[31mPlayback failed: {}\x1b[0m", fe);
            } else {
                println!("Playback finished.");
            }
        }
    }
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
