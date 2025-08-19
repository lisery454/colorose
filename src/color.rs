use egui::Color32;
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

impl Into<Color32> for Color {
    fn into(self) -> Color32 {
        Color32::from_rgb(self.r, self.g, self.b)
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

impl HSL {
    pub fn new(h: f32, s: f32, l: f32) -> Self {
        Self { h, s, l }
    }

    pub fn to_rgb(&self) -> Color {
        let h = self.h / 360.0; // 归一化色相到0-1
        let s = self.s;
        let l = self.l;

        // 辅助函数：将0-1范围内的值转换为0-255的u8
        let to_u8 = |x: f32| (x.clamp(0.0, 1.0) * 255.0).round() as u8;

        if s == 0.0 {
            // 无饱和度，直接返回灰度
            let value = to_u8(l);
            return Color {
                r: value,
                g: value,
                b: value,
            };
        }

        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;

        let hue_to_rgb = |t: f32| {
            let t = if t < 0.0 {
                t + 1.0
            } else if t > 1.0 {
                t - 1.0
            } else {
                t
            };
            if t < 1.0 / 6.0 {
                p + (q - p) * 6.0 * t
            } else if t < 1.0 / 2.0 {
                q
            } else if t < 2.0 / 3.0 {
                p + (q - p) * (2.0 / 3.0 - t) * 6.0
            } else {
                p
            }
        };

        Color {
            r: to_u8(hue_to_rgb(h + 1.0 / 3.0)),
            g: to_u8(hue_to_rgb(h)),
            b: to_u8(hue_to_rgb(h - 1.0 / 3.0)),
        }
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
