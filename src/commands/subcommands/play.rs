use crate::commands::types::Commands;

pub fn parse(args: &[String]) -> Result<Commands, String> {
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

pub fn run(input: &str, no_audio: bool, loop_video: bool) {
    println!("Initializing GPU Playback for: {} ...", input);
    let play_result = crate::gpu_pipeline::play_gpu(input, no_audio, loop_video);
    match play_result {
        Ok(_) => println!("Playback finished."),
        Err(e) => {
            eprintln!("\x1b[1m\x1b[31mNative GPU playback failed: {}\x1b[0m", e);
        }
    }
}
