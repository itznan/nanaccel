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
