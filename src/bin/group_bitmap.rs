#![feature(slice_group_by)]

use std::{iter::once, ops::Range};

use itertools::Itertools;

fn main() {
    let bits = [
        true, true, true, false, false, true, true, true, false, true, false, true,
    ];

    high_ranges(&bits).for_each(|r| {
        dbg!(r);
    });
}

fn high_ranges(bits: &[bool]) -> impl Iterator<Item = Range<usize>> + '_ {
    let mut start = None;

    bits.iter()
        .copied()
        .chain(once(false))
        .enumerate()
        .filter_map(move |(i, bit)| match (bit, start) {
            // A new high range starts.
            (true, None) => {
                start = Some(i);
                None
            }
            // The current high range ends.
            (false, Some(s)) => {
                start = None;
                Some(s..i)
            }
            _ => None,
        })
}
