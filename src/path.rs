use std::{
    assert_matches::assert_matches,
    path::{Component, Path},
};

use serde::{Deserialize, Deserializer};

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum CWDPathPart {
    Root,
    DoubleRoot,
    Home,
    // a custom starting directory alias such as ~ for $HOME
    CustomAlias(String),
    Ellipsis,
    Normal(String),
}

fn parts_from_path(path: &Path) -> Vec<CWDPathPart> {
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

    parts
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

#[derive(Debug)]
pub struct CWDPath {
    parts: Vec<CWDPathPart>,
}

impl CWDPath {
    pub fn from_str<S: AsRef<str>>(path: S) -> Self {
        let parts = parts_from_str(path.as_ref());

        assert_matches!(
            parts.get(0),
            Some(CWDPathPart::Root | CWDPathPart::DoubleRoot)
        );

        Self { parts }
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

    // apply the `~` alias along with any custom aliases that are passed (sequentially, in order)
    pub fn apply_aliases<'a, I>(&mut self, home: CWDPattern, aliases: I)
    where
        I: IntoIterator<Item = (&'a String, &'a CWDPattern)>,
    {
        if self.strip_prefix(&home) {
            self.parts.insert(0, CWDPathPart::Home);
        }

        for (alias, prefix) in aliases.into_iter() {
            if self.strip_prefix(prefix) {
                self.parts
                    .insert(0, CWDPathPart::CustomAlias(alias.to_string()))
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
        Self::from_parts(parts_from_str(path.as_ref()))
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        Self::from_parts(parts_from_path(path.as_ref()))
    }
}

impl From<CWDPath> for CWDPattern {
    fn from(value: CWDPath) -> Self {
        Self::from_parts(value.parts)
    }
}
