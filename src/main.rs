use std::fs::read;

use clap::{App, Arg};

mod display;
mod system;

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .about("A CHIP-8 emulator")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("file")
                .value_name("file")
                .about("the file to run")
                .required(true)
                .index(1),
        )
        .get_matches();

    let mut sys = system::System::new();
    let bytes = read(matches.value_of("file").unwrap()).unwrap();
    sys.load_program(&bytes);
    sys.run();
}
