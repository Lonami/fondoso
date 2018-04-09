extern crate rand;
extern crate image;
extern crate argparse;

use std::fs::File;
use std::str::FromStr;
use std::fmt::Display;
use std::process::exit;

use rand::{Rng, thread_rng};
use image::ImageBuffer;
use argparse::{ArgumentParser, StoreTrue, Store};

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

fn parse_or_exit<T>(what: &str, name: &str) -> T
    where
        T: FromStr,
        <T as FromStr>::Err: Display
{
    match what.parse::<T>() {
        Ok(x) => x,
        Err(e) => {
            eprintln!("Could not parse {} into a number ({})", name, e);
            exit(1);
        }
    }
}

fn main() {
    let mut verbose = false;
    let mut size = "500x500".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Create a new fondo.");
        ap.refer(&mut verbose)
            .add_option(&["-v", "--verbose"], StoreTrue,
            "Be verbose");
        ap.refer(&mut size)
            .add_option(&["-s", "--size"], Store,
            "Size for the generated image in WxH format");
        ap.parse_args_or_exit();
    }

    let size: Vec<&str> = size.split('x').collect();
    if size.len() != 2 {
        eprintln!("Incorrect size format (must be WxH)");
        exit(1);
    }
    let w: usize = parse_or_exit(size[0], "width");
    let h: usize = parse_or_exit(size[1], "height");

    let mut img = ImageBuffer::new(w as u32, h as u32);
    let mut added = vec![vec![false; w]; h];
    let mut pending = Vec::new();

    let start_points = vec![(w / 2, h / 2)];
    for (x, y) in start_points {
        pending.push((0u8, 0u8, 0u8, w / 2, h / 2));
        added[y][x] = true;
    }

    let total = w * h;
    let mut done = 0;
    while !pending.is_empty() {
        if verbose && done % 10_000 == 0 {
            println!("{:.2}%", 100.0 * (done as f64 / total as f64));
        }

        let which = thread_rng().gen_range(0, pending.len());
        let (r, g, b, x, y) = pending.remove(which);

        let r = offset(r);
        let g = offset(g);
        let b = offset(b);

        img.put_pixel(x as u32, y as u32, image::Rgb([r, g, b]));
        done += 1;
        for &(x, y) in neighbours(x, y, w, h).iter() {
            if !added[y][x] {
                pending.push((r, g, b, x, y));
                added[y][x] = true; // Moving this outside makes it more sparse
            }
        }
    }

    if verbose {
        println!("100.00%. Saving...");
    }
    let ref mut fp = File::create("ass.png").unwrap();
    image::ImageRgb8(img).save(fp, image::PNG).unwrap();
    if verbose {
        println!("Done.");
    }
}
