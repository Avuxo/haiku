use std::io;
use keystone::{Arch, Mode};

/// Given a string, get the keystone architecture enum.
/// Uses an io error for convenience because of file processing
/// in main. Hence the custom error type. This should honestly get
/// fixed later on.
pub fn get_architecture(arch: &str) -> io::Result<Arch> {
    match arch {
        "arm" => Ok(Arch::ARM),
        "aarch64" => Ok(Arch::ARM64),
        "x86" => Ok(Arch::X86),
        "mips" => Ok(Arch::MIPS),
        "ppc" => Ok(Arch::PPC),
        "sparc" => Ok(Arch::SPARC),
        "systemz" => Ok(Arch::SYSTEMZ),
        "hexagon" => Ok(Arch::HEXAGON),
        "max" => Ok(Arch::MAX),
        _ => Err(io::Error::new(io::ErrorKind::Other, "invalid architecture")),
    }
}

/// Given a list of flags, return a mode bitmask for keystone.
/// endianness true is little, false is big.
pub fn get_mode_flags(address_size: Option<u8>, endianness: bool) -> io::Result<Mode> {
    let mut mode = if endianness { Mode::LITTLE_ENDIAN } else { Mode::BIG_ENDIAN };

    // this really sucks but it's the result of keystone mode initialization.
    // if you try to initialize a keystone engine with an explicit mode on aarch64
    // other than little endian, it will fail. Ideally this would just always
    // default to 64 bit but ¯\_(ツ)_/¯
    if let Some(size) = address_size {
        let bit_mode = match size {
            16 => Mode::MODE_16,
            32 => Mode::MODE_32,
            64 => Mode::MODE_64,
            _ => return Err(io::Error::new(io::ErrorKind::Other, "invalid address size mode")),
        };

        mode |= bit_mode;
    }

    Ok(mode)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn properly_gets_all_architectures() {
        let tests = [
            ("arm", Arch::ARM),
            ("aarch64", Arch::ARM64),
            ("x86", Arch::X86),
            ("mips", Arch::MIPS),
            ("ppc", Arch::PPC),
            ("sparc", Arch::SPARC),
            ("systemz", Arch::SYSTEMZ),
            ("hexagon", Arch::HEXAGON),
            ("max", Arch::MAX),
        ];

        for test in tests {
            let result = get_architecture(test.0).unwrap();
            assert_eq!(result, test.1);
        }
    }

    #[test]
    fn will_error_on_invalid_architecture() {
        let test = "aarch65";
        let result = get_architecture(test);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "invalid architecture");
    }

    #[test]
    fn sets_correct_bitflags_for_mode() {
        let address_size = Some(16 as u8);
        let mode = get_mode_flags(address_size, false).unwrap();

        assert_eq!(mode & Mode::MODE_16, Mode::MODE_16);
        assert_eq!(mode & Mode::LITTLE_ENDIAN, Mode::LITTLE_ENDIAN);
    }
}
