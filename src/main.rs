#![feature(iter_intersperse)]
#![feature(assert_matches)]

mod config;
mod git_branch;
mod path;
mod python_venv;
mod tainted;
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
                PrefixAlias(s) => s.color("#f5c542").bold(),
                Ellipsis => "⋯".color("#444"),
                Normal(s) => s.color("green"),
                Error => "???".color("red"),
            })
            .intersperse("/".normal())
            .for_each(|part| {
                builder.push(part);
            }),
    }
}

fn main() {
    let mut error_occurred = false;

    let config_str = include_str!("../config.toml");

    // TODO: parse config at compile time
    let config = match Config::from_str(config_str) {
        Ok(config) => config,
        err => {
            error_occurred = true;
            #[cfg(debug_assertions)]
            eprintln!("{:?}", err);
            Default::default()
        }
    };

    let path = Command::new("pwd")
        .output()
        .ok()
        .map(|out| out.stdout)
        .and_then(|chars| {
            std::str::from_utf8(&chars)
                .ok()
                .map(|s| CWDPath::from_str(s.trim()))
        });

    #[cfg(debug_assertions)]
    eprintln!("path: {:?}", path);

    match path {
        Some(path) => {
            let mut path = untaint!(path, bool error_occurred);

            if let Some(home_path) = home_path() {
                let home = untaint!(CWDPattern::from_path(home_path), bool error_occurred);
                path.apply_home_alias(home);
            } else {
                error_occurred = true;
            }

            if let Some(aliases) = &config.aliases {
                path.apply_aliases(aliases);
            }

            path.shorten(1);

            let venv = current_python_venv();
            let branch = current_git_branch();

            let builder = &mut ColoredStringBuilder::new();

            if error_occurred {
                builder.push("!".color("red"));
            }

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
