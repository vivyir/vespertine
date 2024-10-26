mod widgets;
pub use widgets::{DevWidgets, DevWidgetsDisplay};

mod aes_key_gen;
pub use aes_key_gen::AesKeyGen;

mod encrypt_payload;
pub use encrypt_payload::EncryptPayload;

mod decrypt_payload;
pub use decrypt_payload::DecryptPayload;

pub trait View {
    fn ui(&mut self, ui: &mut egui::Ui);
}

pub trait DevWidget {
    fn is_enabled(&self, _ctx: &egui::Context) -> bool {
        true
    }

    // static so we can use it as a key to store open/close state
    fn name(&self) -> &'static str;

    // show the damn thing
    fn show(&mut self, ctx: &egui::Context, open: &mut bool);
}
