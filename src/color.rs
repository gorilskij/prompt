// subset of supported colors
enum Color {
    Red,
    Green,
    Blue,
    TrueColor { r: u8, g: u8, b: u8 },
}

impl<T: AsRef<str>> From<T> for Color {
    fn from(value: T) -> Self {
        use Color::*;

        match value.as_ref() {
            "red" => Red,
            "green" => Green,
            "blue" => Blue,
        }
    }
}