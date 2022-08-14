extern crate keystone;

use std::fs::File;
use std::io::{prelude::*, BufReader};

use keystone::{Keystone, Arch, Mode};

use crate::{ips, macros};

enum ScannerState {
    Scanning,
    InInstructionPatch,
    InBytesPatch,
}

struct Haiku {
    start_offset: u32,
    bytes_len: u32, // IPS patches are limited to 2^24
}

struct ParserState {
    padding_bytes: Vec<u8>,
    instruction_padding: bool,
    state: ScannerState
}

/// Parse a haiku file.
/// takes in filename and will read buffered as lines.
pub fn parse_haiku(filename: &str) -> Result<Vec<ips::IpsEntry>, String> {
    // initialize the keystone engine for assembly.
    // TODO: read assembler from command line option
    let engine = Keystone::new(Arch::ARM64, Mode::LITTLE_ENDIAN)
        .expect("Could not initialize Keystone engine");

    let mut state = ParserState {
        padding_bytes: vec![0x1f, 0x20, 0x03, 0xd5], // aarch64 nop
        instruction_padding: true,
        state: ScannerState::Scanning
    };

    let file = match File::open(filename) {
        Ok(file) => file,
        Err(error) => {
            return Err(format!("File read error on [{}] {}", filename, error.to_string()));
        }
    };

    let buf_reader = BufReader::new(file);

    let mut cur_haiku = Haiku { start_offset: 0, bytes_len: 0 };
    let mut remaining_bytes = 0; // how many bytes left in current haiku?

    let mut ips_entries = Vec::<ips::IpsEntry>::new();

    // bytes for current patch
    let mut patch_bytes = Vec::<u8>::new();

    for l in buf_reader.lines() {
        let raw_line = l.unwrap();
        let line = raw_line.trim_start();

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

                    let patch_info = parse_patch_definition(line)?;
                    cur_haiku = Haiku{ start_offset: patch_info.0, bytes_len: patch_info.1,};

                    remaining_bytes = patch_info.1;
                } else if line.starts_with("instrs ") {
                    state.state = ScannerState::InInstructionPatch;

                    let patch_info = parse_patch_definition(line)?;
                    cur_haiku = Haiku{ start_offset: patch_info.0, bytes_len: patch_info.1,};

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

                    // pad remaining instructions with nops etc.
                    if state.instruction_padding {
                        let padding_entry_length = state.padding_bytes.len() as u32;
                        while remaining_bytes > 0 {
                            patch_bytes.extend_from_slice(&state.padding_bytes);
                            remaining_bytes -= padding_entry_length;
                        }
                    }

                    // build the result based on the current
                    // entries in the bytes vector. Copied
                    // so that the buffer can be cleared.
                    ips_entries.push(ips::IpsEntry{
                        offset: cur_haiku.start_offset,
                        patch: patch_bytes.clone(),
                    });

                    patch_bytes.clear();

                    continue;
                }

                let mut instruction = line.to_string();

                if let Some(instr) = macros::get_macro(line, cur_haiku.start_offset + patch_bytes.len() as u32) {
                    instruction = instr;
                }

                let assembled = engine.asm(instruction, 0).expect(
                    &format!("Failed to assemble [{}]", line)
                );

                // does space remain for new assembly?
                if remaining_bytes < assembled.bytes.len() as u32 {
                    return Err(format!(
                        "Max length exceeded for haiku @ 0x{:#x} on instruction [{}]",
                        cur_haiku.start_offset,
                        line
                    ));
                }

                remaining_bytes -= assembled.bytes.len() as u32;

                patch_bytes.extend_from_slice(&assembled.bytes);
            },

            ScannerState::InBytesPatch => {
                if line.starts_with("}") {
                    state.state = ScannerState::Scanning;

                    ips_entries.push(ips::IpsEntry{
                        offset: cur_haiku.start_offset,
                        patch: patch_bytes.clone(),
                    });

                    continue;
                }

                // bytes in a patch are just separated by spaces.
                let split: Vec<&str> = line.split_whitespace().collect();

                for byte in split {
                    let b = match i64::from_str_radix(&byte, 16) {
                        Ok(b) => b as u8,
                        Err(_) => return Err(
                            format!(
                                "Invalid digit in byte patch for haiku @ 0x{:#x} byte {}",
                                cur_haiku.start_offset,
                                &byte
                            )
                        ),
                    };

                    if remaining_bytes > 0 {
                        patch_bytes.push(b);
                        remaining_bytes -= 1;
                    } else {
                        return Err(format!(
                            "Maximum size of {} bytes exceeded for haiku @ 0x{:#x} with byte {:#x}",
                            cur_haiku.bytes_len,
                            cur_haiku.start_offset,
                            b
                        ));
                    }
                }
            },
        }
    }

    Ok(ips_entries)
}

/// Given a definition line, parse out the start address and length.
/// returns a tuple of the start address and the max allowed length.
///
/// Assumes that this line has already been checked to start with either
/// `bytes' or `instrs'
fn parse_patch_definition(line: &str) -> Result<(u32, u32), String> {
    // whether its a byte patch or instruction patch doesn't matter.
    // the last token will also always be `{' but that can be ignored.
    // TODO: more robust handling of spaces rather than dumb split.
    let tokens: Vec<&str> = line.split(" ").collect();

    let address = match i64::from_str_radix(tokens[1], 16) {
        Ok(addr) => addr as u32,
        Err(e) => return Err(e.to_string()),
    };

    let byte_len = i64::from_str_radix(tokens[2], 16).unwrap() as u32;

    Ok((address, byte_len))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bytes_patch_definition() {
        let definition = "bytes 304F1 10 {";

        let result: (u32, u32) = parse_patch_definition(definition)?;

        assert_eq!(result.0, 0x304F1);
        assert_eq!(result.1, 0x10);
    }

    #[test]
    fn parses_instrs_patch_definition() {
        let definition = "instrs 145b78 2F {";

        let result: (u32, u32) = parse_patch_definition(definition)?;

        assert_eq!(result.0, 0x145b78);
        assert_eq!(result.1, 0x2f);
    }
}
