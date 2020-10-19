mod display;
mod system;

fn main() -> Result<(), anyhow::Error> {
  let matches = clap::App::new(env!("CARGO_PKG_NAME"))
    .about("A CHIP-8 emulator")
    .version(env!("CARGO_PKG_VERSION"))
    .arg(
      clap::Arg::new("file")
        .value_name("file")
        .about("the file to run")
        .required(true)
        .index(1),
    )
    .get_matches();

  let mut sys = system::System::new();
  let bytes = std::fs::read(matches.value_of("file").unwrap())?;
  sys.load_program(&bytes);

  sys.run()
}
