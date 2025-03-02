use super::{register_singleton_callback, AppWindow, RoboInterface};
use crate::ZoomFactor;
use geo::LineString;
use image::ImageBuffer;
use image::Rgba;
use serde::{Deserialize, Serialize};
use slint::ComponentHandle;

const FLOOR_PLAN_IMAGE_HEIGHT: u32 = 1040;
const FLOOR_PLAN_IMAGE_WIDTH: u32 = 1180;
const FLOOR_CIRCLE_RADIUS: u32 = 10;

pub const AGNLE_STEP: f64 = 1.8;
pub const NUMBER_OF_STEPS: usize = 3700;

pub const ROBO_PATH: [[f64; 2]; 30] = [
    [785.0, 355.0],
    [640.0, 355.0],
    [640.0, 370.0],
    [400.0, 370.0],
    [400.0, 400.0],
    [745.0, 400.0],
    [745.0, 430.0],
    [400.0, 430.0],
    [400.0, 460.0],
    [745.0, 460.0],
    [745.0, 490.0],
    [400.0, 490.0],
    [400.0, 520.0],
    [745.0, 520.0],
    [745.0, 550.0],
    [575.0, 550.0],
    [575.0, 590.0],
    [400.0, 590.0],
    [400.0, 620.0],
    [1130.0, 620.0],
    [1130.0, 650.0],
    [400.0, 650.0],
    [400.0, 680.0],
    [785.0, 680.0],
    [785.0, 710.0],
    [400.0, 710.0],
    [400.0, 740.0],
    [785.0, 740.0],
    [785.0, 770.0],
    [400.0, 770.0],
];

pub fn robo_path_iter() -> std::iter::Enumerate<std::vec::IntoIter<geo::LineString>> {
    let robo_path: LineString = ROBO_PATH.into_iter().collect();
    let robo_path_iter = geo::LineStringSegmentize::line_segmentize(&robo_path, NUMBER_OF_STEPS)
        .unwrap()
        .into_iter()
        .enumerate();
    robo_path_iter
}

#[derive(Debug, Copy, Clone)]
pub struct RoboPos {
    pub angle: f64,
    pub x1: u32,
    pub y1: u32,
    pub x2: u32,
    pub y2: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Position {
    pub x: u32,
    pub y: u32,
    pub direction: f64,
}
struct FloorImage {
    width: u32,
    height: u32,
    pixel_buffer: slint::SharedPixelBuffer<slint::Rgba8Pixel>,
    scale_factor: f32,
    circle_radius: i32,
}

pub struct RoboBackend {
    floor_image: FloorImage,
    ui_handle: slint::Weak<AppWindow>,
    pub pos_buffer: Vec<RoboPos>,
    pos_index: usize,
}

impl RoboBackend {
    pub fn new(ui: &AppWindow) -> std::rc::Rc<std::cell::RefCell<Self>> {
        let backend = Self {
            floor_image: Self::create_floor_image(&ui),
            ui_handle: ui.as_weak(),
            pos_buffer: Self::build_position_buffer(),
            pos_index: 0,
        };

        let backend = std::rc::Rc::new(std::cell::RefCell::new(backend));
        register_singleton_callback!(ui::RoboInterface::on_robo_zoom_factor_requested => backend::change_zoom_factor());
        backend
    }

    fn create_floor_image(ui: &AppWindow) -> FloorImage {
        let scale_factor = match ui.global::<RoboInterface>().get_robo_zoom_factor() {
            ZoomFactor::Zoom1x => 0.5f32,
            ZoomFactor::Zoom2x => 1f32,
        };

        let width = (FLOOR_PLAN_IMAGE_WIDTH as f32 * scale_factor) as u32;
        let height = (FLOOR_PLAN_IMAGE_HEIGHT as f32 * scale_factor) as u32;
        let circle_radius = (FLOOR_CIRCLE_RADIUS as f32 * scale_factor) as i32;
        FloorImage {
            width,
            height,
            pixel_buffer: slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(width, height),
            scale_factor,
            circle_radius,
        }
    }

    fn build_position_buffer() -> Vec<RoboPos> {
        let mut pos_buffer = Vec::new();
        let mut previous_angle = 0f64;

        robo_path_iter().for_each(|(index, line_string)| {
            // determine angle of LineString
            let angle = line_string_angle_degrees(&line_string).unwrap();

            // determine tip and tail of LineString
            let line_string_coord_first = line_string.coords().nth(0).unwrap();
            let x_tail = line_string_coord_first.x as u32;
            let y_tail = line_string_coord_first.y as u32;
            let line_string_coord_last = line_string.coords().last().unwrap();
            let x_tip = line_string_coord_last.x as u32;
            let y_tip = line_string_coord_last.y as u32;

            if index != 0 && angle != previous_angle {
                let mut a_angle = previous_angle;
                let diff_angle_cw = (angle - previous_angle).rem_euclid(360f64);
                let diff_angle_acw = (previous_angle - angle).rem_euclid(360f64);

                if diff_angle_acw > diff_angle_cw {
                    while a_angle < angle {
                        a_angle += AGNLE_STEP;

                        // println!("    CW  ROT --- push index: {} angle {}", index, a_angle);

                        pos_buffer.push(RoboPos {
                            angle: a_angle,
                            x1: x_tip,
                            y1: y_tip,
                            x2: x_tail,
                            y2: y_tail,
                        });
                    }
                } else {
                    while a_angle > angle {
                        a_angle -= AGNLE_STEP;

                        // println!("    ACW ROT --- push index: {} angle {}", index, a_angle);

                        pos_buffer.push(RoboPos {
                            angle: a_angle,
                            x1: x_tip,
                            y1: y_tip,
                            x2: x_tail,
                            y2: y_tail,
                        });
                    }
                }
            }

            pos_buffer.push(RoboPos {
                angle: angle,
                x1: x_tip,
                y1: y_tip,
                x2: x_tail,
                y2: y_tail,
            });

            previous_angle = angle;
        });
        pos_buffer
    }

    pub fn task(&mut self) {
        let ui = self.ui_handle.unwrap();
        let demo_interface = ui.global::<RoboInterface>();

        // let zoom_factor = demo_interface.get_robo_zoom_factor();
        let robo_pos = self.robo_demo_render();
        let image = slint::Image::from_rgba8(self.floor_image.pixel_buffer.clone());

        demo_interface.set_robo_angle(robo_pos.angle as f32 * -1.0 - 90.0);
        demo_interface.set_robo_pos_x(robo_pos.x1 as f32 * self.floor_image.scale_factor);
        demo_interface.set_robo_pos_y(robo_pos.y1 as f32 * self.floor_image.scale_factor);
        demo_interface.set_robo_floor_image(image);
    }

    pub fn current_position(&self) -> Position {
        let pos = self.pos_buffer[self.pos_index % self.pos_buffer.len()];
        Position {
            x: pos.x1,
            y: pos.y1,
            direction: pos.angle,
        }
    }

    fn change_zoom_factor(&mut self) {
        let ui = self.ui_handle.unwrap();
        let demo_interface = ui.global::<RoboInterface>();

        let old_value = demo_interface.get_robo_zoom_factor();
        let new_value = match old_value {
            ZoomFactor::Zoom1x => ZoomFactor::Zoom2x,
            ZoomFactor::Zoom2x => ZoomFactor::Zoom1x,
        };
        demo_interface.set_robo_zoom_factor(new_value);
        self.floor_image = Self::create_floor_image(&ui);
    }

    fn robo_demo_render(&mut self) -> RoboPos {
        //let (index, line_string) = iter.next().unwrap();

        // reset buffer in case we reached the end of the robo path
        if (self.pos_index % self.pos_buffer.len()) == 0 {
            let ui = self.ui_handle.unwrap();
            self.floor_image = Self::create_floor_image(&ui);
            self.pos_index = 0;
        }

        let pos = self.pos_buffer[self.pos_index];

        // convert the image_buffer to an image
        let image_buffer_as_image = image::ImageBuffer::from_raw(
            self.floor_image.width,
            self.floor_image.height,
            self.floor_image.pixel_buffer.make_mut_bytes(),
        );
        let mut image_buffer_as_image: ImageBuffer<Rgba<u8>, &mut [u8]> =
            image_buffer_as_image.unwrap();

        // draw line along LineString
        let scale_factor = self.floor_image.scale_factor;
        let start = (pos.x1 as f32 * scale_factor, pos.y1 as f32 * scale_factor);
        let end = (pos.x2 as f32 * scale_factor, pos.y2 as f32 * scale_factor);
        for (x, y) in imageproc::drawing::BresenhamLineIter::new(start, end) {
            imageproc::drawing::draw_filled_circle_mut(
                &mut image_buffer_as_image,
                (x, y),
                self.floor_image.circle_radius,
                image::Rgba([255, 255, 255, 255]),
            );
        }

        // println!(
        //     "Step = {}, A = {}, C1 = ({},{}), C2 = ({},{})",
        //     self.pos_index, pos.angle, pos.x1, pos.y1, pos.x2, pos.y2
        // );

        self.pos_index += 1;

        pos
    }
}

fn line_string_angle_degrees(line_string: &LineString) -> Option<f64> {
    let line = geo::Line::new(*line_string.coords().nth(0)?, *line_string.coords().last()?);
    let angle = line.dy().atan2(line.dx()).to_degrees();
    Some(angle)
}
