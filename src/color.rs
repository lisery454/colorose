use std::fmt::Display;


#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct HSL {
    pub h: f32, // 0..=360
    pub s: f32, // 0..=1
    pub l: f32, // 0..=1
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "r:{:} g:{:} b:{:}", self.r, self.g, self.b)
    }
}

impl Into<String> for Color {
    fn into(self) -> String {
        format!("r:{:} g:{:} b:{:}", self.r, self.g, self.b)
    }
}

impl Color {
    pub fn revert(&self) -> Self {
        return Color {
            r: 255 - self.r,
            g: 255 - self.g,
            b: 255 - self.b,
        };
    }

    pub fn to_hsl(&self) -> HSL {
        // 转成 0..=1 浮点
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        // 计算亮度
        let l = (max + min) / 2.0;

        // 计算饱和度
        let s = if delta == 0.0 {
            0.0
        } else {
            delta / (1.0 - (2.0 * l - 1.0).abs())
        };

        // 计算色相
        let mut h = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };

        if h < 0.0 {
            h += 360.0;
        }

        HSL { h, s, l }
    }
}

impl Into<String> for HSL {
    fn into(self) -> String {
        format!(
            "h:{:.1} s:{:.1} l:{:.1}",
            self.h,
            self.s * 100.0,
            self.l * 100.0
        )
    }
}
