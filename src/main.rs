use clap::Parser;
use colored::*;
use env_logger::{self};
use log::{error, info};
use skye::{CliArgs, read_repos_from_file, safe_write_to_file, sync_commits};
use std::process::exit;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        std::env::set_var("RUST_LOG", "debug");
    }

    env_logger::init();

    let mut args = CliArgs::parse();

    if args.sync == true {
        if let Some(ref target) = args.target {
            if args.output.is_none() {
                args.output = Some(target.clone());
            };

            info!("source_path: {:?}", args.source);
            info!("target_path: {:?}", &target);
            info!("new_target_path: {:?}", args.output);

            let source_repos = read_repos_from_file(&args.source)
                .expect(&format!("failed to read from file {:?}", &args.source));

            let target_repos = read_repos_from_file(&target).unwrap_or_else(|_| {
                info!("not found valid file, create a new file: {:?}", target);
                Vec::new()
            });

            info!(
                "start sync {:?} -> {:?} to {:?}",
                args.source, target, args.output
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
        } else {
            error!("{}", "target not provided!");
            exit(1);
        }
    }

    if args.clone == true {
        info!("clone projects");
        let source_repos = read_repos_from_file(&args.source)
            .expect(&format!("failed to read from file {:?}", &args.source));

        for repo in source_repos {
            println!("repo: {repo:?}");
        }
        unimplemented!()
    }
    Ok(())
}
