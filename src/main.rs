mod get_cursor_color;
mod utils;

use std::{
    error::Error,
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver},
    },
    thread,
    time::Duration,
};

use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, ViewportBuilder};

use crate::{
    get_cursor_color::{Color, Position, get_mouse_position, get_pixel_color_and_tip_position},
    utils::set_dpi_awareness,
};

#[derive(Debug, Default)]
struct ScreenColorAppState {
    position: Position,
    color: Color,
    current_tip_position: Position,
}

#[derive(Debug)]
struct ScreenColorApp {
    state: Arc<Mutex<ScreenColorAppState>>,
    receiver: Receiver<ScreenColorAppState>,
}

impl ScreenColorApp {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        let state = Arc::new(Mutex::new(ScreenColorAppState::default()));
        let state_clone = state.clone();
        thread::spawn(move || {
            loop {
                let position = get_mouse_position().unwrap();
                let old_tip_position = state_clone.lock().unwrap().current_tip_position;
                let (color, current_tip_position) =
                    get_pixel_color_and_tip_position(position, old_tip_position).unwrap();
                sender
                    .send(ScreenColorAppState {
                        position,
                        color,
                        current_tip_position,
                    })
                    .unwrap();
                thread::sleep(Duration::from_millis(33));
            }
        });

        Self { state, receiver }
    }
}

impl eframe::App for ScreenColorApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(new_state) = self.receiver.try_recv() {
            *(self.state.lock().unwrap()) = new_state;
        }

        let state = self.state.lock().unwrap();

        let curr_tip_x = state.current_tip_position.x as f32;
        let curr_tip_y = state.current_tip_position.y as f32;

        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
            curr_tip_x / ctx.pixels_per_point(),
            curr_tip_y / ctx.pixels_per_point(),
        )));

        let color = Color32::from_rgb(state.color.r, state.color.g, state.color.b);
        let color_revert = Color32::from_rgb(
            state.color.revert().r,
            state.color.revert().g,
            state.color.revert().b,
        );

        egui::CentralPanel::default().show(ctx, |ui| {
            Frame {
                fill: color,
                corner_radius: CornerRadius::same(5),
                inner_margin: Margin::same(5),
                stroke: Stroke::new(2.0, color_revert),
                ..Default::default()
            }
            .show(ui, |ui| {
                ui.label(RichText::new(state.position).color(color_revert).strong());
                ui.label(RichText::new(state.color).color(color_revert).strong());
            });
        });

        // ctx.request_repaint();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    set_dpi_awareness()?;

    let app = ScreenColorApp::new();

    eframe::run_native(
        "get screen color",
        eframe::NativeOptions {
            viewport: ViewportBuilder::default()
                .with_always_on_top()
                .with_has_shadow(true)
                .with_decorations(false)
                .with_inner_size((300.0, 60.0))
                .with_transparent(true)
                .with_taskbar(false),

            ..Default::default()
        },
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals {
                panel_fill: Color32::from_rgba_premultiplied(0, 0, 0, 0), // RGB + Alpha
                ..Default::default()
            });
            cc.egui_ctx.style_mut(|style| {
                style.text_styles.insert(
                    egui::TextStyle::Body,
                    egui::FontId::new(12.0, egui::FontFamily::Monospace),
                );
            });
            Ok(Box::new(app))
        }),
    )?;

    Ok(())
}
