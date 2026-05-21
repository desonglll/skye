use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::Path,
};
use tempfile;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepoInfo {
    repo: String,
    commit: String,
    license: Option<String>,
    notes: Option<String>,
    path: String,
    updated_at: Option<DateTime<Utc>>,
}

pub fn sync_commits(source: Vec<RepoInfo>, target: Vec<RepoInfo>, append: bool) -> Vec<RepoInfo> {
    let mut target_map: IndexMap<String, RepoInfo> = target
        .into_iter()
        .map(|item| (item.path.clone(), item))
        .collect();

    let current_time = Utc::now();
    for source_item in source {
        println!("checking {:?}", source_item.path);
        if let Some(target_item) = target_map.get_mut(&source_item.path) {
            // sync src_item -> target_item
            // sync repo
            if target_item.repo != source_item.repo {
                println!(
                    "update repo\t of {}: {} -> {}",
                    target_item.path, target_item.repo, source_item.repo
                );
                target_item.repo = source_item.repo;
                target_item.updated_at = Some(current_time);
            }

            // sync commit
            if target_item.commit != source_item.commit {
                println!(
                    "update commit\t of {}: {} -> {}",
                    target_item.path, target_item.commit, source_item.commit
                );
                target_item.commit = source_item.commit;
                target_item.updated_at = Some(current_time);
            }

            // sync license
            if target_item.license != source_item.license && source_item.license.is_some() {
                if let Some(license) = source_item.license {
                    println!(
                        "update license\t of {}: {} -> {}",
                        target_item.path,
                        target_item.license.clone().unwrap_or(String::from("None")),
                        license.clone()
                    );
                    target_item.license = Some(license);
                }
                target_item.updated_at = Some(current_time);
            }

            // sync notes
            if target_item.notes != source_item.notes && source_item.notes.is_some() {
                if let Some(notes) = source_item.notes {
                    println!(
                        "update note\t of {}: {} -> {}",
                        source_item.path,
                        target_item.notes.clone().unwrap_or(String::from("None")),
                        notes
                    );
                    target_item.notes = Some(notes);
                }
                target_item.updated_at = Some(current_time);
            }

            // check if updated_at exists.
            if target_item.updated_at.is_none() {
                target_item.updated_at = Some(current_time);
            }
        } else {
            if append {
                println!(
                    "found new repo:\t {}, add {}",
                    source_item.path, source_item.repo
                );
                let mut new_item = source_item;
                new_item.updated_at = Some(current_time);
                target_map.insert(new_item.path.clone(), new_item);
            } else {
                println!("skip {}: {}", source_item.path, source_item.repo)
            }
        }
    }
    target_map.into_values().collect()
}

pub fn read_repos_from_file<P: AsRef<Path>>(
    file_path: P,
) -> Result<Vec<RepoInfo>, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let repos: Vec<RepoInfo> = serde_json::from_reader(reader)?;
    Ok(repos)
}

pub fn safe_write_to_file<P: AsRef<Path>>(
    path: P,
    data: &Vec<RepoInfo>,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = path.as_ref();
    let dir = path.parent().unwrap_or(Path::new("."));
    let temp_file = tempfile::NamedTempFile::new_in(dir)?;
    let writer = BufWriter::new(&temp_file);
    serde_json::to_writer_pretty(writer, data)?;
    let mut file = temp_file.persist(path)?;
    file.flush()?;
    Ok(())
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;
    fn create_mock_repo(
        path: &str,
        commit: &str,
        license: Option<&str>,
        notes: Option<&str>,
    ) -> RepoInfo {
        RepoInfo {
            repo: format!("git@github.com:user/{}.git", path),
            commit: commit.to_string(),
            license: license.map(|s| s.to_string()),
            notes: notes.map(|s| s.to_string()),
            path: path.to_string(),
            updated_at: None,
        }
    }

    #[test]
    fn test_sync_new_repo_added() {
        let source = vec![create_mock_repo("repo-a", "commit-1", Some("MIT"), None)];
        let target = vec![];

        let result = sync_commits(source, target, false);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path, "repo-a");

        assert!(result[0].updated_at.is_some());
    }

    #[test]
    fn test_sync_commit_and_repo_update() {
        let source = vec![RepoInfo {
            repo: "git@github.com:user/new-url.git".to_string(),
            commit: "commit-new".to_string(),
            license: None,
            notes: None,
            path: "repo-a".to_string(),
            updated_at: None,
        }];
        let target = vec![create_mock_repo("repo-a", "commit-old", None, None)];

        let result = sync_commits(source, target, false);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].commit, "commit-new");
        assert_eq!(result[0].repo, "git@github.com:user/new-url.git");
        assert!(result[0].updated_at.is_some());
    }

    #[test]
    fn test_sync_option_fields() {
        let source = vec![create_mock_repo(
            "repo-a",
            "same-commit",
            Some("MIT"),
            Some("New Note"),
        )];
        let target = vec![create_mock_repo(
            "repo-a",
            "same-commit",
            None,
            Some("Old Note"),
        )];

        let result = sync_commits(source, target, false);

        assert_eq!(result.len(), 1);

        assert_eq!(result[0].license, Some("MIT".to_string()));

        assert_eq!(result[0].notes, Some("New Note".to_string()));
        assert!(result[0].updated_at.is_some());
    }

    #[test]
    fn test_sync_no_changes() {
        let source = vec![create_mock_repo("repo-a", "commit-1", Some("MIT"), None)];
        let target = vec![create_mock_repo("repo-a", "commit-1", Some("MIT"), None)];

        let result = sync_commits(source, target, false);

        assert_eq!(result.len(), 1);

        assert!(result[0].updated_at.is_none());
    }
    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
