use std::fmt::Display;
use egui::Color32;

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
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct HSV {
    pub h: f32, // 色相 (0°~360°)
    pub s: f32, // 饱和度 (0.0~1.0)
    pub v: f32, // 明度 (0.0~1.0)
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
        Color {
            r: 255 - self.r,
            g: 255 - self.g,
            b: 255 - self.b,
        }
    }

    pub fn to_hsl(&self) -> HSL {
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;

        let max = r.max(g.max(b));
        let min = r.min(g.min(b));
        let delta = max - min;

        // 计算亮度(Lightness)
        let l = (max + min) / 2.0;

        // 计算色相(Hue)
        let h = if delta == 0.0 {
            0.0 // 无色相（灰度）
        } else if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * ((b - r) / delta + 2.0)
        } else {
            60.0 * ((r - g) / delta + 4.0)
        };

        // 计算饱和度(Saturation)
        let s = if delta == 0.0 {
            0.0
        } else {
            delta / (1.0 - (2.0 * l - 1.0).abs())
        };

        HSL {
            h: if h < 0.0 { h + 360.0 } else { h }, // 确保在0-360 范围内
            s: s.max(0.0).min(1.0),                 // 钳制到0-1
            l: l.max(0.0).min(1.0),                 // 钳制到0-1
        }
    }

    pub fn to_hsv(&self) -> HSV {
        // 将RGB归一化到[0,1]
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;

        let max = r.max(g.max(b));
        let min = r.min(g.min(b));
        let delta = max - min;

        // 计算色相(Hue)
        let h = if delta == 0.0 {
            0.0 // 无色相（灰度）
        } else if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * ((b - r) / delta + 2.0)
        } else {
            60.0 * ((r - g) / delta + 4.0)
        };

        // 计算饱和度(Saturation)
        let s = if max == 0.0 { 0.0 } else { delta / max };

        HSV {
            h: if h < 0.0 { h + 360.0 } else { h }, // 确保色相在0-360°
            s: s.clamp(0.0, 1.0),                   // 钳制饱和度
            v: max.clamp(0.0, 1.0),                 // 明度直接取最大值
        }
    }
}

impl From<Color32> for Color {
    fn from(value: Color32) -> Self {
        Color {
            r: value.r(),
            g: value.g(),
            b: value.b(),
        }
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

impl Into<String> for HSV {
    fn into(self) -> String {
        format!(
            "h:{:.1} s:{:.1} v:{:.1}",
            self.h,
            self.s * 100.0,
            self.v * 100.0
        )
    }
}