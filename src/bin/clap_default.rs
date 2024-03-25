use clap::{ArgMatches, CommandFactory, FromArgMatches, Parser};

#[derive(clap::Parser, Default, Debug)]
struct Opts {
    #[clap(long)]
    a: i32,

    #[clap(long)]
    b: Option<i32>,

    #[clap(long, default_value = "88")]
    c: Option<i32>,
}

fn main() {
    let mut opts = Opts {
        a: 42,
        c: Some(99),
        ..Default::default()
    };
    opts.update_from(&["<ignored argv0>", "--b", "66"]);

    println!("{:?}", opts);
}
