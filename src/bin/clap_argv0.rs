use clap::Parser;

#[derive(Debug, Clone, Parser)]
pub struct Opts {
    #[clap(long, default_value = "127.0.0.1:5690")]
    addr: String,
}

fn main() {
    Opts::parse_from(["argv0 must be provided", "--addr", "localhost:8888"]);
}
