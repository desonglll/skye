use clap::Parser;
use env_logger::{self};
use log::{error, info};
use skye::{
    Cli, build_ssh_builder, read_repos_from_file, safe_write_to_file, ssh_clone_repository,
    sync_commits,
};
use std::process::exit;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        std::env::set_var("RUST_LOG", "debug");
    }

    env_logger::init();

    let args = Cli::parse();
    match args.command {
        skye::Commands::Sync {
            ref source,
            ref target,
            ref output,
            append: _,
            ignore: _,
            with_update_at: _,
        } => match output {
            Some(output) => {
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

                safe_write_to_file(output, &updated_target, &args)?;
            }
            None => {
                eprintln!("not a valid output path: None");
                exit(1);
            }
        },
        skye::Commands::Clone {
            source,
            clone_dir,
            shallow: _,
            ignore: _,
        } => {
            info!("clone projects");
            let source_repos = read_repos_from_file(&source)
                .expect(&format!("failed to read from file {:?}", &source));

            let ssh_builder = build_ssh_builder();

            match ssh_builder {
                Ok(mut builder) => {
                    for mut repo in source_repos {
                        println!("repo: {:?}", repo.repo);

                        if repo.repo.starts_with("git@github.com") {
                            repo.repo = repo.repo.replace(":", "/");
                            repo.repo = format!("ssh://{}", repo.repo);
                        }

                        match ssh_clone_repository(&mut builder, &repo.repo, clone_dir.as_path()) {
                            Ok(_) => continue,
                            Err(e) => {
                                error!("error to clone {}: {}", repo.repo, e);
                            }
                        }
                    }
                }
                Err(_) => todo!(),
            }

            unimplemented!()
        }
    }

    Ok(())
}
