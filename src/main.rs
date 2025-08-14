#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod get_cursor_color;
mod utils;

use std::{
    error::Error,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use egui::{Color32, Context, CornerRadius, Frame, Margin, RichText, Stroke, ViewportBuilder};
use tray_item::{IconSource, TrayItem};

use crate::{
    get_cursor_color::{Color, Position, get_mouse_position, get_pixel_color_and_tip_position},
    utils::set_dpi_awareness,
};

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct ScreenColorAppState {
    visible: bool,
    position: Position,
    color: Color,
    current_tip_position: Position,
}

struct ScreenColorApp {
    state: Arc<Mutex<ScreenColorAppState>>,
}

impl ScreenColorApp {
    pub fn new(context: &Context) -> Self {
        let state = Arc::new(Mutex::new(ScreenColorAppState {
            visible: true,
            ..Default::default()
        }));
        let state_clone = state.clone();
        let context_clone = context.clone();
        thread::spawn(move || {
            loop {
                // let start = Instant::now();
                // let visible = state_lock.visible;
                let position = get_mouse_position().unwrap();
                // println!("pos: {:?}", position);

                let old_tip_position = state_clone.lock().unwrap().current_tip_position;
                let (color, current_tip_position) =
                    get_pixel_color_and_tip_position(position, old_tip_position).unwrap();

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

                thread::sleep(Duration::from_millis(16));
            }
        });

        Self { state }
    }
}

impl eframe::App for ScreenColorApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // let start = Instant::now();

        let state = self.state.lock().unwrap();

        let color = Color32::from_rgb(state.color.r, state.color.g, state.color.b);
        let color_revert = Color32::from_rgb(
            state.color.revert().r,
            state.color.revert().g,
            state.color.revert().b,
        );

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
                });
            });
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    set_dpi_awareness()?;

    let mut tray = TrayItem::new("My App", IconSource::Resource("app-icon")).unwrap();

    tray.add_menu_item("Show/Hide", move || {}).unwrap();

    tray.add_menu_item("Exit", || {
        std::process::exit(0);
    })
    .unwrap();

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

            vsync: true,
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

            Ok(Box::new(ScreenColorApp::new(&cc.egui_ctx.clone())))
        }),
    )?;

    Ok(())
}
