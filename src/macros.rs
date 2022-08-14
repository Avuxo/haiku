
/// Check if the current line is a macro / pseudo-instruction
/// Offset given is the base offset for the patch + the offset
/// to the pseudo-instruction. This is necessary for absolute
/// addressing.
/// Returns None when no macro exists
pub fn get_macro(line: &str, offset: u32) -> Option<String> {
    // TODO: macros
    None
}
