#![feature(iter_intersperse)]

use std::env;
use std::iter::once;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use colored::*;

mod tests;

// const SYMBOLS: &str = "⌘ⵞⵘⵙⴲⴵⵥꙮ◬✡⚛☸❀❁ꔮ❃ꕤꖛꖜꗝ";

fn home_path() -> Option<PathBuf> {
    env::var("HOME").ok().map(PathBuf::from)
}

enum GitBranch {
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

fn current_branch() -> Option<GitBranch> {
    // git symbolic-ref --short HEAD
    git_command("git", &["symbolic-ref", "--short", "HEAD"])
        .map(|out| GitBranch::Branch(out.trim().to_string()))
        .or_else(||
            // git show-ref --head -s --abbrev | head -n1
            git_command("git", &["show-ref", "--head", "-s", "--abbrev"])
                .map(|out|
                    GitBranch::Detached(out.lines().next().unwrap().trim().to_string())))
}

#[derive(Eq, PartialEq, Clone, Debug)]
enum CWDPathPart {
    Root,
    DoubleRoot,
    Home,
    Ellipsis,
    Normal(String),
}

#[derive(Debug)]
struct CWDPath {
    parts: Vec<CWDPathPart>,
}

impl CWDPath {
    fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();

        let parts = path
            .components()
            .map(|comp| match comp {
                Component::RootDir => CWDPathPart::Root,
                Component::Normal(s) => {
                    CWDPathPart::Normal(s.to_str().expect("non-utf8 name in path").to_string())
                }
                other => panic!("unexpected {other:?} in path"),
            })
            .collect();

        Self { parts }
    }

    fn from_str<S: AsRef<str>>(path: S) -> Self {
        let path = path.as_ref();

        let first = if path.starts_with("//") {
            CWDPathPart::DoubleRoot
        } else {
            assert!(path.starts_with('/'));
            CWDPathPart::Root
        };

        let parts = once(first)
            .chain(
                path.split('/')
                    .filter(|p| !p.is_empty())
                    .map(|p| CWDPathPart::Normal(p.to_string())),
            )
            .collect();

        Self { parts }
    }

    #[must_use]
    fn strip_prefix(&mut self, prefix: &Self) -> bool {
        match self.parts.strip_prefix(prefix.parts.as_slice()) {
            None => false,
            Some(rest) => {
                self.parts = rest.to_vec();
                true
            }
        }
    }

    fn strip_home(&mut self) {
        let home = Self::from_path(home_path().expect("failed to get home path"));
        if self.strip_prefix(&home) {
            self.parts.insert(0, CWDPathPart::Home);
        }
    }

    // always keeps / or ~ at the beginning and the last part of the path
    // plus, `additional`-many single-letter parts
    fn shorten(&mut self, mut additional: usize) {
        let mut new_parts = vec![self.parts.remove(0)];
        let last = self.parts.pop();
        if self.parts.len() == 1 {
            additional = 1;
        }
        if self.parts.len() > additional {
            new_parts.push(CWDPathPart::Ellipsis);
        }
        if !self.parts.is_empty() {
            new_parts.extend(
                self.parts[self.parts.len() - additional..]
                    .iter()
                    .map(|part| match part {
                        CWDPathPart::Normal(s) => CWDPathPart::Normal(
                            s.chars().next().expect("empty name in path").to_string(),
                        ),
                        other => other.clone(),
                    }),
            );
        }
        new_parts.extend(last);
        self.parts = new_parts;
    }
}

fn format_branch(branch: &GitBranch, builder: &mut ColoredStringBuilder) {
    // const BRANCH_COLOR: &str = "#32a8a8";
    // const DETACHED_COLOR: &str = "#bdb12f";

    const BRANCH_COLOR: &str = "cyan";
    const DETACHED_COLOR: &str = "yellow";

    let cs = match branch {
        GitBranch::Branch(s) => s.color(BRANCH_COLOR),
        GitBranch::Detached(s) => s.color(DETACHED_COLOR),
    };
    builder.push(cs);
}

fn format_path(path: &CWDPath, builder: &mut ColoredStringBuilder) {
    match &*path.parts {
        &[CWDPathPart::Root] => {
            builder.push("/".normal());
        }
        &[CWDPathPart::DoubleRoot] => {
            builder.push("//".normal());
        }
        parts => parts
            .iter()
            .map(|part| match part {
                CWDPathPart::Root => "".normal(),
                CWDPathPart::DoubleRoot => "//".normal(),
                CWDPathPart::Home => "~".color("red"),
                CWDPathPart::Ellipsis => "⋯".color("#444"),
                CWDPathPart::Normal(s) => s.color("green"),
            })
            .intersperse("/".normal())
            .for_each(|part| {
                builder.push(part);
            }),
    }
}

fn main() {
    let path = Command::new("pwd")
        .output()
        .ok()
        .map(|out| out.stdout)
        .and_then(|chars| {
            std::str::from_utf8(&chars)
                .ok()
                .map(|s| CWDPath::from_str(s.trim()))
        });

    match path {
        Some(mut path) => {
            path.strip_home();
            path.shorten(1);

            let branch = current_branch();

            let builder = &mut ColoredStringBuilder::new();
            if let Some(branch) = branch {
                builder.push("⟨".color("blue").bold());
                format_branch(&branch, builder);
            }
            builder.push("|".color("blue").bold());
            format_path(&path, builder);
            builder.push("⟩ ".color("blue").bold());
            print!("{}", builder.build());
        }
        None => {
            let s = ColoredStringBuilder::new()
                .push("|".color("blue").bold())
                .push("???".color("red"))
                .push("⟩ ".color("blue").bold())
                .build();
            print!("{s}");
        }
    }
}
