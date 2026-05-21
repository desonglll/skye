use clap::Parse;
use skye::{read_repos_from_file, safe_write_to_file, sync_commits};
use std::path::PathBuf;

#[derive(Parse, Debug)]
#[command(
    version = "0.0.1",
    author = "desonglll",
    about = "A cli for sync setup.json of bizyair cce dockerfile."
)]
struct CliArgs {
    #[arg(short, long)]
    pub source: PathBuf,
    #[arg(short, long)]
    pub target: PathBuf,
    #[arg(short, long)]
    pub new_target: Option<PathBuf>,
    #[arg(short, long)]
    pub append: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();

    if args.new_target.is_none() {
        args.new_target = args.target.clone();
    };

    println!("source_path: {source_path:?}");
    println!("target_path: {target_path:?}");
    println!("new_target_path: {new_target_path:?}");

    let source_repos = read_repos_from_file(&source_path)
        .expect(&format!("failed to read from file {:?}", source_path));

    let target_repos = read_repos_from_file(&target_path).unwrap_or_else(|_| {
        println!("not found valid file, create a new file: {target_path:?}");
        Vec::new()
    });

    println!(
        "start sync {:?} -> {:?} to {:?}",
        source_path, target_path, new_target_path
    );
    let updated_target = sync_commits(source_repos, target_repos, false);
    println!("safely write back...");
    safe_write_to_file(new_target_path, &updated_target)?;
    Ok(())
}
