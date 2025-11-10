use std::io::Error;

pub fn transform_shamt(opecode: &str) -> Result<i32, Error> {
    match opecode {
        "sll" => Ok(0x00),
        "srl" => Ok(0x02),
        _ => Ok(0x00),
    }
}