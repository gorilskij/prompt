use std::{
    assert_matches::assert_matches,
    path::{Component, Path},
};

use serde::{Deserialize, Deserializer};

use crate::tainted::Tainted;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum CWDPathPart {
    Root,
    DoubleRoot,
    Home,
    // a custom starting directory alias such as ~ for $HOME
    PrefixAlias(String),
    Ellipsis,
    Normal(String),
    // an invalid part (such as a non-unicode part)
    Error,
}

#[derive(Debug)]
pub struct PathParsingTaint {
    non_unicode: bool,
    unexpected_parts: Vec<String>,
}

fn parts_from_path(path: &Path) -> Tainted<Vec<CWDPathPart>, PathParsingTaint> {
    let mut taint = PathParsingTaint {
        non_unicode: false,
        unexpected_parts: vec![],
    };

    let parts = path
        .components()
        .map(|comp| match comp {
            Component::RootDir => CWDPathPart::Root,
            Component::Normal(s) => match s.to_str() {
                Some(s) => CWDPathPart::Normal(s.to_string()),
                None => {
                    taint.non_unicode = true;
                    CWDPathPart::Error
                }
            },
            unexpected => {
                taint.unexpected_parts.push(format!("{:?}", unexpected));
                CWDPathPart::Error
            }
        })
        .collect();

    Tainted {
        value: parts,
        taint: (taint.non_unicode || !taint.unexpected_parts.is_empty()).then_some(taint),
    }
}

#[derive(Debug)]
pub struct EmptyPartsTaint;

fn parts_from_str(path: &str) -> Tainted<Vec<CWDPathPart>, EmptyPartsTaint> {
    let mut parts = Vec::new();

    let leading_slashes = path.chars().take(3).take_while(|c| *c == '/').count();
    match leading_slashes {
        0 => {}
        1 => parts.push(CWDPathPart::Root),
        2 => parts.push(CWDPathPart::DoubleRoot),
        _ => todo!("error"),
    }

    let path = path.split_at(leading_slashes).1;

    let mut has_empty_parts = false;
    if !path.is_empty() {
        let mut iter = path.split("/").peekable();

        if iter.peek() == Some(&"~") && leading_slashes == 0 {
            iter.next();
            parts.push(CWDPathPart::Home);
        }

        while let Some(part) = iter.next() {
            if part.is_empty() && iter.peek() != None {
                has_empty_parts = true;
                parts.push(CWDPathPart::Error)
            } else {
                parts.push(CWDPathPart::Normal(part.to_string()))
            }
        }
    }

    Tainted {
        value: parts,
        taint: has_empty_parts.then_some(EmptyPartsTaint),
    }
}

#[derive(Debug)]
pub struct CWDPath {
    parts: Vec<CWDPathPart>,
}

impl CWDPath {
    pub fn from_str<S: AsRef<str>>(path: S) -> Tainted<Self, EmptyPartsTaint> {
        parts_from_str(path.as_ref()).map(|parts| Self { parts })
    }

    pub fn parts(&self) -> &[CWDPathPart] {
        &self.parts
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

    // apply the `~` alias
    pub fn apply_home_alias(&mut self, home: CWDPattern) {
        if self.strip_prefix(&home) {
            self.parts.insert(0, CWDPathPart::Home);
        }
    }

    // apply custom aliases (sequentially, in order)
    pub fn apply_aliases<'a, I>(&mut self, aliases: I)
    where
        I: IntoIterator<Item = (&'a String, &'a CWDPattern)>,
    {
        for (alias, prefix) in aliases.into_iter() {
            if self.strip_prefix(prefix) {
                self.parts
                    .insert(0, CWDPathPart::PrefixAlias(alias.to_string()))
            }
        }
    }

    // always keeps / or ~ at the beginning and the last part of the path
    // plus, `additional`-many single-letter parts
    pub fn shorten(&mut self, mut additional: usize) {
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
pub struct CWDPattern {
    parts: Vec<CWDPathPart>,
}

// used by Config
impl<'de> Deserialize<'de> for CWDPattern {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s: String = Deserialize::deserialize(deserializer)?;
        let tainted = CWDPattern::from_str(s);
        match tainted.taint {
            Some(EmptyPartsTaint) => Err(serde::de::Error::custom("empty parts in pattern")),
            None => Ok(tainted.value),
        }
    }
}

impl CWDPattern {
    fn from_parts(parts: Vec<CWDPathPart>) -> Self {
        use CWDPathPart::*;

        assert!(!parts.is_empty(), "`parts` must not be empty");
        match parts[0] {
            Root | DoubleRoot | Home | PrefixAlias(_) => {}
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

    fn from_str<S: AsRef<str>>(path: S) -> Tainted<Self, EmptyPartsTaint> {
        parts_from_str(path.as_ref()).map(|parts| Self::from_parts(parts))
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Tainted<Self, PathParsingTaint> {
        parts_from_path(path.as_ref()).map(Self::from_parts)
    }
}

impl From<CWDPath> for CWDPattern {
    fn from(value: CWDPath) -> Self {
        Self::from_parts(value.parts)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_shorten() {
        use crate::{CWDPath, CWDPathPart};

        let path = CWDPath::from_str("/tmp");

        assert!(path.taint.is_none());
        let mut path = path.value;

        path.shorten(0);
        assert_eq!(
            path.parts,
            vec![CWDPathPart::Root, CWDPathPart::Normal("tmp".to_string())]
        )
    }
}
