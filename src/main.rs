// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod charger;
mod energy_monitor;
mod industrial;
mod robo;
use clap::Parser;
use rumqttc::{Client, Event, Incoming, MqttOptions, QoS};
use serde_json;

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

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    mqtt_host: Option<String>,
    mqtt_port: Option<u16>,
}

fn main() -> Result<(), slint::PlatformError> {
    let args = Args::parse();
    let mqtt_broker_host_address = args.mqtt_host.unwrap_or(String::from("127.0.0.1"));
    let mqtt_broker_port = args.mqtt_port.unwrap_or(1883);

    let ui = AppWindow::new()?;

    // setup MQTT client
    let mqtt_options = MqttOptions::new("TI-demo-box", mqtt_broker_host_address, mqtt_broker_port);
    let (mqtt_client, mut mqtt_eventloop) = Client::new(mqtt_options, 10);

    let charger_backend = charger::ChargerDemoBackend::new(&ui);

    let energy_monitor_backend = energy_monitor::EnergyMonitorBackend::new(&ui);

    let industrial_assets = industrial::IndustrialDemoAssets::new();
    let industrial_backend = industrial::IndustrialDemoBackend::new(&ui, industrial_assets);

    // let robo_path_iter = robo::robo_path_iter();
    let robo_backend = robo::RoboBackend::new(&ui);

    let demo_loader = ui.global::<DemoLoader>();
    demo_loader.on_index_changed({
        let demo_update_timer = slint::Timer::default();
        let mqtt_update_timer = slint::Timer::default();
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            let demo_loader = ui.global::<DemoLoader>();
            let demo_index = demo_loader.get_selected_index();
            println!("DEMO IDX CHANGED TO: {}", demo_index);

            demo_update_timer.stop();

            match demo_index {
                1 => {
                    demo_update_timer.start(
                        slint::TimerMode::Repeated,
                        std::time::Duration::from_millis(33),
                        {
                            let robo_backend = robo_backend.clone();
                            move || {
                                robo_backend.borrow_mut().task();
                            }
                        },
                    );
                    mqtt_update_timer.start(
                        slint::TimerMode::Repeated,
                        std::time::Duration::from_millis(500),
                        {
                            let mqtt_client = mqtt_client.clone();
                            let robo_backend = robo_backend.clone();
                            move || {
                                let robo_backend = robo_backend.borrow();
                                let position = robo_backend.current_position();
                                let _ = mqtt_client.publish(
                                    "robo-vac",
                                    QoS::AtMostOnce,
                                    true,
                                    serde_json::to_string(&position).unwrap(),
                                );
                            }
                        },
                    );
                }
                2 => {
                    demo_update_timer.start(
                        slint::TimerMode::Repeated,
                        std::time::Duration::from_millis(40),
                        {
                            let industrial_backend = industrial_backend.clone();
                            move || {
                                industrial_backend.borrow_mut().task();
                            }
                        },
                    );
                    mqtt_update_timer.start(
                        slint::TimerMode::Repeated,
                        std::time::Duration::from_millis(500),
                        {
                            let mqtt_client = mqtt_client.clone();
                            let industrial_backend = industrial_backend.clone();
                            move || {
                                let industrial_backend = industrial_backend.borrow_mut();
                                let position = industrial_backend.current_position();

                                let _ = mqtt_client.publish(
                                    "industrial-box",
                                    QoS::AtMostOnce,
                                    true,
                                    serde_json::to_string(&position).unwrap(),
                                );
                            }
                        },
                    );
                }
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
                    },
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

    let _mqtt_thread = std::thread::spawn(move || {
        println!("MQTT thread started");

        while let Ok(event) = mqtt_eventloop.recv() {
            match event {
                Ok(event) => {
                    if let Event::Incoming(Incoming::Publish(_message)) = event {
                        println!("MQTT Incoming message");
                    }
                }
                Err(err) => {
                    println!("MQTT error: {}", err);
                    break;
                }
            }
        }
        println!("MQTT quit polling event loop");
    });

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
