use base64::{engine::general_purpose::STANDARD, Engine as _};
use aes_gcm_siv::{
    aead::{Aead, KeyInit, OsRng},
    Aes256GcmSiv, Nonce
};
use argon2::{
    password_hash::{
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use crate::{View, DevWidget, AesPayload};
use std::time::SystemTime;

pub struct DecryptPayload {
    // base 64
    payload_base64: String,
    aes_key: String,
    timestamp_text: String,
    timestamp_unix: String,
    message: String,
}

impl DecryptPayload {
    pub fn new(payload: String, aes_key_base64: String) -> Self {
        Self {
            payload_base64: payload,
            aes_key: aes_key_base64,
            ..Default::default()
        }
    }
}

impl Default for DecryptPayload {
    fn default() -> Self {
        Self {
            payload_base64: "".into(),
            aes_key: "".into(),
            timestamp_text: "".into(),
            timestamp_unix: "".into(),
            message: "".into(),
        }
    }
}

impl DevWidget for DecryptPayload {
    fn name(&self) -> &'static str {
        "Decrypt message"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .max_height(80.0)
            .show(ctx, |ui| self.ui(ui));
    }
}

impl View for DecryptPayload {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Payload: ");
            egui::TextEdit::singleline(&mut self.payload_base64).show(ui);
        });
        ui.horizontal(|ui| {
            ui.label("AES key: ");
            ui.text_edit_singleline(&mut self.aes_key);
        });
        let mut timestamp: &str = self.timestamp_text.as_str();
        ui.horizontal(|ui| {
            ui.label("Timestamp: ");
            ui.add_sized(ui.available_size(), egui::TextEdit::singleline(&mut timestamp));
        });
        let mut timestamp_unix: &str = self.timestamp_unix.as_str();
        ui.horizontal(|ui| {
            ui.label("Timestamp (UNIX): ");
            ui.add_sized(ui.available_size(), egui::TextEdit::singleline(&mut timestamp_unix));
        });
        let mut message: &str = self.message.as_str();
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.label("Message: ");
            ui.add_sized(ui.available_size(), egui::TextEdit::multiline(&mut message));
        });
        if ui.add_sized(ui.available_size(), egui::Button::new("Decrypt")).clicked() {
            let ( timestamp, message_text ) = AesPayload::decrypt(self.aes_key.clone(), self.payload_base64.clone());
            self.timestamp_text = timestamp;
            let timestamp = humantime::parse_rfc3339(self.timestamp_text.as_str()).unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
            self.timestamp_unix = format!("{timestamp}");
            self.message = message_text;
        }
    }
}
