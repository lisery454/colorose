use egui::{Color32, Frame, Margin, Pos2, Rect, Stroke, TextureHandle, TextureOptions, Ui, Vec2};

pub fn show_screen_img(
    ui: &mut Ui,
    texture: &mut Option<TextureHandle>,
    size: f32,
    tex_size: usize,
    pixels: Vec<Color32>,
    color_revert: Color32,
    screen_sample_size: &mut usize,
    current_tex_size: &mut usize,
) {
    let current_screen_sample_size = *screen_sample_size;

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
        if current_screen_sample_size > tex_size {
            *screen_sample_size = tex_size;
        }
        *current_tex_size = tex_size;
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

            let half_tex_size = (tex_size as f32 / 2.0).floor();
            let half_sample_size = (current_screen_sample_size as f32 / 2.0).floor();
            let half_sample_size = half_sample_size.min(half_tex_size);
            let min_factor = (half_tex_size - half_sample_size) / (tex_size as f32);
            let max_factor = (half_tex_size + half_sample_size + 1.0) / (tex_size as f32);
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
    } else {
        Frame {
            inner_margin: Margin::same(5),
            ..Default::default()
        }
        .show(ui, |ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());
            let painter = ui.painter();
            painter.rect(
                rect,
                0.0,
                Color32::TRANSPARENT,
                Stroke::NONE,
                egui::StrokeKind::Middle,
            );
        });
    }
}
