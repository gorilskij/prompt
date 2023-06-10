#![feature(iter_intersperse)]

use std::{env};
use std::path::{Component, Path, PathBuf};
use std::process::Command;

// const SYMBOLS: &str = "⌘ⵞⵘⵙⴲⴵⵥꙮ◬✡⚛☸❀❁ꔮ❃ꕤꖛꖜꗝ";

fn home_path() -> Option<PathBuf> {
    env::var("HOME")
        .ok()
        .map(|s| PathBuf::from(s))
}

enum GitBranch {
    Branch(String),
    Detached(String),
}

fn current_branch() -> Option<GitBranch> {
    // Command::new("git")
    //     .arg("branch")
    //     .arg("--show-current")
    //     .output()
    //     .ok()
    //     .and_then(|out| match out.status.success() {
    //         true => String::from_utf8(out.stdout).ok(),
    //         false => None,
    //     })
    //     .map(|s| s.trim().to_string())

    // git symbolic-ref --short HEAD
    Command::new("git")
        .args(["symbolic-ref", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|out| match out.status.success() {
            true => std::str::from_utf8(&out.stdout)
                .ok()
                .map(|s| GitBranch::Branch(s.trim().to_string())),
            false => None,
        })
        .or_else(|| // git show-ref --head -s --abbrev | head -n1
            Command::new("git")
                .args(["show-ref", "--head", "-s", "--abbrev"])
                .output()
                .ok()
                .and_then(|out| match out.status.success() {
                    true => std::str::from_utf8(&out.stdout)
                        .ok()
                        .map(|s|
                            GitBranch::Detached(s.lines().next().unwrap().trim().to_string())),
                    false => None,
                }))
}

#[derive(Eq, PartialEq, Clone, Debug)]
enum CWDPathPart {
    RootDir,
    HomeDir,
    Ellipsis,
    Normal(String),
}

struct CWDPath {
    parts: Vec<CWDPathPart>,
}

impl<T: AsRef<Path>> From<T> for CWDPath {
    fn from(path: T) -> Self {
        let path = path.as_ref();
        let parts = path.components()
            .map(|comp| match comp {
                Component::RootDir => CWDPathPart::RootDir,
                Component::Normal(s) =>
                    CWDPathPart::Normal(s.to_str().expect("non-utf8 name in path").to_string()),
                other => panic!("unexpected {other:?} in path"),
            })
            .collect();
        Self { parts }
    }
}

impl CWDPath {
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
        let home = Self::from(&home_path().expect("failed to get home path"));
        if self.strip_prefix(&home) {
            self.parts.insert(0, CWDPathPart::HomeDir);
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
            new_parts.extend(self.parts[self.parts.len() - additional..].iter()
                .map(|part| match part {
                    CWDPathPart::Normal(s) => CWDPathPart::Normal(
                        s.chars().next().expect("empty name in path").to_string()),
                    other => other.clone(),
                }));
        }
        new_parts.extend(last);
        self.parts = new_parts;
    }
}

fn fish_print(string: &str, color: &str) {
    print!("set_color \"{}\";printf \"{}\";", color, string);
}

fn fish_print_branch(branch: &GitBranch) {
    const BRANCH_COLOR: &str = "#32a8a8";
    const DETACHED_COLOR: &str = "#bdb12f";
    match branch {
        GitBranch::Branch(s) => fish_print(s, BRANCH_COLOR),
        GitBranch::Detached(s) => fish_print(s, DETACHED_COLOR),
    }
}

fn fish_print_path(path: &CWDPath) {
    if path.parts.len() == 1 && path.parts[0] == CWDPathPart::RootDir {
        print!("set_color \"normal\";printf \"/\";");
    } else {
        path.parts.iter()
            .flat_map(|part| match part {
                CWDPathPart::RootDir => Some("".to_string()),
                CWDPathPart::HomeDir => Some("set_color \"red\";printf \"~\";".to_string()),
                CWDPathPart::Ellipsis => Some("set_color \"#444\"; printf \"⋯\";".to_string()),
                CWDPathPart::Normal(s) => Some(format!("set_color green; printf \"{}\";", s)),
            })
            .intersperse_with(|| "set_color \"normal\";printf \"/\";".to_string())
            .for_each(|s| print!("{}", s));
    }
}

fn fish_done() {
    print!("set_color \"normal\";printf \" \"");
}

fn main() {
    let mut path = CWDPath::from(
        env::current_dir().expect("failed to get current path"));

    path.strip_home();
    path.shorten(1);

    let branch = current_branch();

    // let symbol_idx = 0;
    // let _symbol = SYMBOLS
    //     .chars()
    //     .nth(symbol_idx)
    //     .unwrap()
    //     .to_string();

    // let new_path_str = new_path.as_os_str().to_str().expect("corrupted path");

    if let Some(branch) = branch {
        fish_print("⟨", "blue");
        fish_print_branch(&branch);
        fish_print("|", "blue");
        // fish_print(&new_path_str, "normal");
        fish_print_path(&path);
        fish_print("⟩", "blue");
        fish_done();
    } else {
        fish_print("|", "blue");
        // fish_print(&new_path_str, "normal");
        fish_print_path(&path);
        fish_print("⟩", "blue");
        fish_done();
    }
}
