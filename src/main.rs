#[warn(unused_imports)]
use clap::Parser;
use log::{debug, error, info, trace, warn};
use std::{env, path::PathBuf};

mod cakecutter;
mod errors;
mod generate;
mod template;
mod user_config;

use cakecutter::cakecutter;
use template::parse_template_input;

#[derive(Parser, Debug)]
#[command(name="CakeCutter", author="Boudewijn Aasman", version="1.0", about="Something", long_about = None)]
#[command(propagate_version = true)]
struct Main {
    template: String,
    #[arg(short, long)]
    directory: Option<PathBuf>,
    #[arg(short, long)]
    config: Option<PathBuf>,
    #[arg(default_value_t = 0)]
    verbose: u8,
    #[arg(long, default_value_t = false)]
    overwrite_if_exists: bool,
    #[arg(short, long)]
    output_dir: Option<PathBuf>,
    #[arg(short, long, default_value_t = false)]
    replay: bool,
    #[arg(short, long, default_value_t = false)]
    no_input: bool,
    #[arg(long)]
    checkout: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    colog::init();
    info!("Initializing cakecutter...");
    let args = Main::parse();

    let template_type = parse_template_input(args.template, args.directory);
    let config = args
        .config
        .unwrap_or_else(|| PathBuf::from("./config.json".to_string()));

    cakecutter(
        template_type,
        args.output_dir,
        config,
        args.replay,
        args.no_input,
        args.overwrite_if_exists,
    )?;
    Ok(())
}
