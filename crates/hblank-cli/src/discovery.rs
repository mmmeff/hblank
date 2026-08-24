use std::path::{Path, PathBuf};

use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use thiserror::Error;
use walkdir::WalkDir;

use crate::Config;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveredFixtureFile {
    pub relative_path: PathBuf,
    pub absolute_path: PathBuf,
    pub module_name: String,
}

/// Finds configured fixture files in deterministic relative-path order.
///
/// # Errors
/// Returns an error for invalid globs, unreadable directory entries, or non-UTF-8 paths.
pub fn discover_fixture_files(
    project_root: &Path,
    config: &Config,
) -> Result<Vec<DiscoveredFixtureFile>, DiscoveryError> {
    let include = build_glob_set(&config.fixtures)?;
    let ignore = build_glob_set(&config.ignore)?;
    let mut fixture_files = Vec::new();

    for entry in WalkDir::new(project_root).follow_links(false) {
        let entry = entry.map_err(DiscoveryError::Walk)?;
        if !entry.file_type().is_file() {
            continue;
        }
        let absolute_path = entry.path().to_path_buf();
        let relative_path = absolute_path
            .strip_prefix(project_root)
            .map_err(|_| DiscoveryError::OutsideProject(absolute_path.clone()))?
            .to_path_buf();
        let relative = portable_path(&relative_path)?;
        if include.is_match(&relative) && !ignore.is_match(&relative) {
            fixture_files.push(DiscoveredFixtureFile {
                module_name: module_name(&relative),
                relative_path,
                absolute_path,
            });
        }
    }

    fixture_files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(fixture_files)
}

fn build_glob_set(patterns: &[String]) -> Result<GlobSet, DiscoveryError> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = GlobBuilder::new(pattern)
            .literal_separator(true)
            .build()
            .map_err(|source| DiscoveryError::Glob {
                pattern: pattern.clone(),
                source,
            })?;
        builder.add(glob);
    }
    builder.build().map_err(DiscoveryError::GlobSet)
}

fn portable_path(path: &Path) -> Result<String, DiscoveryError> {
    let path = path
        .to_str()
        .ok_or_else(|| DiscoveryError::NonUtf8Path(path.to_path_buf()))?;
    Ok(path.replace(std::path::MAIN_SEPARATOR, "/"))
}

fn module_name(relative_path: &str) -> String {
    let mut sanitized = String::with_capacity(relative_path.len());
    for character in relative_path.chars() {
        if character.is_ascii_alphanumeric() {
            sanitized.push(character.to_ascii_lowercase());
        } else if !sanitized.ends_with('_') {
            sanitized.push('_');
        }
    }
    let sanitized = sanitized.trim_matches('_');
    format!(
        "__hblank_{sanitized}_{:016x}",
        stable_hash(relative_path.as_bytes())
    )
}

const fn stable_hash(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    let mut index = 0;
    while index < bytes.len() {
        hash ^= bytes[index] as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        index += 1;
    }
    hash
}

#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("invalid fixture file glob '{pattern}': {source}")]
    Glob {
        pattern: String,
        source: globset::Error,
    },
    #[error("could not build fixture file glob matcher: {0}")]
    GlobSet(globset::Error),
    #[error("could not walk project files: {0}")]
    Walk(walkdir::Error),
    #[error("discovered path is outside the project: {0}")]
    OutsideProject(PathBuf),
    #[error("Hblank requires UTF-8 source paths, received {0}")]
    NonUtf8Path(PathBuf),
}
