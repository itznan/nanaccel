use crate::commands::types::Commands;

pub fn parse(args: &[String]) -> Result<Commands, String> {
    if args.len() < 5 {
        return Err("Usage: color <operation> <input> <output> [options]\n\
                    Operations: hdr2sdr, sdr2hdr, lut, gamma, grading, colorspace, whitebalance, adjust, tonemap".to_string());
    }
    let operation = args[2].to_lowercase();
    let input = args[3].clone();
    let output = args[4].clone();

    let mut lut_file = None;
    let mut gamma = None;
    let mut shadows = None;
    let mut midtones = None;
    let mut highlights = None;
    let mut colorspace = None;
    let mut temperature = None;
    let mut brightness = None;
    let mut contrast = None;
    let mut saturation = None;
    let mut tonemap = None;

    let mut i = 5;
    while i < args.len() {
        match args[i].as_str() {
            "-l" | "--lut" | "--lut-file" => {
                if i + 1 < args.len() {
                    lut_file = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --lut-file".to_string());
                }
            }
            "--gamma" => {
                if i + 1 < args.len() {
                    gamma = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --gamma".to_string());
                }
            }
            "--shadows" => {
                if i + 1 < args.len() {
                    shadows = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --shadows".to_string());
                }
            }
            "--midtones" => {
                if i + 1 < args.len() {
                    midtones = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --midtones".to_string());
                }
            }
            "--highlights" => {
                if i + 1 < args.len() {
                    highlights = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --highlights".to_string());
                }
            }
            "--colorspace" | "--space" => {
                if i + 1 < args.len() {
                    colorspace = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --colorspace".to_string());
                }
            }
            "--temp" | "--temperature" => {
                if i + 1 < args.len() {
                    temperature = Some(
                        args[i + 1]
                            .parse::<f32>()
                            .map_err(|_| "Invalid temperature value")?,
                    );
                    i += 2;
                } else {
                    return Err("Missing value for --temp".to_string());
                }
            }
            "--brightness" => {
                if i + 1 < args.len() {
                    brightness = Some(
                        args[i + 1]
                            .parse::<f32>()
                            .map_err(|_| "Invalid brightness value")?,
                    );
                    i += 2;
                } else {
                    return Err("Missing value for --brightness".to_string());
                }
            }
            "--contrast" => {
                if i + 1 < args.len() {
                    contrast = Some(
                        args[i + 1]
                            .parse::<f32>()
                            .map_err(|_| "Invalid contrast value")?,
                    );
                    i += 2;
                } else {
                    return Err("Missing value for --contrast".to_string());
                }
            }
            "--saturation" => {
                if i + 1 < args.len() {
                    saturation = Some(
                        args[i + 1]
                            .parse::<f32>()
                            .map_err(|_| "Invalid saturation value")?,
                    );
                    i += 2;
                } else {
                    return Err("Missing value for --saturation".to_string());
                }
            }
            "--tonemap" => {
                if i + 1 < args.len() {
                    tonemap = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --tonemap".to_string());
                }
            }
            other => {
                return Err(format!("Unknown option for color: {}", other));
            }
        }
    }

    let valid_ops = [
        "hdr2sdr",
        "sdr2hdr",
        "lut",
        "gamma",
        "grading",
        "colorspace",
        "whitebalance",
        "adjust",
        "tonemap",
    ];
    if !valid_ops.contains(&operation.as_str()) {
        return Err(format!(
            "Unknown color operation: '{}'. Available: {}",
            operation,
            valid_ops.join(", ")
        ));
    }

    Ok(Commands::Color {
        input,
        output,
        operation,
        lut_file,
        gamma,
        shadows,
        midtones,
        highlights,
        colorspace,
        temperature,
        brightness,
        contrast,
        saturation,
        tonemap,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn run(
    input: &str,
    output: &str,
    operation: &str,
    lut_file: Option<&str>,
    gamma: Option<&str>,
    shadows: Option<&str>,
    midtones: Option<&str>,
    highlights: Option<&str>,
    colorspace: Option<&str>,
    temperature: Option<f32>,
    brightness: Option<f32>,
    contrast: Option<f32>,
    saturation: Option<f32>,
    tonemap: Option<&str>,
) {
    println!(
        "Starting color processing operation '{}' on {} to {}...",
        operation, input, output
    );
    match run_color_operation(
        input,
        output,
        operation,
        lut_file,
        gamma,
        shadows,
        midtones,
        highlights,
        colorspace,
        temperature,
        brightness,
        contrast,
        saturation,
        tonemap,
    ) {
        Ok(_) => {
            println!("\x1b[1m\x1b[32mColor processing completed successfully!\x1b[0m")
        }
        Err(e) => eprintln!("\x1b[1m\x1b[31mColor processing failed: {}\x1b[0m", e),
    }
}

#[allow(clippy::too_many_arguments)]
fn run_color_operation(
    _input: &str,
    _output: &str,
    operation: &str,
    _lut_file: Option<&str>,
    _gamma: Option<&str>,
    _shadows: Option<&str>,
    _midtones: Option<&str>,
    _highlights: Option<&str>,
    _colorspace: Option<&str>,
    _temperature: Option<f32>,
    _brightness: Option<f32>,
    _contrast: Option<f32>,
    _saturation: Option<f32>,
    _tonemap: Option<&str>,
) -> Result<(), String> {
    eprintln!(
        "\x1b[33mNotice: In NanAccel, video color processing operations ({}) are handled natively \
        on the GPU using Direct3D 11 Video Processors and custom shaders to maintain zero-CPU copy overhead. \
        Please launch GPU accelerated color operations during playback using:
          nanaccel play <input>
        Or during hardware-accelerated transcoding using:
          nanaccel transcode <input> <output>\x1b[0m",
        operation
    );
    Err(
        "Subprocess execution is disabled. Please run native GPU transcode/playback options."
            .to_string(),
    )
}
