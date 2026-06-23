#[allow(clippy::too_many_arguments)]
pub fn run(
    input: &str,
    output: &str,
    codec: &str,
    preset: &str,
    bitrate: Option<&str>,
    scale: Option<&str>,
    transcode_audio: bool,
    audio_codec: Option<&str>,
) {
    let use_native_gpu = (codec == "h264" || codec == "hevc") && !transcode_audio && audio_codec.is_none();

    if use_native_gpu {
        let ext = std::path::Path::new(output)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        if ext != "mp4" && ext != "mov" && ext != "m4v" && ext != "3gp" && !ext.is_empty() {
            println!(
                "\x1b[33mWarning: NanAccel natively encodes and muxes to standard ISO-MP4/MOV formats. \
                The output stream will be written as a valid MP4/MOV container structure inside the requested '.{}' file.\x1b[0m",
                ext
            );
        }
        println!("Starting GPU transcode: {} -> {} ...", input, output);
        let transcode_result =
            crate::gpu_pipeline::transcode_gpu(input, output, codec, preset, bitrate, scale);

        match transcode_result {
            Ok(_) => {
                println!("\x1b[1m\x1b[32mTranscoding completed successfully via GPU!\x1b[0m");
            }
            Err(e) => {
                eprintln!("\x1b[1m\x1b[31mTranscoding failed: {}\x1b[0m", e);
            }
        }
    } else {
        eprintln!(
            "\x1b[1m\x1b[31mTranscoding error: NanAccel's native GPU pipeline only supports H.264 and HEVC video stream transcoding without audio transcoding.\x1b[0m"
        );
    }
}
