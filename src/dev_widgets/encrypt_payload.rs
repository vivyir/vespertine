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

pub struct EncryptPayload {
    // base 64
    aes_key: String,
    custom_timestamp: bool,
    custom_timestamp_text: String,
    text: String,
    output_base64: String,
}

impl EncryptPayload {
    pub fn new(aes_key_base64: String) -> Self {
        Self {
            aes_key: aes_key_base64,
            ..Default::default()
        }
    }

    pub fn encrypt_as_payload_to_base64(&self) -> String {
        //STANDARD.encode(self.encrypt())
        let timestamp;
        if self.custom_timestamp {
            timestamp = humantime::parse_rfc3339_weak(&self.custom_timestamp_text).unwrap() // FIXME
        } else {
            timestamp = SystemTime::now();
        }
        let (nonce, ciphertext) = AesPayload::encrypt(self.aes_key.clone(), self.text.clone());
        let aes_payload = AesPayload { ciphertext, nonce, timestamp };
        let encoded_binary = bincode::serialize(&aes_payload).unwrap(); //FIXME

        STANDARD.encode(encoded_binary)
    }
}

impl Default for EncryptPayload {
    fn default() -> Self {
        Self {
            aes_key: "".into(),
            custom_timestamp: false,
            custom_timestamp_text: "".into(),
            text: "".into(),
            output_base64: "".into(),
        }
    }
}

impl DevWidget for EncryptPayload {
    fn name(&self) -> &'static str {
        "Encrypt message"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .max_height(80.0)
            .show(ctx, |ui| self.ui(ui));
    }
}

impl View for EncryptPayload {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.custom_timestamp, "Custom timestamp");
        if self.custom_timestamp {
            ui.horizontal(|ui| {
                ui.label("Timestamp: ");
                egui::TextEdit::singleline(&mut self.custom_timestamp_text).hint_text("2024-01-01 12:30:00 (in the UTC tz)").show(ui);
            });
        }
        ui.horizontal(|ui| {
            ui.label("AES key: ");
            ui.text_edit_singleline(&mut self.aes_key);
        });
        ui.horizontal(|ui| {
            ui.label("Message: ");
            ui.text_edit_singleline(&mut self.text);
        });
        let mut output: &str = self.output_base64.as_str();
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_sized(ui.available_size(), egui::TextEdit::multiline(&mut output));
        });
        if ui.add_sized(ui.available_size(), egui::Button::new("Encrypt")).clicked() {
            self.output_base64 = self.encrypt_as_payload_to_base64();
        }
    }
}
