extern crate rand;
extern crate image;

#[macro_use]
extern crate structopt;

use std::fs::File;
use std::str::FromStr;
use std::fmt::Display;
use std::process::exit;
use std::collections::BTreeSet;

use rand::{Rng, SmallRng, SeedableRng, thread_rng};
use image::ImageBuffer;
use structopt::StructOpt;

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
struct Point {
    // TODO Changing the order here is what gives interesting
    // results for the BTreeSet. Mostly alternating rgb order.
    // So implement PartialOrd/Ord ourselves to allow settings.
    r: u8,
    g: u8,
    b: u8,
    x: usize,
    y: usize,
}

#[derive(Debug)]
enum PendingKind {
    VecPopRandom(Vec<Point>),
    VecShuffleNeighbours(Vec<Point>, u8), // chance
    SetBTree(BTreeSet<Point>),
    SetBTreeRev(BTreeSet<Point>),
}

impl PendingKind {
    fn add(&mut self, point: Point) {
        match self {
            &mut PendingKind::VecPopRandom(ref mut x)
            | &mut PendingKind::VecShuffleNeighbours(ref mut x, _) => {
                x.push(point)
            },
            &mut PendingKind::SetBTree(ref mut set)
            | &mut PendingKind::SetBTreeRev(ref mut set) => {
                set.insert(point);
            },
        }
    }

    fn pop(&mut self, rng: &mut SmallRng) -> Point {
        match self {
            &mut PendingKind::VecPopRandom(ref mut vec) => {
                let which = rng.gen_range(0, vec.len());
                vec.remove(which)
            },
            &mut PendingKind::VecShuffleNeighbours(ref mut vec, _) => {
                vec.pop().unwrap()
            },
            &mut PendingKind::SetBTree(ref mut set) => {
                let point = set.iter().next().unwrap().clone();
                set.take(&point).unwrap()
            },
            &mut PendingKind::SetBTreeRev(ref mut set) => {
                let point = set.iter().rev().next().unwrap().clone();
                set.take(&point).unwrap()
            },
        }
    }

    fn has_any(&self) -> bool {
        !match self {
            &PendingKind::VecPopRandom(ref x)
            | &PendingKind::VecShuffleNeighbours(ref x, _) => x.is_empty(),

            &PendingKind::SetBTree(ref x)
            | &PendingKind::SetBTreeRev(ref x) => x.is_empty()
        }
    }

    fn shuffle_chance(&self) -> u8 {
        match self {
            &PendingKind::VecShuffleNeighbours(_, chance) => chance,
            _ => 0
        }
    }
}

const VALUE_SEPARATOR: char = ',';
const LIST_SEPARATOR: char = ':';

/// fondoso, to create beautiful images and wallpapers
#[derive(StructOpt, Debug)]
#[structopt(name = "fondoso")]
struct Opt {
    /// Be verbose
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,

    /// Size for the generated image, in WxH format
    #[structopt(short = "s", long = "size", default_value = "500x500")]
    size: String,

    /// Number of random points to add to the list of positions
    #[structopt(short = "n", long = "number", default_value = "0")]
    point_count: usize,

    /// Colon-separated list of comma-separated points x,y
    #[structopt(short = "p", long = "positions", default_value = "")]
    positions: String,

    /// Colon-separated list of comma-separated colours r,g,b.
    /// The last color is repeated until it fills all positions
    #[structopt(short = "c", long = "colours", default_value = "")]
    colours: String,

    /// Randomise colours instead repeating the last one
    #[structopt(short = "r", long = "random")]
    randomise_colours: bool,

    /// Output filename
    #[structopt(short = "o", long = "output", default_value = "output.png")]
    output: String,

    /// Delta offset when updating the colour at each step
    #[structopt(short = "d", long = "delta", default_value = "4")]
    delta: u32,

    /// The kind of list/point choosing to use. If a number is used
    /// it should be an integer between 0 and 100 indicating the chance
    /// to shuffle the list of neighbours (100 always, 0 never).
    /// Other values are 'tree' and 'treerev'
    #[structopt(short = "k", long = "kind", default_value = "default")]
    kind: String,

    /// The seed to be used for the random number generator
    #[structopt(short = "f", long = "fixed-seed")]
    seed: Option<u64>
}

fn offset(value: u8, delta: i32, rng: &mut SmallRng) -> u8 {
    let mut random: i32 = 0;
    while random == 0 {
        random = rng.gen_range(-delta, delta + 1);
    }
    match value as i32 + random {
        x if x < 0   => 0,
        x if x > 255 => 255,
        x => x as u8
    }
}

fn neighbours(x: usize, y: usize, w: usize, h: usize, shuffle_chance: u8,
              rng: &mut SmallRng)
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
    if shuffle_chance != 0 && rng.gen_range(0, 100) < shuffle_chance {
        rng.shuffle(&mut result);
    }
    result
}

fn parse_or_exit<T>(what: &str, name: &str) -> T
    where
        T: FromStr,
        <T as FromStr>::Err: Display
{
    match what.trim().parse::<T>() {
        Ok(x) => x,
        Err(e) => {
            eprintln!("Could not parse {} into a number ({})", name, e);
            exit(1);
        }
    }
}

fn parse_points(w: usize, h: usize, opt: &Opt, rng: &mut SmallRng)
    -> Vec<Point>
{
    let (w, h) = (w as i32, h as i32);
    let mut positions: Vec<(usize, usize)> = if opt.positions.is_empty() {
        Vec::new()
    } else {
        opt.positions.split(LIST_SEPARATOR).map(|point| {
            let point: Vec<&str> = point.split(VALUE_SEPARATOR).collect();
            if point.len() != 2 {
                eprintln!("Incorrect point format (must be x{}y)",
                          VALUE_SEPARATOR);
                exit(1);
            }
            let x = match parse_or_exit::<f64>(point[0], "x coordinate") {
                x if x < 0.0 => w + x as i32 % w,
                x if 0.0 < x && x < 1.0 => {
                    let tmp = (x * w as f64) as i32;
                    if tmp == w { w - 1 } else { tmp }
                },
                x => x as i32 % w
            };
            let y = match parse_or_exit::<f64>(point[1], "y coordinate") {
                y if y < 0.0 => h + y as i32 % h,
                y if 0.0 < y && y < 1.0 => {
                    let tmp = (y * h as f64) as i32;
                    if tmp == h { h - 1 } else { tmp }
                },
                y => y as i32 % h
            };
            (x as usize, y as usize)
        }).collect()
    };

    let mut colours: Vec<(u8, u8, u8)> = if opt.colours.is_empty() {
        Vec::new()
    } else {
        opt.colours.split(LIST_SEPARATOR).map(|point| {
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

    let (w, h) = (w as usize, h as usize);
    if opt.point_count == 0 && positions.is_empty() {
        positions.push((w / 2, h / 2));
    } else {
        while positions.len() < opt.point_count {
            positions.push((rng.gen_range(0, w), rng.gen_range(0, h)));
        }
    }

    if opt.randomise_colours {
        while colours.len() < positions.len() {
            colours.push((rng.gen_range(0, 255),
                          rng.gen_range(0, 255),
                          rng.gen_range(0, 255)));
        }
    } else {
        let last = if colours.is_empty() {
            (0, 0, 0)
        } else {
            colours[colours.len() - 1].clone()
        };
        while colours.len() < positions.len() {
            colours.push(last.clone());
        }
    }

    (0..positions.len()).map(|i| {
        let (x, y) = positions[i];
        let (r, g, b) = colours[i];
        Point {x, y, r, g, b}
    }).collect()
}

fn main() {
    let opt = Opt::from_args();
    let size: Vec<&str> = opt.size.split('x').collect();

    if size.len() != 2 {
        eprintln!("Incorrect size format (must be WxH)");
        exit(1);
    }
    let w: usize = parse_or_exit(size[0], "width");
    let h: usize = parse_or_exit(size[1], "height");
    let delta = opt.delta as i32;

    let mut img = ImageBuffer::new(w as u32, h as u32);
    let mut added = vec![vec![false; w]; h];
    let mut rng = match opt.seed {
        Some(seed) => {
            let mut seed = seed;
            let mut array = [0u8; 16];
            for i in 0..array.len() {
                array[i] = (seed % 256) as u8;
                seed /= 256;
            }
            SmallRng::from_seed(array)
        },
        _ => SmallRng::from_rng(thread_rng()).unwrap()
    };

    let mut pending = match opt.kind.parse() {
        Ok(x) if x <= 100 => PendingKind::VecShuffleNeighbours(Vec::new(), x),
        _ => {
            match &opt.kind[..] {
                "tree" => PendingKind::SetBTree(BTreeSet::new()),
                "treerev" => PendingKind::SetBTreeRev(BTreeSet::new()),
                _ => PendingKind::VecPopRandom(Vec::new())
            }
        }
    };

    for point in parse_points(w, h, &opt, &mut rng)
    {
        added[point.y][point.x] = true;
        pending.add(point);
    }

    let total = w * h;
    let mut done = 0;
    while pending.has_any() {
        if opt.verbose && done % 10_000 == 0 {
            println!("{:.2}%", 100.0 * (done as f64 / total as f64));
        }

        let point = pending.pop(&mut rng);
        let (r, g, b) = (point.r, point.g, point.b);
        let r = offset(r, delta, &mut rng);
        let g = offset(g, delta, &mut rng);
        let b = offset(b, delta, &mut rng);

        let (x, y) = (point.x, point.y);
        img.put_pixel(x as u32, y as u32, image::Rgb([r, g, b]));
        done += 1;
        for &(x, y) in neighbours(x, y, w, h, pending.shuffle_chance(),
                                  &mut rng).iter()
        {
            if !added[y][x] {
                pending.add(Point {r, g, b, x, y});
                added[y][x] = true; // Moving this outside makes it more sparse
            }
        }
    }

    if opt.verbose {
        println!("100.00%. Saving...");
    }
    let ref mut fp = File::create(opt.output).unwrap();
    image::ImageRgb8(img).save(fp, image::PNG).unwrap();
    if opt.verbose {
        println!("Done.");
    }
}
