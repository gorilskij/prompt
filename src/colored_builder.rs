use colored::ColoredString;

pub struct ColoredStringBuilder {
    parts: Vec<ColoredString>,
}

impl ColoredStringBuilder {
    pub fn new() -> Self {
        Self { parts: vec![] }
    }

    pub fn push(&mut self, s: ColoredString) -> &mut Self {
        self.parts.push(s);
        self
    }

    pub fn build(&self) -> String {
        self.parts.iter().map(|s| s.to_string()).collect()
    }
}
