use crate::app::app_state::AppState;
use crate::model::wheel_mode::WheelMode;
use crate::service::cursor_color::{get_mouse_position, get_screen_data};
use crate::service::utils::{load_icon_data, set_dpi_awareness};
use crate::ui::{screen::show_screen_img, wheel::show_wheel};
use eframe::epaint::StrokeKind;
use egui::{
    Button, Color32, Context, CornerRadius, Frame, Margin, PointerButton, RichText, Stroke,
    TextureHandle, Vec2, ViewportBuilder, ViewportCommand,
};
#[cfg(target_os = "windows")]
use raw_window_handle::HasWindowHandle;
use std::{
    error::Error,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

pub struct App {
    pub state: Arc<Mutex<AppState>>,
    pub wheel_texture: Option<TextureHandle>,
    pub screen_texture: Option<TextureHandle>,
    pub current_screen_tex_size: usize,
}

// init
impl App {
    fn new(state: Arc<Mutex<AppState>>, context: &Context) -> Self {
        context.set_visuals(egui::Visuals {
            panel_fill: Color32::from_rgba_premultiplied(0, 0, 0, 0), // RGB + Alpha
            ..Default::default()
        });
        context.style_mut(|style| {
            style.text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::new(15.0, egui::FontFamily::Monospace),
            );
        });
        context.send_viewport_cmd(ViewportCommand::EnableButtons {
            minimized: true,
            maximize: false,
            close: true,
        });

        let state_clone = state.clone();
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(16));

                let position = match get_mouse_position() {
                    Ok(p) => p,
                    Err(_) => {
                        continue;
                    }
                };

                let screen_tex_size = state_clone.lock().unwrap().screen_tex_size;
                let screen_sample_size = state_clone.lock().unwrap().screen_sample_size;
                let screen_data =
                    match get_screen_data(position, screen_tex_size, screen_sample_size) {
                        Ok(v) => v,
                        Err(_) => {
                            continue;
                        }
                    };
                let color = screen_data.cursor_pixel_color;
                let colors = screen_data.screen_pixel_colors;

                let _ = {
                    let mut s = state_clone.lock().unwrap();
                    if position != s.position || color != s.color {
                        s.screen_colors = colors;
                        s.position = position;
                        s.color = color;

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
            current_screen_tex_size: 0,
        }
    }

    pub fn run() -> Result<(), Box<dyn Error>> {
        set_dpi_awareness()?;
        let state = AppState::new();
        eframe::run_native(
            "Colorose",
            eframe::NativeOptions {
                viewport: ViewportBuilder::default()
                    .with_always_on_top()
                    .with_has_shadow(true)
                    .with_decorations(true)
                    .with_inner_size((450.0, 300.0))
                    .with_icon(load_icon_data("resources/app-icon.png").unwrap())
                    .with_taskbar(true)
                    .with_drag_and_drop(true)
                    .with_resizable(false)
                    .with_transparent(true),

                vsync: true,
                renderer: eframe::Renderer::Glow,
                multisampling: 8,
                ..Default::default()
            },
            Box::new(|cc| {
                #[cfg(target_os = "windows")]
                if let Ok(handle) = cc.window_handle() {
                    use raw_window_handle::RawWindowHandle;

                    if let RawWindowHandle::Win32(handle) = handle.as_raw() {
                        use windows::Win32::Foundation::HWND;

                        use crate::service::utils::enable_acrylic_effect;
                        let hwnd = HWND(handle.hwnd.get() as *mut _);
                        enable_acrylic_effect(hwnd).expect("Failed to enable acrylic effect");
                    }
                }

                Ok(Box::new(App::new(state.clone(), &cc.egui_ctx.clone())))
            }),
        )?;
        Ok(())
    }
}

// ui
impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let mut state = self.state.lock().unwrap();

        let color = Color32::from_rgb(state.color.r, state.color.g, state.color.b);
        let color_revert = Color32::from_rgb(
            state.color.revert().r,
            state.color.revert().g,
            state.color.revert().b,
        );
        let hsv = state.color.to_hsv();
        let hsl = state.color.to_hsl();

        let fg_color = Color32::from_rgb(219, 214, 201);
        // let bg_color = Color32::from_rgb(43, 43, 43);

        egui::CentralPanel::default()
            .frame(Frame {
                fill: Color32::TRANSPARENT,
                inner_margin: Margin::same(10),
                // corner_radius: CornerRadius::same(5),
                // stroke: Stroke::new(2.0, fg_color),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    Frame {
                        inner_margin: Margin::same(5),
                        ..Default::default()
                    }
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            let desired_size = egui::vec2(50.0, 50.0); // 正方形大小
                            let (rect, _) =
                                ui.allocate_exact_size(desired_size, egui::Sense::hover());

                            let rounding = CornerRadius::same(10); // 圆角半径
                            let stroke = Stroke::new(2.0, fg_color);

                            ui.painter()
                                .rect(rect, rounding, color, stroke, StrokeKind::Middle);
                        });
                        ui.vertical(|ui| {
                            Frame {
                                inner_margin: Margin {
                                    left: 15,
                                    right: 0,
                                    top: 0,
                                    bottom: 0,
                                },

                                ..Default::default()
                            }
                            .show(ui, |ui| {
                                ui.set_width(210.0);
                                ui.label(RichText::new(state.position).color(fg_color).strong());
                                ui.label(RichText::new(state.color).color(fg_color).strong());
                                ui.label(RichText::new(hsv).color(fg_color).strong());
                                ui.label(RichText::new(hsl).color(fg_color).strong());
                            });
                        });
                        ui.vertical(|ui| {
                            Frame {
                                inner_margin: Margin {
                                    left: 15,
                                    right: 0,
                                    top: 0,
                                    bottom: 0,
                                },
                                ..Default::default()
                            }
                            .show(ui, |ui| {
                                Frame {
                                    inner_margin: Margin {
                                        left: 10,
                                        right: 10,
                                        top: 2,
                                        bottom: 2,
                                    },
                                    ..Default::default()
                                }
                                .show(ui, |ui| {
                                    ui.set_width(120.0);
                                    ui.set_height(25.0);
                                    let sample_size = state.screen_sample_size;
                                    let sample_size_btn =
                                        Button::new(format!("sample: {:2}", sample_size))
                                            .min_size(Vec2::new(100.0, 20.0));
                                    let sample_size_btn_response = ui.add(sample_size_btn);
                                    if sample_size_btn_response.clicked_by(PointerButton::Secondary) {
                                        if sample_size < state.screen_tex_size {
                                            state.screen_sample_size = sample_size + 2;
                                        }
                                    }
                                    if sample_size_btn_response.clicked_by(PointerButton::Primary)
                                    {
                                        if sample_size > 1 {
                                            state.screen_sample_size = sample_size - 2;
                                        }
                                    }
                                });
                                Frame {
                                    inner_margin: Margin {
                                        left: 10,
                                        right: 10,
                                        top: 2,
                                        bottom: 2,
                                    },
                                    ..Default::default()
                                }
                                .show(ui, |ui| {
                                    ui.set_width(120.0);
                                    ui.set_height(25.0);
                                    let screen_tex_size = state.screen_tex_size;
                                    let screen_tex_size_btn =
                                        Button::new(format!("screen: {:2}", screen_tex_size))
                                            .min_size(Vec2::new(100.0, 20.0));
                                    let screen_tex_size_btn_response = ui.add(screen_tex_size_btn);
                                    if screen_tex_size_btn_response
                                        .clicked_by(PointerButton::Secondary)
                                    {
                                        if screen_tex_size < 25 {
                                            state.screen_tex_size = screen_tex_size + 2;
                                        }
                                    }
                                    if screen_tex_size_btn_response
                                        .clicked_by(PointerButton::Primary)
                                    {
                                        if screen_tex_size > 1 {
                                            state.screen_tex_size = screen_tex_size - 2;
                                        }
                                    }
                                });
                                Frame {
                                    inner_margin: Margin {
                                        left: 10,
                                        right: 10,
                                        top: 2,
                                        bottom: 2,
                                    },
                                    ..Default::default()
                                }
                                .show(ui, |ui| {
                                    ui.set_width(120.0);
                                    ui.set_height(25.0);
                                    let wheel_mode = state.wheel_mode;
                                    let wheel_mode_btn =
                                        Button::new(format!("mode: {:?}", wheel_mode))
                                            .min_size(Vec2::new(100.0, 20.0));
                                    let wheel_mode_btn_response = ui.add(wheel_mode_btn);
                                    if wheel_mode_btn_response.clicked_by(PointerButton::Primary) {
                                        match wheel_mode {
                                            WheelMode::HSL => {
                                                state.wheel_mode = WheelMode::HSV;
                                            }
                                            WheelMode::HSV => {
                                                state.wheel_mode = WheelMode::HSL;
                                            }
                                        };
                                    }
                                });
                            });
                        });
                    });
                });
                ui.horizontal(|ui| {
                    show_wheel(
                        ui,
                        &mut self.wheel_texture,
                        color,
                        color_revert,
                        80.0,
                        12.0,
                        &state.wheel_mode,
                    );

                    show_screen_img(
                        ui,
                        &mut self.screen_texture,
                        160.0,
                        state.screen_tex_size,
                        state.screen_colors.iter().map(|c| c.to_color32()).collect(),
                        color_revert,
                        &mut state.screen_sample_size,
                        &mut self.current_screen_tex_size,
                    );
                });
            });

        ctx.request_repaint_after(Duration::from_millis(34));
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }
}
