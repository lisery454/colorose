use std::fmt::Display;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x:{:} y:{:}", self.x, self.y)
    }
}

impl Into<String> for Position {
    fn into(self) -> String {
        format!("x:{:} y:{:}", self.x, self.y)
    }
}
