#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WheelMode {
    HSL,
    HSV,
}

impl Default for WheelMode {
    fn default() -> Self {
        WheelMode::HSV
    }
}