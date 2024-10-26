use aes_gcm_siv::{
    aead::{Aead, KeyInit, OsRng},
    Aes256GcmSiv, Nonce
};
use std::collections::BTreeSet;
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;
use std::time::{Duration, SystemTime};
use egui::{RichText, Widget};
use base64::{engine::general_purpose::STANDARD, Engine as _};

use crate::{AesPayload, DevWidgetsDisplay};

// terrible code, pure dogshit, absolute horseshit, fuming Excrement. BUT AT LEAST IT WORKS
// TODO: clean the foaming dogshit code

pub type MessageManifest = Vec<MessageEntry>;

#[serde_with::serde_as]
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageEntry {
    pub name: String,
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    pub timestamp: Duration,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct FireApp {
    // Example stuff:
    message: String,
    timestamp_fmt: String,

    source_url: String,
    aes_key_base64: String,

    #[serde(skip)]
    show_spinner: bool,
    
    #[serde(skip)]
    startup: bool,

    #[serde(skip)]
    src_url_viewport: bool,
    #[serde(skip)]
    aes_key_viewport: bool,
    #[serde(skip)]
    dev_widgets_display: DevWidgetsDisplay,
    #[serde(skip)]
    open_widgets: BTreeSet<String>,

    #[serde(skip)]
    channels: (Sender<Signals>, Receiver<Signals>),

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
}

pub enum Signals {
    RetrievedManifest(MessageManifest),
    // Timestamp text, Message
    Base64DecryptedPayload(String, String),
}

impl Default for FireApp {
    fn default() -> Self {
        Self {
            message: "Loading latest message…".to_owned(),
            timestamp_fmt: "1970-01-01 00:00:00 UTC".to_owned(),
            
            source_url: "".to_owned(),
            aes_key_base64: "".to_owned(),

            show_spinner: false,
            startup: true,

            dev_widgets_display: DevWidgetsDisplay::default(),
            open_widgets: BTreeSet::new(),

            channels: mpsc::channel::<Signals>(),

            src_url_viewport: false,
            aes_key_viewport: false,
            value: 2.7,
        }
    }
}

impl FireApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    pub fn get_manifest(&mut self) {
        self.show_spinner = true;
        // .0 is tx
        let reqtx = self.channels.0.clone();
        let mut manifest_url = self.source_url.clone();
        manifest_url.push_str("manifest.json");
        thread::spawn(move || {
            // FIXME
            //let result = ureq::get("http://example.com").call().unwrap().into_string().unwrap();
            //reqtx.send(Signals::Text(result)).unwrap();

            // FIXME: handle network errors gratefully
            let manifest_file = ureq::get(manifest_url.as_str()).call().unwrap().into_string().unwrap();
            let mut manifest: MessageManifest = serde_json::from_str(manifest_file.as_str()).unwrap();

            manifest.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            reqtx.send(Signals::RetrievedManifest(manifest)).unwrap();
        });
    }

    pub fn get_message(&mut self, filename: String) {
        self.show_spinner = true;

        let reqtx = self.channels.0.clone();
        let mut file_url = self.source_url.clone();
        let mut aes_key = self.aes_key_base64.clone();
        file_url.push_str(filename.as_str());
        thread::spawn(move || {
            let message_req = ureq::get(file_url.as_str()).call().unwrap().into_string().unwrap();
            let message_req = message_req.trim();
            
            let ( timestamp_text, plaintext ) = AesPayload::decrypt(aes_key, message_req.to_string());

            reqtx.send(Signals::Base64DecryptedPayload(
                timestamp_text,
                plaintext,
            ));
        });
    }
}

impl eframe::App for FireApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        if self.startup {
            self.get_manifest();
            self.startup = false;
        }

        // .1 is rx
        if let Ok(sugnal) = self.channels.1.try_recv() {
            self.show_spinner = false;
            match sugnal {
                Signals::RetrievedManifest(v) => {
                    //FIXME
                    let current_unix_timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
                    for entry in v {
                        if entry.timestamp <= current_unix_timestamp {
                            self.get_message(entry.name.clone());
                        }
                    }
                }
                Signals::Base64DecryptedPayload(timestamp_text, message) => {
                    self.timestamp_fmt = timestamp_text;
                    self.message = message;
                }
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                ui.menu_button("Settings", |ui| {
                    egui::widgets::global_theme_preference_buttons(ui);
                    if ui.button("Set source URL").clicked() {
                        self.src_url_viewport = true;
                    }
                    if ui.button("Set AES key").clicked() {
                        self.aes_key_viewport = true;
                    }
                    if ui.button("Refresh").clicked() {
                        self.startup = true;
                    }
                    if self.source_url.to_lowercase() == "developer mode" {
                        ui.menu_button("Developer tools", |ui| {
                            self.dev_widgets_display.widget_list_ui(ui);
                        });
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            //ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.columns(2, |cols| {
                        cols[0].with_layout(egui::Layout::left_to_right(egui::Align::BOTTOM), |ui| {
                            ui.label("Made with ♥ by Vivian for her amazing girlfriend.");
                        });
                        cols[1].with_layout(egui::Layout::right_to_left(egui::Align::BOTTOM), |ui| {
                            if self.show_spinner {
                                egui::Spinner::new().ui(ui);
                            }
                            ui.hyperlink_to("Source", "https://github.com/vivyir/vespertine");
                            egui::warn_if_debug_build(ui);
                        });
                    });
                });
            //});
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut timestamp: &str = self.timestamp_fmt.as_str();
                ui.horizontal(|ui| {
                    ui.label("Timestamp: ");
                    ui.add_sized(ui.available_size(), egui::TextEdit::singleline(&mut timestamp));
                });
                ui.separator();
                ui.label(RichText::new(self.message.as_str()).size(15.0));
            });
        });

        self.dev_widgets_display.show_windows(ctx);

        if self.src_url_viewport {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("src_url_viewport"),
                egui::ViewportBuilder::default()
                    .with_title("Change source URL")
                    .with_inner_size([380.0, 50.0]),
                |ctx, class| {
                    assert!(
                        class == egui::ViewportClass::Immediate,
                        "egui backend doesn't support multiple viewports"
                    );

                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            ui.label("Source URL: ");
                            let response = ui.text_edit_singleline(&mut self.source_url);
                            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                self.src_url_viewport = false;
                            }
                        })
                    });

                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.src_url_viewport = false;
                    }
                })
        }

        if self.aes_key_viewport {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("aes_key_viewport"),
                egui::ViewportBuilder::default()
                    .with_title("Change AES key")
                    .with_inner_size([380.0, 50.0]),
                |ctx, class| {
                    assert!(
                        class == egui::ViewportClass::Immediate,
                        "egui backend doesn't support multiple viewports"
                    );

                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            ui.label("AES key (base64): ");
                            let response = ui.text_edit_singleline(&mut self.aes_key_base64);
                            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                self.aes_key_viewport = false;
                            }
                        })
                    });

                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.aes_key_viewport = false;
                    }
                })
        }
    }
}
