use crate::commands::types::Commands;

pub fn parse(args: &[String]) -> Result<Commands, String> {
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
            eprintln!("\x1b[1m\x1b[31mScreenshot extraction failed: {}\x1b[0m", e);
        }
    }
}
