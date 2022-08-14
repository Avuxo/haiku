
mod ips;
mod parse;
mod macros;

fn main() {
    let example = vec![
        "// comment",
        "// comment 2",
        "bytes 304F1 8 {",
        "  00 43 11 FF",
        "  31 24 31 12",
        "}",
        "",
        "",
        "instrs 145b78 8 {",
        "  fmov s0, wzr",
        "  ldrh w0, [x8, #0x30]",
        "}",
        "// end comment",
    ];

    // Although clean this means that only one error will be reported at a time.
    // In the future this might benefit from parsing haikus one at a time rather
    // than in bulk the way it's done now.
    let result = parse::parse_haiku(&example);
    match result {
        Ok(_) => println!("no errors"),
        Err(message) => println!("Error: {}", message),
    };
}
