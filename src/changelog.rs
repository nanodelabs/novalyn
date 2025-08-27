use std::path::Path;

/// Write or prepend the new release block to CHANGELOG.md.
/// Returns Ok(true) if file changed, false if idempotent (identical top block already present).
pub fn write_or_update_changelog(path: &Path, new_block: &str) -> std::io::Result<bool> {
    let file_path = path.join("CHANGELOG.md");
    let existing = std::fs::read_to_string(&file_path).unwrap_or_else(|_| "# Changelog\n".into());
    let mut normalized_new = new_block.trim_end().to_string();
    normalized_new.push('\n');
    // Extract current first block (skip optional title line beginning with '# ' but not '## ')
    let top_block = extract_top_block(&existing);
    if let Some(tb) = top_block {
        if tb.trim_end() == normalized_new.trim_end() {
            return Ok(false);
        }
    }
    // Direct quick check: if existing (after possible title) already begins with normalized_new
    let existing_after_title = existing
        .strip_prefix("# Changelog\n")
        .unwrap_or(existing.as_str());
    if existing_after_title.starts_with(&normalized_new) {
        return Ok(false);
    }
    // Prepend new block before existing content (keeping single newline separation)
    let mut out = String::new();
    out.push_str(&normalized_new);
    if !existing.starts_with('#') {
        // unlikely
        out.push('\n');
    }
    out.push_str(&existing);
    std::fs::write(&file_path, out)?;
    Ok(true)
}

fn extract_top_block(existing: &str) -> Option<String> {
    let mut lines = existing.lines().peekable();
    // Skip single title line if present
    if let Some(first) = lines.peek() {
        if first.starts_with("# ") && !first.starts_with("## ") {
            lines.next();
        }
    }
    let mut collected = Vec::new();
    let mut in_block = false;
    for line in lines {
        if line.starts_with("## ") {
            if in_block {
                break;
            }
            in_block = true;
            collected.push(line.to_string());
        } else if in_block {
            if line.starts_with("## ") {
                break;
            }
            collected.push(line.to_string());
        }
    }
    if collected.is_empty() {
        None
    } else {
        Some(collected.join("\n") + "\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn prepends_when_missing() {
        let dir = tempdir().unwrap();
        let changed = write_or_update_changelog(dir.path(), "## v1.0.0\nNotes\n").unwrap();
        assert!(changed);
        let txt = std::fs::read_to_string(dir.path().join("CHANGELOG.md")).unwrap();
        assert!(txt.starts_with("## v1.0.0"));
    }

    #[test]
    fn second_release_inserts_above() {
        let dir = tempdir().unwrap();
        write_or_update_changelog(dir.path(), "## v1.0.0\nOld\n").unwrap();
        let changed = write_or_update_changelog(dir.path(), "## v1.1.0\nNew stuff\n").unwrap();
        assert!(changed);
        let txt = std::fs::read_to_string(dir.path().join("CHANGELOG.md")).unwrap();
        assert!(txt.starts_with("## v1.1.0"));
        assert!(txt.contains("## v1.0.0"));
    }

    #[test]
    fn idempotent_same_top_block() {
        let dir = tempdir().unwrap();
        let block = "## v1.0.0\nBody\n";
        write_or_update_changelog(dir.path(), block).unwrap();
        let changed = write_or_update_changelog(dir.path(), block).unwrap();
        assert!(!changed);
    }
}
