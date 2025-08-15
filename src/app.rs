use std::{
    error::Error,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{
    color::Color,
    cursor_color::{get_mouse_position, get_pixel_color_and_tip_position_and_img},
    position::Position,
    tray::init_tray,
    utils::set_dpi_awareness,
};
use egui::{
    Color32, Context, CornerRadius, Frame, Margin, Mesh, Pos2, Rect, RichText, Stroke,
    TextureHandle, TextureOptions, Ui, Vec2, ViewportBuilder, epaint::Hsva,
};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct AppState {
    pub visible: bool,
    pub position: Position,
    pub color: Color,
    pub current_tip_position: Position,
    pub screen_colors: Vec<Color32>,
    pub screen_tex_size: usize,
}

impl AppState {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(AppState {
            visible: true,
            screen_tex_size: 21,
            ..Default::default()
        }))
    }
}

pub struct App {
    pub state: Arc<Mutex<AppState>>,
    pub wheel_texture: Option<TextureHandle>,
    pub screen_texture: Option<TextureHandle>,
}

// init
impl App {
    pub fn new(state: Arc<Mutex<AppState>>, context: &Context) -> Self {
        context.set_visuals(egui::Visuals {
            panel_fill: Color32::from_rgba_premultiplied(0, 0, 0, 0), // RGB + Alpha
            ..Default::default()
        });
        context.style_mut(|style| {
            style.text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::new(12.0, egui::FontFamily::Monospace),
            );
        });

        let state_clone = state.clone();
        let context_clone = context.clone();
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(16));

                let position = match get_mouse_position() {
                    Ok(p) => p,
                    Err(_) => {
                        continue;
                    }
                };

                let old_tip_position = state_clone.lock().unwrap().current_tip_position;
                let screen_tex_size = state_clone.lock().unwrap().screen_tex_size;
                let (color, current_tip_position, colors) =
                    match get_pixel_color_and_tip_position_and_img(
                        position,
                        old_tip_position,
                        screen_tex_size as usize,
                    ) {
                        Ok(v) => v,
                        Err(_) => {
                            continue;
                        }
                    };

                let _ = {
                    let mut s = state_clone.lock().unwrap();
                    if position != s.position
                        || color != s.color
                        || current_tip_position != s.current_tip_position
                    {
                        if current_tip_position != s.current_tip_position {
                            context_clone.send_viewport_cmd(egui::ViewportCommand::OuterPosition(
                                egui::pos2(
                                    current_tip_position.x as f32
                                        / context_clone.pixels_per_point(),
                                    current_tip_position.y as f32
                                        / context_clone.pixels_per_point(),
                                ),
                            ));
                        }
                        s.screen_colors = colors;
                        s.position = position;
                        s.color = color;
                        s.current_tip_position = current_tip_position;

                        true
                    } else {
                        false
                    }
                };
            }
        });

        Self {
            state,
            wheel_texture: None,
            screen_texture: None,
        }
    }

    pub fn run() -> Result<(), Box<dyn Error>> {
        set_dpi_awareness()?;
        let state = AppState::new();
        let _tray_item = init_tray();
        eframe::run_native(
            "Colorose",
            eframe::NativeOptions {
                viewport: ViewportBuilder::default()
                    .with_always_on_top()
                    .with_has_shadow(true)
                    .with_decorations(false)
                    // .with_inner_size((400.0, 250.0))
                    .with_transparent(true)
                    .with_taskbar(false),
                vsync: true,
                ..Default::default()
            },
            Box::new(|cc| Ok(Box::new(App::new(state.clone(), &cc.egui_ctx.clone())))),
        )?;
        Ok(())
    }
}

// ui
impl eframe::App for App {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let state = self.state.lock().unwrap();

        let color = Color32::from_rgb(state.color.r, state.color.g, state.color.b);
        let color_revert = Color32::from_rgb(
            state.color.revert().r,
            state.color.revert().g,
            state.color.revert().b,
        );
        let hsl = state.color.to_hsl();

        let fg_color = Color32::from_rgb(219, 214, 201);
        let bg_color = Color32::from_rgb(31, 36, 48);

        if state.visible {
            egui::Window::new("window title")
                .auto_sized()
                .resizable(false)
                .title_bar(false)
                .frame(egui::Frame::NONE.fill(egui::Color32::TRANSPARENT))
                .show(ctx, |ui| {
                    Frame {
                        fill: bg_color,
                        corner_radius: CornerRadius::same(5),
                        inner_margin: Margin::same(5),
                        stroke: Stroke::new(2.0, fg_color),
                        ..Default::default()
                    }
                    .show(ui, |ui| {
                        ui.label(RichText::new(state.position).color(fg_color).strong());
                        ui.label(RichText::new(state.color).color(fg_color).strong());
                        ui.label(RichText::new(hsl).color(fg_color).strong());
                        ui.horizontal(|ui| {
                            Frame {
                                inner_margin: Margin::same(5),
                                ..Default::default()
                            }
                            .show(ui, |ui| {
                                show_wheel(
                                    ui,
                                    &mut self.wheel_texture,
                                    color,
                                    color_revert,
                                    75.0,
                                    10.0,
                                );
                            });

                            show_screen_img(
                                ui,
                                &mut self.screen_texture,
                                150.0,
                                state.screen_tex_size,
                                state.screen_colors.clone(),
                                color_revert,
                            );
                        });
                    });
                });
        }
    }
}

pub fn show_screen_img(
    ui: &mut Ui,
    texture: &mut Option<TextureHandle>,
    size: f32,
    tex_size: usize,
    pixels: Vec<Color32>,
    color_revert: Color32,
) {
    if tex_size * tex_size == pixels.len() {
        *texture = Some(ui.ctx().load_texture(
            "screen_image",
            egui::epaint::ColorImage {
                size: [tex_size, tex_size],
                source_size: Vec2::new(tex_size as f32, tex_size as f32),
                pixels,
            },
            TextureOptions::NEAREST,
        ));
    }

    if let Some(tex) = texture {
        Frame {
            inner_margin: Margin::same(5),
            ..Default::default()
        }
        .show(ui, |ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());
            let painter = ui.painter();
            painter.image(
                tex.id(),
                rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );

            let min_factor = (tex_size as f32 / 2.0).floor() / (tex_size as f32);
            let max_factor = ((tex_size as f32 / 2.0).floor() + 1.0) / (tex_size as f32);
            let min_x = (rect.max.x - rect.min.x) * min_factor + rect.min.x;
            let max_x = (rect.max.x - rect.min.x) * max_factor + rect.min.x;
            let min_y = (rect.max.y - rect.min.y) * min_factor + rect.min.y;
            let max_y = (rect.max.y - rect.min.y) * max_factor + rect.min.y;
            let square_rect = Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y));
            painter.rect_stroke(
                square_rect,
                0.0,
                Stroke::new(2.0, color_revert),
                egui::StrokeKind::Outside,
            );
        });
    }
}

pub fn generate_wheel_texture(
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
) {
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
        // ui.image((tex.id(), egui::vec2(size, size)));

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

        let hsva: Hsva = color.into();

        // ===== 绘制 SV 方块 =====
        let square_size = (outer_radius - ring_thickness) * 2.0 / 1.414 - 3.0;
        let square_rect = Rect::from_center_size(center, Vec2::splat(square_size));
        // 底层水平渐变
        {
            let mut mesh = Mesh::default();
            let top_left = square_rect.left_top();
            let top_right = square_rect.right_top();
            let bottom_left = square_rect.left_bottom();
            let bottom_right = square_rect.right_bottom();

            let col_left = Color32::from(Hsva::new(hsva.h, 0.0, 1.0, 1.0));
            let col_right = Color32::from(Hsva::new(hsva.h, 1.0, 1.0, 1.0));

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

        // ===== 绘制标记点 =====
        // 外环 hue 点
        let hue_angle = hsva.h * std::f32::consts::TAU;
        let hue_pos = center
            + Vec2::angled(hue_angle) * ((outer_radius + outer_radius - ring_thickness) / 2.0);
        painter.circle_stroke(hue_pos, 4.0, Stroke::new(2.0, Color32::WHITE));

        // SV 方块点
        let square_size = (outer_radius - ring_thickness) * 2.0 / 1.414 - 3.0;
        let square_rect = Rect::from_center_size(center, Vec2::splat(square_size));

        let sv_x = square_rect.left() + hsva.s * square_size;
        let sv_y = square_rect.top() + (1.0 - hsva.v) * square_size;
        painter.circle_stroke(Pos2::new(sv_x, sv_y), 4.0, Stroke::new(2.0, color_revert));
    }
}
