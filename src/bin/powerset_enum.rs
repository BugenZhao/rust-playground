#![feature(never_type, exhaustive_patterns, proc_macro_hygiene)]

struct Red;
struct Green;
struct Blue;

#[powerset_enum::powerset_enum]
enum Color {
    Red(Red),
    Green(Green),
    Blue(Blue),
}

fn foo(x: bool) -> Color![Red, Green] {
    if x {
        Color::Red(Red)
    } else {
        Color::Green(Green)
    }
}

fn main() {
    match foo(rand::random()) {
        Color::Red(_) => todo!(),
        Color::Green(_) => todo!(),
    }
}
