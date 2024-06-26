slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    let ui_handle = ui.as_weak();

    slint::Timer::single_shot(std::time::Duration::from_secs(5), move || {
        ui_handle.unwrap().set_rect_x(0.0);
    });

    ui.run()
}
