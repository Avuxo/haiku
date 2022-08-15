use std::io;

use clap::Parser;

mod ips;
mod parse;
mod macros;
mod arch;
mod args;

use args::Args;

fn main() -> io::Result<()>{
    let cli = Args::parse();

    let arch = arch::get_architecture(&cli.architecture)?;

    // keystone flags
    let mode_flags = arch::get_mode_flags(
        cli.address_size,
        cli.little_endian
    )?;

    // Although clean this means that only one error will be reported at a time.
    // In the future this might benefit from parsing haikus one at a time rather
    // than in bulk the way it's done now.
    let result = match parse::parse_haiku(&cli.haiku_name, mode_flags, arch) {
        Ok(ips) => ips,
        Err(message) => panic!("Error: {}", message),
    };

    ips::generate_ips(&result, &cli.ips_name)?;

    Ok(())
}
