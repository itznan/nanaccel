mod commands;
mod gpu_pipeline;
mod gpu_utils;
mod mux;

fn main() {
    if !gpu_utils::check_nvidia_gpu() {
        println!("gpu not detected");
        std::process::exit(1);
    }

    let args = match commands::parse_args() {
        Ok(cmd) => cmd,
        Err(err) => {
            eprintln!("\x1b[31mError: {}\x1b[0m", err);
            commands::print_help();
            std::process::exit(1);
        }
    };

    commands::execute(args);
}
