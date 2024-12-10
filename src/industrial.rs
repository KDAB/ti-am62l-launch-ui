use super::{AppWindow, IndustrialInterface};
use image::{imageops, EncodableLayout, ImageBuffer, Rgba};
use slint::ComponentHandle;

pub struct IndustrialDemoAssets { // TODO -> rename to img buffer ...
    // text_font: rusttype::Font, // TODO -> check with Leon
    image_box_background: ImageBuffer<Rgba<u8>, Vec<u8>>,
    image_box_fill: ImageBuffer<Rgba<u8>, Vec<u8>>,
    image_box_foreground: ImageBuffer<Rgba<u8>, Vec<u8>>,
    image_box_white: ImageBuffer<Rgba<u8>, Vec<u8>>,
}

impl IndustrialDemoAssets {
    pub fn new() -> IndustrialDemoAssets {
        Self {
            // text_font: {
            //     let bytes = include_bytes!("../ui/resources/fonts/OpenSans-VariableFont.ttf");
            //     rusttype::Font::try_from_bytes(bytes).unwrap()
            // },
            image_box_background: {
                let bytes = include_bytes!("../ui/resources/demos/industrial/box-translucent.png");
                image::load_from_memory(bytes).unwrap().into_rgba8()
            },
            image_box_fill: {
                let bytes = include_bytes!("../ui/resources/demos/industrial/box-fill.png");
                image::load_from_memory(bytes).unwrap().into_rgba8()
            },
            image_box_foreground: {
                let bytes = include_bytes!("../ui/resources/demos/industrial/box-edges-front.png");
                image::load_from_memory(bytes).unwrap().into_rgba8()
            },
            image_box_white: {
                let bytes = include_bytes!("../ui/resources/demos/industrial/box-white.png");
                image::load_from_memory(bytes).unwrap().into_rgba8()
            },
        }
    }
}

pub struct IndustrialDemoBackend {
    assets: IndustrialDemoAssets,
    // font = rusttype::Font::try_from_bytes(font_data).unwrap();
    animation_counter: i8,
    animation_progress_permill: u16,
    image_buffer: image::ImageBuffer<Rgba<u8>, Vec<u8>>,
    ui_handle: slint::Weak<AppWindow>,
}

impl IndustrialDemoBackend {
    pub fn new(
        ui: &AppWindow,
        assets: IndustrialDemoAssets,
    ) -> std::rc::Rc<std::cell::RefCell<Self>> {
        let this = std::rc::Rc::new(std::cell::RefCell::new(Self {
            assets: assets,
            animation_counter: -1,
            animation_progress_permill: 401,
            image_buffer: image::ImageBuffer::new(159,174),
            ui_handle: ui.as_weak(),
        }));

        this
    }

    pub fn task(&mut self) {
        let box_position_delta = self.animation_progress_permill as f32;

        if self.animation_progress_permill <= 200 {
            let alpha_factor_percent = self.animation_progress_permill as f32 / 200 as f32;
            render_box_translucent(
                &mut self.image_buffer,
                &self.assets.image_box_background,
                &self.assets.image_box_foreground,
                &self.assets.image_box_fill,
                alpha_factor_percent,
            );
        }

        if self.animation_progress_permill > 200 && self.animation_progress_permill <= 400 {
            let alpha_factor_percent = (self.animation_progress_permill - 200) as f32 / 200 as f32;
            render_box_white(
                &mut self.image_buffer,
                &self.assets.image_box_white,
                alpha_factor_percent,
            );
        }

        let pixel_buffer =
            slint::SharedPixelBuffer::clone_from_slice(self.image_buffer.as_raw(), 159, 174);
        let image = slint::Image::from_rgba8(pixel_buffer);
        
        let ui = self.ui_handle.unwrap();
        let industrial_demo_inteface = ui.global::<IndustrialInterface>();
        industrial_demo_inteface.set_box_image(image);
        industrial_demo_inteface.set_box_x(box_position_delta);
        industrial_demo_inteface.set_box_y(box_position_delta / 1.7);

        if self.animation_progress_permill < 400 {
            self.animation_progress_permill += 1;
        } else {
            self.animation_progress_permill = 0;

            if self.animation_counter < 99 {
                self.animation_counter += 1;
            } else {
                self.animation_counter = 0;
            }

            // TODO -> use fonts from assets
            let bytes = include_bytes!("../ui/resources/fonts/OpenSans-VariableFont.ttf");
            let text_font = rusttype::Font::try_from_bytes(bytes).unwrap();

            let text_image = render_text(
                text_font,
                format!("count: {}", self.animation_counter).as_str(),
            );
            industrial_demo_inteface.set_text_image(text_image);
        }
    }
}

pub fn render_box_translucent(
    image_buffer: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    image_buffer_box_background: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    image_buffer_box_foreground: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    image_buffer_box_fill: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    alpha_factor: f32,
) {
    *image_buffer = image_buffer_box_background.clone();

    let mut overlay = image_buffer_box_fill.clone();
    alpha(&mut overlay, alpha_factor);
    overlay_image(image_buffer, &overlay, 0, 5);

    let overlay = image_buffer_box_foreground;
    overlay_image(image_buffer, overlay, 32, 60);
}

pub fn render_box_white(
    image_buffer: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    image_buffer_box_white: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    alpha_factor: f32,
) {
    if alpha_factor < 1.0 {
        let mut overlay = image_buffer_box_white.clone();
        alpha(&mut overlay, alpha_factor);
        overlay_image(image_buffer, &overlay, 20, 20);
    } else {
        *image_buffer = ImageBuffer::new(159, 174);
        let overlay = image_buffer_box_white.clone();
        overlay_image(image_buffer, &overlay, 20, 20);
    }
}

pub fn render_text(font: rusttype::Font, text: &str) -> slint::Image {
    let mut image_buffer = image::RgbaImage::new(400, 200);
    let color = image::Rgba([0x23, 0x47, 0x70, 0xff]);
    let scale = rusttype::Scale::uniform(60.0);
    let x = 0 as f32;
    let y = image_buffer.height() as f32 / 2.0;

    //
    // TODO -> next: fix font rendering with fontdue or use rusttype
    // let font_data = include_bytes!("../ui/resources/fonts/OpenSans-VariableFont.ttf");
    // let font = fontdue::Font::from_bytes(font_data as &[u8], fontdue::FontSettings::default()).unwrap();
    // draw_text_fontdue(image_buffer.borrow_mut(), font.borrow(), 100.0, x, y, color, text);
    //
    draw_text_rusttype(&mut image_buffer, &font, scale, x, y, color, text);

    let mut shadow_buffer = image::RgbaImage::new(400, 200);
    let shadow_color = image::Rgba([0x4c, 0x84, 0xb3, 0x1b]);
    draw_text_rusttype(
        &mut shadow_buffer,
        &font,
        scale,
        x,
        y,
        shadow_color,
        text,
    );
    let shadow = shadow_buffer.clone();
    let shadow = imageops::fast_blur(&shadow, 1.0);
    // let shadow = imageops::unsharpen(shadow.borrow(), 10.0, 100);
    let shadow = imageops::flip_vertical(&shadow);
    imageops::overlay(&mut image_buffer, &shadow, x as i64, 20);

    // image_buffer = image::imageops::blur(image_buffer.borrow(), 2.0);
    // image_buffer = image::imageops::unsharpen(image_buffer.borrow(), 10.0, 100);
    // image_buffer = image::imageops::rotate90(image_buffer.borrow());
    // image::imageops::flip_horizontal(image_buffer.borrow_mut());

    // let angle_deg: f32 = -20.0;
    // let cos = angle_deg.to_radians().cos();
    // let sin = angle_deg.to_radians().sin();
    // let matrix = [
    //     cos, -sin, 0.0,
    //     sin, cos, 0.0,
    //     0.0012, 0.0015, 1.0,
    // ];
    let matrix = [0.6, 0.0, 0.0, 0.35, 1.0, 0.0, 0.0, 0.0, 1.0];
    // let matrix = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
    let projection = imageproc::geometric_transformations::Projection::from_matrix(matrix).unwrap();
    let image = imageproc::geometric_transformations::warp(
        &image_buffer,
        &projection,
        imageproc::geometric_transformations::Interpolation::Bilinear,
        image::Rgba([0x00, 0x00, 0x00, 0x00]),
    );
    // let mut image_shadow = imageops::fast_blur(image.borrow(), 0.1);
    // image_shadow = imageops::flip_vertical(image_shadow.borrow());
    let pixel_buffer =
        slint::SharedPixelBuffer::clone_from_slice(image.as_bytes(), image.width(), image.height());
    // let pixel_buffer = slint::SharedPixelBuffer::clone_from_slice(image_shadow.as_bytes(), image_shadow.width(), image_shadow.height());

    // let pixel_buffer =
    //     slint::SharedPixelBuffer::clone_from_slice(image_buffer.as_bytes(), 300, 100);
    slint::Image::from_rgba8(pixel_buffer)
}

fn alpha(image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, alpha_factor: f32) -> () {
    for pixel in image.pixels_mut() {
        let alpha = pixel[3] as f32;
        pixel[3] = (alpha * alpha_factor).min(255.0) as u8;
    }
}

fn overlay_image(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    overlay: &ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    offset_x: u32,
    offset_y: u32,
) -> () {
    // Loop through each pixel in the overlay image
    for y in 0..overlay.height() {
        for x in 0..overlay.width() {
            // Get the pixel from the overlay image
            let overlay_pixel = overlay.get_pixel(x, y);

            // Calculate the position on the background image
            let bg_x = offset_x + x;
            let bg_y = offset_y + y;

            // Skip if the position is out of bounds
            if bg_x >= image.width() || bg_y >= image.height() {
                continue;
            }

            // Blend the overlay pixel with the background pixel
            let background_pixel = image.get_pixel(bg_x, bg_y);
            let blended_pixel = blend_pixels(&background_pixel, &overlay_pixel);

            // Set the blended pixel on the background image
            image.put_pixel(bg_x, bg_y, blended_pixel);
        }
    }
}

// Blend two RGBA pixels with alpha compositing
fn blend_pixels(bg_pixel: &Rgba<u8>, overlay_pixel: &Rgba<u8>) -> Rgba<u8> {
    let bg_alpha = bg_pixel[3] as f32 / 255.0;
    let overlay_alpha = overlay_pixel[3] as f32 / 255.0;
    let out_alpha = overlay_alpha + bg_alpha * (1.0 - overlay_alpha);

    let blend = |bg: u8, over: u8| {
        ((overlay_alpha * over as f32 + bg_alpha * bg as f32 * (1.0 - overlay_alpha)) / out_alpha)
            as u8
    };

    Rgba([
        blend(bg_pixel[0], overlay_pixel[0]),
        blend(bg_pixel[1], overlay_pixel[1]),
        blend(bg_pixel[2], overlay_pixel[2]),
        (out_alpha * 255.0) as u8,
    ])
}

// TODO -> are there good reasons to use fontdue instad of fonttype?
// fn draw_text_fontdue(
//     img: &mut image::RgbaImage,
//     font: &fontdue::Font,
//     font_size: f32,
//     x: f32,
//     y: f32,
//     color: image::Rgba<u8>,
//     text: &str,
// ) {
//     let (metrics, bitmap) = font.rasterize('S', font_size);

//     for (i, &pixel) in bitmap.iter().enumerate() {
//         let gx = i % metrics.width;
//         let gy = i / metrics.width;
//         let px = x as i32 + gx as i32;
//         let py = y as i32 + gy as i32;

//         if px >= 0 && py >= 0 && (px as u32) < img.width() && (py as u32) < img.height() {
//             let pixel_intensity = (pixel as f32 / 255.0 * color[0] as f32) as u8;
//             img.put_pixel(px as u32, py as u32, Rgba([pixel_intensity, pixel_intensity, pixel_intensity, pixel_intensity]));
//         }
//     }
// }

fn draw_text_rusttype(
    img: &mut image::RgbaImage,
    font: &rusttype::Font,
    scale: rusttype::Scale,
    x: f32,
    y: f32,
    color: image::Rgba<u8>,
    text: &str,
) {
    for glyph in font.layout(text, scale, rusttype::point(x, y)) {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|gx, gy, v| {
                let px = bounding_box.min.x + gx as i32;
                let py = bounding_box.min.y + gy as i32;
                if px >= 0 && py >= 0 && px < img.width() as i32 && py < img.height() as i32 {
                    let pixel = img.get_pixel_mut(px as u32, py as u32);
                    let alpha = (v * color[3] as f32) as u8;
                    *pixel = image::Rgba([color[0], color[1], color[2], alpha]);
                }
            });
        }
    }
}
