// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod charger;
mod energy_monitor;
mod industrial;
mod robo;

slint::include_modules!();

#[macro_export]
macro_rules! register_singleton_callback {
    ($ui:ident::$datatype:ident::$callback:ident => $this:ident::$fun:ident($($args:ident),*)) => {
        {
            let this = std::rc::Rc::clone(&$this);
            $ui.global::<$datatype>().$callback(move |$($args),*| {
                this.borrow_mut().$fun($($args),*);
            });
        }
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    let charger_backend = charger::ChargerDemoBackend::new(&ui);

    let energy_monitor_backend = energy_monitor::EnergyMonitorBackend::new(&ui);

    let industrial_assets = industrial::IndustrialDemoAssets::new();
    let industrial_backend = industrial::IndustrialDemoBackend::new(&ui, industrial_assets);

    let robo_path_iter = robo::robo_path_iter();
    let robo_backend = robo::RoboBackend::new(&ui, robo_path_iter);

    let demo_loader = ui.global::<DemoLoader>();
    demo_loader.on_index_changed({
        let demo_update_timer: slint::Timer = slint::Timer::default();
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            let demo_loader = ui.global::<DemoLoader>();
            let demo_index = demo_loader.get_selected_index();
            println!("DEMO IDX CHANGED TO: {}", demo_index);

            demo_update_timer.stop();

            match demo_index {
                1 => demo_update_timer.start(
                    slint::TimerMode::Repeated,
                    std::time::Duration::from_millis(1000),
                    {
                        let robo_backend = robo_backend.clone();
                        move || {
                            robo_backend.borrow_mut().task();
                        }
                    },
                ),
                2 => demo_update_timer.start(
                    slint::TimerMode::Repeated,
                    std::time::Duration::from_millis(40),
                    {
                        let industrial_backend = industrial_backend.clone();
                        move || {
                            industrial_backend.borrow_mut().task();
                        }
                    },
                ),
                3 => {
                    energy_monitor_backend.borrow_mut().task(); // do an initial task run first
                    demo_update_timer.start(
                        slint::TimerMode::Repeated,
                        std::time::Duration::from_millis(1000),
                        {
                            let energy_monitor_backend = energy_monitor_backend.clone();
                            move || {
                                energy_monitor_backend.borrow_mut().task();
                            }
                        },
                    )
                }
                4 => demo_update_timer.start(
                    slint::TimerMode::Repeated,
                    std::time::Duration::from_millis(2000),
                    {
                        let charger_backend = charger_backend.clone();
                        move || {
                            charger_backend.borrow_mut().task();
                        }
                    }
                ),
                _ => println!("UNHANDLED DEMO INDEX"),
            }
        }
    });

    let sys_update_timer = slint::Timer::default();
    sys_update_timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(1000),
        {
            let app_window_backend = AppWindowBackend::new(&ui);
            move || {
                let app_window_backend = app_window_backend.clone();
                app_window_backend.borrow_mut().task();
            }
        },
    );

    ui.run()
}

struct AppWindowBackend {
    sys_info: sysinfo::System,
    ui_handle: slint::Weak<AppWindow>,
}

impl AppWindowBackend {
    pub fn new(ui: &AppWindow) -> std::rc::Rc<std::cell::RefCell<Self>> {
        let this = std::rc::Rc::new(std::cell::RefCell::new(Self {
            sys_info: sysinfo::System::new_with_specifics(
                sysinfo::RefreshKind::new()
                    .with_cpu(sysinfo::CpuRefreshKind::everything().without_frequency()),
            ),
            ui_handle: ui.as_weak(),
        }));

        this
    }

    pub fn task(&mut self) {
        self.sys_info.refresh_cpu_usage();
        let cpu_load_total = self
            .sys_info
            .cpus()
            .iter()
            .map(sysinfo::Cpu::cpu_usage)
            .sum::<f32>()
            / self.sys_info.cpus().len() as f32;

        self.sys_info
            .refresh_memory_specifics(sysinfo::MemoryRefreshKind::everything().without_swap());
        let ram_load = self.sys_info.used_memory() as f32 / self.sys_info.total_memory() as f32;

        let ui = self.ui_handle.unwrap();
        let system_resources_data = ui.global::<SystemResourcesData>();
        system_resources_data.set_cpu_load_total(cpu_load_total as i32);
        system_resources_data.set_ram_load(ram_load as i32);
    }
}
