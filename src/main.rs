#![feature(iter_intersperse)]
#![feature(assert_matches)]

use std::assert_matches::assert_matches;
use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use colored::*;
use serde::{Deserialize, Deserializer};

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
    // a custom starting directory alias such as ~ for $HOME
    CustomAlias(String),
    Ellipsis,
    Normal(String),
}

#[derive(Debug)]
struct CWDPath {
    parts: Vec<CWDPathPart>,
}

fn parts_from_str(path: &str) -> Vec<CWDPathPart> {
    let mut parts = Vec::new();

    let mut iter = path.split('/').peekable();

    if iter.peek().is_some_and(|part| part.is_empty()) {
        iter.next();
        if iter.peek().is_some_and(|part| part.is_empty()) {
            iter.next();
            parts.push(CWDPathPart::DoubleRoot);
        } else {
            parts.push(CWDPathPart::Root);
        }
    } else if iter.peek() == Some(&"~") {
        iter.next();
        parts.push(CWDPathPart::Home);
    }

    for part in iter {
        // TODO: error handling
        assert!(!part.is_empty());
        parts.push(CWDPathPart::Normal(part.to_string()))
    }

    parts
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
        let parts = parts_from_str(path.as_ref());

        assert_matches!(
            parts.get(0),
            Some(CWDPathPart::Root | CWDPathPart::DoubleRoot)
        );

        Self { parts }
    }

    #[must_use]
    fn strip_prefix(&mut self, prefix: &CWDPattern) -> bool {
        match self.parts.strip_prefix(prefix.parts.as_slice()) {
            None => false,
            Some(rest) => {
                self.parts = rest.to_vec();
                true
            }
        }
    }

    // apply the `~` alias along with any custom aliases that are passed (sequentially, in order)
    fn apply_aliases<'a, I>(&mut self, custom_aliases: I)
    where
        I: IntoIterator<Item = (&'a String, &'a CWDPattern)>,
    {
        let home = Self::from_path(home_path().expect("failed to get home path")).into();
        if self.strip_prefix(&home) {
            self.parts.insert(0, CWDPathPart::Home);
        }

        for (alias, prefix) in custom_aliases.into_iter() {
            if self.strip_prefix(prefix) {
                self.parts
                    .insert(0, CWDPathPart::CustomAlias(alias.to_string()))
            }
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

#[derive(Debug)]
struct CWDPattern {
    parts: Vec<CWDPathPart>,
}

impl<'de> Deserialize<'de> for CWDPattern {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s: String = Deserialize::deserialize(deserializer)?;
        Ok(CWDPattern::from_str(s))
    }
}

impl CWDPattern {
    fn from_parts(parts: Vec<CWDPathPart>) -> Self {
        use CWDPathPart::*;

        assert!(!parts.is_empty(), "`parts` must not be empty");
        match parts[0] {
            Root | Home | CustomAlias(_) => {}
            DoubleRoot => assert_eq!(
                parts.len(),
                1,
                "a pattern starting with `//` cannot contain any more parts"
            ),
            _ => {
                panic!("the first part of a pattern can only be `/`, `//`, `~`, or a custom alias")
            }
        }
        // this also ensures that `...` is not present anywhere in the pattern
        for part in &parts[1..] {
            assert_matches!(
                part,
                Normal(_),
                "all parts of a pattern except the first must be normal"
            );
        }

        Self { parts }
    }

    fn from_str<S: AsRef<str>>(path: S) -> Self {
        let parts = parts_from_str(path.as_ref());
        Self::from_parts(parts)
    }
}

impl From<CWDPath> for CWDPattern {
    fn from(value: CWDPath) -> Self {
        Self::from_parts(value.parts)
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
    use CWDPathPart::*;

    match &*path.parts {
        &[Root] => {
            builder.push("/".normal());
        }
        &[DoubleRoot] => {
            builder.push("//".normal());
        }
        parts => parts
            .iter()
            .map(|part| match part {
                Root => "".normal(),
                DoubleRoot => "//".normal(),
                Home => "~".color("red"),
                CustomAlias(s) => s.color("#f5c542").bold(),
                Ellipsis => "⋯".color("#444"),
                Normal(s) => s.color("green"),
            })
            .intersperse("/".normal())
            .for_each(|part| {
                builder.push(part);
            }),
    }
}

#[derive(Deserialize, Default, Debug)]
struct Config {
    // Aliases are applied sequentially, each alias must rely on
    // the path already having previous aliases applied
    aliases: Option<HashMap<String, CWDPattern>>,
}

fn load_config(path: impl AsRef<Path>) -> Config {
    let Ok(config_str) = fs::read_to_string(path) else {
        return Default::default();
    };
    let Ok(config) = toml::from_str(&config_str) else {
        return Default::default();
    };
    config
}

fn main() {
    let config = load_config("config.toml");

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
            if let Some(aliases) = &config.aliases {
                path.apply_aliases(aliases);
            }
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
