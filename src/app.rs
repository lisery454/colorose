use std::{
    error::Error,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{
    color::Color,
    cursor_color::{get_mouse_position, get_pixel_color_and_tip_position},
    position::Position,
    tray::init_tray,
    utils::set_dpi_awareness,
};
use egui::{Color32, Context, CornerRadius, Frame, Margin, RichText, Stroke, ViewportBuilder};

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct AppState {
    pub visible: bool,
    pub position: Position,
    pub color: Color,
    pub current_tip_position: Position,
}

impl AppState {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(AppState {
            visible: true,
            ..Default::default()
        }))
    }
}

pub struct App {
    pub state: Arc<Mutex<AppState>>,
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
                let (color, current_tip_position) =
                    match get_pixel_color_and_tip_position(position, old_tip_position) {
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

        Self { state }
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
                    .with_inner_size((300.0, 70.0))
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

        if state.visible {
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
                    ui.label(RichText::new(hsl).color(color_revert).strong());
                });
            });
        }
    }
}
