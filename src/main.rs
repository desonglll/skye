use clap::Parser;
use colored::*;
use env_logger::{self};
use log::{debug, info};
use skye::{CliArgs, read_repos_from_file, safe_write_to_file, sync_commits};
use std::process::exit;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        std::env::set_var("RUST_LOG", "debug");
    }

    env_logger::init();

    let mut args = CliArgs::parse();

    if args.output.is_none() {
        args.output = Some(args.target.clone());
    };

    info!("source_path: {:?}", args.source);
    info!("target_path: {:?}", args.target);
    info!("new_target_path: {:?}", args.output);

    let source_repos = read_repos_from_file(&args.source)
        .expect(&format!("failed to read from file {:?}", &args.source));

    let target_repos = read_repos_from_file(&args.target).unwrap_or_else(|_| {
        info!("not found valid file, create a new file: {:?}", args.target);
        Vec::new()
    });

    info!(
        "start sync {:?} -> {:?} to {:?}",
        args.source, args.target, args.output
    );
    let updated_target = sync_commits(source_repos, target_repos, &args);
    info!("safely write back...");
    match args.output.clone() {
        None => {
            eprintln!("not a valid output path: None");
            exit(1);
        }
        Some(output) => {
            safe_write_to_file(output, &updated_target, &args)?;
        }
    }
    info!("{}", String::from("success!").green());
    Ok(())
}
