#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::path::PathBuf;

use eframe::{App, CreationContext};
use egui_extras::Column;
use file_receiver::{FileReceievers, FileReceiverResult, FileReceiverSource};

mod file_receiver;

pub mod watermark;

#[derive(Default)]
pub struct Watermarker {
    file_recievers: FileReceievers,
    sources: Vec<PathBuf>,
    watermark: Option<PathBuf>,
    destination: Option<PathBuf>,
    marking: Option<usize>,
}

impl Watermarker {
    pub fn new(cc: &CreationContext) -> Box<dyn App> {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        Box::new(Self::default())
    }
}

impl App for Watermarker {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.file_recievers.receive_all();
        if self.marking.is_none() {
            if let Some(r) = self
                .file_recievers
                .get_receiver(FileReceiverSource::SourceImages)
            {
                if let FileReceiverResult::File(paths) = r.get_file() {
                    self.sources = paths.to_vec();
                }
            }
            if let Some(r) = self
                .file_recievers
                .get_receiver(FileReceiverSource::Watermark)
            {
                if let FileReceiverResult::File(paths) = r.get_file() {
                    self.watermark = paths.first().cloned();
                }
            }
            if let Some(r) = self
                .file_recievers
                .get_receiver(FileReceiverSource::DestinationFolder)
            {
                if let FileReceiverResult::File(paths) = r.get_file() {
                    self.destination = paths.first().cloned();
                }
            }
        }
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Quit").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });
        // Draw things here.
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                add_button_and_display_result(
                    ui,
                    FileReceiverSource::SourceImages,
                    &mut self.file_recievers,
                );
                add_button_and_display_result(
                    ui,
                    FileReceiverSource::Watermark,
                    &mut self.file_recievers,
                );
                add_button_and_display_result(
                    ui,
                    FileReceiverSource::DestinationFolder,
                    &mut self.file_recievers,
                );
                ui.horizontal(|ui| {
                    ui.add_enabled_ui(
                        self.marking.is_none()
                            && self.destination.is_some()
                            && self.watermark.is_some()
                            && !self.sources.is_empty(),
                        |ui| {
                            if ui.button("Apply watermarks").clicked() {
                                self.marking = Some(0);
                                ctx.request_repaint();
                            }
                        },
                    )
                })
            });
            ui.separator();
            let width = ui.available_width();
            let min_width = 200.0;
            let columns = (width / min_width).floor() as usize;
            let images = self
                .sources
                .iter()
                .enumerate()
                .map(|(idx, p)| {
                    let should_be_green = if let Some(marking) = self.marking {
                        idx < marking
                    } else {
                        false
                    };
                    egui::Image::new(format!("file://{}", p.to_string_lossy()))
                        .maintain_aspect_ratio(true)
                        .max_size([200.0, 200.0].into())
                        .fit_to_original_size(10.0)
                        .tint(if should_be_green {
                            egui::Color32::GREEN
                        } else {
                            egui::Color32::WHITE
                        })
                })
                .collect::<Vec<_>>();
            let row_count = (images.len() as f32 / columns as f32).ceil() as usize;
            egui::scroll_area::ScrollArea::new([true, false])
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui_extras::TableBuilder::new(ui)
                        .columns(Column::auto(), columns)
                        .striped(true)
                        .body(|ui| {
                            ui.rows(200.0, row_count, |row_index, mut row| {
                                for i in 0..columns {
                                    let index = row_index * columns + i;
                                    if index >= images.len() {
                                        break;
                                    }
                                    row.col(|ui| {
                                        ui.add(images[index].clone());
                                    });
                                }
                            });
                        });
                });
        });
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Watermarker");
                ui.label("By Julius de Jeu");
            });
        });
        if let Some(marking) = self.marking {
            if let Some(source) = self.sources.get(marking) {
                if let Some(destination) = &self.destination {
                    if let Some(watermark) = &self.watermark {
                        if let Err(e) = watermark::watermark_image(
                            source,
                            destination,
                            watermark,
                            0.4,
                        ) {
                            eprintln!("Error: {}", e);
                        }
                    }
                }
            }
            self.marking = Some(marking + 1);
            if self.marking.unwrap() >= self.sources.len() {
                self.marking = None;
            }
            ctx.request_repaint();
        }
    }
}

fn add_button_and_display_result(
    ui: &mut egui::Ui,
    source: FileReceiverSource,
    receivers: &mut FileReceievers,
) {
    ui.horizontal(|ui| {
        if ui.button(source.to_string()).clicked() {
            receivers.new_receiver(source);
        }
        if let Some(receiver) = receivers.get_receiver(source) {
            match receiver.get_file() {
                FileReceiverResult::File(path) => {
                    if path.len() > 1 {
                        ui.label("Multiple files selected");
                    } else {
                        ui.label(path[0].to_str().unwrap());
                    }
                }
                FileReceiverResult::NoFile => {
                    ui.label("No file selected");
                }
                FileReceiverResult::Waiting => {
                    ui.label("Waiting for file");
                }
            }
        } else {
            ui.label("");
        }
    });
}
