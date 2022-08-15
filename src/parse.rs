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

struct ParserState {
    padding_bytes: Vec<u8>,
    instruction_padding: bool,
    state: ScannerState
}

/// Parse a haiku file.
/// takes in filename and will read buffered as lines, flag for endianness,
/// and arch assembler to use.
pub fn parse_haiku(filename: &str, mode: Mode, arch: Arch) -> Result<Vec<ips::IpsEntry>, String> {
    // initialize the keystone engine for assembly.
    let engine = Keystone::new(arch, mode)
        .expect("Could not initialize Keystone engine");

    let mut state = ParserState {
        padding_bytes: vec![0x1f, 0x20, 0x03, 0xd5], // aarch64 nop
        instruction_padding: true,
        state: ScannerState::Scanning
    };

    let file = match File::open(filename) {
        Ok(file) => file,
        Err(error) => {
            return Err(format!("File read error on [{}] {}", filename, error));
        }
    };

    let buf_reader = BufReader::new(file);

    let mut start_offset = 0;
    let mut remaining_bytes = 0; // how many bytes left in current haiku?

    let mut ips_entries = Vec::<ips::IpsEntry>::new();

    // bytes for current patch
    let mut patch_bytes = Vec::<u8>::new();

    for l in buf_reader.lines() {
        let raw_line = l.unwrap();
        let line = raw_line.trim_start();

        // skip comments regardless of current state.
        if line.starts_with("//") || line.is_empty() {
            continue;
        }

        // handle script directives.
        // TODO: handle script directives properly
        if line.starts_with('#') {
            //handle_script_directive(line, state);
            continue;
        }

        match state.state {
            ScannerState::Scanning => {
                // byte patch
                if line.starts_with("bytes ") {
                    state.state = ScannerState::InBytesPatch;

                    let patch_info = parse_patch_definition(line)?;
                    start_offset = patch_info.0;

                    remaining_bytes = patch_info.1;
                } else if line.starts_with("instrs ") {
                    state.state = ScannerState::InInstructionPatch;

                    let patch_info = parse_patch_definition(line)?;
                    start_offset = patch_info.0;

                    remaining_bytes = patch_info.1;
                } else {
                    return Err(
                        format!("Unexpected token on line {}", line)
                    );
                }

            },

            ScannerState::InInstructionPatch => {
                if line.starts_with('}') {
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
                        offset: start_offset,
                        patch: patch_bytes.clone(),
                    });

                    patch_bytes.clear();

                    continue;
                }

                let mut instruction = line.to_string();

                if let Some(instr) = macros::get_macro(line, start_offset + patch_bytes.len() as u32) {
                    instruction = instr;
                }

                let assembled = engine
                    .asm(instruction, 0)
                    .unwrap_or_else(|_| panic!("Failed to assemble [{}]", line));

                // does space remain for new assembly?
                if remaining_bytes < assembled.bytes.len() as u32 {
                    return Err(format!(
                        "Max length exceeded for haiku @ {:#x} on instruction [{}]",
                        start_offset,
                        line
                    ));
                }

                remaining_bytes -= assembled.bytes.len() as u32;

                patch_bytes.extend_from_slice(&assembled.bytes);
            },

            ScannerState::InBytesPatch => {
                if line.starts_with('}') {
                    state.state = ScannerState::Scanning;

                    ips_entries.push(ips::IpsEntry{
                        offset: start_offset,
                        patch: patch_bytes.clone(),
                    });

                    continue;
                }

                let bytes = parse_byte_list(
                    line,
                    start_offset,
                    remaining_bytes,
                )?;

                patch_bytes.extend_from_slice(&bytes);
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
    let tokens: Vec<&str> = line.split_whitespace().collect();

    let address = match i64::from_str_radix(tokens[1], 16) {
        Ok(addr) => addr as u32,
        Err(e) => return Err(e.to_string()),
    };

    let byte_len = i64::from_str_radix(tokens[2], 16).unwrap() as u32;

    Ok((address, byte_len))
}

/// Given a line of bytes, get the correct u8s
/// Input example: 80 1F  00   03 13
fn parse_byte_list(line: &str, start_offset: u32, mut remaining_bytes: u32) -> Result<Vec<u8>, String> {
    let mut patch_bytes: Vec<u8> = vec![];
    // bytes in a patch are just separated by spaces.
    let split: Vec<&str> = line.split_whitespace().collect();

    for byte in split {
        let b = match i64::from_str_radix(byte, 16) {
            Ok(b) => b as u8,
            Err(_) => return Err(
                format!(
                    "Invalid digit in byte patch for haiku @ {:#x} byte {}",
                    start_offset,
                    &byte
                )
            ),
        };

        if remaining_bytes > 0 {
            patch_bytes.push(b);
            remaining_bytes -= 1;
        } else {
            return Err(format!(
                "Maximum size exceeded for haiku @ {:#x} with byte {:#x}",
                start_offset,
                b
            ));
        }
    }

    Ok(patch_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bytes_patch_definition() {
        let definition = "bytes 304F1 10 {";

        let result: (u32, u32) = parse_patch_definition(definition).unwrap();

        assert_eq!(result.0, 0x304F1);
        assert_eq!(result.1, 0x10);
    }

    #[test]
    fn parses_instrs_patch_definition() {
        let definition = "instrs 145b78 2F {";

        let result: (u32, u32) = parse_patch_definition(definition).unwrap();

        assert_eq!(result.0, 0x145b78);
        assert_eq!(result.1, 0x2f);
    }

    #[test]
    fn parse_path_def_should_have_error_for_invalid_digits() {
        let definition = "instrs 0x145b78 2F {";
        
        let result = parse_patch_definition(definition);

        assert!(result.is_err());
        assert_eq!("invalid digit found in string", result.unwrap_err().to_string());
    }

    #[test]
    fn parse_out_bytes_from_string_when_enough_space() {
        let line = "80 1F 00 03";
        let result = parse_byte_list(line, 0xDEADBEEF, 0x04).unwrap();

        assert_eq!(result, vec![0x80, 0x1f, 0x00, 0x03]);
    }

    #[test]
    fn parse_bytes_should_have_error_for_invalid_digits() {
        let line = "g80 1F 00 03";
        let result = parse_byte_list(line, 0xDEADBEEF, 0x04);

        assert!(result.is_err());
        assert_eq!(
            "Invalid digit in byte patch for haiku @ 0xdeadbeef byte g80",
            result.unwrap_err().to_string()
        );
    }

    #[test]
    fn parse_bytes_should_have_error_when_out_of_space() {
        let line = "80 1F 00 03";
        let result = parse_byte_list(line, 0xDEADBEEF, 0x03);

        assert!(result.is_err());
        assert_eq!(
            "Maximum size exceeded for haiku @ 0xdeadbeef with byte 0x3",
            result.unwrap_err().to_string()
        );
    }
}
