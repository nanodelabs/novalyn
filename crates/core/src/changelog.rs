use ecow::EcoString;
use std::path::Path;
use tokio::fs;

/// Write or prepend a new release block to CHANGELOG.md asynchronously.
///
/// This function handles idempotent updates - if the exact same release block
/// already exists at the top of the changelog, no write occurs.
///
/// # Arguments
/// * `path` - Directory containing CHANGELOG.md
/// * `new_block` - New release block to prepend
///
/// # Returns
/// * `Ok(true)` - File was modified with new content
/// * `Ok(false)` - File unchanged (idempotent operation)
/// * `Err` - I/O error occurred
/// Internal helper to determine if changelog update is needed and prepare new content.
/// Returns None if no update is needed, Some(new_content) if update should occur.
fn prepare_changelog_update(existing: &str, new_block: &EcoString) -> Option<String> {
    let mut normalized_new = new_block.trim_end().to_string();
    normalized_new.push('\n');

    // Extract current first block (skip optional title line beginning with '# ' but not '## ')
    let top_block = extract_top_block(existing);
    if let Some(tb) = top_block {
        if tb.trim_end() == normalized_new.trim_end() {
            return None;
        }
    }

    // Direct quick check: if existing (after possible title) already begins with normalized_new
    let existing_after_title = existing.strip_prefix("# Changelog\n").unwrap_or(existing);
    if existing_after_title.starts_with(&normalized_new) {
        return None;
    }

    // Prepend new block before existing content (keeping single newline separation)
    let mut out = String::new();
    out.push_str(&normalized_new);
    if !existing.starts_with('#') {
        // unlikely
        out.push('\n');
    }
    out.push_str(existing);
    Some(out)
}

pub async fn write_or_update_changelog_async(
    path: &Path,
    new_block: &EcoString,
) -> std::io::Result<bool> {
    let file_path = path.join("CHANGELOG.md");
    let existing = fs::read_to_string(&file_path)
        .await
        .unwrap_or_else(|_| "# Changelog\n".into());
    if let Some(new_content) = prepare_changelog_update(&existing, new_block) {
        fs::write(&file_path, new_content).await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Synchronous version for backward compatibility.
///
/// Consider using `write_or_update_changelog_async` for better performance
/// when in an async context.
pub fn write_or_update_changelog(path: &Path, new_block: &EcoString) -> std::io::Result<bool> {
    let file_path = path.join("CHANGELOG.md");
    let existing = std::fs::read_to_string(&file_path).unwrap_or_else(|_| "# Changelog\n".into());
    if let Some(new_content) = prepare_changelog_update(&existing, new_block) {
        std::fs::write(&file_path, new_content)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Extract the top release block from a changelog file.
///
/// Parses the changelog to find the first `## ` header and all content
/// until the next `## ` header.
///
/// # Arguments
/// * `existing` - Changelog file content
///
/// # Returns
/// The top release block if found, None if no release blocks exist
fn extract_top_block(existing: &str) -> Option<EcoString> {
    let mut lines = existing.lines().peekable();
    // Skip single title line if present
    if let Some(first) = lines.peek() {
        if first.starts_with("# ") && !first.starts_with("## ") {
            lines.next();
        }
    }
    let mut collected: Vec<EcoString> = Vec::new();
    let mut in_block = false;
    for line in lines {
        if line.starts_with("## ") {
            if in_block {
                break;
            }
            in_block = true;
            collected.push(line.into());
        } else if in_block {
            if line.starts_with("## ") {
                break;
            }
            collected.push(line.into());
        }
    }
    if collected.is_empty() {
        None
    } else {
        Some((collected.join("\n") + "\n").into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn prepends_when_missing() {
        let dir = tempdir().unwrap();
        let changed =
            write_or_update_changelog(dir.path(), &EcoString::from("## v1.0.0\nNotes\n")).unwrap();
        assert!(changed);
        let txt = std::fs::read_to_string(dir.path().join("CHANGELOG.md")).unwrap();
        assert!(txt.starts_with("## v1.0.0"));
    }

    #[test]
    fn second_release_inserts_above() {
        let dir = tempdir().unwrap();
        write_or_update_changelog(dir.path(), &EcoString::from("## v1.0.0\nOld\n")).unwrap();
        let changed =
            write_or_update_changelog(dir.path(), &EcoString::from("## v1.1.0\nNew stuff\n"))
                .unwrap();
        assert!(changed);
        let txt = std::fs::read_to_string(dir.path().join("CHANGELOG.md")).unwrap();
        assert!(txt.starts_with("## v1.1.0"));
        assert!(txt.contains("## v1.0.0"));
    }

    #[test]
    fn idempotent_same_top_block() {
        let dir = tempdir().unwrap();
        let block = "## v1.0.0\nBody\n";
        write_or_update_changelog(dir.path(), &EcoString::from(block)).unwrap();
        let changed = write_or_update_changelog(dir.path(), &EcoString::from(block)).unwrap();
        assert!(!changed);
    }
}
