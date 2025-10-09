use eframe::egui::{self, FontData, FontDefinitions, FontFamily};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

mod converter;
mod file_handler;
mod types;

use converter::AudioConverter;
use types::*;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "音频格式转换工具",
        options,
        Box::new(|cc| {
            // Install image loaders
            egui_extras::install_image_loaders(&cc.egui_ctx);

            // Load SimHei.ttf (located at project root ../SimHei.ttf relative to src)
            // and register it as the primary font for Chinese display.
            let mut fonts = FontDefinitions::default();
            fonts.font_data.insert(
                "SimHei".to_owned(),
                FontData::from_static(include_bytes!("../SimHei.ttf")),
            );

            // Prepend SimHei to the proportional (default) family so it's used for UI text.
            fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .insert(0, "SimHei".to_owned());

            // Also make it available for monospace if needed.
            fonts
                .families
                .entry(FontFamily::Monospace)
                .or_default()
                .push("SimHei".to_owned());

            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(AudioConverterApp::new()))
        }),
    )
}

struct AudioConverterApp {
    converter: Arc<Mutex<AudioConverter>>,
    runtime: Arc<Runtime>,
    min_duration: f32,
    concurrent_tasks: usize,
    queue: Arc<Mutex<Vec<FileTask>>>,
    drop_zone_text: String,
}

impl AudioConverterApp {
    fn new() -> Self {
        Self {
            converter: Arc::new(Mutex::new(AudioConverter::new())),
            runtime: Arc::new(Runtime::new().unwrap()),
            min_duration: 10.0,
            concurrent_tasks: 2,
            queue: Arc::new(Mutex::new(Vec::new())),
            drop_zone_text: "拖放文件或文件夹到这里".to_string(),
        }
    }

    fn add_files(&mut self, paths: Vec<PathBuf>) {
        let mut queue = self.queue.lock().unwrap();

        for path in paths {
            if path.is_file() {
                if file_handler::is_audio_file(&path) && !file_handler::is_wav_file(&path) {
                    queue.push(FileTask {
                        input_path: path.clone(),
                        output_path: file_handler::get_output_path(&path),
                        status: TaskStatus::Pending,
                        progress: 0.0,
                        error_message: None,
                    });
                }
            } else if path.is_dir() {
                let files = file_handler::find_audio_files(&path);
                for file in files {
                    if !file_handler::is_wav_file(&file) {
                        queue.push(FileTask {
                            input_path: file.clone(),
                            output_path: file_handler::get_output_path(&file),
                            status: TaskStatus::Pending,
                            progress: 0.0,
                            error_message: None,
                        });
                    }
                }
            }
        }
    }

    fn start_conversion(&mut self) {
        let queue = self.queue.clone();
        let converter = self.converter.clone();
        let runtime = self.runtime.clone();
        let min_duration = self.min_duration;
        let concurrent_tasks = self.concurrent_tasks;

        runtime.spawn(async move {
            let mut handles = vec![];
            let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrent_tasks));

            loop {
                let task_opt = {
                    let mut q = queue.lock().unwrap();
                    q.iter_mut()
                        .find(|t| matches!(t.status, TaskStatus::Pending))
                        .map(|t| {
                            t.status = TaskStatus::Processing;
                            t.clone()
                        })
                };

                if let Some(task) = task_opt {
                    let permit = semaphore.clone().acquire_owned().await.unwrap();
                    let queue_clone = queue.clone();
                    let converter_clone = converter.clone();

                    let handle = tokio::spawn(async move {
                        let result = converter_clone.lock().unwrap().convert_audio(
                            &task.input_path,
                            &task.output_path,
                            min_duration,
                        );

                        let mut q = queue_clone.lock().unwrap();
                        if let Some(t) = q.iter_mut().find(|t| t.input_path == task.input_path) {
                            match result {
                                Ok(_) => {
                                    t.status = TaskStatus::Completed;
                                    t.progress = 100.0;
                                }
                                Err(e) => {
                                    t.status = TaskStatus::Failed;
                                    t.error_message = Some(e.to_string());
                                }
                            }
                        }

                        drop(permit);
                    });

                    handles.push(handle);
                } else {
                    break;
                }
            }

            for handle in handles {
                let _ = handle.await;
            }
        });
    }

    fn clear_completed(&mut self) {
        let mut queue = self.queue.lock().unwrap();
        queue.retain(|t| !matches!(t.status, TaskStatus::Completed));
    }

    fn clear_all(&mut self) {
        let mut queue = self.queue.lock().unwrap();
        queue.clear();
    }

    fn get_stats(&self) -> (usize, usize, usize, usize) {
        let queue = self.queue.lock().unwrap();
        let total = queue.len();
        let completed = queue
            .iter()
            .filter(|t| matches!(t.status, TaskStatus::Completed))
            .count();
        let processing = queue
            .iter()
            .filter(|t| matches!(t.status, TaskStatus::Processing))
            .count();
        let failed = queue
            .iter()
            .filter(|t| matches!(t.status, TaskStatus::Failed))
            .count();
        (total, completed, processing, failed)
    }
}

impl eframe::App for AudioConverterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        // Handle drag and drop
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                let paths: Vec<PathBuf> = i
                    .raw
                    .dropped_files
                    .iter()
                    .filter_map(|f| f.path.clone())
                    .collect();
                self.add_files(paths);
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎵 音频格式转换工具");
            ui.add_space(10.0);

            // Settings panel
            ui.group(|ui| {
                ui.label("⚙️ 设置");
                ui.horizontal(|ui| {
                    ui.label("最小时长 (秒):");
                    ui.add(egui::Slider::new(&mut self.min_duration, 1.0..=300.0));
                });
                ui.horizontal(|ui| {
                    ui.label("并发任务数:");
                    ui.add(egui::Slider::new(&mut self.concurrent_tasks, 1..=8));
                });
            });

            ui.add_space(10.0);

            // Drop zone
            let drop_zone = ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), 100.0),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add_space(30.0);
                    ui.label(
                        egui::RichText::new(&self.drop_zone_text)
                            .size(20.0)
                            .color(egui::Color32::GRAY),
                    );
                },
            );

            let drop_rect = drop_zone.response.rect;
            ui.painter().rect_stroke(
                drop_rect,
                5.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 100, 100)),
            );

            ui.add_space(10.0);

            // Control buttons
            ui.horizontal(|ui| {
                let (total, completed, processing, _failed) = self.get_stats();
                let can_start = total > 0 && processing == 0 && completed < total;

                if ui
                    .add_enabled(can_start, egui::Button::new("▶ 开始转换"))
                    .clicked()
                {
                    self.start_conversion();
                }

                if ui.button("🗑 清除已完成").clicked() {
                    self.clear_completed();
                }

                if ui.button("🗑 清空队列").clicked() {
                    self.clear_all();
                }
            });

            ui.add_space(10.0);

            // Statistics
            let (total, completed, processing, failed) = self.get_stats();
            ui.horizontal(|ui| {
                ui.label(format!("📊 总计: {}", total));
                ui.separator();
                ui.label(format!("✅ 已完成: {}", completed));
                ui.separator();
                ui.label(format!("⏳ 处理中: {}", processing));
                ui.separator();
                ui.label(format!("❌ 失败: {}", failed));
            });

            if total > 0 {
                let progress = completed as f32 / total as f32;
                ui.add(egui::ProgressBar::new(progress).show_percentage());
            }

            ui.add_space(10.0);

            // Task list
            egui::ScrollArea::vertical()
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    let queue = self.queue.lock().unwrap();
                    for task in queue.iter() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                let icon = match task.status {
                                    TaskStatus::Pending => "⏸",
                                    TaskStatus::Processing => "⏳",
                                    TaskStatus::Completed => "✅",
                                    TaskStatus::Failed => "❌",
                                };
                                ui.label(icon);

                                ui.vertical(|ui| {
                                    ui.label(
                                        task.input_path
                                            .file_name()
                                            .unwrap()
                                            .to_string_lossy()
                                            .to_string(),
                                    );

                                    if let TaskStatus::Processing = task.status {
                                        ui.add(
                                            egui::ProgressBar::new(task.progress / 100.0)
                                                .show_percentage(),
                                        );
                                    }

                                    if let Some(ref err) = task.error_message {
                                        ui.colored_label(egui::Color32::RED, err);
                                    }
                                });
                            });
                        });
                        ui.add_space(5.0);
                    }
                });
        });
    }
}
