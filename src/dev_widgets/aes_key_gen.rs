use base64::{engine::general_purpose::STANDARD, Engine as _};
use aes_gcm_siv::{
    aead::{Aead, KeyInit, OsRng},
    Aes256GcmSiv, Nonce
};
use crate::{View, DevWidget};

pub struct AesKeyGen {
    aes_key: String,
}

impl AesKeyGen {
    pub fn generate_key() -> String {
        let aes_key_slice = Aes256GcmSiv::generate_key(&mut OsRng);
        let aes_key_slice = aes_key_slice.as_slice();

        STANDARD.encode(aes_key_slice)
    }
}

impl Default for AesKeyGen {
    fn default() -> Self {
        Self {
            aes_key: AesKeyGen::generate_key(),
        }
    }
}

impl DevWidget for AesKeyGen {
    fn name(&self) -> &'static str {
        "AES key (base64)"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .resizable(false)
            .max_height(50.0)
            .show(ctx, |ui| self.ui(ui));
    }
}

impl View for AesKeyGen {
    fn ui(&mut self, ui: &mut egui::Ui) {
        let mut s: &str = self.aes_key.as_str();
        ui.add_sized(ui.available_size(), egui::TextEdit::multiline(&mut s));
        ui.vertical_centered(|ui| {
            if ui.button("Regenerate").clicked() {
                self.aes_key = AesKeyGen::generate_key();
            }
        });
    }
}
