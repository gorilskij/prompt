use std::{env, ffi::OsString};

use colored::Colorize;

use crate::colored_builder::ColoredStringBuilder;

pub enum PythonVenv {
    Normal(String),
    NotUnicode(OsString),
}

pub fn current_python_venv() -> Option<PythonVenv> {
    match env::var("VIRTUAL_ENV_PROMPT") {
        Ok(s) => Some(PythonVenv::Normal(s)),
        Err(env::VarError::NotUnicode(s)) => Some(PythonVenv::NotUnicode(s)),
        Err(env::VarError::NotPresent) => None,
    }
}

pub fn format_python_venv(venv: &PythonVenv, builder: &mut ColoredStringBuilder) {
    const PYTHON_VENV_COLOR: &str = "#4b8bbe";

    match venv {
        PythonVenv::Normal(s) => builder.push(s.color(PYTHON_VENV_COLOR)),
        // TODO: implement a solution
        PythonVenv::NotUnicode(_s) => builder.push("<non-unicode>".color("red")),
    };
}
