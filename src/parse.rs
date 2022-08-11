enum ScannerState {
    Scanning,
    InInstructionPatch,
    InBytesPatch,
}

struct Haiku {
    start_address: u64,
    bytes_len: u32, // IPS patches are limited to 2^24
}

struct ParserState {
    padding_bytes: Vec<u8>,
    instruction_padding: bool,
    state: ScannerState
}

/// Parse a haiku file.
/// takes in array of lines.
pub fn parse_haiku(lines: &[&str]) -> Result<(), String>{
    let mut state = ParserState {
        padding_bytes: vec![0x00],
        instruction_padding: false,
        state: ScannerState::Scanning
    };

    let mut cur_haiku = Haiku { start_address: 0, bytes_len: 0 };
    let mut remaining_bytes = 0; // how many bytes left in current haiku?

    for raw_line in lines.iter() {
        let line = raw_line.trim_left();
        // skip comments regardless of current state.
        if line.starts_with("//") || line.len() == 0 {
            continue;
        }

        // handle script directives.
        // TODO: handle script directives properly
        if line.starts_with("#") {
            //handle_script_directive(line, state);
            continue;
        }

        match state.state {
            ScannerState::Scanning => {
                // byte patch
                if line.starts_with("bytes ") {
                    state.state = ScannerState::InBytesPatch;

                    let patch_info = parse_patch_definition(line);
                    cur_haiku = Haiku{ start_address: patch_info.0, bytes_len: patch_info.1,};

                    remaining_bytes = patch_info.1;
                } else if line.starts_with("instrs ") {
                    state.state = ScannerState::InInstructionPatch;

                    let patch_info = parse_patch_definition(line);
                    cur_haiku = Haiku{ start_address: patch_info.0, bytes_len: patch_info.1,};

                    remaining_bytes = patch_info.1;
                } else {
                    return Err(
                        format!("Unexpected token on line {}", line)
                    );
                }

            },
            ScannerState::InInstructionPatch => {
                if line.starts_with("}") {
                    state.state = ScannerState::Scanning;
                    continue;
                }

                // process each instruction line by line
                if remaining_bytes == 0 {
                    return Err(format!(
                        // TODO: more debug info
                        "Maximum size of {} bytes exceeded for haiku @ 0x{:#x}",
                        cur_haiku.bytes_len,
                        cur_haiku.start_address,
                    ));
                }
            },
            ScannerState::InBytesPatch => {
                if line.starts_with("}") {
                    state.state = ScannerState::Scanning;
                    continue;
                }

                // process each instruction line by line
                if remaining_bytes == 0 {
                    return Err(format!(
                        // TODO: more debug info
                        "Maximum size of {} bytes exceeded for haiku @ 0x{:#x}",
                        cur_haiku.bytes_len,
                        cur_haiku.start_address,
                    ));
                }
                
            },
        }
    }

    Ok(())
}

/// Given a definition line, parse out the start address and length.
/// returns a tuple of the start address and the max allowed length.
///
/// Assumes that this line has already been checked to start with either
/// `bytes' or `instrs'
fn parse_patch_definition(line: &str) -> (u64, u32) {
    // whether its a byte patch or instruction patch doesn't matter.
    // the last token will also always be `{' but that can be ignored.
    // TODO: more robust handling of spaces rather than dumb split.
    let tokens: Vec<&str> = line.split(" ").collect();

    let address = i64::from_str_radix(tokens[1], 16).unwrap() as u64;

    let byte_len = i64::from_str_radix(tokens[2], 16).unwrap() as u32;

    (address, byte_len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bytes_patch_definition() {
        let definition = "bytes 304F1 10 {";

        let result: (u64, u32) = parse_patch_definition(definition);

        assert_eq!(result.0, 0x304F1);
        assert_eq!(result.1, 0x10);
    }

    #[test]
    fn parses_instrs_patch_definition() {
        let definition = "instrs 145b78 2F {";

        let result: (u64, u32) = parse_patch_definition(definition);

        assert_eq!(result.0, 0x145b78);
        assert_eq!(result.1, 0x2f);
    }
}
