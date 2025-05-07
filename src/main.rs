#![feature(iter_intersperse)]
#![feature(assert_matches)]

mod config;
mod git_branch;
mod path;
mod python_venv;
mod tests;

use std::env;
use std::path::PathBuf;
use std::process::Command;

use colored::*;

use config::*;
use git_branch::*;
use path::*;
use python_venv::*;

// const SYMBOLS: &str = "⌘ⵞⵘⵙⴲⴵⵥꙮ◬✡⚛☸❀❁ꔮ❃ꕤꖛꖜꗝ";

fn home_path() -> Option<PathBuf> {
    env::var("HOME").ok().map(PathBuf::from)
}

fn format_path(path: &CWDPath, builder: &mut ColoredStringBuilder) {
    use CWDPathPart::*;

    match path.parts() {
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
            let home = CWDPattern::from_path(home_path().expect("failed to get home path"));

            if let Some(aliases) = &config.aliases {
                path.apply_aliases(home, aliases);
            } else {
                path.apply_aliases(home, []);
            }

            path.shorten(1);

            let venv = current_python_venv();
            let branch = current_git_branch();

            let builder = &mut ColoredStringBuilder::new();

            const LEFT_SEPARATOR: &str = "|";
            const RIGHT_SPARATOR: &str = "|";

            match (venv, branch) {
                (Some(venv), Some(branch)) => {
                    builder.push("⟨".color("blue").bold());
                    format_python_venv(&venv, builder);
                    builder.push(LEFT_SEPARATOR.color("blue").bold());
                    format_git_branch(&branch, builder);
                    builder.push(RIGHT_SPARATOR.color("blue").bold());
                    format_path(&path, builder);
                    builder.push("⟩ ".color("blue").bold());
                }
                (Some(venv), None) => {
                    builder.push("⟨".color("blue").bold());
                    format_python_venv(&venv, builder);
                    builder.push(RIGHT_SPARATOR.color("blue").bold());
                    format_path(&path, builder);
                    builder.push("⟩ ".color("blue").bold());
                }
                (None, Some(branch)) => {
                    builder.push("⟨".color("blue").bold());
                    format_git_branch(&branch, builder);
                    builder.push(RIGHT_SPARATOR.color("blue").bold());
                    format_path(&path, builder);
                    builder.push("⟩ ".color("blue").bold());
                }
                (None, None) => {
                    builder.push(LEFT_SEPARATOR.color("blue").bold());
                    format_path(&path, builder);
                    builder.push("⟩ ".color("blue").bold());
                }
            }

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
