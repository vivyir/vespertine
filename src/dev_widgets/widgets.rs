use std::collections::BTreeSet;
use crate::DevWidget;

use crate::dev_widgets::{AesKeyGen, EncryptPayload, DecryptPayload};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct DevWidgets {
    #[serde(skip)]
    widgets: Vec<Box<dyn DevWidget>>,
    #[serde(skip)]
    open: BTreeSet<String>,
}

impl Default for DevWidgets {
    fn default() -> Self {
        Self::from_widgets(vec![
            Box::<AesKeyGen>::default(),
            Box::<EncryptPayload>::default(),
            Box::<DecryptPayload>::default(),
        ])
    }
}

impl DevWidgets {
    pub fn from_widgets(widgets: Vec<Box<dyn DevWidget>>) -> Self {
        let mut open = BTreeSet::new();
        // here we can add default ones but we wont
        Self { widgets, open }
    }

    pub fn checkboxes(&mut self, ui: &mut egui::Ui) {
        let Self { widgets, open } = self;
        for widget in widgets {
            if widget.is_enabled(ui.ctx()) {
                let mut is_open = open.contains(widget.name());
                ui.toggle_value(&mut is_open, widget.name());
                set_open(open, widget.name(), is_open);
            }
        }
    }

    pub fn windows(&mut self, ctx: &egui::Context) {
        let Self { widgets, open } = self;
        for widget in widgets {
            let mut is_open = open.contains(widget.name());
            widget.show(ctx, &mut is_open);
            set_open(open, widget.name(), is_open);
        }
    }
}

fn set_open(open: &mut BTreeSet<String>, key: &'static str, is_open: bool) {
    if is_open {
        if !open.contains(key) {
            open.insert(key.to_owned());
        }
    } else {
        open.remove(key);
    }
}

pub struct DevWidgetsDisplay {
    widgets: DevWidgets,
}

impl Default for DevWidgetsDisplay {
    fn default() -> Self {
        Self {
            widgets: DevWidgets::default(),
        }
    }
}

impl DevWidgetsDisplay {
    pub fn show_windows(&mut self, ctx: &egui::Context) {
        self.widgets.windows(ctx);
    }

    pub fn widget_list_ui(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.widgets.checkboxes(ui);
        });
    }
}
