use super::{AppWindow, RoboInterface, register_singleton_callback};
use crate::ZoomFactor;
use geo::LineString;
use image::imageops;
use image::ImageBuffer;
use image::Rgba;
use slint::{ComponentHandle, Rgba8Pixel};

const FLOOR_PLAN_IMAGE_HEIGHT: u32 = 1040;
const FLOOR_PLAN_IMAGE_WIDTH: u32 = 1180;

pub const NUMBER_OF_STEPS: usize = 120;

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

pub fn robo_path_iter() -> std::iter::Enumerate<std::iter::Cycle<std::vec::IntoIter<geo::LineString>>> {
    let robo_path: LineString = ROBO_PATH.into_iter().collect();
    let robo_path_iter = geo::LineStringSegmentize::line_segmentize(
        &robo_path,
        NUMBER_OF_STEPS,
    )
    .unwrap()
    .into_iter()
    .cycle()
    .enumerate();
    robo_path_iter
}

pub struct RoboPos {
    pub angle: f64,
    pub pos_x: u32,
    pub pos_y: u32,
}

pub struct RoboBackend {
    pixel_buffer: slint::SharedPixelBuffer::<slint::Rgba8Pixel>,
    robo_path_iter: std::iter::Enumerate<std::iter::Cycle<std::vec::IntoIter<geo::LineString>>>,
    ui_handle: slint::Weak<AppWindow>,
}

impl RoboBackend {
    pub fn new(ui: &AppWindow, robo_path_iter: std::iter::Enumerate<std::iter::Cycle<std::vec::IntoIter<geo::LineString>>>) -> std::rc::Rc<std::cell::RefCell<Self>> {
        let this = std::rc::Rc::new(std::cell::RefCell::new(Self {
            pixel_buffer: slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(1000, 880),
            robo_path_iter: robo_path_iter,
            ui_handle: ui.as_weak(),
        }));

        register_singleton_callback!(ui::RoboInterface::on_robo_zoom_factor_requested => this::change_zoom_factor());

        this
    }

    pub fn task(&mut self) {    
        let ui = self.ui_handle.unwrap();
        let demo_interface = ui.global::<RoboInterface>();

        let zoom_factor = demo_interface.get_robo_zoom_factor();
        let mut robo_pos = robo_demo_render(
            &mut self.robo_path_iter,
            &mut self.pixel_buffer,
        );
        let image =
            create_img_from_buffer(&self.pixel_buffer, &mut robo_pos, zoom_factor);

        demo_interface.set_robo_angle(robo_pos.angle as f32 * -1.0 - 90.0);
        demo_interface.set_robo_pos_x(robo_pos.pos_x as f32);
        demo_interface.set_robo_pos_y(robo_pos.pos_y as f32);
        demo_interface.set_robo_floor_image(image);
    }

    fn with_mut_ui_data(&mut self, fun: impl Fn(&mut Self, RoboInterface<'_>)) {
        self.ui_handle
            .upgrade()
            .map(|ui| fun(self, ui.global::<RoboInterface>()));
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
        self.task(); // TODO -> this will also cause the robo to go to the next path increment, maybe split task function into two functions
    }
}

fn robo_demo_render(
    iter: &mut dyn Iterator<Item = (usize, LineString<f64>)>,
    image_buffer: &mut slint::SharedPixelBuffer<slint::Rgba8Pixel>,
) -> RoboPos {
    let (index, line_string) = iter.next().unwrap();

    // reset buffer in case we reached the end of the robo path
    if (index % NUMBER_OF_STEPS) == 0 {
        *image_buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(
            FLOOR_PLAN_IMAGE_WIDTH,
            FLOOR_PLAN_IMAGE_HEIGHT,
        );
    }

    // convert the image_buffer to an image
    let image_buffer_as_bytes = image_buffer.make_mut_bytes();
    let image_buffer_as_image = image::ImageBuffer::from_raw(
        FLOOR_PLAN_IMAGE_WIDTH,
        FLOOR_PLAN_IMAGE_HEIGHT,
        image_buffer_as_bytes,
    );
    let mut image_buffer_as_image = image_buffer_as_image.unwrap();

    // determine angle of LineString
    let angle = line_string_angle_degrees(&line_string).unwrap();

    // determine tip and tail of LineString
    let line_string_coord_first = line_string.coords().nth(0).unwrap();
    let x_tail = line_string_coord_first.x as u32;
    let y_tail = line_string_coord_first.y as u32;
    let line_string_coord_last = line_string.coords().last().unwrap();
    let x_tip = line_string_coord_last.x as u32;
    let y_tip = line_string_coord_last.y as u32;

    // draw line along LineString
    let start = (x_tail as f32, y_tail as f32);
    let end = (x_tip as f32, y_tip as f32);
    for (x, y) in imageproc::drawing::BresenhamLineIter::new(start, end) {
        imageproc::drawing::draw_filled_circle_mut(
            &mut image_buffer_as_image,
            (x, y),
            10,
            image::Rgba([255, 255, 255, 255]),
        );
    }

    println!(
        "Step = {}, A = {}, C1 = ({},{}), C2 = ({},{})",
        index, angle, x_tail, y_tail, x_tip, y_tip
    );

    RoboPos {
        angle: angle,
        pos_x: x_tip,
        pos_y: y_tip,
    }
}

fn create_img_from_buffer(
    image_buffer: &slint::SharedPixelBuffer<Rgba8Pixel>,
    robo_pos: &mut RoboPos,
    zoom_factor: ZoomFactor,
) -> slint::Image {
    if zoom_factor == ZoomFactor::Zoom1x {
        let image_buffer_as_bytes = image_buffer.as_bytes();
        let image_buffer_as_image = image::ImageBuffer::from_raw(
            FLOOR_PLAN_IMAGE_WIDTH,
            FLOOR_PLAN_IMAGE_HEIGHT,
            image_buffer_as_bytes,
        );
        let image_buffer_as_image: ImageBuffer<Rgba<u8>, &[u8]> = image_buffer_as_image.unwrap();

        let resized = imageops::resize(
            &image_buffer_as_image,
            FLOOR_PLAN_IMAGE_WIDTH / 2,
            FLOOR_PLAN_IMAGE_HEIGHT / 2,
            imageops::FilterType::Nearest,
        );

        // let subimage = imageops::crop_imm(&image_buffer_as_image, FLOOR_PLAN_IMAGE_WIDTH/4, FLOOR_PLAN_IMAGE_HEIGHT/4, FLOOR_PLAN_IMAGE_WIDTH/2, FLOOR_PLAN_IMAGE_HEIGHT/2);
        // let image = subimage.to_image();
        // let buffer = image.as_raw();
        let spb: slint::SharedPixelBuffer<Rgba8Pixel> = slint::SharedPixelBuffer::clone_from_slice(
            resized.as_raw(),
            FLOOR_PLAN_IMAGE_WIDTH / 2,
            FLOOR_PLAN_IMAGE_HEIGHT / 2,
        );
        // (slint::Image::from_rgba8(spb), angle, x_tip/2, y_tip/2)
        robo_pos.pos_x /= 2;
        robo_pos.pos_y /= 2;

        slint::Image::from_rgba8(spb)
    } else {
        slint::Image::from_rgba8(image_buffer.clone())
    }
}

fn line_string_angle_degrees(line_string: &LineString) -> Option<f64> {
    let line = geo::Line::new(*line_string.coords().nth(0)?, *line_string.coords().last()?);
    let angle = line.dy().atan2(line.dx()).to_degrees();
    Some(angle)
}
