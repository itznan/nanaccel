use crate::commands::types::Commands;

pub fn parse(args: &[String]) -> Result<Commands, String> {
    if args.len() < 5 {
        return Err("Usage: edit <operation> <input> <output> [options]\n\
                    Operations: trim, cut, split, join, concat, crop, rotate, flip, scale, stabilize, denoise, sharpen, deblock, deinterlace, reverse, loop, fade, crossfade, overlay, watermark".to_string());
    }
    let operation = args[2].to_lowercase();
    let input = args[3].clone();
    let output = args[4].clone();

    let mut start_time = None;
    let mut end_time = None;
    let mut duration = None;
    let mut crop = None;
    let mut rotate = None;
    let mut flip = None;
    let mut scale = None;
    let mut loop_count = None;
    let mut fade_in = None;
    let mut fade_out = None;
    let mut overlay_file = None;
    let mut watermark_text = None;
    let mut position = None;
    let mut additional_inputs = Vec::new();

    let mut i = 5;
    while i < args.len() {
        match args[i].as_str() {
            "-ss" | "--start" => {
                if i + 1 < args.len() {
                    start_time = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for -ss".to_string());
                }
            }
            "-to" | "--end" => {
                if i + 1 < args.len() {
                    end_time = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for -to".to_string());
                }
            }
            "-t" | "--duration" => {
                if i + 1 < args.len() {
                    duration = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for -t".to_string());
                }
            }
            "--crop" => {
                if i + 1 < args.len() {
                    crop = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --crop".to_string());
                }
            }
            "--rotate" => {
                if i + 1 < args.len() {
                    rotate = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --rotate".to_string());
                }
            }
            "--flip" => {
                if i + 1 < args.len() {
                    flip = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --flip".to_string());
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
            "--loop-count" => {
                if i + 1 < args.len() {
                    loop_count = Some(
                        args[i + 1]
                            .parse::<i32>()
                            .map_err(|_| "Invalid loop count")?,
                    );
                    i += 2;
                } else {
                    return Err("Missing value for --loop-count".to_string());
                }
            }
            "--fade-in" => {
                if i + 1 < args.len() {
                    fade_in = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --fade-in".to_string());
                }
            }
            "--fade-out" => {
                if i + 1 < args.len() {
                    fade_out = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --fade-out".to_string());
                }
            }
            "-f" | "--file" | "--overlay" => {
                if i + 1 < args.len() {
                    overlay_file = Some(args[i + 1].clone());
                    additional_inputs.push(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for file option".to_string());
                }
            }
            "--watermark-text" => {
                if i + 1 < args.len() {
                    watermark_text = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --watermark-text".to_string());
                }
            }
            "--position" => {
                if i + 1 < args.len() {
                    position = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --position".to_string());
                }
            }
            other => {
                return Err(format!("Unknown option for edit: {}", other));
            }
        }
    }

    let valid_ops = [
        "trim",
        "cut",
        "split",
        "join",
        "concat",
        "crop",
        "rotate",
        "flip",
        "scale",
        "stabilize",
        "denoise",
        "sharpen",
        "deblock",
        "deinterlace",
        "reverse",
        "loop",
        "fade",
        "crossfade",
        "overlay",
        "watermark",
    ];
    if !valid_ops.contains(&operation.as_str()) {
        return Err(format!(
            "Unknown edit operation: '{}'. Available: {}",
            operation,
            valid_ops.join(", ")
        ));
    }

    Ok(Commands::Edit {
        input,
        output,
        operation,
        start_time,
        end_time,
        duration,
        crop,
        rotate,
        flip,
        scale,
        loop_count,
        fade_in,
        fade_out,
        overlay_file,
        watermark_text,
        position,
        additional_inputs,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn run(
    input: &str,
    output: &str,
    operation: &str,
    start_time: Option<&str>,
    end_time: Option<&str>,
    duration: Option<&str>,
    crop: Option<&str>,
    rotate: Option<&str>,
    flip: Option<&str>,
    scale: Option<&str>,
    loop_count: Option<i32>,
    fade_in: Option<&str>,
    fade_out: Option<&str>,
    overlay_file: Option<&str>,
    watermark_text: Option<&str>,
    position: Option<&str>,
    additional_inputs: &[String],
) {
    println!(
        "Starting video edit operation '{}' on {} ...",
        operation, input
    );
    match run_video_edit_operation(
        input,
        output,
        operation,
        start_time,
        end_time,
        duration,
        crop,
        rotate,
        flip,
        scale,
        loop_count,
        fade_in,
        fade_out,
        overlay_file,
        watermark_text,
        position,
        additional_inputs,
    ) {
        Ok(_) => println!("\x1b[1m\x1b[32mVideo edit operation completed successfully!\x1b[0m"),
        Err(e) => eprintln!("\x1b[1m\x1b[31mVideo edit operation failed: {}\x1b[0m", e),
    }
}

#[allow(clippy::too_many_arguments)]
fn run_video_edit_operation(
    _input: &str,
    _output: &str,
    operation: &str,
    _start_time: Option<&str>,
    _end_time: Option<&str>,
    _duration: Option<&str>,
    _crop: Option<&str>,
    _rotate: Option<&str>,
    _flip: Option<&str>,
    _scale: Option<&str>,
    _loop_count: Option<i32>,
    _fade_in: Option<&str>,
    _fade_out: Option<&str>,
    _overlay_file: Option<&str>,
    _watermark_text: Option<&str>,
    _position: Option<&str>,
    _additional_inputs: &[String],
) -> Result<(), String> {
    eprintln!(
        "\x1b[33mNotice: In NanAccel, video editing operations ({}) are handled natively on the GPU \
        using NVDEC hardware decoding, Direct3D 11 Video Processors, and NVENC encoding. \
        Please launch GPU-accelerated scaling, cropping, or transcoding parameters using:
          nanaccel transcode <input> <output> --scale <w>x<h>\x1b[0m",
        operation
    );
    Err(
        "Subprocess execution is disabled. Please run native GPU transcode/playback options."
            .to_string(),
    )
}
