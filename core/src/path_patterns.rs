//! Gitignore-style patterns shared by file discovery and rule configuration.

use crate::{LintError, Result};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::path::{Component, Path, PathBuf};

/// Ordered gitignore-style patterns matched against a file path.
#[derive(Debug, Clone)]
pub struct PathPatterns(Gitignore);

impl PathPatterns {
    /// Compile patterns with their original ordering preserved.
    pub fn new(patterns: &[String]) -> Result<Self> {
        let mut builder = GitignoreBuilder::new(".");
        for (index, pattern) in patterns.iter().enumerate() {
            // Keep the pattern index to resolve matches against parent directories.
            builder
                .add_line(Some(PathBuf::from(index.to_string())), pattern)
                .map_err(|error| {
                    LintError::ConfigError(format!("Invalid path pattern {pattern:?}: {error}"))
                })?;
        }
        builder
            .build()
            .map(Self)
            .map_err(|error| LintError::ConfigError(format!("Invalid path patterns: {error}")))
    }

    /// Match a file, including patterns that match one of its ancestors.
    pub fn matches(&self, path: &Path) -> bool {
        if self.0.is_empty() {
            return false;
        }
        let candidate: PathBuf = path
            .components()
            .skip_while(|part| {
                matches!(
                    part,
                    Component::Prefix(_) | Component::RootDir | Component::CurDir
                )
            })
            .collect();
        candidate
            .ancestors()
            .enumerate()
            .map(|(depth, ancestor)| self.0.matched(ancestor, depth != 0))
            .filter_map(|matched| {
                let index = matched.inner()?.from()?.to_str()?.parse::<usize>().ok()?;
                Some((index, matched.is_ignore()))
            })
            .max_by_key(|(index, _)| *index)
            .is_some_and(|(_, included)| included)
    }
}

/// Express paths under the current directory relative to it for anchored patterns.
pub fn matching_path(path: &Path) -> PathBuf {
    if path.is_absolute()
        && let Ok(current_dir) = std::env::current_dir()
        && let Ok(relative) = path.strip_prefix(current_dir)
    {
        return relative.to_path_buf();
    }
    path.to_path_buf()
}
