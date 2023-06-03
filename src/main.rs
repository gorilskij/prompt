use std::env;
use std::ffi::OsStr;
use std::path::{Component, PathBuf};
use std::process::Command;

const SYMBOLS: &str = "⌘ⵞⵘⵙⴲⴵⵥꙮ◬✡⚛☸❀❁ꔮ❃ꕤꖛꖜꗝ";

fn home_path() -> Option<PathBuf> {
    env::var("HOME")
        .ok()
        .map(|s| PathBuf::from(s))
}

fn current_branch() -> Option<String> {
    Command::new("git")
        .arg("branch")
        .arg("--show-current")
        .output()
        .ok()
        .and_then(|out| match out.status.success() {
            true => String::from_utf8(out.stdout).ok(),
            false => None,
        })
        .map(|s| s.trim().to_string())
}

fn fish_print(string: &str, color: &str) {
    print!("set_color \"{}\";printf \"{}\";", color, string);
}

fn fish_done() {
    print!("set_color \"normal\";printf \" \"");
}

fn main() {
    let path = env::current_dir().expect("failed to get current path");
    let home = home_path().expect("failed to get home path");

    let (path, path_stripped) = match path.strip_prefix(&home) {
        Ok(stripped) => (PathBuf::from(stripped), true),
        Err(_) => (path, false),
    };

    let parts: Vec<_> = path.components().collect();
    let mut new_path: PathBuf = if parts.len() >= 3 {
        [Component::Normal(OsStr::new("⋯")), *parts.last().unwrap()]
            .into_iter()
            .collect()
    } else {
        parts
            .iter()
            .copied()
            .take(parts.len() - 1)
            .map(|p| match p {
                Component::Normal(s) => Component::Normal(OsStr::new(s
                    .to_str()
                    .expect("non-utf8 name in path")
                    .get(0..=0)
                    .expect("empty name in path"))),
                other => other,
            })
            .chain(parts.last().copied())
            .collect()
    };

    if path_stripped {
        if new_path.as_os_str().len() > 0 {
            new_path = PathBuf::from("~").join(new_path);
        } else {
            new_path = PathBuf::from("~");
        }
    }

    let branch = current_branch();

    let symbol_idx = 0;
    let _symbol = SYMBOLS
        .chars()
        .nth(symbol_idx)
        .unwrap()
        .to_string();

    let new_path_str = new_path.as_os_str().to_str().expect("corrupted path");

    if let Some(branch) = branch {
        fish_print("⟨", "blue");
        fish_print(&branch, "#32a8a8");
        fish_print("|", "blue");
        fish_print(&new_path_str, "normal");
        fish_print("⟩", "blue");
        fish_done();
    } else {
        fish_print("|", "blue");
        fish_print(&new_path_str, "normal");
        fish_print("⟩", "blue");
        fish_done();
    }
}
