use super::{AppWindow, ChargerInterface};
use slint::ComponentHandle;

pub struct ChargerDemoBackend {
    cycle_counter: u8,
    ui_handle: slint::Weak<AppWindow>,
}

impl ChargerDemoBackend {
    pub fn new(ui: &AppWindow) -> std::rc::Rc<std::cell::RefCell<Self>> {
        let this = std::rc::Rc::new(std::cell::RefCell::new(Self {
            cycle_counter: 0,
            ui_handle: ui.as_weak(),
        }));

        this
    }

    pub fn task(&mut self) {
        self.cycle_counter();
    }

    fn cycle_counter(&mut self) {
        if self.cycle_counter < 4 {
            self.set_counter_value(self.cycle_counter + 1);
        } else {
            self.set_counter_value(0);
        }
    }

    fn set_counter_value(&mut self, value: u8) {
        self.with_mut_ui_data(|this, ui_data| {
            this.cycle_counter = value;
            ui_data.set_cycle_counter(this.cycle_counter as i32);
        });
    }

    // TODO -> create this function (and the ui_handle struct member?) with another macro
    fn with_mut_ui_data(&mut self, fun: impl Fn(&mut Self, ChargerInterface<'_>)) {
        self.ui_handle
            .upgrade()
            .map(|ui| fun(self, ui.global::<ChargerInterface>()));
    }
}
