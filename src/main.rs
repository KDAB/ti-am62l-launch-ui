slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let ui = std::rc::Rc::new(AppWindow::new().unwrap());

    let mut sys = sysinfo::System::new_with_specifics(
        sysinfo::RefreshKind::new().with_cpu(sysinfo::CpuRefreshKind::everything().without_frequency()),
    );
    let timer = slint::Timer::default();
    let ui_handle = std::rc::Rc::clone(&ui);
    timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(1000),
        move || {
            sys.refresh_cpu();
            let cpu_load_total = sys.cpus().iter().map(sysinfo::Cpu::cpu_usage).sum::<f32>() / sys.cpus().len() as f32;
            sys.refresh_memory_specifics(sysinfo::MemoryRefreshKind::everything().without_swap());
            let ram_load = sys.used_memory() as f32 / sys.total_memory() as f32;

            let system_resources_data = SystemResourcesData {
                cpu_load_total: cpu_load_total as i32,
                ram_load: (ram_load * 100.0) as i32,
            };

            let demo_interface = ui_handle.global::<DemoInterface>();
            demo_interface.set_system_resources_data(system_resources_data);
        },
    );

    ui.run()
}
