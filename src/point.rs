use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Point {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub x: usize,
    pub y: usize,
    pub sort_mode: u16
}

impl Point {
    pub fn as_tuple(&self) -> (usize, usize, usize, usize, usize) {
        (
            self.get_field(self.sort_mode >>  0),
            self.get_field(self.sort_mode >>  3),
            self.get_field(self.sort_mode >>  6),
            self.get_field(self.sort_mode >>  9),
            self.get_field(self.sort_mode >> 12),
        )
    }

    pub fn get_field(&self, field: u16) -> usize {
        match field & 0b111 {
            1 => self.x,
            2 => self.y,
            3 => self.r as usize,
            4 => self.g as usize,
            5 => self.b as usize,
            _ => 0
        }
    }

    pub fn get_sort_mode(mode: &str) -> u16 {
        let mut mode = mode.to_lowercase();
        for c in "rgbxy".chars() {
            if !mode.contains(c) {
                mode.push(c);
            }
        }
        let mut result = 0u16;
        for c in mode.chars().rev() {
            result = (result << 3) | match c {
                'x' => 1,
                'y' => 2,
                'r' => 3,
                'g' => 4,
                'b' => 5,
                _ => 0
            }
        }
        result
    }
}

impl Ord for Point {
    fn cmp(&self, other: &Point) -> Ordering {
        self.as_tuple().cmp(&other.as_tuple())
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Point) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
