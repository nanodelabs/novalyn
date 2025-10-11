use ecow::{EcoString, EcoVec};
use git2::{Oid, Repository, StatusOptions};

#[derive(Debug, Clone)]
pub struct RawCommit {
    pub id: EcoString,
    pub short_id: EcoString,
    pub summary: EcoString,
    pub body: EcoString,
    pub author_name: EcoString,
    pub author_email: EcoString,
    pub timestamp: i64,
}

pub fn detect_repo(path: &std::path::Path) -> Result<Repository, git2::Error> {
    Repository::discover(path)
}

pub fn last_tag(repo: &Repository) -> Result<Option<EcoString>, git2::Error> {
    let tags = repo.tag_names(None)?;
    let mut latest: Option<(EcoString, i64, semver::Version)> = None;
    for name_opt in tags.iter() {
        let name = match name_opt {
            Some(n) => n,
            None => continue,
        };
        let ver_str = name.trim_start_matches('v');
        if let Ok(parsed) = semver::Version::parse(ver_str)
            && let Ok(oid) = repo.refname_to_id(&format!("refs/tags/{}", name))
            && let Ok(object) = repo.find_object(oid, None)
            && let Ok(commit) = object.peel_to_commit()
        {
            let time = commit.time().seconds();
            match &latest {
                None => latest = Some((name.into(), time, parsed)),
                Some((_, lt_time, lt_ver)) => {
                    if time > *lt_time || (time == *lt_time && &parsed > lt_ver) {
                        latest = Some((name.into(), time, parsed));
                    }
                }
            }
        }
    }
    Ok(latest.map(|(n, _, _)| n))
}

pub fn current_ref(repo: &Repository) -> Result<Option<EcoString>, git2::Error> {
    let head = match repo.head() {
        Ok(h) => h,
        Err(e) => {
            return if e.code() == git2::ErrorCode::UnbornBranch {
                Ok(None)
            } else {
                Err(e)
            };
        }
    };
    if head.is_branch() {
        return Ok(head.shorthand().map(|s| s.into()));
    }
    // detached: see if it points at a tag
    let oid = head.target();
    if let Some(oid) = oid {
        let tags = repo.tag_names(None)?;
        for name_opt in tags.iter() {
            if let Some(name) = name_opt
                && let Ok(tag_oid) = repo.refname_to_id(&format!("refs/tags/{}", name))
                && tag_oid == oid
            {
                return Ok(Some(name.into()));
            }
        }
    }
    Ok(Some(
        format!(
            "DETACHED@{}",
            oid.map(|o| o.to_string()).unwrap_or_default()
        )
        .into(),
    ))
}

pub fn commits_between(
    repo: &Repository,
    from: Option<&str>,
    to: &str,
) -> Result<EcoVec<RawCommit>, git2::Error> {
    let to_obj = repo.revparse_single(to)?;
    let to_commit = to_obj.peel_to_commit()?;

    let mut revwalk = repo.revwalk()?;
    revwalk.push(to_commit.id())?;
    if let Some(from_ref) = from
        && let Ok(from_obj) = repo.revparse_single(from_ref)
    {
        revwalk.hide(from_obj.id())?;
    }
    revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)?;

    let mut commits: EcoVec<RawCommit> = EcoVec::new();
    for oid_res in revwalk {
        if let Ok(oid) = oid_res
            && let Ok(commit) = repo.find_commit(oid)
        {
            commits.push(to_raw_commit(&commit));
        }
    }
    commits.make_mut().reverse(); // chronological oldest->newest
    Ok(commits)
}

fn to_raw_commit(commit: &git2::Commit) -> RawCommit {
    let id = commit.id().to_string().into();
    let short_id = commit.id().to_string()[0..7].to_string().into();
    let message = commit.message().unwrap_or("");
    let mut lines = message.lines();
    let summary = lines.next().unwrap_or("").into();
    let body = lines.collect::<Vec<_>>().join("\n").into();
    let sig = commit.author();
    let time = commit.time();
    RawCommit {
        id,
        short_id,
        summary,
        body,
        author_name: sig.name().unwrap_or("").into(),
        author_email: sig.email().unwrap_or("").into(),
        timestamp: time.seconds(),
    }
}

pub fn is_dirty(repo: &Repository, include_untracked: bool) -> Result<bool, git2::Error> {
    let mut opts = StatusOptions::new();
    opts.include_untracked(include_untracked);
    let statuses = repo.statuses(Some(&mut opts))?;
    for entry in statuses.iter() {
        let s = entry.status();
        if s.intersects(
            git2::Status::WT_MODIFIED
                | git2::Status::WT_DELETED
                | git2::Status::WT_RENAMED
                | git2::Status::WT_TYPECHANGE
                | git2::Status::INDEX_NEW
                | git2::Status::INDEX_MODIFIED
                | git2::Status::INDEX_DELETED
                | git2::Status::INDEX_RENAMED
                | git2::Status::INDEX_TYPECHANGE
                | git2::Status::WT_NEW,
        ) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn add_and_commit(repo: &Repository, message: &str) -> Result<Oid, git2::Error> {
    let mut index = repo.index()?;
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    let sig = repo.signature()?;
    let parent_commits: Vec<git2::Commit> = match repo.head() {
        Ok(head) => {
            if let Some(oid) = head.target() {
                vec![repo.find_commit(oid)?]
            } else {
                vec![]
            }
        }
        Err(_) => vec![],
    };
    let parents: Vec<&git2::Commit> = parent_commits.iter().collect();
    let oid = repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)?;
    Ok(oid)
}

pub fn create_tag(
    repo: &Repository,
    name: &str,
    message: &str,
    annotated: bool,
) -> Result<Oid, git2::Error> {
    let head = repo.head()?;
    let target = head.peel(git2::ObjectType::Commit)?;
    let commit = target
        .into_commit()
        .map_err(|_| git2::Error::from_str("HEAD is not a commit"))?;
    let sig = repo.signature()?;
    if annotated {
        repo.tag(name, commit.as_object(), &sig, message, false)
    } else {
        repo.reference(&format!("refs/tags/{}", name), commit.id(), false, message)?;
        Ok(commit.id())
    }
}
