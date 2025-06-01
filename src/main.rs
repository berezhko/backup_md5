use anyhow::Result;
use std::env;

mod config;
mod directory;
mod file_processor;
mod hash;

use config::Config;
use file_processor::process_files_with_extensions;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!(
            "Usage: {} <source_dir> <target_base> <config_file>",
            args[0]
        );
        eprintln!("Example: {} ./src ./target config.toml", args[0]);
        std::process::exit(1);
    }

    let source_dir = &args[1];
    let target_base = &args[2];
    let config_file = &args[3];

    let config = Config::from_file(config_file)?;
    process_files_with_extensions(source_dir, target_base, &config.extensions)?;

    Ok(())
}
