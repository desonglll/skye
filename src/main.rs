use clap::Parser;
use colored::*;
use env_logger::{self};
use log::{error, info};
use skye::{Cli, read_repos_from_file, safe_write_to_file, sync_commits};
use std::process::exit;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        std::env::set_var("RUST_LOG", "debug");
    }

    env_logger::init();

    let mut args = Cli::parse();
    match &mut args.command {
        skye::Commands::Sync {
            source,
            target,
            output,
            append,
            ignore,
            with_update_at,
        } => {
            if output.is_none() {
                *output = Some(target.clone());
            };

            info!("source_path: {:?}", source);
            info!("target_path: {:?}", &target);
            info!("new_target_path: {:?}", output);

            let source_repos = read_repos_from_file(&source)
                .expect(&format!("failed to read from file {:?}", &source));

            let target_repos = read_repos_from_file(&target).unwrap_or_else(|_| {
                info!("not found valid file, create a new file: {:?}", target);
                Vec::new()
            });

            info!("start sync {:?} -> {:?} to {:?}", source, target, output);
            let updated_target = sync_commits(source_repos, target_repos, &args);
            info!("safely write back...");
            match output.clone() {
                None => {
                    eprintln!("not a valid output path: None");
                    exit(1);
                }
                Some(output) => {
                    safe_write_to_file(output, &updated_target, &args)?;
                }
            }
        }
        skye::Commands::Clone {
            source,
            clone_dir,
            shallow,
            ignore,
        } => {
            info!("clone projects");
            let source_repos = read_repos_from_file(&source)
                .expect(&format!("failed to read from file {:?}", &source));

            for repo in source_repos {
                println!("repo: {:?}", repo.repo);
            }
            unimplemented!()
        }
    }

    Ok(())
}
