use chrono::{DateTime, FixedOffset, Utc};
use colored::*;
use indexmap::IndexMap;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::Path,
};
use tempfile;

/// Structure of the json object.
///
/// ```json
/// {
///     "repo": "git@github.com:siliconflow/BizyDraft.git",
///     "commit": "9d7bcbad2a8b6d17165a1c2ccc27ca53d4136d24",
///     "license": "MIT License",
///     "notes": "BizyDraft 核心引擎与前端",
///     "path": "BizyDraft",
///     "updated_at": "2026-05-21T08:45:00.391906Z"
///   }
/// ```
///
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepoInfo {
    repo: String,
    commit: String,
    license: Option<String>,
    notes: Option<String>,
    path: String,
    updated_at: Option<DateTime<FixedOffset>>,
}

pub fn sync_commits(
    source: Vec<RepoInfo>,
    target: Vec<RepoInfo>,
    append: bool,
    ignore: Option<Vec<String>>,
) -> Vec<RepoInfo> {
    let mut target_map: IndexMap<String, RepoInfo> = target
        .into_iter()
        .map(|item| (item.path.clone(), item))
        .collect();
    let ignore_list = ignore.unwrap_or(vec![]);
    let shanghai_tz = FixedOffset::east_opt(8 * 3600).unwrap();
    let current_time: DateTime<FixedOffset> = Utc::now().with_timezone(&shanghai_tz);
    for source_item in source {
        if ignore_list.contains(&source_item.path) {
            info!(
                "ignore repo:\t {}: {}",
                source_item.path.magenta(),
                source_item.repo
            );
            continue;
        }
        if let Some(target_item) = target_map.get_mut(&source_item.path) {
            info!("checking:\t {}", source_item.path.green());
            let mut is_updated = false;
            // sync src_item -> target_item
            // sync repo
            if target_item.repo != source_item.repo {
                debug!(
                    "updating: \t {} -> {}",
                    target_item.repo.red(),
                    source_item.repo.blue()
                );
                target_item.repo = source_item.repo;
                target_item.updated_at = Some(current_time);
                is_updated = true;
            }

            // sync commit
            if target_item.commit != source_item.commit {
                debug!(
                    "updating: \t {} -> {}",
                    target_item.commit.red(),
                    source_item.commit.blue()
                );
                target_item.commit = source_item.commit;
                target_item.updated_at = Some(current_time);
                is_updated = true;
            }

            // sync license
            if target_item.license != source_item.license && source_item.license.is_some() {
                if let Some(license) = source_item.license {
                    debug!(
                        "updating: \t {} -> {}",
                        target_item
                            .license
                            .as_deref()
                            .filter(|s| !s.is_empty())
                            .unwrap_or("None")
                            .red(),
                        &license.blue()
                    );
                    target_item.license = Some(license);
                    is_updated = true;
                }
                target_item.updated_at = Some(current_time);
            }

            // sync notes
            if target_item.notes != source_item.notes && source_item.notes.is_some() {
                if let Some(notes) = source_item.notes {
                    debug!(
                        "updating: \t {} -> {}",
                        target_item
                            .notes
                            .as_deref()
                            .filter(|s| !s.is_empty())
                            .unwrap_or("None")
                            .red(),
                        notes.blue()
                    );
                    target_item.notes = Some(notes);
                    is_updated = true;
                }
                target_item.updated_at = Some(current_time);
            }

            // check if updated_at exists.
            if target_item.updated_at.is_none() {
                target_item.updated_at = Some(current_time);
                debug!(
                    "updating: \t {} -> {}",
                    "None".red(),
                    current_time.to_string().blue()
                );
                is_updated = true;
            }

            if is_updated {
                info!("updated: \t {}", source_item.path.blue())
            }
        } else {
            if append {
                info!(
                    "new repo:\t {}, add {}",
                    source_item.path.green(),
                    source_item.repo.blue()
                );
                debug!("adding: \t {}", source_item.repo.blue());
                debug!("adding: \t {}", source_item.commit.blue());
                debug!(
                    "adding: \t {}",
                    source_item
                        .license
                        .as_deref()
                        .filter(|s| !s.is_empty())
                        .unwrap_or("None")
                        .blue()
                );
                debug!(
                    "adding: \t {}",
                    source_item
                        .notes
                        .as_deref()
                        .filter(|s| !s.is_empty())
                        .unwrap_or("None")
                        .blue()
                );
                debug!(
                    "adding: \t {}",
                    source_item
                        .updated_at
                        .unwrap_or(current_time)
                        .to_string()
                        .blue()
                );
                let mut new_item = source_item;
                new_item.updated_at = Some(current_time);
                target_map.insert(new_item.path.clone(), new_item);
            } else {
                info!(
                    "skip repo:\t {}: {}",
                    source_item.path.magenta(),
                    source_item.repo
                )
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

        let result = sync_commits(source, target, true, None);

        println!("Result length: {}", result.len());

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

        let result = sync_commits(source, target, false, Some(Vec::new()));

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

        let result = sync_commits(source, target, false, Some(Vec::new()));

        assert_eq!(result.len(), 1);

        assert_eq!(result[0].license, Some("MIT".to_string()));

        assert_eq!(result[0].notes, Some("New Note".to_string()));
        assert!(result[0].updated_at.is_some());
    }

    #[test]
    fn test_sync_no_changes() {
        let shanghai_tz = FixedOffset::east_opt(8 * 3600).unwrap();
        let current_time = Utc::now().with_timezone(&shanghai_tz);
        let source = vec![create_mock_repo("repo-a", "commit-1", Some("MIT"), None)];

        let mut target_item = create_mock_repo("repo-a", "commit-1", Some("MIT"), None);
        target_item.updated_at = Some(current_time);
        let target = vec![target_item];

        let result = sync_commits(source, target, false, Some(Vec::new()));

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].updated_at, Some(current_time));
    }
    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
