use std::{cell::RefCell, collections::VecDeque, f64::consts::PI};

use super::{AppWindow, EnergyMonitorInterface};
use once_cell::sync::Lazy; // lazy_static
use plotters::prelude::*;
use rand::{rngs::ThreadRng, Rng};
use slint::ComponentHandle;
use std::time::SystemTime;

pub struct EnergyMonitorBackend {
    balance: EnergyMonitorData,
    consumption: EnergyMonitorData,
    production: EnergyMonitorData,
    rng: ThreadRng,
    ui_handle: slint::Weak<AppWindow>,
}

impl EnergyMonitorBackend {
    pub fn new(ui: &AppWindow) -> std::rc::Rc<RefCell<Self>> {
        let this = std::rc::Rc::new(RefCell::new(Self {
            consumption: EnergyMonitorData::default(),
            production: EnergyMonitorData::default(),
            balance: EnergyMonitorData::default(),
            rng: rand::thread_rng(),
            ui_handle: ui.as_weak(),
        }));

        this
    }

    pub fn task(&mut self) {
        generate_data(&mut self.rng, &mut self.consumption, &mut self.production, 24);

        // consumption_current += rng.gen_range(0.0..0.1);
        // consumption_today += consumption_current;
        // consumption_week += consumption_current;
        // consumption_month += consumption_current;

        // production_current += rng.gen_range(0.0..0.2);
        // production_today += production_current;
        // production_week += production_current;
        // production_month += production_current;

        self.balance.current = self.production.current - self.consumption.current;
        self.balance.today = self.production.today - self.consumption.today;
        self.balance.week = self.production.week - self.consumption.week;
        self.balance.month = self.production.month - self.consumption.month;

        let ui = self.ui_handle.unwrap();
        let energy_monitor_interface = ui.global::<EnergyMonitorInterface>();
        energy_monitor_interface.set_balance_current(format!("{:.} kWh", self.balance.current).into());
        energy_monitor_interface.set_balance_today(format!("{:.1} kWh", self.balance.today).into());
        energy_monitor_interface.set_balance_week(format!("{:.1} kWh", self.balance.week).into());
        energy_monitor_interface.set_balance_month(format!("{:.1} kWh", self.balance.month).into());
        energy_monitor_interface
            .set_consumption_current(format!("{:.} kWh", self.consumption.current).into());
        energy_monitor_interface
            .set_consumption_today(format!("{:.1} kWh", self.consumption.today).into());
        energy_monitor_interface
            .set_consumption_week(format!("{:.1} kWh", self.consumption.week).into());
        energy_monitor_interface
            .set_consumption_month(format!("{:.1} kWh", self.consumption.month).into());
        energy_monitor_interface
            .set_production_current(format!("{:.1} kWh", self.production.current).into());
        energy_monitor_interface
            .set_production_today(format!("{:.1} kWh", self.production.today).into());
        energy_monitor_interface.set_production_week(format!("{:.1} kWh", self.production.week).into());
        energy_monitor_interface
            .set_production_month(format!("{:.1} kWh", self.production.month).into());

        let graph =
            plot_data(self.production.current, self.consumption.current, self.balance.current, 24);
        energy_monitor_interface.set_graph(graph);
    }

    // TODO -> create this function (and the ui_handle struct member?) with another macro
    fn with_mut_ui_data(&mut self, fun: impl Fn(&mut Self, EnergyMonitorInterface<'_>)) {
        self.ui_handle
            .upgrade()
            .map(|ui| fun(self, ui.global::<EnergyMonitorInterface>()));
    }
}

#[derive(Clone, Default)]
pub struct EnergyMonitorData {
    pub current: f64,
    pub today: f64, // last 24h
    pub week: f64,
    pub month: f64,
}

pub fn generate_data(
    rng: &mut ThreadRng,
    consumption: &mut EnergyMonitorData,
    production: &mut EnergyMonitorData,
    cache_size: usize,
) {
    static mut CACHE_24H: Lazy<Vec<VecDeque<f64>>> = Lazy::new(|| {
        let vec = vec![VecDeque::new(); 2];
        vec
    });

    let seconds_since_epoch = seconds_since_epoch();
    if seconds_since_epoch.is_none() {
        return;
    }

    let seconds_since_epoch = seconds_since_epoch.unwrap();

    production.current =
        sine_gen(seconds_since_epoch, 0.6, 0.01, None, Some(0.6)) + rng.gen_range(0.0..0.2);
    consumption.current = sine_gen(seconds_since_epoch, 0.4, 0.01, Some(-PI / 2.5), Some(0.4))
        + rng.gen_range(0.0..0.1);

    unsafe {
        CACHE_24H[0].push_back(production.current);
        CACHE_24H[1].push_back(consumption.current);

        if CACHE_24H[0].len() > cache_size {
            CACHE_24H[0].pop_front();
        }

        if CACHE_24H[1].len() > cache_size {
            CACHE_24H[1].pop_front();
        }

        production.today = CACHE_24H[0].iter().sum();
        consumption.today = CACHE_24H[1].iter().sum();
    }

    // TODO -> kept it simple here
    consumption.week = consumption.today * 7.0;
    consumption.month = consumption.week * 4.0;

    production.week = production.today * 7.0;
    production.month = production.week * 4.0;
}

pub fn plot_data(
    production: f64,
    consumption: f64,
    balance: f64,
    cache_size: usize,
) -> slint::Image {
    const COLOR_PRODUCTION: RGBColor = RGBColor(0x00, 0xbc, 0x00);
    const COLOR_CONSUMPTION: RGBColor = RGBColor(0xbc, 0x00, 0x00);
    const COLOR_BALANCE: RGBColor = RGBColor(0x7f, 0x7f, 0x7f);
    const COLOR_BY_PLOT: [RGBColor; 3] = [COLOR_PRODUCTION, COLOR_CONSUMPTION, COLOR_BALANCE];

    static mut CACHE: Lazy<Vec<VecDeque<f64>>> = Lazy::new(|| {
        let vec = vec![VecDeque::new(); 2];
        vec
    });

    unsafe {
        CACHE[0].push_back(production);
        CACHE[1].push_back(consumption);
        // CACHE[2].push_back(balance);

        if CACHE[0].len() > cache_size {
            CACHE[0].pop_front();
        }

        if CACHE[1].len() > cache_size {
            CACHE[1].pop_front();
        }

        // if CACHE[2].len() > cache_size {
        //     CACHE[2].pop_front();
        // }
    }

    let mut pixel_buffer = slint::SharedPixelBuffer::new(580, 320);
    let size = (pixel_buffer.width(), pixel_buffer.height());

    let backend = BitMapBackend::with_buffer(pixel_buffer.make_mut_bytes(), size);
    let drawing_area = backend.into_drawing_area();

    drawing_area
        .fill(&RGBColor(0xfa, 0xfa, 0xfa))
        .expect("error filling drawing area");

    let mut chart = ChartBuilder::on(&drawing_area)
        .margin_top(20)
        .margin_bottom(20)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .build_cartesian_2d(0..(cache_size - 1), 0.0..2.0)
        .expect("error building chart context");

    chart
        .configure_mesh()
        .disable_x_mesh()
        .y_max_light_lines(0)
        .bold_line_style(BLACK.mix(0.3))
        .draw()
        .expect("error drawing chart");

    unsafe {
        for (idx, data) in (0..).zip(CACHE.iter()) {
            let color = COLOR_BY_PLOT[idx];
            chart
                .draw_series(
                    AreaSeries::new(
                        (0..).zip(data.iter()).map(|(x, y)| (x, *y)),
                        0.0,
                        color.mix(0.2),
                    )
                    .border_style(color),
                )
                .expect("error drawing series");
        }
    }

    drawing_area.present().expect("error presenting plot");
    drop(chart);
    drop(drawing_area);

    slint::Image::from_rgb8(pixel_buffer)
}

fn seconds_since_epoch() -> Option<f64> {
    let time = SystemTime::now();
    let time_since_epoch = time.duration_since(SystemTime::UNIX_EPOCH);
    let result = (|| {
        if time_since_epoch.is_ok() {
            return Some(time_since_epoch.unwrap().as_secs_f64());
        }
        None
    })();
    result
}

fn sine_gen(
    seconds_since_epoch: f64,
    amplitude: f64,
    frequency: f64,
    x_offset: Option<f64>,
    y_offset: Option<f64>,
) -> f64 {
    let x_offset = x_offset.unwrap_or(0.0);
    let y_offset = y_offset.unwrap_or(0.0);
    let sine = amplitude
        * (frequency * 2.0 * std::f64::consts::PI * seconds_since_epoch + x_offset).sin()
        + y_offset;
    sine
}
