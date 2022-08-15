use std::io;
use clap::Parser;

mod ips;
mod parse;
mod macros;
mod arch;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Input file
    #[clap(value_parser)]
    haiku_name: String,

    /// Generated output file
    #[clap(value_parser)]
    ips_name: String,

    /// Keystone assembler backend to use
    #[clap(short, long, value_parser, default_value_t = String::from("aarch64"))]
    architecture: String,

    /// Determine endianness of ASSEMBLER output. byte patches are unaffected.
    #[clap(short, long, action, default_value_t = true)]
    little_endian: bool,

    // n-bit address. Valid values: 16, 32, 64 (optional)
    #[clap(short, short='s', long, value_parser)]
    address_size: Option<u8>
}

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
