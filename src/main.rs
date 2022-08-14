use clap::Parser;

mod ips;
mod parse;
mod macros;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
   #[clap(value_parser)]
   haiku_name: String,

   /// Keystone assembler backend to use
   #[clap(short, long, value_parser, default_value_t = String::from("aarch64"))]
   assembler: String,
}

fn main() {
    let cli = Args::parse();

    // Although clean this means that only one error will be reported at a time.
    // In the future this might benefit from parsing haikus one at a time rather
    // than in bulk the way it's done now.
    let result = parse::parse_haiku(&cli.haiku_name);
    match result {
        Ok(_) => println!("no errors"),
        Err(message) => println!("Error: {}", message),
    };
}
