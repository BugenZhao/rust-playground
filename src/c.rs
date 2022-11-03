#![feature(bench_black_box)]

use std::hint::black_box;

enum DirectionKind {
    Forward,
    Backward,
}

trait Direction {
    fn direction() -> DirectionKind;
}

struct Forward;
struct Backward;

impl Direction for Forward {
    #[inline(always)]
    fn direction() -> DirectionKind {
        DirectionKind::Forward
    }
}

impl Direction for Backward {
    #[inline(always)]
    fn direction() -> DirectionKind {
        DirectionKind::Backward
    }
}

#[inline(never)]
fn test<D>(i: u64) -> usize
where
    D: Direction,
{
    [1, 2, 3].partition_point(|&n| match D::direction() {
        DirectionKind::Forward => n >= i,
        DirectionKind::Backward => n < i,
    })
}

pub fn main() {
    let a1 = test::<Forward>(black_box(233));
    let a2 = test::<Backward>(black_box(233));

    black_box(a1);
    black_box(a2);
}
