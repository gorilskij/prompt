use std::{
    env,
    fs,
    path::PathBuf,
};

use colored::Colorize;

use crate::colored_builder::ColoredStringBuilder;

pub enum GitBranch {
    Branch(String),
    Detached(String),
}

fn find_git_dir() -> Option<PathBuf> {
    let mut dir = env::var("PWD").ok().map(PathBuf::from)?;
    loop {
        let candidate = dir.join(".git");
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

pub fn current_git_branch() -> Option<GitBranch> {
    let git_dir = find_git_dir()?;
    let head = fs::read_to_string(git_dir.join("HEAD")).ok()?;
    let head = head.trim();

    if let Some(branch) = head.strip_prefix("ref: refs/heads/") {
        Some(GitBranch::Branch(branch.to_string()))
    } else {
        // detached HEAD — abbreviate to 7 chars like git does
        let hash = &head[..head.len().min(7)];
        Some(GitBranch::Detached(hash.to_string()))
    }
}

pub fn format_git_branch(branch: &GitBranch, builder: &mut ColoredStringBuilder) {
    // const BRANCH_COLOR: &str = "cyan";
    // const BRANCH_COLOR: &str = "#ac51b8";
    const BRANCH_COLOR: &str = "#9d4ac4";
    // const BRANCH_COLOR: &str = "#da8534";
    const DETACHED_COLOR: &str = "#bdb12f";

    let cs = match branch {
        GitBranch::Branch(s) => s.color(BRANCH_COLOR),
        GitBranch::Detached(s) => s.color(DETACHED_COLOR),
    };
    builder.push(cs);
}
