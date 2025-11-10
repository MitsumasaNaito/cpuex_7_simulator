use std::collections::HashMap;
use std::io::{Error, ErrorKind};
// use crate::PROGRAM_START;

pub fn transform_operand(operand: &str, symbol_table: &HashMap<String, i32>) -> Result<i32, Error> {
    if operand.parse::<i32>().is_ok() {
        return Ok(operand.parse::<i32>().unwrap());
    }
    if operand.starts_with("$") {
        let reg_number = match operand {
            "$zero" => 0,
            "$at" => 1,
            "$v0" => 2,
            "$v1" => 3,
            "$a0" => 4,
            "$a1" => 5,
            "$a2" => 6,
            "$a3" => 7,
            "$t0" => 8,
            "$t1" => 9,
            "$t2" => 10,
            "$t3" => 11,
            "$t4" => 12,
            "$t5" => 13,
            "$t6" => 14,
            "$t7" => 15,
            "$s0" => 16,
            "$s1" => 17,
            "$s2" => 18,
            "$s3" => 19,
            "$s4" => 20,
            "$s5" => 21,
            "$s6" => 22,
            "$s7" => 23,
            "$t8" => 24,
            "$t9" => 25,
            "$k0" => 26,
            "$k1" => 27,
            "$gp" => 28,
            "$sp" => 29,
            "$fp" => 30,
            "$ra" => 31,
            _ => return Err(Error::new(ErrorKind::InvalidInput, "Invalid operand")),
        };
        return Ok(reg_number);
    }
    let line_number: i32 = match symbol_table.get(operand) {
        Some(line_number) => *line_number,
        None => return Err(Error::new(ErrorKind::InvalidInput, "Invalid symbol")),
    };
    Ok(line_number)
}