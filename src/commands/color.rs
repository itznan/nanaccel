use std::process::Command;

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
        "Starting color processing operation '{}' on {} ...",
        operation, input
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
            println!("\x1b[1m\x1b[32mColor processing operation completed successfully!\x1b[0m")
        }
        Err(e) => eprintln!(
            "\x1b[1m\x1b[31mColor processing operation failed: {}\x1b[0m",
            e
        ),
    }
}

fn run_color_operation(
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
) -> Result<(), String> {
    let mut args = vec![];

    match operation {
        "hdr2sdr" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());

            let tm = tonemap.unwrap_or("mobius");
            args.push("-vf".to_string());
            args.push(format!("zscale=t=linear:npl=100,format=gbrpf32le,tonemap=tonemap={}:desat=2,zscale=p=bt709:t=bt709:m=bt709,format=yuv420p", tm));

            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "sdr2hdr" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());

            args.push("-vf".to_string());
            args.push("zscale=p=bt2020:t=arib-std-b67:m=bt2020nc,format=yuv420p10le".to_string());

            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "lut" => {
            let lf = lut_file.ok_or_else(|| {
                "For 'lut' operation, please specify a LUT file with -l/--lut option.".to_string()
            })?;
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());

            let escaped_lut_path = lf.replace("\\", "/").replace(":", "\\:");
            args.push("-vf".to_string());
            args.push(format!("lut3d=file='{}'", escaped_lut_path));

            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "gamma" => {
            let g = gamma.unwrap_or("1.0");
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            args.push(format!("eq=gamma={}", g));
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "grading" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());

            let mut balance = vec![];
            if let Some(s) = shadows {
                let parts: Vec<&str> = s.split(':').collect();
                if parts.len() == 3 {
                    balance.push(format!("rs={}:gs={}:bs={}", parts[0], parts[1], parts[2]));
                }
            }
            if let Some(m) = midtones {
                let parts: Vec<&str> = m.split(':').collect();
                if parts.len() == 3 {
                    balance.push(format!("rm={}:gm={}:bm={}", parts[0], parts[1], parts[2]));
                }
            }
            if let Some(h) = highlights {
                let parts: Vec<&str> = h.split(':').collect();
                if parts.len() == 3 {
                    balance.push(format!("rh={}:gh={}:bh={}", parts[0], parts[1], parts[2]));
                }
            }

            args.push("-vf".to_string());
            if balance.is_empty() {
                args.push("copy".to_string());
            } else {
                args.push(format!("colorbalance={}", balance.join(":")));
            }

            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "colorspace" => {
            let cs = colorspace.unwrap_or("bt709");
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            args.push(format!("colorspace=all={}", cs));
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "whitebalance" => {
            let temp = temperature.unwrap_or(6500.0);
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            args.push(format!("colortemperature=temperature={}", temp));
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "adjust" => {
            let b = brightness.unwrap_or(0.0);
            let c = contrast.unwrap_or(1.0);
            let s = saturation.unwrap_or(1.0);
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            args.push(format!(
                "eq=brightness={}:contrast={}:saturation={}",
                b, c, s
            ));
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "tonemap" => {
            let tm = tonemap.unwrap_or("mobius");
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            args.push(format!("tonemap=tonemap={}", tm));
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        _ => return Err(format!("Unsupported color operation: {}", operation)),
    }

    println!("Running color command: ffmpeg {}", args.join(" "));

    let status = Command::new("ffmpeg")
        .args(&args)
        .status()
        .map_err(|e| format!("Failed to execute ffmpeg for color operation: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err("ffmpeg color operation failed".to_string())
    }
}
