extern crate rand;
extern crate image;

use std::fs::File;

use rand::{Rng, thread_rng};
use image::ImageBuffer;

fn offset(value: u8) -> u8 {
    let mut random: i32 = 0;
    while random == 0 {
        random = thread_rng().gen_range(-4, 4+1);
    }
    match value as i32 + random {
        x if x < 0   => 0,
        x if x > 255 => 255,
        x => x as u8
    }
}

fn neighbours(x: usize, y: usize, w: usize, h: usize)
    -> Vec<(usize, usize)>
{
    let mut result = Vec::new();

    let (x, y, w, h) = (x as i32, y as i32, w as i32, h as i32);
    let offsets = [
        (-1, -1), (-1, 0), (-1, 1),
        ( 0, -1),          ( 0, 1),
        ( 1, -1), ( 1, 0), ( 1, 1)
    ];
    for &(dx, dy) in offsets.iter() {
        let (nx, ny) = (x + dx, y + dy);
        if nx >= 0 && nx < w && ny >= 0 && ny < h {
            result.push((nx as usize, ny as usize));
        }
    }
    result
}

fn main() {
    const W: usize = 500;
    const H: usize = 500;
    let mut img = ImageBuffer::new(W as u32, H as u32);
    let mut added = [[false; W]; H];
    let mut pending = Vec::new();

    let start_points = vec![(W / 2, H / 2)];
    for (x, y) in start_points {
        pending.push((0u8, 0u8, 0u8, W / 2, H / 2));
        added[y][x] = true;
    }

    let total = W * H;
    let mut done = 0;
    while !pending.is_empty() {
        if done % 10_000 == 0 {
            println!("{:.2}%", 100.0 * (done as f64 / total as f64));
        }

        let which = thread_rng().gen_range(0, pending.len());
        let (r, g, b, x, y) = pending.remove(which);

        let r = offset(r);
        let g = offset(g);
        let b = offset(b);

        img.put_pixel(x as u32, y as u32, image::Rgb([r, g, b]));
        done += 1;
        for &(x, y) in neighbours(x, y, W, H).iter() {
            if !added[y][x] {
                pending.push((r, g, b, x, y));
                added[y][x] = true; // Moving this outside makes it more sparse
            }
        }
    }

    println!("100.00%. Saving...");
    let ref mut fp = File::create("ass.png").unwrap();
    image::ImageRgb8(img).save(fp, image::PNG).unwrap();
    println!("Done.");
}
