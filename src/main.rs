use std::io;
use clap::Parser;

mod ips;
mod parse;
mod macros;

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
    assembler: String,
}

fn main() -> io::Result<()>{
    let cli = Args::parse();

    // Although clean this means that only one error will be reported at a time.
    // In the future this might benefit from parsing haikus one at a time rather
    // than in bulk the way it's done now.
    let result = match parse::parse_haiku(&cli.haiku_name) {
        Ok(ips) => ips,
        Err(message) => panic!("Error: {}", message),
    };

    ips::generate_ips(&result, &cli.ips_name)?;

    Ok(())
}
