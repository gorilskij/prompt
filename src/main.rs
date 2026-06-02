#![feature(iter_intersperse)]
#![feature(try_blocks)]

mod config;
mod git_branch;
mod path;
mod python_venv;
mod tainted;

use std::{error::Error, fs};

use colored::*;

use config::*;
use dirs::home_dir;
use git_branch::*;
use path::*;
use python_venv::*;

// const SYMBOLS: &str = "⌘ⵞⵘⵙⴲⴵⵥꙮ◬✡⚛☸❀❁ꔮ❃ꕤꖛꖜꗝ";

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

    let home = home_dir();
    if home.is_none() {
        #[cfg(debug_assertions)]
        eprintln!("no home directory");
    }

    #[cfg(debug_assertions)]
    println!("home path: {:?}", home);

    let config: Result<Config, Box<dyn Error>> = try {
        if let Some(mut config_path) = home {
            config_path.push(".config/prompt/config.toml");

            #[cfg(debug_assertions)]
            println!("config path: {:?}", config_path);

            if fs::exists(&config_path).map_err(|e| Box::new(e) as Box<dyn Error>)? {
                #[cfg(debug_assertions)]
                println!("config file exists");

                let config_str =
                    fs::read_to_string(config_path).map_err(|e| Box::new(e) as Box<dyn Error>)?;
                Config::from_str(&config_str).map_err(|e| Box::new(e) as Box<dyn Error>)?
            } else {
                #[cfg(debug_assertions)]
                println!("config file does not exist");

                Default::default()
            }
        } else {
            Default::default()
        }
    };
    let config = match config {
        Ok(config) => config,
        Err(err) => {
            error_occurred = true;
            #[cfg(debug_assertions)]
            println!("{:?}", err);
            Default::default()
        }
    };

    let path = std::env::var("PWD")
        .ok()
        .map(|s| CWDPath::from_str(s.trim()));

    #[cfg(debug_assertions)]
    eprintln!("path: {:?}", path);

    let path = path.map(|path| {
        let mut path = untaint!(path, bool error_occurred);

        if let Some(home_path) = home_dir() {
            let home = untaint!(CWDPattern::from_path(home_path), bool error_occurred);
            path.apply_home_alias(home);
        } else {
            error_occurred = true;
        }

        if let Some(aliases) = &config.aliases {
            path.apply_aliases(aliases);
        }

        path.shorten(1);

        path
    });

    let venv = current_python_venv();
    let branch = current_git_branch();

    let builder = &mut ColoredStringBuilder::new();

    if error_occurred {
        builder.push("!".color("red"));
    }

    const LEFT_DELIMITER: &str = "⟨";
    const SEPARATOR: &str = "|";
    const RIGHT_DELIMITER: &str = "⟩ ";

    if venv.is_none() && branch.is_none() {
        builder.push(SEPARATOR.color("blue").bold());
    } else {
        builder.push(LEFT_DELIMITER.color("blue").bold());
    }

    if let Some(venv) = venv {
        format_python_venv(&venv, builder);
        builder.push(SEPARATOR.color("blue").bold());
    }

    if let Some(branch) = branch {
        format_git_branch(&branch, builder);
        builder.push(SEPARATOR.color("blue").bold());
    }

    if let Some(path) = path {
        format_path(&path, builder);
    } else {
        builder.push("???".color("red"));
    }

    builder.push(RIGHT_DELIMITER.color("blue").bold());

    print!("{}", builder.build());
}
