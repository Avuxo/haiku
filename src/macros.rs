
/// Check if the current line is a macro / pseudo-instruction
/// Offset given is the base offset for the patch + the offset
/// to the pseudo-instruction. This is necessary for absolute
/// addressing.
///
/// All macros are prefixed with `!' so this can be more or less portable.
///
/// Returns None when no macro exists
pub fn get_macro(line: &str, offset: u32) -> Option<String> {
    // TODO: architecture-specific handling.
    // TODO: properly tokenize instead of just doing dumb check.
    if line.starts_with("!call") {
        return Some(parse_jump_statement(line, offset, true));
    } else if line.starts_with("!jump") {
        return Some(parse_jump_statement(line, offset, false));
    }

    None
}

/// Generate jump statement for fixed-width instruction set.
/// Currently only supports aarch64
/// When link is set, it will use bl. When link is not set, b will be used.
///
/// Note: All call macros are expected to be prefixed with the immediate #.
/// Example: call #0x13cc8c
fn parse_jump_statement(instr: &str, instr_address: u32, link: bool) -> String {
    // TODO: proper parsing of instrction. right now just breaks after #0x
    let dest_address = i64::from_str_radix(&instr[9..], 16).unwrap() as i32;

    let jump: i32 = dest_address - (instr_address as i32);

    // TODO: handle multiple instructions necessary for longer jumps.
    return format!(
        "{} #{}{:#x}",
        if link {"bl"} else {"b"},
        if jump > 0 { "" } else { "-" },
        jump.abs()
    );
}
