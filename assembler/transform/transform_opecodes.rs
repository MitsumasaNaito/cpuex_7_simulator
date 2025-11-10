use std::io::{Error, ErrorKind};

pub fn transform_opecode(opecode: &str) -> Result<i32, Error> {
    match opecode {
        "add"
        | "sub"
        | "and"
        | "or"
        | "slt"
        | "sll"
        | "srl"
        | "jr"
        | "syscall" => Ok(0x00),
        "addi" => Ok(0x08),
        "lw" => Ok(0x23),
        "sw" => Ok(0x2B),
        "beq" => Ok(0x04),
        "bne" => Ok(0x05),
        "slti" => Ok(0x0A),
        "j" => Ok(0x02),
        "jal" => Ok(0x03),
        _ => Err(Error::new(ErrorKind::InvalidInput, "Invalid opecode")),
    }
}