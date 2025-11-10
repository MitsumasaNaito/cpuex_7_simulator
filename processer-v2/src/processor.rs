//! MIPSプロセッサコアの実装

use std::fmt;

use crate::instructions::{Instruction, InstructionType, Register, Word};
use crate::memory::{Memory, MemoryAddress, MemoryError};
use crate::cache::{Cache, CacheStats};

/// MIPSプロセッサのレジスタ数
pub const REGISTER_COUNT: usize = 32;

/// プログラムカウンタの初期値
pub const PC_INITIAL: MemoryAddress = 0x00400000;

/// スタックポインタの初期値
pub const SP_INITIAL: MemoryAddress = 0x7FFFFFFC;

/// MIPSプロセッサ
#[derive(Debug)]
pub struct Processor {
    /// 汎用レジスタ（$0-$31）
    registers: [Word; REGISTER_COUNT],
    /// プログラムカウンタ
    pc: MemoryAddress,
    /// ハイレジスタ（乗除算用）
    hi: Word,
    /// ローレジスタ（乗除算用）
    lo: Word,
    /// メモリシステム
    memory: Memory,
    /// キャッシュシステム
    cache: Cache,
    /// 実行統計
    stats: ProcessorStats,
}

/// プロセッサ統計情報
#[derive(Debug, Clone, Default)]
pub struct ProcessorStats {
    /// 実行命令数
    pub instructions_executed: u64,
    /// 分岐命令数
    pub branches_taken: u64,
    /// ロード命令数
    pub loads_executed: u64,
    /// ストア命令数
    pub stores_executed: u64,
}

impl Processor {
    /// 新しいプロセッサを作成
    #[allow(dead_code)]
    pub fn new() -> Self {
        let mut processor = Self {
            registers: [0; REGISTER_COUNT],
            pc: PC_INITIAL,
            hi: 0,
            lo: 0,
            memory: Memory::new(),
            cache: Cache::new(),
            stats: ProcessorStats::default(),
        };
        
        // スタックポインタを初期化
        processor.registers[29] = SP_INITIAL; // $sp
        
        processor
    }

    /// 指定されたサイズのメモリでプロセッサを作成
    pub fn with_memory_size(memory_size: usize) -> Self {
        let mut processor = Self {
            registers: [0; REGISTER_COUNT],
            pc: PC_INITIAL,
            hi: 0,
            lo: 0,
            memory: Memory::with_size(memory_size),
            cache: Cache::new(),
            stats: ProcessorStats::default(),
        };
        
        // スタックポインタを初期化
        processor.registers[29] = SP_INITIAL; // $sp
        
        processor
    }

    /// レジスタの値を取得
    pub fn get_register(&self, reg: Register) -> Word {
        if reg == 0 {
            0 // $0は常に0
        } else {
            self.registers[reg as usize]
        }
    }

    /// レジスタに値を設定
    pub fn set_register(&mut self, reg: Register, value: Word) {
        if reg != 0 {
            self.registers[reg as usize] = value;
        }
    }

    /// プログラムカウンタを取得
    pub fn get_pc(&self) -> MemoryAddress {
        self.pc
    }

    /// プログラムカウンタを設定
    #[allow(dead_code)]
    pub fn set_pc(&mut self, pc: MemoryAddress) {
        self.pc = pc;
    }

    /// メモリから命令を読み込む
    pub fn fetch_instruction(&mut self) -> Result<Instruction, MemoryError> {
        println!("PC=0x{:08X} から命令をフェッチ", self.pc);
        let instruction = self.cache.read_word(&mut self.memory, self.pc)?;
        println!("フェッチした命令: 0x{:08X}", instruction);
        Ok(instruction)
    }

    /// 命令を実行
    pub fn execute_instruction(&mut self, instruction: Instruction) -> Result<bool, ProcessorError> {
        let instruction_type = InstructionType::decode(instruction);
        
        match instruction_type {
            InstructionType::Add { rd, rs, rt } => {
                let rs_val = self.get_register(rs);
                let rt_val = self.get_register(rt);
                self.set_register(rd, rs_val.wrapping_add(rt_val));
            }
            
            InstructionType::Sub { rd, rs, rt } => {
                let rs_val = self.get_register(rs);
                let rt_val = self.get_register(rt);
                self.set_register(rd, rs_val.wrapping_sub(rt_val));
            }
            
            InstructionType::And { rd, rs, rt } => {
                let rs_val = self.get_register(rs);
                let rt_val = self.get_register(rt);
                self.set_register(rd, rs_val & rt_val);
            }
            
            InstructionType::Or { rd, rs, rt } => {
                let rs_val = self.get_register(rs);
                let rt_val = self.get_register(rt);
                self.set_register(rd, rs_val | rt_val);
            }
            
            InstructionType::Slt { rd, rs, rt } => {
                let rs_val = self.get_register(rs) as i32;
                let rt_val = self.get_register(rt) as i32;
                self.set_register(rd, if rs_val < rt_val { 1 } else { 0 });
            }
            
            InstructionType::Sll { rd, rt, shamt } => {
                let rt_val = self.get_register(rt);
                self.set_register(rd, rt_val << shamt);
            }
            
            InstructionType::Srl { rd, rt, shamt } => {
                let rt_val = self.get_register(rt);
                self.set_register(rd, rt_val >> shamt);
            }
            
            InstructionType::Jr { rs } => {
                let rs_val = self.get_register(rs);
                self.pc = rs_val;
                self.stats.branches_taken += 1;
                return Ok(true); // 分岐が発生
            }
            
            InstructionType::Addi { rt, rs, imm } => {
                let rs_val = self.get_register(rs) as i32;
                let result = rs_val.wrapping_add(imm as i32) as u32;
                self.set_register(rt, result);
            }
            
            InstructionType::Lw { rt, rs, imm } => {
                let rs_val = self.get_register(rs);
                let address = rs_val.wrapping_add(imm as u32);
                let value = self.cache.read_word(&mut self.memory, address)
                    .map_err(|e| ProcessorError::MemoryError(e))?;
                self.set_register(rt, value);
                self.stats.loads_executed += 1;
            }
            
            InstructionType::Sw { rt, rs, imm } => {
                let rs_val = self.get_register(rs);
                let rt_val = self.get_register(rt);
                let address = rs_val.wrapping_add(imm as u32);
                self.cache.write_word(&mut self.memory, address, rt_val)
                    .map_err(|e| ProcessorError::MemoryError(e))?;
                self.stats.stores_executed += 1;
            }
            
            InstructionType::Beq { rs, rt, imm } => {
                let rs_val = self.get_register(rs);
                let rt_val = self.get_register(rt);
                if rs_val == rt_val {
                    self.pc = self.pc.wrapping_add((imm as i32 * 4) as u32);
                    self.stats.branches_taken += 1;
                    return Ok(true); // 分岐が発生
                }
            }
            
            InstructionType::Bne { rs, rt, imm } => {
                let rs_val = self.get_register(rs);
                let rt_val = self.get_register(rt);
                if rs_val != rt_val {
                    self.pc = self.pc.wrapping_add((imm as i32 * 4) as u32);
                    self.stats.branches_taken += 1;
                    return Ok(true); // 分岐が発生
                }
            }
            
            InstructionType::Slti { rt, rs, imm } => {
                let rs_val = self.get_register(rs) as i32;
                self.set_register(rt, if rs_val < imm as i32 { 1 } else { 0 });
            }
            
            InstructionType::J { addr } => {
                println!(
                    "[JUMP] From: 0x{:08X}, To: 0x{:08X} (addr field: 0x{:07X})",
                    self.pc, (self.pc & 0xF0000000) | (addr << 2), addr
                );
                self.pc = (self.pc & 0xF0000000) | (addr << 2);
                self.stats.branches_taken += 1;
                return Ok(true); // 分岐が発生
            }
            
            InstructionType::Jal { addr } => {
                self.set_register(31, self.pc + 4); // $raに戻りアドレスを保存
                self.pc = (self.pc & 0xF0000000) | (addr << 2);
                self.stats.branches_taken += 1;
                return Ok(true); // 分岐が発生
            }
            
            InstructionType::Syscall => {
                // システムコールの実装
                // self.set_register(2, 10); // $v0 = 10 (exit syscall)
                let syscall_number = self.get_register(2); // $v0レジスタからシステムコール番号を取得
                println!("Syscall実行: $v0 = {} (syscall番号: {})", self.get_register(2), syscall_number);
                
                match syscall_number {
                    1 => {
                        // print_int: $a0レジスタの値を整数として出力
                        let value = self.get_register(4); // $a0レジスタ
                        println!("{}", value as i32);
                    }
                    4 => {
                        // print_string: $a0レジスタのアドレスから文字列を出力
                        let address = self.get_register(4); // $a0レジスタ
                        self.print_string(address)?;
                    }
                    10 => {
                        // exit: プログラム終了
                        println!("プログラムが終了しました");
                        return Err(ProcessorError::ProgramEnd); // プログラム終了
                    }
                    11 => {
                        // print_char: $a0レジスタの値を文字として出力
                        let value = self.get_register(4); // $a0レジスタ
                        print!("{}", value as u8 as char);
                    }
                    _ => {
                        println!("未対応のシステムコール: {}", syscall_number);
                        return Err(ProcessorError::InvalidInstruction(instruction));
                    }
                }
            }
            
            InstructionType::Invalid => {
                return Err(ProcessorError::InvalidInstruction(instruction));
            }
        }
        
        // 通常の命令の場合、分岐なしを返す
        Ok(false) // 分岐なし
    }

    /// 1命令を実行（フェッチ + 実行）
    pub fn step(&mut self) -> Result<bool, ProcessorError> {
        let instruction = self.fetch_instruction()
            .map_err(|e| {
                println!("命令フェッチエラー: PC=0x{:08X}, エラー={}", self.pc, e);
                ProcessorError::MemoryError(e)
            })?;
        
        let instruction_type = InstructionType::decode(instruction);
        println!("実行: 0x{:08X} ({})", instruction, instruction_type);
        
        let branch_taken = self.execute_instruction(instruction)
            .map_err(|e| {
                println!("命令実行エラー: PC=0x{:08X}, エラー={}", self.pc, e);
                e
            })?;
        
        // 分岐が発生しなかった場合のみPCを4進める
        if !branch_taken {
            self.pc = self.pc.wrapping_add(4);
            self.stats.instructions_executed += 1;
        }
        
        Ok(branch_taken)
    }

    /// プログラムを実行（無限ループまたはシステムコールまで）
    pub fn run(&mut self) -> Result<(), ProcessorError> {
        let mut instruction_count = 0;
        loop {
            // 無効なアドレスの場合は終了
            if self.pc == 0xFFFFFFFF {
                break;
            }
            
            // デバッグ出力
            if instruction_count < 10 {
                println!("命令 {}: PC=0x{:08X}", instruction_count, self.pc);
            }
            
            match self.step()? {
                true => {
                    println!("分岐が発生: PC=0x{:08X}", self.pc);
                    continue; // 分岐が発生した場合
                }
                false => {
                    // システムコールの場合は終了
                    // println!("self.registers[2] = {}", self.registers[2]);
                    // エラーでなくループを抜けることによってプログラムを終了させるように修正したい！！
                    if self.registers[2] == 10 {
                        println!("システムコールで終了\n\n");
                        println!("=== 計算結果 ===\n");
                        println!("{}", self.get_register(4));
                        break;
                    }
                    // 通常の命令の場合は次のループで続行
                }
            }
            
            instruction_count += 1;
            
            // 安全のため、1000命令で強制終了
            if instruction_count > 100000 {
                println!("警告: 100000命令を超えました。強制終了します。");
                break;
            }
        }
        Ok(())
    }

    /// メモリにプログラムをロード
    pub fn load_program(&mut self, program: &[Instruction], start_address: MemoryAddress) -> Result<(), MemoryError> {
        println!("プログラムをロード中: {} 命令", program.len());
        for (i, instruction) in program.iter().enumerate() {
            let address = start_address + (i * 4) as u32;
            println!("命令 {}: 0x{:08X} をアドレス 0x{:08X} に書き込み", i, instruction, address);
            self.memory.write_instruction(address, *instruction)?;
        }
        self.pc = start_address;
        println!("PCを 0x{:08X} に設定", self.pc);
        Ok(())
    }

    /// メモリからデータを読み込む
    #[allow(dead_code)]
    pub fn read_memory(&self, address: MemoryAddress) -> Result<Word, MemoryError> {
        self.memory.read_word(address)
    }

    /// メモリにデータを書き込む
    #[allow(dead_code)]
    pub fn write_memory(&mut self, address: MemoryAddress, value: Word) -> Result<(), MemoryError> {
        self.memory.write_word(address, value)
    }

    /// プロセッサの状態をダンプ
    pub fn dump_state(&self) -> String {
        let mut result = String::new();
        
        result.push_str("=== プロセッサ状態 ===\n");
        result.push_str(&format!("PC: 0x{:08X}\n", self.pc));
        result.push_str(&format!("HI: 0x{:08X}\n", self.hi));
        result.push_str(&format!("LO: 0x{:08X}\n", self.lo));
        result.push_str("\n=== レジスタ ===\n");
        
        for i in 0..REGISTER_COUNT {
            let reg_name = match i {
                0 => "$zero",
                1 => "$at",
                2 => "$v0", 3 => "$v1",
                4 => "$a0", 5 => "$a1", 6 => "$a2", 7 => "$a3",
                8 => "$t0", 9 => "$t1", 10 => "$t2", 11 => "$t3",
                12 => "$t4", 13 => "$t5", 14 => "$t6", 15 => "$t7",
                16 => "$s0", 17 => "$s1", 18 => "$s2", 19 => "$s3",
                20 => "$s4", 21 => "$s5", 22 => "$s6", 23 => "$s7",
                24 => "$t8", 25 => "$t9",
                26 => "$k0", 27 => "$k1",
                28 => "$gp", 29 => "$sp", 30 => "$fp", 31 => "$ra",
                _ => "???",
            };
            
            result.push_str(&format!("{}: 0x{:08X} ({})\n", 
                reg_name, self.registers[i], self.registers[i] as i32));
        }
        
        result.push_str(&format!("\n=== 統計情報 ===\n{}", self.stats));
        result.push_str(&format!("\n=== キャッシュ統計 ===\n{}", self.cache.get_stats()));
        
        result
    }

    /// 統計情報を取得
    pub fn get_stats(&self) -> &ProcessorStats {
        &self.stats
    }

    /// キャッシュ統計を取得
    pub fn get_cache_stats(&self) -> &CacheStats {
        self.cache.get_stats()
    }

    /// 統計情報をリセット
    #[allow(dead_code)]
    pub fn reset_stats(&mut self) {
        self.stats = ProcessorStats::default();
        self.cache.reset_stats();
    }

    /// 文字列を出力（システムコール用）
    fn print_string(&self, address: MemoryAddress) -> Result<(), MemoryError> {
        let mut current_addr = address;
        let mut result = String::new();
        
        loop {
            let byte = self.memory.read_byte(current_addr)?;
            if byte == 0 {
                break; // null文字で終了
            }
            result.push(byte as char);
            current_addr += 1;
        }
        
        print!("{}", result);
        Ok(())
    }
}

/// プロセッサエラー
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessorError {
    MemoryError(MemoryError),
    InvalidInstruction(Instruction),
    ProgramEnd,
}

impl From<MemoryError> for ProcessorError {
    fn from(err: MemoryError) -> Self {
        ProcessorError::MemoryError(err)
    }
}

impl fmt::Display for ProcessorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessorError::MemoryError(e) => write!(f, "メモリエラー: {}", e),
            ProcessorError::InvalidInstruction(inst) => write!(f, "無効な命令: 0x{:08X}", inst),
            ProcessorError::ProgramEnd => write!(f, "プログラムが終了しました"),
        }
    }
}

impl std::error::Error for ProcessorError {}

impl fmt::Display for ProcessorStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "実行命令数: {}\n", self.instructions_executed)?;
        write!(f, "分岐命令数: {}\n", self.branches_taken)?;
        write!(f, "ロード命令数: {}\n", self.loads_executed)?;
        write!(f, "ストア命令数: {}", self.stores_executed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_creation() {
        let processor = Processor::new();
        assert_eq!(processor.get_pc(), PC_INITIAL);
        assert_eq!(processor.get_register(0), 0); // $zeroは常に0
        assert_eq!(processor.get_register(29), SP_INITIAL); // $sp
    }

    #[test]
    fn test_register_operations() {
        let mut processor = Processor::new();
        
        processor.set_register(1, 0x12345678);
        assert_eq!(processor.get_register(1), 0x12345678);
        
        // $0は変更できない
        processor.set_register(0, 0xFFFFFFFF);
        assert_eq!(processor.get_register(0), 0);
    }

    #[test]
    fn test_add_instruction() {
        let mut processor = Processor::new();
        
        // add $1, $2, $3
        let instruction = 0x00430820u32;
        processor.set_register(2, 10);
        processor.set_register(3, 20);
        
        processor.execute_instruction(instruction).unwrap();
        assert_eq!(processor.get_register(1), 30);
    }
}
