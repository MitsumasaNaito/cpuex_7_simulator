use std::io::{Error, ErrorKind};

pub fn transform_funct(opecode: &str) -> Result<i32, Error> {
    match opecode {
        "add" => Ok(0x20),
        "sub" => Ok(0x22),
        "and" => Ok(0x24),
        "or" => Ok(0x25),
        "slt" => Ok(0x2A),
        "sll" => Ok(0x00),
        "srl" => Ok(0x02),
        "jr" => Ok(0x08),
        "syscall" => Ok(0x0C),
        _ => Err(Error::new(ErrorKind::InvalidInput, "Invalid funct")),
    }
}