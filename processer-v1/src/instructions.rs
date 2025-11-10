//! MIPS命令セットの定義と実装

use std::fmt;

/// MIPSレジスタ番号（0-31）
pub type Register = u8;

/// 32ビットの命令
pub type Instruction = u32;

/// 32ビットのデータ
pub type Word = u32;

/// 16ビットの即値
pub type Immediate = i16;

/// 26ビットのアドレス
pub type Address = u32;

/// MIPS命令の種類
#[derive(Debug, Clone, PartialEq)]
pub enum InstructionType {
    // R形式命令
    Add { rd: Register, rs: Register, rt: Register },
    Sub { rd: Register, rs: Register, rt: Register },
    And { rd: Register, rs: Register, rt: Register },
    Or { rd: Register, rs: Register, rt: Register },
    Slt { rd: Register, rs: Register, rt: Register },
    Sll { rd: Register, rt: Register, shamt: u8 },
    Srl { rd: Register, rt: Register, shamt: u8 },
    Jr { rs: Register },
    
    // I形式命令
    Addi { rt: Register, rs: Register, imm: Immediate },
    Lw { rt: Register, rs: Register, imm: Immediate },
    Sw { rt: Register, rs: Register, imm: Immediate },
    Beq { rs: Register, rt: Register, imm: Immediate },
    Bne { rs: Register, rt: Register, imm: Immediate },
    Slti { rt: Register, rs: Register, imm: Immediate },
    
    // J形式命令
    J { addr: Address },
    Jal { addr: Address },
    
    // システムコール
    Syscall,
    
    // 無効な命令
    Invalid,
}

impl InstructionType {
    /// 32ビット命令から命令をデコードする関数
    pub fn decode(instruction: Instruction) -> Self {
        let opcode = (instruction >> 26) as u8;
        let rs = ((instruction >> 21) & 0x1F) as u8;
        let rt = ((instruction >> 16) & 0x1F) as u8;
        let rd = ((instruction >> 11) & 0x1F) as u8;
        let shamt = ((instruction >> 6) & 0x1F) as u8;
        let funct = (instruction & 0x3F) as u8;
        let imm = (instruction & 0xFFFF) as i16;
        let addr = instruction & 0x3FFFFFF;

        match opcode {
            0x00 => {
                // R形式命令
                match funct {
                    0x20 => InstructionType::Add { rd, rs, rt },
                    0x22 => InstructionType::Sub { rd, rs, rt },
                    0x24 => InstructionType::And { rd, rs, rt },
                    0x25 => InstructionType::Or { rd, rs, rt },
                    0x2A => InstructionType::Slt { rd, rs, rt },
                    0x00 => InstructionType::Sll { rd, rt, shamt },
                    0x02 => InstructionType::Srl { rd, rt, shamt },
                    0x08 => InstructionType::Jr { rs },
                    0x0C => InstructionType::Syscall,
                    _ => InstructionType::Invalid,
                }
            }
            0x08 => InstructionType::Addi { rt, rs, imm },
            0x23 => InstructionType::Lw { rt, rs, imm },
            0x2B => InstructionType::Sw { rt, rs, imm },
            0x04 => InstructionType::Beq { rs, rt, imm },
            0x05 => InstructionType::Bne { rs, rt, imm },
            0x0A => InstructionType::Slti { rt, rs, imm },
            0x02 => InstructionType::J { addr },
            0x03 => InstructionType::Jal { addr },
            _ => InstructionType::Invalid,
        }
    }

    /// 命令のサイズを返す（MIPSは全て4バイト）
    #[allow(dead_code)]
    pub fn size(&self) -> usize {
        4
    }
}

// 命令の詳細を表示する

impl fmt::Display for InstructionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstructionType::Add { rd, rs, rt } => {
                write!(f, "add ${}, ${}, ${}", rd, rs, rt)
            }
            InstructionType::Sub { rd, rs, rt } => {
                write!(f, "sub ${}, ${}, ${}", rd, rs, rt)
            }
            InstructionType::And { rd, rs, rt } => {
                write!(f, "and ${}, ${}, ${}", rd, rs, rt)
            }
            InstructionType::Or { rd, rs, rt } => {
                write!(f, "or ${}, ${}, ${}", rd, rs, rt)
            }
            InstructionType::Slt { rd, rs, rt } => {
                write!(f, "slt ${}, ${}, ${}", rd, rs, rt)
            }
            InstructionType::Sll { rd, rt, shamt } => {
                write!(f, "sll ${}, ${}, {}", rd, rt, shamt)
            }
            InstructionType::Srl { rd, rt, shamt } => {
                write!(f, "srl ${}, ${}, {}", rd, rt, shamt)
            }
            InstructionType::Jr { rs } => {
                write!(f, "jr ${}", rs)
            }
            InstructionType::Addi { rt, rs, imm } => {
                write!(f, "addi ${}, ${}, {}", rt, rs, imm)
            }
            InstructionType::Lw { rt, rs, imm } => {
                write!(f, "lw ${}, {}({})", rt, imm, rs)
            }
            InstructionType::Sw { rt, rs, imm } => {
                write!(f, "sw ${}, {}({})", rt, imm, rs)
            }
            InstructionType::Beq { rs, rt, imm } => {
                write!(f, "beq ${}, ${}, {}", rs, rt, imm)
            }
            InstructionType::Bne { rs, rt, imm } => {
                write!(f, "bne ${}, ${}, {}", rs, rt, imm)
            }
            InstructionType::Slti { rt, rs, imm } => {
                write!(f, "slti ${}, ${}, {}", rt, rs, imm)
            }
            InstructionType::J { addr } => {
                write!(f, "j 0x{:08X}", addr << 2)
            }
            InstructionType::Jal { addr } => {
                write!(f, "jal 0x{:08X}", addr << 2)
            }
            InstructionType::Syscall => {
                write!(f, "syscall")
            }
            InstructionType::Invalid => {
                write!(f, "invalid")
            }
        }
    }
}

//　decodeが正常に機能しているかのテスト

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_add() {
        // add $1, $2, $3
        let instruction = 0x00430820u32;
        let decoded = InstructionType::decode(instruction);
        
        if let InstructionType::Add { rd, rs, rt } = decoded {
            assert_eq!(rd, 1);
            assert_eq!(rs, 2);
            assert_eq!(rt, 3);
        } else {
            panic!("Expected Add instruction");
        }
    }

    #[test]
    fn test_decode_addi() {
        // addi $1, $2, 100
        let instruction = 0x20410064u32;
        let decoded = InstructionType::decode(instruction);
        
        if let InstructionType::Addi { rt, rs, imm } = decoded {
            assert_eq!(rt, 1);
            assert_eq!(rs, 2);
            assert_eq!(imm, 100);
        } else {
            panic!("Expected Addi instruction");
        }
    }
}
