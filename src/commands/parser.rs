use super::help;
use super::subcommands::{
    audio, color, edit, info, mux, overlay, play, record, screenshot, shader, subtitle, transcode,
};
use super::types::Commands;

pub fn parse_args() -> Result<Commands, String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        return Err("No subcommand specified".to_string());
    }

    match args[1].as_str() {
        "help" | "-h" | "--help" => {
            help::print_help();
            std::process::exit(0);
        }
        "info" => info::parse(&args),
        "play" => play::parse(&args),
        "transcode" => transcode::parse(&args),
        "screenshot" => screenshot::parse(&args),
        "subtitle" | "sub" => subtitle::parse(&args),
        "edit" => edit::parse(&args),
        "color" => color::parse(&args),
        "audio" => audio::parse(&args),
        "record" => record::parse(&args),
        "overlay" => overlay::parse(&args),
        "shader" => shader::parse(&args),
        "mux" => mux::parse(&args),
        sub => Err(format!("Unknown subcommand: {}", sub)),
    }
}
