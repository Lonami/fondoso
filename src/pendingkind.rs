use std::collections::{BTreeSet, BinaryHeap};
use rand::{Rng, SmallRng};

use point::Point;

#[derive(Debug)]
pub enum PendingKind {
    VecPopRandom(Vec<Point>),
    VecShuffleNeighbours(Vec<Point>, u8), // chance
    SetBTree(BTreeSet<Point>),
    SetBTreeRev(BTreeSet<Point>),
    Heap(BinaryHeap<Point>)
}

impl PendingKind {
    pub fn add(&mut self, point: Point) {
        use PendingKind::*;
        match self {
            &mut VecPopRandom(ref mut x)
            | &mut VecShuffleNeighbours(ref mut x, _) => {
                x.push(point);
            },
            &mut SetBTree(ref mut set)
            | &mut SetBTreeRev(ref mut set) => {
                set.insert(point);
            },
            &mut Heap(ref mut heap) => {
                heap.push(point);
            }
        }
    }

    pub fn pop(&mut self, rng: &mut SmallRng) -> Point {
        use PendingKind::*;
        match self {
            &mut VecPopRandom(ref mut vec) => {
                let which = rng.gen_range(0, vec.len());
                vec.remove(which)
            },
            &mut VecShuffleNeighbours(ref mut vec, _) => {
                vec.pop().unwrap()
            },
            &mut SetBTree(ref mut set) => {
                let point = set.iter().next().unwrap().clone();
                set.take(&point).unwrap()
            },
            &mut SetBTreeRev(ref mut set) => {
                let point = set.iter().rev().next().unwrap().clone();
                set.take(&point).unwrap()
            },
            &mut Heap(ref mut heap) => {
                heap.pop().unwrap()
            }
        }
    }

    pub fn has_any(&self) -> bool {
        use PendingKind::*;
        !match self {
            &VecPopRandom(ref x)
            | &VecShuffleNeighbours(ref x, _) => x.is_empty(),

            &SetBTree(ref x)
            | &SetBTreeRev(ref x) => x.is_empty(),

            &Heap(ref x) => x.is_empty()
        }
    }

    pub fn shuffle_chance(&self) -> u8 {
        match self {
            &PendingKind::VecShuffleNeighbours(_, chance) => chance,
            _ => 0
        }
    }
}
