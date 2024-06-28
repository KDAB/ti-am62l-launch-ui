use std::rc::Rc;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let ui = Rc::new(AppWindow::new().unwrap());

    let mut sys = sysinfo::System::new_with_specifics(
        sysinfo::RefreshKind::new().with_cpu(sysinfo::CpuRefreshKind::everything()),
    );
    let sysinfo_sys_timer = slint::Timer::default();
    let ui_handle = Rc::clone(&ui);
    sysinfo_sys_timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(1000),
        move || {
            let hw_info = ui_handle.global::<HwInfo>();

            sys.refresh_cpu();
            let cpus = sys.cpus();

            let cpu_load_total =
                cpus.iter().map(sysinfo::Cpu::cpu_usage).sum::<f32>() / cpus.len() as f32;
            let cpu_load_cpu_1 = cpus.get(0).unwrap().cpu_usage();
            let cpu_load_cpu_2 = cpus.get(1).unwrap().cpu_usage();
            hw_info.set_cpu_load_total(cpu_load_total as i32);
            hw_info.set_cpu_load_cpu_1(cpu_load_cpu_1 as i32);
            hw_info.set_cpu_load_cpu_2(cpu_load_cpu_2 as i32);

            let cpu_frequency_cpu_1 = cpus.get(0).unwrap().frequency();
            let cpu_frequency_cpu_2 = cpus.get(1).unwrap().frequency();
            hw_info.set_cpu_freq_cpu_1(cpu_frequency_cpu_1 as i32);
            hw_info.set_cpu_freq_cpu_2(cpu_frequency_cpu_2 as i32);

            sys.refresh_memory_specifics(sysinfo::MemoryRefreshKind::everything().without_swap());
            let ram_load = sys.used_memory() / sys.total_memory();
            let ram_used_mb = sys.used_memory() / 1000000;
            hw_info.set_ram_load(ram_load as i32);
            hw_info.set_ram_used(ram_used_mb as i32);
        },
    );

    let mut comps = sysinfo::Components::new_with_refreshed_list();
    let sysinfo_comps_timer = slint::Timer::default();
    let ui_handle = Rc::clone(&ui);
    sysinfo_comps_timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(2500),
        move || {
            let hw_info = ui_handle.global::<HwInfo>();

            // TODO -> no components available on BeaglePlay, /sys/class/hwmon missing
            // comps.refresh_list();
            // for comp in &comps {
            //     println!("{} curr {}°C, crit {:?}°C", comp.label(), comp.temperature(), comp.critical());
            // }
        },
    );

    let mut sys_info_string = "SYSTEM INFO\n===========\n\n".to_string();

    sys_info_string.push_str("System\n");
    sys_info_string.push_str(&format!("- Name: {}\n", sysinfo::System::name().unwrap()));
    sys_info_string.push_str(&format!(
        "- Kernel Version: {}\n",
        sysinfo::System::kernel_version().unwrap()
    ));
    sys_info_string.push_str(&format!(
        "- OS Version: {}\n",
        sysinfo::System::long_os_version().unwrap()
    ));
    sys_info_string.push_str(&format!(
        "- Host Name: {}\n",
        sysinfo::System::host_name().unwrap()
    ));
    sys_info_string.push_str("\n");

    let networks = sysinfo::Networks::new_with_refreshed_list();
    sys_info_string.push_str("Networks\n");
    for (interface_name, network) in &networks {
        sys_info_string.push_str(&format!(
            "- {}: {}\n",
            interface_name,
            network.mac_address()
        ));
    }
    let hw_info = ui.global::<HwInfo>();
    hw_info.set_sys_info(slint::SharedString::from(sys_info_string));

    ui.run()
}
