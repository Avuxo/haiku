use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Input file
    #[clap(value_parser)]
    pub haiku_name: String,

    /// Generated output file
    #[clap(value_parser)]
    pub ips_name: String,

    /// Keystone assembler backend to use
    #[clap(short, long, value_parser, default_value_t = String::from("aarch64"))]
    pub architecture: String,

    /// Determine endianness of ASSEMBLER output. byte patches are unaffected.
    #[clap(short, long, action, default_value_t = true)]
    pub little_endian: bool,

    // n-bit address. Valid values: 16, 32, 64 (optional)
    #[clap(short, short='s', long, value_parser)]
    pub address_size: Option<u8>
}
