use std::process::Command;

use colored::{ColoredStringBuilder, Colorize};

pub enum GitBranch {
    Branch(String),
    Detached(String),
}

fn git_command(command: &str, args: &[&str]) -> Option<String> {
    Command::new(command)
        .args(args)
        .output()
        .ok()
        .and_then(|out| match out.status.success() {
            true => std::str::from_utf8(&out.stdout)
                .ok()
                .map(|s| s.trim().to_string()),
            false => None,
        })
}

pub fn current_git_branch() -> Option<GitBranch> {
    // git symbolic-ref --short HEAD
    git_command("git", &["symbolic-ref", "--short", "HEAD"])
        .map(|out| GitBranch::Branch(out.trim().to_string()))
        .or_else(||
            // git show-ref --head -s --abbrev | head -n1
            git_command("git", &["show-ref", "--head", "-s", "--abbrev"])
                .map(|out|
                    GitBranch::Detached(out.lines().next().unwrap().trim().to_string())))
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
