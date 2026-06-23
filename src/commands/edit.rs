use std::process::Command;

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
) -> Result<(), String> {
    let mut args = vec![];

    match operation {
        "trim" | "cut" => {
            args.push("-y".to_string());
            if let Some(ss) = start_time {
                args.push("-ss".to_string());
                args.push(ss.to_string());
            }
            args.push("-i".to_string());
            args.push(input.to_string());
            if let Some(to) = end_time {
                args.push("-to".to_string());
                args.push(to.to_string());
            }
            if let Some(t) = duration {
                args.push("-t".to_string());
                args.push(t.to_string());
            }
            args.push("-c".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "split" => {
            let dur = duration.unwrap_or("60");
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-f".to_string());
            args.push("segment".to_string());
            args.push("-segment_time".to_string());
            args.push(dur.to_string());
            args.push("-c".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "join" | "concat" => {
            let mut files_list = vec![input.to_string()];
            files_list.extend(additional_inputs.iter().cloned());

            let list_path = format!("{}_concat_list.txt", output);
            let mut list_content = String::new();
            for f in &files_list {
                let escaped = f.replace("\\", "/").replace("'", "'\\''");
                list_content.push_str(&format!("file '{}'\n", escaped));
            }
            std::fs::write(&list_path, list_content)
                .map_err(|e| format!("Failed to write temporary concat list file: {}", e))?;

            args.push("-y".to_string());
            args.push("-f".to_string());
            args.push("concat".to_string());
            args.push("-safe".to_string());
            args.push("0".to_string());
            args.push("-i".to_string());
            args.push(list_path.clone());
            args.push("-c".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());

            println!("Running join command: ffmpeg {}", args.join(" "));
            let status = Command::new("ffmpeg").args(&args).status().map_err(|e| {
                let _ = std::fs::remove_file(&list_path);
                format!("Failed to execute concat: {}", e)
            })?;

            let _ = std::fs::remove_file(&list_path);
            if status.success() {
                return Ok(());
            } else {
                return Err("ffmpeg concat failed".to_string());
            }
        }
        "crop" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            args.push(format!("crop={}", crop.unwrap_or("in_w:in_h:0:0")));
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "rotate" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            let trans = match rotate.unwrap_or("90") {
                "90" | "clock" => "transpose=1".to_string(),
                "180" => "hflip,vflip".to_string(),
                "270" | "cclock" => "transpose=2".to_string(),
                other => other.to_string(),
            };
            args.push(trans);
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "flip" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            let fl = match flip.unwrap_or("h") {
                "h" | "horizontal" => "hflip".to_string(),
                "v" | "vertical" => "vflip".to_string(),
                "both" => "hflip,vflip".to_string(),
                other => other.to_string(),
            };
            args.push(fl);
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "scale" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            let sc = scale.unwrap_or("1280x720");
            let parts: Vec<&str> = sc.split('x').collect();
            if parts.len() == 2 {
                args.push(format!("scale={}:{}", parts[0], parts[1]));
            } else {
                args.push(format!("scale={}", sc));
            }
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "stabilize" => {
            println!("Starting Pass 1 for video stabilization (detecting shakiness)...");
            let status1 = Command::new("ffmpeg")
                .args([
                    "-y",
                    "-i",
                    input,
                    "-vf",
                    "vidstabdetect=shakiness=10:accuracy=15:result=transforms.trf",
                    "-f",
                    "null",
                    "-",
                ])
                .status()
                .map_err(|e| format!("Failed to execute stabilization pass 1: {}", e))?;

            if !status1.success() {
                let _ = std::fs::remove_file("transforms.trf");
                return Err("Stabilization pass 1 failed".to_string());
            }

            println!("Starting Pass 2 for video stabilization (applying transforms)...");
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            args.push("vidstabtransform=input=transforms.trf:zoom=2:smoothing=30".to_string());
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());

            let status2 = Command::new("ffmpeg").args(&args).status().map_err(|e| {
                let _ = std::fs::remove_file("transforms.trf");
                format!("Failed to execute stabilization pass 2: {}", e)
            })?;

            let _ = std::fs::remove_file("transforms.trf");
            if status2.success() {
                return Ok(());
            } else {
                return Err("Stabilization pass 2 failed".to_string());
            }
        }
        "denoise" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            args.push("hqdn3d".to_string());
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "sharpen" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            args.push("unsharp=5:5:1.0:5:5:0.0".to_string());
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "deblock" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            args.push("deblock".to_string());
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "deinterlace" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            args.push("yadif".to_string());
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "reverse" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-vf".to_string());
            args.push("reverse".to_string());
            args.push("-af".to_string());
            args.push("areverse".to_string());
            args.push(output.to_string());
        }
        "loop" => {
            let count = loop_count.unwrap_or(3);
            args.push("-y".to_string());
            args.push("-stream_loop".to_string());
            args.push(count.to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-c".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "fade" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());

            let mut vfilters = vec![];
            let mut afilters = vec![];

            if let Some(fi) = fade_in {
                vfilters.push(format!("fade=in:{}", fi));
                afilters.push(format!("afade=in:{}", fi));
            }
            if let Some(fo) = fade_out {
                vfilters.push(format!("fade=out:{}", fo));
                afilters.push(format!("afade=out:{}", fo));
            }

            if !vfilters.is_empty() {
                args.push("-vf".to_string());
                args.push(vfilters.join(","));
            }
            if !afilters.is_empty() {
                args.push("-af".to_string());
                args.push(afilters.join(","));
            }
            args.push(output.to_string());
        }
        "crossfade" => {
            let f2 = overlay_file.ok_or_else(|| "For 'crossfade' operation, please specify a second video file with -f/--file option.".to_string())?;
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-i".to_string());
            args.push(f2.to_string());
            args.push("-filter_complex".to_string());
            args.push("xfade=transition=fade:duration=1:offset=5".to_string());
            args.push(output.to_string());
        }
        "overlay" => {
            let f2 = overlay_file.ok_or_else(|| {
                "For 'overlay' operation, please specify an overlay file with -f/--file option."
                    .to_string()
            })?;
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());
            args.push("-i".to_string());
            args.push(f2.to_string());
            args.push("-filter_complex".to_string());
            let pos = position.unwrap_or("10:10");
            args.push(format!("overlay={}", pos));
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push(output.to_string());
        }
        "watermark" => {
            args.push("-y".to_string());
            args.push("-i".to_string());
            args.push(input.to_string());

            if let Some(txt) = watermark_text {
                args.push("-vf".to_string());
                let pos = position.unwrap_or("w-tw-10:h-th-10");
                args.push(format!(
                    "drawtext=text='{}':x={}:y={}:fontsize=24:fontcolor=white",
                    txt,
                    pos.split(':').next().unwrap_or("w-tw-10"),
                    pos.split(':').nth(1).unwrap_or("h-th-10")
                ));
                args.push("-c:a".to_string());
                args.push("copy".to_string());
            } else if let Some(img) = overlay_file {
                args.push("-i".to_string());
                args.push(img.to_string());
                args.push("-filter_complex".to_string());
                let pos = position.unwrap_or("W-w-10:H-h-10");
                args.push(format!("overlay={}", pos));
                args.push("-c:a".to_string());
                args.push("copy".to_string());
            } else {
                return Err("For 'watermark' operation, please specify either --watermark-text or an image with -f/--file option.".to_string());
            }
            args.push(output.to_string());
        }
        _ => return Err(format!("Unsupported edit operation: {}", operation)),
    }

    println!("Running edit command: ffmpeg {}", args.join(" "));

    let status = Command::new("ffmpeg")
        .args(&args)
        .status()
        .map_err(|e| format!("Failed to execute ffmpeg for video edit operation: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err("ffmpeg video edit operation failed".to_string())
    }
}
