mod logger;
mod mock_platform;
mod platform;

use std::time::Instant;

use egui::{Color32, RichText, ScrollArea};

use crate::logger::{LogLevel, Logger};
use crate::mock_platform::MockPlatform;
use crate::platform::{IPlatformBackend, PlatformKind, PlatformResult};

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;
const STATUS_BAR_HEIGHT: f32 = 36.0;
const LOG_PANEL_MIN_HEIGHT: f32 = 120.0;
const FPS_UPDATE_INTERVAL_SECS: f64 = 0.5;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(WINDOW_WIDTH, WINDOW_HEIGHT)),
        min_window_size: Some(egui::vec2(WINDOW_WIDTH, WINDOW_HEIGHT)),
        ..Default::default()
    };

    eframe::run_native(
        "Jade Engine - MVP",
        native_options,
        Box::new(|_cc| Box::new(EngineEditor::new())),
    )
}

pub struct EngineEditor {
    platform: Box<dyn IPlatformBackend>,
    logger: Logger,

    app_start: Instant,
    last_status_update: Instant,
    last_frame_time: Instant,
    frame_count_since_update: u32,
    current_fps: f64,

    auto_refresh_time: bool,
    last_auto_refresh: Instant,

    last_result: String,
    last_result_ok: bool,

    fps_recent_samples: Vec<f64>,
}

impl EngineEditor {
    fn new() -> Self {
        let mut logger = Logger::new();
        logger.info("Jade Engine MVP starting up...");
        logger.info("Loading Mock virtual platform adapter...");

        let platform: Box<dyn IPlatformBackend> = Box::new(MockPlatform::new());

        logger.info(format!(
            "Platform backend initialized: {}",
            platform.platform_kind().label()
        ));
        logger.info("PAL abstraction layer ready.");

        Self {
            platform,
            logger,
            app_start: Instant::now(),
            last_status_update: Instant::now(),
            last_frame_time: Instant::now(),
            frame_count_since_update: 0,
            current_fps: 60.0,
            auto_refresh_time: false,
            last_auto_refresh: Instant::now(),
            last_result: "(no test executed yet)".to_string(),
            last_result_ok: true,
            fps_recent_samples: Vec::new(),
        }
    }

    fn run_platform_test(&mut self, name: &str, test: impl FnOnce(&dyn IPlatformBackend) -> PlatformResult<String>) {
        match test(self.platform.as_ref()) {
            Ok(value) => {
                self.last_result = value.clone();
                self.last_result_ok = true;
                self.logger.info(format!("{} => {}", name, value));
            }
            Err(err) => {
                self.last_result = format!("{}", err);
                self.last_result_ok = false;
                self.logger.error(format!("{} failed: {}", name, err));
            }
        }
    }

    fn refresh_fps(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_frame_time).as_secs_f64();
        self.last_frame_time = now;

        if elapsed > 0.0 {
            self.fps_recent_samples.push(1.0 / elapsed);
            if self.fps_recent_samples.len() > 30 {
                self.fps_recent_samples.remove(0);
            }
        }

        let update_elapsed = now.duration_since(self.last_status_update).as_secs_f64();
        if update_elapsed >= FPS_UPDATE_INTERVAL_SECS {
            self.current_fps = if self.fps_recent_samples.is_empty() {
                60.0
            } else {
                let sum: f64 = self.fps_recent_samples.iter().sum();
                sum / self.fps_recent_samples.len() as f64
            };
            self.last_status_update = now;
        }
    }

    fn ui_top_status(&mut self, ctx: &egui::Context) {
        let top_frame = egui::Frame::panel()
            .inner_margin(egui::Margin::symmetric(10.0, 6.0));

        egui::TopBottomPanel::top("status_bar")
            .resizable(false)
            .min_height(STATUS_BAR_HEIGHT)
            .exact_height(STATUS_BAR_HEIGHT)
            .frame(top_frame)
            .show(ctx, |ui| {
                let platform_label = self.platform.platform_kind();
                let platform_color = match platform_label {
                    PlatformKind::Virtual => Color32::LIGHT_GRAY,
                    _ => Color32::LIGHT_GREEN,
                };

                let uptime = self.app_start.elapsed().as_secs_f64();
                let memory = self.platform.memory_usage_mb().unwrap_or(0.0);
                let fps = self.current_fps;

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 16.0;

                    ui.label(RichText::new("▶ Jade Engine").strong());

                    ui.separator();

                    ui.label(RichText::new(platform_label.label()).color(platform_color));

                    ui.separator();

                    ui.label(format!("Uptime: {:.2}s", uptime));
                    ui.label(format!("Memory: {:.2} MB", memory));
                    ui.label(format!("FPS: {:.1}", fps));
                });
            });
    }

    fn ui_bottom_log(&mut self, ctx: &egui::Context) {
        let log_frame = egui::Frame::panel()
            .inner_margin(egui::Margin::symmetric(8.0, 6.0));

        egui::TopBottomPanel::bottom("log_panel")
            .resizable(true)
            .default_height(WINDOW_HEIGHT * 0.25)
            .min_height(LOG_PANEL_MIN_HEIGHT)
            .frame(log_frame)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("System Log").strong());
                    ui.separator();

                    let mut show_ts = self.logger.show_timestamp();
                    ui.checkbox(&mut show_ts, "Timestamp");
                    self.logger.set_show_timestamp(show_ts);

                    let mut show_lv = self.logger.show_level();
                    ui.checkbox(&mut show_lv, "Level");
                    self.logger.set_show_level(show_lv);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Clear").clicked() {
                            self.logger.clear();
                        }
                    });
                });

                ui.separator();

                let entries: Vec<_> = self.logger.entries().to_vec();
                ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for entry in entries {
                            let color = match entry.level {
                                LogLevel::Info => Color32::WHITE,
                                LogLevel::Warn => Color32::YELLOW,
                                LogLevel::Error => Color32::LIGHT_RED,
                            };

                            let mut parts = String::new();
                            if self.logger.show_timestamp() {
                                parts.push_str(&format!("[{}] ", entry.timestamp));
                            }
                            if self.logger.show_level() {
                                parts.push_str(&format!("[{}] ", entry.level.label()));
                            }
                            parts.push_str(&entry.message);

                            ui.label(RichText::new(parts).color(color).monospace());
                        }
                    });
            });
    }

    fn ui_left_panel(&mut self, ctx: &egui::Context) {
        let left_frame = egui::Frame::panel()
            .inner_margin(egui::Margin::symmetric(8.0, 8.0));

        egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(WINDOW_WIDTH * 0.28)
            .width_range(180.0..=300.0)
            .frame(left_frame)
            .show(ctx, |ui| {
                ui.label(RichText::new("PAL Test Console").strong().size(14.0));
                ui.add_space(4.0);

                ui.collapsing("Platform Info", |ui| {
                    if ui.button("Get Platform Name").clicked() {
                        self.run_platform_test("platform_name", |b| b.platform_name());
                    }
                    if ui.button("Get System Version").clicked() {
                        self.run_platform_test("system_version", |b| b.system_version());
                    }
                    if ui.button("Get CPU Core Count").clicked() {
                        self.run_platform_test("cpu_core_count", |b| {
                            b.cpu_core_count().map(|n| n.to_string())
                        });
                    }
                });

                ui.collapsing("Time & Clock", |ui| {
                    if ui.button("High Precision Time").clicked() {
                        self.run_platform_test("high_precision_time", |b| {
                            b.high_precision_time().map(|t| format!("{:.6}s", t))
                        });
                    }
                    if ui.button("Frame Interval (10 frames)").clicked() {
                        self.run_platform_test("frame_interval_sample", |b| {
                            b.frame_interval_sample(10)
                                .map(|t| format!("avg {:.6}s / frame", t / 10.0))
                        });
                    }

                    ui.checkbox(&mut self.auto_refresh_time, "Auto refresh time");
                });

                ui.collapsing("Logging System", |ui| {
                    if ui.button("Info Log").clicked() {
                        self.logger.info("User-triggered INFO log entry.");
                    }
                    if ui.button("Warn Log").clicked() {
                        self.logger.warn("User-triggered WARN log entry.");
                    }
                    if ui.button("Error Log").clicked() {
                        self.logger.error("User-triggered ERROR log entry.");
                    }
                });
            });
    }

    fn ui_right_panel(&mut self, ctx: &egui::Context) {
        let right_frame = egui::Frame::panel()
            .inner_margin(egui::Margin::symmetric(8.0, 8.0));

        egui::SidePanel::right("right_panel")
            .resizable(true)
            .default_width(WINDOW_WIDTH * 0.12)
            .width_range(80.0..=200.0)
            .frame(right_frame)
            .show(ctx, |ui| {
                ui.label(RichText::new("Reserved").strong().size(14.0));
                ui.add_space(4.0);
                ui.label(RichText::new("(future extensions)").color(Color32::DARK_GRAY));
                ui.separator();
                ui.label("This area is intentionally reserved for upcoming engine modules:");
                ui.add_space(4.0);
                ui.label("• Asset browser");
                ui.label("• Scene graph");
                ui.label("• Inspector");
                ui.label("• Console REPL");
            });
    }

    fn ui_center_panel(&mut self, ctx: &egui::Context) {
        let center_frame = egui::Frame::panel()
            .inner_margin(egui::Margin::symmetric(10.0, 8.0));

        egui::CentralPanel::default()
            .frame(center_frame)
            .show(ctx, |ui| {
                ui.label(RichText::new("Test Result").strong().size(14.0));
                ui.add_space(4.0);

                let result_color = if self.last_result_ok {
                    Color32::WHITE
                } else {
                    Color32::LIGHT_RED
                };

                let result_frame = egui::Frame::group()
                    .inner_margin(egui::Margin::symmetric(10.0, 10.0));

                result_frame.show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label("Last PAL call returned:");
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(self.last_result.clone())
                                .color(result_color)
                                .monospace()
                                .size(13.0),
                        );
                    });
                });

                ui.add_space(8.0);

                ui.separator();

                ui.add_space(6.0);
                ui.label(RichText::new("Architecture Overview").strong().size(14.0));
                ui.add_space(4.0);

                let overview = egui::Frame::group()
                    .inner_margin(egui::Margin::symmetric(10.0, 10.0));

                overview.show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("PAL Abstraction Layer").strong());
                        ui.label("The engine interacts with every platform solely through the trait IPlatformBackend.");
                        ui.add_space(4.0);
                        ui.label(RichText::new("Current backend:").strong());
                        ui.label(format!("  • {}", self.platform.platform_kind().label()));
                        ui.add_space(6.0);
                        ui.label(RichText::new("Next development steps:").strong());
                        ui.label("  1. Add a WindowsPlatform real adapter.");
                        ui.label("  2. Add a LinuxPlatform (PikaOS) real adapter.");
                        ui.label("  3. Extend PAL with file I/O, windowing, and input traits.");
                        ui.label("  4. Plug in the permanent runtime entry point.");
                    });
                });
            });
    }

    fn handle_auto_refresh(&mut self) {
        if !self.auto_refresh_time {
            return;
        }
        let elapsed = self.last_auto_refresh.elapsed().as_secs_f64();
        if elapsed >= 1.0 {
            if let Ok(t) = self.platform.high_precision_time() {
                self.last_result = format!("{:.6}s (auto)", t);
                self.last_result_ok = true;
            }
            self.last_auto_refresh = Instant::now();
        }
    }
}

impl eframe::App for EngineEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.refresh_fps();
        self.handle_auto_refresh();

        self.ui_top_status(ctx);
        self.ui_bottom_log(ctx);
        self.ui_left_panel(ctx);
        self.ui_right_panel(ctx);
        self.ui_center_panel(ctx);

        ctx.request_repaint();
    }
}
