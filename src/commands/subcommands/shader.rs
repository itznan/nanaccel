use crate::commands::types::Commands;

pub fn parse(args: &[String]) -> Result<Commands, String> {
    if args.len() < 5 {
        return Err("Usage: nanaccel shader <input> <output> <shader_file>".to_string());
    }
    let input = args[2].clone();
    let output = args[3].clone();
    let shader_file = args[4].clone();
    Ok(Commands::Shader {
        input,
        output,
        shader_file,
    })
}

pub fn run(input: &str, output: &str, shader_file: &str) {
    println!("\x1b[1m\x1b[32mStarting HLSL Shader Effect Pipeline...\x1b[0m");
    println!("  Source Input     : {}", input);
    println!("  Destination File : {}", output);
    println!("  HLSL Shader File : {}", shader_file);

    let result = crate::gpu_pipeline::shader_gpu(input, output, shader_file);
    match result {
        Ok(_) => {
            println!(
                "\x1b[1m\x1b[32mShader filter pipeline completed successfully! Saved to: {}\x1b[0m",
                output
            );
        }
        Err(e) => {
            eprintln!("\x1b[1m\x1b[31mShader pipeline failed: {}\x1b[0m", e);
        }
    }
}
