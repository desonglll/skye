use clap::Parser;
use colored::*;
use env_logger::{self};
use log::info;
use skye::{read_repos_from_file, safe_write_to_file, sync_commits};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    version = "0.0.1",
    author = "desonglll",
    about = "A cli for sync setup.json of bizyair cce dockerfile."
)]
struct CliArgs {
    /// Source file path with json format.
    #[arg(short, long)]
    pub source: PathBuf,
    /// Target file path with json format.
    #[arg(short, long)]
    pub target: PathBuf,
    /// New target file saved path with json format.
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    /// Whether to append missing object from source to target.
    #[arg(short, long)]
    pub append: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        std::env::set_var("RUST_LOG", "info");
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
    let updated_target = sync_commits(source_repos, target_repos, args.append);
    info!("safely write back...");
    safe_write_to_file(args.output.unwrap_or(args.target), &updated_target)?;
    info!("{}", String::from("success!").green());
    Ok(())
}
