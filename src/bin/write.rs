use std::fmt::Write;

fn write_generic(w: &mut impl Write) {
    write_dyn(w)
}

fn write_dyn(mut w: &mut dyn Write) {
    write_generic(&mut w)
}

fn main() {}
