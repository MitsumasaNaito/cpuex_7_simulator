use std::io::{Error, ErrorKind};
use crate::Instruction;
use std::collections::HashMap;
use crate::transform::transform_operands;
use crate::transform::transform_opecodes;
use crate::transform::transform_shamts;
use crate::transform::transform_functs;
use crate::PROGRAM_START;

pub fn transform_instruction(instruction: &Instruction, symbol_table: &HashMap<String, i32>, line_number: &i32) -> Result<i32, Error> {
    let decoded_instruction: i32;
    let transformed_opecode: i32 = transform_opecodes::transform_opecode(&instruction.opecode)?;
    // println!("transformed_opecode: {}", transformed_opecode);

    match transformed_opecode {
        0x00 => {
            let transformed_funct: i32 = transform_functs::transform_funct(&instruction.opecode)?;
            match transformed_funct {
                0x0C => {
                    // システムコール
                    decoded_instruction = transformed_funct;
                },
                _ => {
                    // R形式命令
                    let transformed_rd: i32 = transform_operands::transform_operand(&instruction.operands[0], &symbol_table)?;
                    let transformed_rs: i32 = transform_operands::transform_operand(&instruction.operands[1], &symbol_table)?;
                    let transformed_rt: i32 = transform_operands::transform_operand(&instruction.operands[2], &symbol_table)?;
                    let transformed_shamt: i32 = transform_shamts::transform_shamt(&instruction.opecode)?;
                    decoded_instruction = transformed_opecode << 26 | transformed_rs << 21 | transformed_rt << 16 | transformed_rd << 11 | transformed_shamt << 6 | transformed_funct;
                },
            }
        },
        0x08 | 0x0A => {
            // I形式命令(即値演算)
            let transformed_rt: i32 = transform_operands::transform_operand(&instruction.operands[0], &symbol_table)?;
            let transformed_rs: i32 = transform_operands::transform_operand(&instruction.operands[1], &symbol_table)?;
            let transformed_imm: i32 = transform_operands::transform_operand(&instruction.operands[2], &symbol_table)?;
            decoded_instruction = transformed_opecode << 26 | transformed_rs << 21 | transformed_rt << 16 | transformed_imm;
        },
        0x23 | 0x2B => {
            // I形式命令(ロード・ストア)
            let transformed_rt: i32 = transform_operands::transform_operand(&instruction.operands[0], &symbol_table)?;
            let transformed_imm: i32 = transform_operands::transform_operand(&instruction.operands[1], &symbol_table)?;
            let transformed_rs: i32 = transform_operands::transform_operand(&instruction.operands[2], &symbol_table)?;
            decoded_instruction = transformed_opecode << 26 | transformed_rs << 21 | transformed_rt << 16 | transformed_imm;
        },
        0x04 | 0x05 => {
            // I形式命令(条件分岐)
            let transformed_rs: i32 = transform_operands::transform_operand(&instruction.operands[0], &symbol_table)?;
            let transformed_rt: i32 = transform_operands::transform_operand(&instruction.operands[1], &symbol_table)?;
            let transformed_imm: i32 = transform_operands::transform_operand(&instruction.operands[2], &symbol_table)? - line_number - 1; // 最後の定数は要調整
            decoded_instruction = transformed_opecode << 26 | transformed_rs << 21 | transformed_rt << 16 | transformed_imm;
        },
        0x02 | 0x03 => {
            // J形式命令
            let transformed_addr: i32 = transform_operands::transform_operand(&instruction.operands[0], &symbol_table)?;
            decoded_instruction = transformed_opecode << 26 | transformed_addr + (PROGRAM_START >> 2);
        },
        _ => {
            return Err(Error::new(ErrorKind::InvalidInput, "Invalid opecode"));
        },
    }
    Ok(decoded_instruction)
}