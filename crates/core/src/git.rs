use ecow::{EcoString, EcoVec};
use gix::Repository;
use gix::date::parse::TimeBuf;

/// A raw commit extracted from the git repository.
///
/// Contains essential commit metadata needed for changelog generation.
/// This is a lightweight representation optimized for performance.
#[derive(Debug, Clone)]
pub struct RawCommit {
    pub id: EcoString,
    pub short_id: EcoString,
    pub summary: EcoString,
    pub body: EcoString,
    pub author_name: EcoString,
    pub author_email: EcoString,
    /// Unix timestamp of commit
    pub timestamp: i64,
}

/// Detect and open a git repository at the given path.
///
/// Searches for a .git directory starting from the given path and
/// walking up the directory tree.
///
/// # Arguments
/// * `path` - Starting directory for repository search
///
/// # Returns
/// * `Ok(Repository)` - Successfully opened repository
/// * `Err` - No repository found or access error
pub fn detect_repo(path: &std::path::Path) -> anyhow::Result<Repository> {
    gix::discover(path).map_err(anyhow::Error::from)
}

/// Find the most recent semantic version tag in the repository.
///
/// Scans all tags matching semantic versioning format (with optional 'v' prefix)
/// and returns the most recent one based on commit timestamp and version comparison.
///
/// # Arguments
/// * `repo` - Git repository to search
///
/// # Returns
/// * `Ok(Some(tag_name))` - Most recent semantic version tag
/// * `Ok(None)` - No semantic version tags found
/// * `Err` - Repository access error
pub fn last_tag(repo: &Repository) -> anyhow::Result<Option<EcoString>> {
    use gix::object::Kind;
    let mut latest: Option<(EcoString, i64, semver::Version)> = None;
    let refs = repo.references().map_err(anyhow::Error::from)?;
    for result in refs.all()? {
        let mut tag_ref = match result {
            Ok(r) => r,
            Err(_) => continue,
        };
        let name_bstr = tag_ref.name().as_bstr();
        if !name_bstr.starts_with(b"refs/tags/") {
            continue;
        }
        let tag_name_bstr = &name_bstr[b"refs/tags/".len()..];
        let tag_name = String::from_utf8_lossy(tag_name_bstr).to_string();
        let ver_str = tag_name.trim_start_matches('v');
        let parsed = match semver::Version::parse(ver_str) {
            Ok(v) => v,
            Err(_) => continue,
        };
        // Peel to commit for annotated tags, or use target for lightweight
        let target_commit_oid = match tag_ref.peel_to_kind(Kind::Commit) {
            Ok(obj) => obj.id,
            Err(_) => continue,
        };
        let commit = match repo.find_commit(target_commit_oid) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let time = commit.time().map(|t| t.seconds).unwrap_or(0);
        match &latest {
            None => latest = Some((tag_name.clone().into(), time, parsed)),
            Some((_, lt_time, lt_ver)) => {
                if time > *lt_time || (time == *lt_time && &parsed > lt_ver) {
                    latest = Some((tag_name.clone().into(), time, parsed));
                }
            }
        }
    }
    Ok(latest.map(|(n, _, _)| n))
}

/// Get the current HEAD reference name.
///
/// Returns the current branch name, tag name, or detached HEAD identifier.
///
/// # Arguments
/// * `repo` - Git repository
///
/// # Returns
/// * `Ok(Some(ref_name))` - Reference name (branch, tag, or detached HEAD)
/// * `Ok(None)` - Unborn HEAD (no commits yet)
/// * `Err` - Repository access error
pub fn current_ref(repo: &Repository) -> anyhow::Result<Option<EcoString>> {
    let head = repo.head().map_err(anyhow::Error::from)?;
    if head.is_unborn() {
        return Ok(None);
    }
    if head.is_detached() {
        if let Some(target_id) = head.id() {
            let refs = repo.references().map_err(anyhow::Error::from)?;
            for result in refs.all()? {
                let tag_ref = match result {
                    Ok(r) => r,
                    Err(_) => continue,
                };
                let name_bstr = tag_ref.name().as_bstr();
                if name_bstr.starts_with(b"refs/tags/") {
                    if let Some(tag_oid) = tag_ref.target().try_id() {
                        if *tag_oid == *target_id {
                            let tag_name_bstr = &name_bstr[b"refs/tags/".len()..];
                            let tag_name = String::from_utf8_lossy(tag_name_bstr).to_string();
                            return Ok(Some(tag_name.into()));
                        }
                    }
                }
            }
            return Ok(Some(format!("DETACHED@{:?}", target_id).into()));
        }
    } else if let Some(name) = head.referent_name() {
        let branch_bstr = name.as_bstr();
        let branch = branch_bstr
            .strip_prefix(b"refs/heads/")
            .unwrap_or(branch_bstr);
        return Ok(Some(String::from_utf8_lossy(branch).to_string().into()));
    }
    Ok(None)
}

/// Collect all commits between two references.
///
/// Performs a git log operation from `from` (exclusive) to `to` (inclusive).
/// If `from` is None, collects all commits up to `to`.
/// Automatically chooses between sequential and parallel processing based on commit count.
///
/// # Arguments
/// * `repo` - Git repository
/// * `from` - Optional starting reference (exclusive)
/// * `to` - Ending reference (inclusive)
///
/// # Returns
/// * `Ok(commits)` - Vector of raw commits in chronological order (oldest first)
/// * `Err` - Git operation error
pub fn commits_between(
    repo: &Repository,
    from: Option<&str>,
    to: &str,
) -> anyhow::Result<EcoVec<RawCommit>> {
    // Sequential processing since gix::Repository is not Sync
    // The actual commit parsing is already quite fast
    let mut commits: EcoVec<RawCommit> = EcoVec::new();
    let to_obj = repo.rev_parse_single(to).map_err(anyhow::Error::from)?;
    let to_id = to_obj.object()?.peel_to_kind(gix::object::Kind::Commit)?.id;
    let mut walk = repo.rev_walk([to_id]);
    if let Some(from_rev) = from {
        let from_obj = repo
            .rev_parse_single(from_rev)
            .map_err(anyhow::Error::from)?;
        let from_id = from_obj
            .object()?
            .peel_to_kind(gix::object::Kind::Commit)?
            .id;
        walk = walk.with_hidden([from_id]);
    }

    for commit_info in walk.all()? {
        let commit_id = commit_info?.id;
        let commit = repo.find_commit(commit_id)?;
        match to_raw_commit(&commit) {
            Ok(raw) => commits.push(raw),
            Err(e) => {
                tracing::warn!("Skipping commit {}: {}", commit.id(), e);
                continue;
            }
        }
    }
    commits.make_mut().reverse();
    Ok(commits)
}

fn to_raw_commit(commit: &gix::Commit) -> anyhow::Result<RawCommit> {
    let id = commit.id().to_string().into();
    let short_id = commit.id().to_string()[0..7].to_string().into();
    let message_bstr = commit
        .message_raw()
        .map_err(|e| anyhow::anyhow!("missing commit message: {}", e))?;
    let message = String::from_utf8_lossy(message_bstr).to_string();
    let mut lines = message.lines();
    let summary = lines.next().unwrap_or("").into();
    let body = lines.collect::<Vec<_>>().join("\n").into();
    let author = commit
        .author()
        .map_err(|e| anyhow::anyhow!("missing author: {}", e))?;
    let author_name = String::from_utf8_lossy(author.name).to_string().into();
    let author_email = String::from_utf8_lossy(author.email).to_string().into();
    let timestamp = commit.time().map(|t| t.seconds).unwrap_or(0);
    Ok(RawCommit {
        id,
        short_id,
        summary,
        body,
        author_name,
        author_email,
        timestamp,
    })
}

/// Check if the working tree has uncommitted changes.
///
/// Examines both staged and unstaged changes in the repository.
///
/// # Arguments
/// * `repo` - Git repository
///
/// # Returns
/// * `Ok(true)` - Working tree has uncommitted changes
/// * `Ok(false)` - Working tree is clean
/// * `Err` - Git operation error
pub fn is_dirty(repo: &Repository) -> anyhow::Result<bool> {
    // Use gix's status functionality to check for changes
    let status_platform = repo.status(gix::progress::Discard)?;

    // Get status iterator and check for any worktree changes
    let status_iter = status_platform.into_iter(None)?;

    // Check for any IndexWorktree status entries (uncommitted changes)
    for status_item in status_iter {
        let item = status_item?;
        // Only IndexWorktree items indicate uncommitted changes (dirty state)
        if matches!(item, gix::status::Item::IndexWorktree(_)) {
            return Ok(true);
        }
        // TreeIndex items are changes between HEAD and index (already staged)
        // We don't consider those as "dirty" - only unstaged changes matter
    }
    Ok(false)
}

pub fn add_and_commit(repo: &mut Repository, message: &str) -> anyhow::Result<gix::ObjectId> {
    // Get the working directory
    let workdir = repo
        .workdir()
        .ok_or_else(|| anyhow::anyhow!("No working directory"))?;

    // Start with empty tree or current HEAD tree
    let base_tree_id = if let Ok(head) = repo.head() {
        if let Some(head_id) = head.id() {
            // Get the tree from HEAD commit
            let head_commit = repo.find_object(head_id)?.peel_to_commit()?;
            head_commit.tree_id()?.detach()
        } else {
            repo.empty_tree().id
        }
    } else {
        repo.empty_tree().id
    };

    // Create tree editor
    let mut tree_editor = repo.edit_tree(base_tree_id)?;

    // Get the status to find files to add
    let status_platform = repo.status(gix::progress::Discard)?;
    let status_iter = status_platform.into_iter(None)?;

    // Process status items to find files to add
    for status_item in status_iter {
        let item = status_item?;

        // Only process IndexWorktree items (files that need to be staged)
        if let gix::status::Item::IndexWorktree(worktree_item) = item {
            let path = worktree_item.rela_path();
            let full_path = workdir.join(std::path::Path::new(std::str::from_utf8(path)?));

            if full_path.is_file() {
                // Read file and create blob
                let content = std::fs::read(&full_path)?;
                let blob_id = repo.write_blob(&content)?;

                // Add to tree
                tree_editor.upsert(path, gix::object::tree::EntryKind::Blob, blob_id)?;
            }
        }
        // Skip TreeIndex items - those are already staged
    }

    // Write the tree
    let tree_id = tree_editor.write()?.detach();

    // Create commit signature
    let sig_ref = repo.committer_or_set_generic_fallback()?;
    let sig = sig_ref.to_owned()?;
    let mut time_buf = gix::date::parse::TimeBuf::default();
    let sig_ref_borrowed = sig.to_ref(&mut time_buf);

    // Get parent commits
    let parents: Vec<gix::ObjectId> = if let Ok(head) = repo.head() {
        if let Some(head_id) = head.id() {
            vec![head_id.into()]
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    // Create the commit
    let commit_id = repo.commit_as(
        sig_ref_borrowed,
        sig_ref_borrowed,
        "HEAD",
        message,
        tree_id,
        parents,
    )?;

    // Update the index to match the committed tree so the worktree appears clean
    // Create a new index from the committed tree and write it to the index file
    let mut new_index = repo.index_from_tree(&tree_id)?;
    new_index.write(gix::index::write::Options::default())?;

    Ok(commit_id.detach())
}

pub fn create_tag(
    repo: &mut Repository,
    name: &str,
    message: &str,
    annotated: bool,
) -> anyhow::Result<gix::ObjectId> {
    // Extract head commit id and signature before mutable borrow
    // Get head commit id without holding a reference to head_commit
    let head_commit_id = repo.head_id().map_err(anyhow::Error::from)?.detach();
    if annotated {
        let sig_ref = repo
            .committer_or_set_generic_fallback()
            .map_err(anyhow::Error::from)?;
        let sig = sig_ref.to_owned().map_err(anyhow::Error::from)?;

        let mut time_buf = TimeBuf::default();
        let sig_ref_borrowed = sig.to_ref(&mut time_buf);

        let tag_ref = repo
            .tag(
                name,
                head_commit_id,
                gix::object::Kind::Commit,
                Some(sig_ref_borrowed),
                message,
                gix::refs::transaction::PreviousValue::MustNotExist,
            )
            .map_err(anyhow::Error::from)?;
        Ok(tag_ref.target().id().to_owned())
    } else {
        let tag_ref = repo
            .tag_reference(
                name,
                head_commit_id,
                gix::refs::transaction::PreviousValue::MustNotExist,
            )
            .map_err(anyhow::Error::from)?;
        Ok(tag_ref.target().id().to_owned())
    }
}

pub fn init_repo(path: &std::path::Path) -> anyhow::Result<Repository> {
    // Initialize repository at path
    let mut repo = gix::init(path)?;

    // Set user configuration
    let mut config = repo.config_snapshot_mut();
    config.set_raw_value(&gix::config::tree::User::NAME, "Tester")?;
    config.set_raw_value(&gix::config::tree::User::EMAIL, "tester@example.com")?;
    config.commit()?;

    Ok(repo)
}
