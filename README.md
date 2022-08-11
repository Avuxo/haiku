# haiku
Binary patch macro assembler generating IPS files.

## Syntax
```
// This is a comment, any line that starts with `//' will be ignored
// by the patch generation system

// empty lines are also ignored.

// lines starting with a # will run some haiku directive
// any assembler supported by the keystone backend can be used.
#aasembler aarch64

#instruction_padding nop

#disable_instruction_padding
#enable_instruction_padding

// patches starting with `bytes' will consist of a series of
// 1 or more 2-hex-digit numbers representing bytes to be inserted
// inline.
// The first number after `bytes' is the IPS start address.
// The second number is how many bytes are allowed to go in
// this patch.
// A patch will not be generated if it exceeds this number.
bytes 304F4 10 {
    // bytes can be separated by any amount of whitespace
    00 0F F1 41 41
    41 41 3F 3F 3F
}

// whatever instruction set is set in the assembler directive
// will be used to assemble all instructions.
instr 30600 1F {
    // an instruction can be prefixed by whitespace
    // there can be a maximum of one instruction per line.
    fmov s0, wzr
    ldrb x0, [x8, #0x30]
}
```

## Directives
- `assembler` set the assembler to be used by Keystone.
- `disable_instruction_padding` stop instructions from being padded with system `nop`.
- `enable_instruction_padding` enable instructions padding with target `nop`.
- `instruction_padding` set the instruction to pad with. Defaults to `nop`.
