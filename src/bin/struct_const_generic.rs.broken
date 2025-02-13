#![allow(incomplete_features)]
#![feature(adt_const_params)]

#[derive(std::marker::ConstParamTy, PartialEq, Eq)]
struct Config {
    name: &'static str,
    version: &'static str,
    good: bool,
}

impl Config {
    const MYSQL: Self = Self {
        name: "mysql",
        version: "8.0.0",
        good: true,
    };

    const POSTGRES: Self = Self {
        name: "postgres",
        version: "13.0.0",
        good: false,
    };
}

trait Access {
    fn config(&self) -> &Config;

    fn is_good(&self) -> bool {
        self.config().good
    }
}

struct Static<const C: Config>;

impl<const C: Config> Access for Static<C> {
    fn config(&self) -> &Config {
        &C
    }
}

struct Dynamic {
    config: Config,
}

impl Access for Dynamic {
    fn config(&self) -> &Config {
        &self.config
    }
}

fn main() {
    let config = Config {
        good: true,
        ..Config::POSTGRES
    };

    let _foo = Static::<{ Config::MYSQL }>.is_good();
    let _bar = Dynamic { config }.is_good();
}
