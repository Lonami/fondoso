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

#[derive(Debug)]
struct Point {
    x: usize,
    y: usize,
    r: u8,
    g: u8,
    b: u8,
}

const VALUE_SEPARATOR: char = ',';
const LIST_SEPARATOR: char = ':';

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

fn parse_points(w: usize, h: usize,
                number: usize, positions: &str, colours: &str)
    -> Vec<Point>
{
    let mut positions: Vec<(usize, usize)> = if positions.is_empty() {
        Vec::new()
    } else {
        positions.split(LIST_SEPARATOR).map(|point| {
            let point: Vec<&str> = point.split(VALUE_SEPARATOR).collect();
            if point.len() != 2 {
                eprintln!("Incorrect point format (must be x{}y)",
                          VALUE_SEPARATOR);
                exit(1);
            }
            let x: usize = parse_or_exit(point[0], "x coordinate");
            let y: usize = parse_or_exit(point[1], "y coordinate");
            (x, y)
        }).collect()
    };

    let mut colours: Vec<(u8, u8, u8)> = if colours.is_empty() {
        Vec::new()
    } else {
            colours.split(LIST_SEPARATOR).map(|point| {
            let colour: Vec<&str> = point.split(VALUE_SEPARATOR).collect();
            if colour.len() != 3 {
                eprintln!("Incorrect colour format (must be r{0}g{0}b)",
                          VALUE_SEPARATOR);
                exit(1);
            }
            let r: u8 = parse_or_exit(colour[0], "red channel");
            let g: u8 = parse_or_exit(colour[1], "green channel");
            let b: u8 = parse_or_exit(colour[2], "blue channel");
            (r, g, b)
        }).collect()
    };

    if number == 0 && positions.is_empty() {
        positions.push((w / 2, h / 2));
    } else {
        while positions.len() < number {
            positions.push((thread_rng().gen_range(0, w),
                            thread_rng().gen_range(0, h)));
        }
    }

    let last = colours.get(colours.len() - 1).unwrap_or(&(0, 0, 0)).clone();
    while colours.len() < positions.len() {
        colours.push(last.clone());
    }

    (0..positions.len()).map(|i| {
        let (x, y) = positions[i];
        let (r, g, b) = colours[i];
        Point { x, y, r, g, b}
    }).collect()
}

fn main() {
    let mut verbose = false;
    let mut size = "500x500".to_string();
    let mut point_count: usize = 0;
    let mut positions = "".to_string();
    let mut colours = "".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Create a new fondo.");
        ap.refer(&mut verbose)
            .add_option(&["-v", "--verbose"], StoreTrue,
            "be verbose");
        ap.refer(&mut size)
            .add_option(&["-s", "--size"], Store,
            "size for the generated image in WxH format");
        ap.refer(&mut point_count)
            .add_option(&["-n", "--number"], Store,
            "number of random points to add to the list of positions");
        ap.refer(&mut positions)
            .add_option(&["-p", "--positions"], Store,
            "colon-separated list of comma-separated points x,y");
        ap.refer(&mut colours)
            .add_option(&["-c", "--colors", "--colours"], Store,
            "colon-separated list of comma-separated colours r,g,b.\n\
            The last color is repeated until it fills all positions");

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

    for point in parse_points(w, h, point_count, &positions, &colours) {
        pending.push((point.r, point.g, point.b, point.x, point.y));
        added[point.y][point.x] = true;
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
