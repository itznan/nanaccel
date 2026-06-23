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
    Err("FFmpeg subprocess execution disabled. Please run native GPU transcode/playback options.".to_string())
}
