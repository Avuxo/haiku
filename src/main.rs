
mod ips;
mod parse;

fn main() {
    let example = vec![
        "// comment",
        "// comment 2",
        "bytes 304F1 10 {",
        "  00 43 11 FF",
        "}",
        "",
        "",
        "instrs 145b78 2F {",
        "  fmov s0, wzr",
        "  ldrh w0, [x8, #0x30]",
        "}",
        "// end comment",
    ];
    let result = parse::parse_haiku(&example);
    match result {
        Ok(_) => println!("no errors"),
        Err(message) => println!("Error: {}", message),
    };
}
