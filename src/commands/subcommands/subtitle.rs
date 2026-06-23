use crate::commands::types::Commands;

pub fn parse(args: &[String]) -> Result<Commands, String> {
    if args.len() < 5 {
        return Err("Usage: subtitle <operation> <input> <output> [options]\n\
                    Operations: extract, convert, burn, sync, merge, remove"
            .to_string());
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
                    track_index = Some(
                        args[i + 1]
                            .parse::<u32>()
                            .map_err(|_| "Invalid track index")?,
                    );
                    i += 2;
                } else {
                    return Err("Missing value for --track".to_string());
                }
            }
            "--shift" => {
                if i + 1 < args.len() {
                    shift_ms = Some(
                        args[i + 1]
                            .parse::<i32>()
                            .map_err(|_| "Invalid shift milliseconds value")?,
                    );
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
        return Err(format!(
            "Unknown subtitle operation: '{}'. Available: extract, convert, burn, sync, merge, remove",
            operation
        ));
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

pub fn run(
    input: &str,
    output: &str,
    operation: &str,
    sub_file: Option<&str>,
    shift_ms: Option<i32>,
    track_index: Option<u32>,
) {
    println!(
        "Starting subtitle operation '{}' on {} ...",
        operation, input
    );
    match run_subtitle_operation(input, output, operation, sub_file, shift_ms, track_index) {
        Ok(_) => println!("\x1b[1m\x1b[32mSubtitle operation completed successfully!\x1b[0m"),
        Err(e) => eprintln!("\x1b[1m\x1b[31mSubtitle operation failed: {}\x1b[0m", e),
    }
}

fn run_subtitle_operation(
    _input: &str,
    _output: &str,
    operation: &str,
    _sub_file: Option<&str>,
    _shift_ms: Option<i32>,
    _track_index: Option<u32>,
) -> Result<(), String> {
    eprintln!(
        "\x1b[33mNotice: In NanAccel, subtitle stream operations ({}) are handled natively through \
        our container parsing logic. Burning subtitles into live video frames can be processed \
        during transcoding commands if specified via source mapping.\x1b[0m",
        operation
    );
    Err(
        "Subprocess execution is disabled. Please run native GPU transcode/playback options."
            .to_string(),
    )
}
