use std::fs::File;
use std::io::{BufRead, BufReader, Write, BufWriter, Error, ErrorKind};
use std::collections::HashMap;
mod parse_mips_line;
mod symbol_table;
mod transform;

pub const PROGRAM_START: i32 = 0x00400000;

pub struct Instruction {
    label: String,
    opecode: String,
    operands: Vec<String>,
}

// 2パスアセンブラ
fn main() -> Result<(), Error>{

    // シンボルテーブルを作成
    let input1 = File::open("input.asm")?;
    let buffered1 = BufReader::new(input1);
    let mut symbol_table = HashMap::new();
    let mut line_number = 0;
    for line in buffered1.lines() {
        let line = line.map_err(|e| Error::new(ErrorKind::Other, e))?;
        symbol_table::make_symbol_table(&mut symbol_table, &line, line_number as i32)?;
        line_number += 1;
    }
    // let symbol = match symbol_table.get("end") {
    //     Some(symbol) => symbol,
    //     None => &0,
    // };
    // println!("symbol: {}", symbol);

    let file = File::create("output.hex")?;
    let mut writer = BufWriter::new(file);

    // 命令を解析
    let input2 = File::open("input.asm")?;
    let buffered2 = BufReader::new(input2);
    let mut line_number = 0;
    for line in buffered2.lines() {
        let line = line.map_err(|e| Error::new(ErrorKind::Other, e))?;
        let instruction: Instruction = parse_mips_line::parse_mips_line(&line)?;

        // if instruction.label != "" {
        //     println!("{}: ", instruction.label);
        // }
        // println!("{}", instruction.opecode);
        // for operand in &instruction.operands {
        //     println!("{}", operand);
        // }
        let decoded_instruction: String = format!("{:08X}", transform::transform_instructions::transform_instruction(&instruction, &symbol_table, &line_number)?);
        writer.write_all(decoded_instruction.as_bytes())?;
        writer.write_all(b"\n")?;
        line_number += 1;
    }
    writer.flush()?;
    Ok(())
}