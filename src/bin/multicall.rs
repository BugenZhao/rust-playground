use clap::{arg, command, Arg, Command};
use itertools::Itertools;

fn main() {
    let arg = || {
        Arg::new("args")
            .num_args(0..)
            .allow_hyphen_values(true)
            .trailing_var_arg(true)
    };

    let applets = || {
        [
            Command::new("frontend").aliases(["fe"]).arg(arg()),
            Command::new("backend").aliases(["be"]).arg(arg()),
        ]
    };

    let matches = command!("multicall")
        .multicall(true)
        .subcommand(
            command!("multicall")
                .subcommand_value_name("APPLET")
                .subcommand_help_heading("Applets")
                .subcommand_required(true)
                .subcommands(applets()),
        )
        .subcommands(applets())
        .disable_help_flag(true)
        .get_matches();

    let subcommand = matches.subcommand().unwrap();
    let applet = subcommand.1.subcommand().unwrap_or(subcommand);

    println!("applet: {}", applet.0);
    println!(
        "args: {:?}",
        applet
            .1
            .get_many::<String>("args")
            .into_iter()
            .flatten()
            .collect_vec()
    )
}
