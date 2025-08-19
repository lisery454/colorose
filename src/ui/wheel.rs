use egui::{
    Color32, Frame, Margin, Mesh, Pos2, Rect, Stroke, TextureHandle, TextureOptions, Ui, Vec2,
    epaint::Hsva,
};

use crate::model::{
    color::{Color, HSL},
    wheel_mode::WheelMode,
};

fn generate_wheel_texture(
    tex_size: usize,   // 纹理大小（正方形）
    outer_radius: f32, // 外圈半径（像素）
    ring_thickness: f32,
) -> Vec<Color32> {
    // let hsva: Hsva = color.into();
    let mut pixels = vec![Color32::TRANSPARENT; tex_size * tex_size];

    let out_radius_mapped = tex_size as f32 / 2.0;
    let ring_thickness_mapped = out_radius_mapped / outer_radius * ring_thickness;
    // let square_size_mapped = (out_radius_mapped - ring_thickness_mapped) * 2.0 / 1.414 - 3.0;

    let center = (tex_size as f32) / 2.0;
    let ring_inner = out_radius_mapped - ring_thickness_mapped;
    // let square_half = square_size_mapped / 2.0;

    let samples = 4; // 每个像素 4×4 采样

    for y in 0..tex_size {
        for x in 0..tex_size {
            let mut r = 0.0;
            let mut g = 0.0;
            let mut b = 0.0;
            let mut a = 0.0;

            for sy in 0..samples {
                for sx in 0..samples {
                    // 子像素坐标（0.5 偏移取中心）
                    let fx = x as f32 + (sx as f32 + 0.5) / samples as f32;
                    let fy = y as f32 + (sy as f32 + 0.5) / samples as f32;

                    let dx = fx - center;
                    let dy = fy - center;
                    let dist = (dx * dx + dy * dy).sqrt();

                    let col = if dist >= ring_inner && dist <= out_radius_mapped {
                        // 色相环
                        let angle = dy.atan2(dx);
                        let hue = (angle / std::f32::consts::TAU).rem_euclid(1.0);
                        let hue = (hue + 5.0 / 12.0).rem_euclid(1.0);
                        Hsva::new(hue, 1.0, 1.0, 1.0)
                    // } else if dx.abs() <= square_half && dy.abs() <= square_half {
                    //     // SV 方块
                    //     let s = (dx + square_half) / square_size_mapped; // 0..1
                    //     let v = 1.0 - (dy + square_half) / square_size_mapped; // 0..1
                    //     Hsva::new(hsva.h, s, v, 1.0)
                    } else {
                        Hsva::new(0.0, 0.0, 0.0, 0.0) // 透明
                    };

                    let rgba = egui::Rgba::from(col);
                    r += rgba.r();
                    g += rgba.g();
                    b += rgba.b();
                    a += rgba.a();
                }
            }

            let scale = (samples * samples) as f32;
            pixels[y * tex_size + x] = Color32::from_rgba_unmultiplied(
                (r / scale * 255.0) as u8,
                (g / scale * 255.0) as u8,
                (b / scale * 255.0) as u8,
                (a / scale * 255.0) as u8,
            );
        }
    }

    pixels
}

pub fn show_wheel(
    ui: &mut Ui,
    texture: &mut Option<TextureHandle>,
    color: Color32,
    color_revert: Color32,
    outer_radius: f32,
    ring_thickness: f32,
    wheel_mode: &WheelMode,
) {
    Frame {
        inner_margin: Margin::same(5),
        ..Default::default()
    }
    .show(ui, |ui| {
        let tex_size = (outer_radius * 2.0 * 2.0) as usize;
        if texture.is_none() {
            let pixels = generate_wheel_texture(tex_size, outer_radius, ring_thickness);
            *texture = Some(ui.ctx().load_texture(
                "color_wheel_texture",
                egui::epaint::ColorImage {
                    size: [tex_size, tex_size],
                    source_size: Vec2::new(tex_size as f32, tex_size as f32),
                    pixels,
                },
                TextureOptions::LINEAR, // 线性采样 = 抗锯齿
            ));
        }

        if let Some(tex) = texture {
            // 色环
            let total_size = Vec2::splat(outer_radius * 2.0);
            let (rect, _) = ui.allocate_exact_size(total_size, egui::Sense::hover());
            let center = rect.center();

            let painter = ui.painter();

            painter.image(
                tex.id(),
                rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );

            let hsv = Color::from(color).to_hsv();
            let hsl = Color::from(color).to_hsl();

            // 中间的图
            match wheel_mode {
                WheelMode::HSL => {
                    let sqrt_3 = 3.0_f32.sqrt();
                    let triangle_size = (outer_radius - ring_thickness) * sqrt_3 - 3.0; // 调整大小

                    // 三角形顶点坐标
                    let top_left = Pos2::new(
                        center.x - triangle_size / sqrt_3 / 2.0,
                        center.y - triangle_size / 2.0,
                    );
                    let middle_left = Pos2::new(center.x - triangle_size / sqrt_3 / 2.0, center.y);
                    let bottom_left = Pos2::new(
                        center.x - triangle_size / sqrt_3 / 2.0,
                        center.y + triangle_size / 2.0,
                    );
                    let middle_right = Pos2::new(center.x + triangle_size / sqrt_3, center.y);

                    {
                        let mut mesh = Mesh::default();

                        let color_top_left = HSL::new(hsl.h, 1.0, 1.0).to_rgb().to_color32();
                        let color_middle_left = HSL::new(hsl.h, 0.0, 0.5).to_rgb().to_color32();
                        let color_bottom_left = HSL::new(hsl.h, 1.0, 0.0).to_rgb().to_color32();
                        let color_middle_right = HSL::new(hsl.h, 1.0, 0.5).to_rgb().to_color32();

                        let idx = mesh.vertices.len() as u32;
                        mesh.colored_vertex(top_left, color_top_left);
                        mesh.colored_vertex(middle_left, color_middle_left);
                        mesh.colored_vertex(bottom_left, color_bottom_left);
                        mesh.colored_vertex(middle_right, color_middle_right);

                        mesh.add_triangle(idx, idx + 1, idx + 3);
                        mesh.add_triangle(idx + 1, idx + 2, idx + 3);

                        painter.add(egui::Shape::mesh(mesh));
                    }
                }
                WheelMode::HSV => {
                    let square_size = (outer_radius - ring_thickness) * 2.0 / 1.414 - 3.0;
                    let square_rect = Rect::from_center_size(center, Vec2::splat(square_size));
                    // 底层水平渐变
                    {
                        let mut mesh = Mesh::default();
                        let top_left = square_rect.left_top();
                        let top_right = square_rect.right_top();
                        let bottom_left = square_rect.left_bottom();
                        let bottom_right = square_rect.right_bottom();

                        let col_left = Color32::from(Hsva::new(hsv.h / 360.0, 0.0, 1.0, 1.0));
                        let col_right = Color32::from(Hsva::new(hsv.h / 360.0, 1.0, 1.0, 1.0));

                        let idx = mesh.vertices.len() as u32;
                        mesh.colored_vertex(top_left, col_left);
                        mesh.colored_vertex(top_right, col_right);
                        mesh.colored_vertex(bottom_right, col_right);
                        mesh.colored_vertex(bottom_left, col_left);
                        mesh.add_triangle(idx, idx + 1, idx + 2);
                        mesh.add_triangle(idx, idx + 2, idx + 3);

                        painter.add(egui::Shape::mesh(mesh));
                    }
                    // 顶层垂直黑色渐变
                    {
                        let mut mesh = Mesh::default();
                        let top_left = square_rect.left_top();
                        let top_right = square_rect.right_top();
                        let bottom_left = square_rect.left_bottom();
                        let bottom_right = square_rect.right_bottom();

                        let col_top = Color32::from_rgba_premultiplied(0, 0, 0, 0); // 透明
                        let col_bottom = Color32::from_rgba_premultiplied(0, 0, 0, 255); // 黑色不透明

                        let idx = mesh.vertices.len() as u32;
                        mesh.colored_vertex(top_left, col_top);
                        mesh.colored_vertex(top_right, col_top);
                        mesh.colored_vertex(bottom_right, col_bottom);
                        mesh.colored_vertex(bottom_left, col_bottom);
                        mesh.add_triangle(idx, idx + 1, idx + 2);
                        mesh.add_triangle(idx, idx + 2, idx + 3);

                        painter.add(egui::Shape::mesh(mesh));
                    }
                }
            }

            // 外环 hue 点
            let hue_angle = (hsv.h - 150.0) / 360.0 * std::f32::consts::TAU;
            let hue_pos = center
                + Vec2::angled(hue_angle) * ((outer_radius + outer_radius - ring_thickness) / 2.0);
            painter.circle_stroke(hue_pos, 4.0, Stroke::new(2.0, Color32::WHITE));

            // 中间的标记点
            match wheel_mode {
                WheelMode::HSL => {
                    let sqrt_3 = 3.0_f32.sqrt();
                    let triangle_size = (outer_radius - ring_thickness) * sqrt_3 - 3.0;
                    let triangle_rect = Rect::from_min_max(
                        Pos2 {
                            x: center.x - triangle_size / sqrt_3 / 2.0,
                            y: center.y - triangle_size / 2.0,
                        },
                        Pos2 {
                            x: center.x + triangle_size / sqrt_3,
                            y: center.y + triangle_size / 2.0,
                        },
                    );
                    let sl_x = triangle_rect.left()
                        + triangle_size * hsl.s * (1.0 - 2.0 * (hsl.l - 0.5).abs()) * sqrt_3 / 2.0;
                    let sl_y = triangle_rect.top() + triangle_size * (1.0 - hsl.l);
                    painter.circle_stroke(
                        Pos2::new(sl_x, sl_y),
                        4.0,
                        Stroke::new(2.0, color_revert),
                    );
                }
                WheelMode::HSV => {
                    let square_size = (outer_radius - ring_thickness) * 2.0 / 1.414 - 3.0;
                    let square_rect = Rect::from_center_size(center, Vec2::splat(square_size));
                    let sv_x = square_rect.left() + hsv.s * square_size;
                    let sv_y = square_rect.top() + (1.0 - hsv.v) * square_size;
                    painter.circle_stroke(
                        Pos2::new(sv_x, sv_y),
                        4.0,
                        Stroke::new(2.0, color_revert),
                    );
                }
            }
        }
    });
}
