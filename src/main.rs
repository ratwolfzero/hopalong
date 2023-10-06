use image::{ImageFormat, RgbImage};
use minifb::{Key, Window, WindowOptions};
use rayon::prelude::*;
use std::env;

const WIDTH: usize = 1000;
const HEIGHT: usize = 1000;

const NUMCOL: usize = COLOR_PALETTE.len();

const COLOR_PALETTE: [(u8, u8, u8, &str); 32] = [
    (0, 0, 0, "Black"),
    (255, 0, 0, "Red"),
    (0, 255, 0, "Green"),
    (0, 0, 255, "Blue"),
    (255, 255, 0, "Yellow"),
    (255, 0, 255, "Magenta"),
    (0, 255, 255, "Cyan"),
    (128, 0, 0, "Maroon"),
    (0, 128, 0, "Lime"),
    (0, 0, 128, "Navy"),
    (255, 165, 0, "Orange"),
    (128, 0, 128, "Purple"),
    (0, 128, 128, "Teal"),
    (192, 192, 192, "Silver"),
    (128, 128, 128, "Gray"),
    (255, 255, 255, "White"),
    (255, 69, 0, "Red-Orange"),
    (218, 165, 32, "Goldenrod"),
    (173, 216, 230, "Light Blue"),
    (255, 20, 147, "Deep Pink"),
    (255, 105, 180, "Pink"),
    (255, 215, 0, "Gold"),
    (0, 139, 139, "Dark Cyan"),
    (205, 133, 63, "Peru"),
    (218, 112, 214, "Orchid"),
    (210, 105, 30, "Chocolate"),
    (60, 179, 113, "Medium Sea Green"),
    (244, 164, 96, "Sandy Brown"),
    (138, 43, 226, "Blue-Violet"),
    (255, 192, 203, "Light Pink"),
    (0, 255, 127, "Spring Green"),
    (128, 0, 0, "Brown"),
];

fn hopalong(num: usize, a: f64, b: f64, c: f64, buffer: &mut [u32], scale_factor: f64) {
    let mut x = 0.0;
    let mut y = 0.0;

    let (min_x, max_x, min_y, max_y) = calculate_bounds(num, a, b, c);
    let x_scale = (WIDTH as f64 / (max_x - min_x)) * scale_factor;
    let y_scale = (HEIGHT as f64 / (max_y - min_y)) * scale_factor;
    let x_offset = (WIDTH as f64 - (max_x - min_x) * x_scale) / 2.0;
    let y_offset = (HEIGHT as f64 - (max_y - min_y) * y_scale) / 2.0;

    // Create a 2D array to keep track of pixel density
    let mut pixel_density: Vec<Vec<usize>> = vec![vec![0; WIDTH]; HEIGHT];

    for _ in 0..num {
        let xx = y - f64::signum(x) * f64::sqrt((b * x - c).abs());
        let yy = a - x;
        x = xx;
        y = yy;

        let px = ((x - min_x) * x_scale + x_offset) as usize;
        let py = ((y - min_y) * y_scale + y_offset) as usize;

        if px < WIDTH && py < HEIGHT {
            // Increment the pixel density for the current pixel
            pixel_density[py][px] += 1;
        }
    }

    // Calculate the maximum density
    let max_density = pixel_density
        .par_iter()
        .flatten()
        .cloned()
        .max()
        .unwrap_or(1);

    // Color the pixels based on density
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let density = pixel_density[y][x];
            let color_index =
                ((density as f64 / max_density as f64) * NUMCOL as f64) as usize % NUMCOL;
            let (r, g, b, _color_name) = COLOR_PALETTE[color_index];
            buffer[y * WIDTH + x] =
                0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
        }
    }
}

fn calculate_bounds(num: usize, a: f64, b: f64, c: f64) -> (f64, f64, f64, f64) {
    // Calculate minimum and maximum x and y values
    let mut x = 0.0;
    let mut y = 0.0;
    let mut min_x: f64 = 0.0;
    let mut max_x: f64 = 0.0;
    let mut min_y: f64 = 0.0;
    let mut max_y: f64 = 0.0;

    for _ in 0..num {
        let xx = y - f64::signum(x) * f64::sqrt((b * x - c).abs());
        let yy = a - x;
        x = xx;
        y = yy;

        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
    }

    (min_x, max_x, max_y, min_y) //to have the same orientation as in Julia and Python
                                 //(min_x, max_x, min_y, max_y)
}

fn main() {
    //let num = 10000000;
    //let a = 2.0;
    //let b = 0.05;
    //let c = 2.0;
    // Parse command-line arguments

    let args: Vec<String> = env::args().collect();
    if args.len() != 5 {
        eprintln!("Usage: {} <a> <b> <c> <num>", args[0]);
        return;
    }

    //let num = args[1].parse::<usize>().unwrap(); //unwrap_or(default);

    let a = args[1].parse::<f64>().unwrap();
    let b = args[2].parse::<f64>().unwrap();
    let c = args[3].parse::<f64>().unwrap();

    let num_str = &args[4];
    let num: usize = if let Ok(parsed) = num_str.parse::<usize>() {
        parsed
    } else if let Ok(parsed_float) = num_str.parse::<f64>() {
        parsed_float as usize
    } else {
        eprintln!("Invalid value for 'num'");
        return;
    };

    let scale_factor = 0.95;

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    hopalong(num, a, b, c, &mut buffer, scale_factor);

    // Generate a unique image file name based on the parameters
    let image_name = format!("num={}_a={}_b={}_c={}.png", num_str, a, b, c); // Customize the format as needed

    // Create an image from the buffer
    let mut image_buffer = RgbImage::new(WIDTH as u32, HEIGHT as u32);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let color = buffer[y * WIDTH + x];
            let r = ((color >> 16) & 0xFF) as u8;
            let g = ((color >> 8) & 0xFF) as u8;
            let b = (color & 0xFF) as u8;
            image_buffer.put_pixel(x as u32, y as u32, image::Rgb([r, g, b]));
        }
    }

    // Save the image with the generated name
    let save_path = format!("/Users/ralf//Projects/hopalong_pictures/{}", image_name); // Specify your desired save path
    if let Err(e) = image_buffer.save_with_format(&save_path, ImageFormat::Png) {
        eprintln!("Error saving image: {}", e);
    } else {
        println!("Image saved to: {}", save_path);
    }

    // Display the image in a window (optional)
    let title = format!(
        "Hopalong - num  = {}  , a = {} , b = {} , c = {}",
        num_str, a, b, c
    );
    let mut window = Window::new(
        &title,
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: false,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
