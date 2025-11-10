//! メモリシステムの実装

use std::fmt;

use crate::Instruction; 

/// メモリアドレス（バイト単位）
pub type MemoryAddress = u32;

/// メモリのサイズ（バイト単位）
#[allow(dead_code)]
pub const MEMORY_SIZE: usize = 1024 * 1024; // 1MB

/// メモリシステム
#[derive(Debug, Clone)]
pub struct Memory {
    /// メモリデータ（バイト配列）
    data: Vec<u8>,
}

impl Memory {
    /// 新しいメモリシステムを作成
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            data: vec![0; MEMORY_SIZE],
        }
    }

    /// 指定されたサイズのメモリを作成
    pub fn with_size(size: usize) -> Self {
        Self {
            data: vec![0; size],
        }
    }

    /// メモリにバイトを書き込む
    #[allow(dead_code)]
    pub fn write_byte(&mut self, address: MemoryAddress, value: u8) -> Result<(), MemoryError> {
        if address as usize >= self.data.len() {
            return Err(MemoryError::AddressOutOfRange(address));
        }
        self.data[address as usize] = value;
        Ok(())
    }

    /// メモリからバイトを読み込む
    #[allow(dead_code)]
    pub fn read_byte(&self, address: MemoryAddress) -> Result<u8, MemoryError> {
        if address as usize >= self.data.len() {
            return Err(MemoryError::AddressOutOfRange(address));
        }
        Ok(self.data[address as usize])
    }

    /// メモリにワード（32ビット）を書き込む（リトルエンディアン）
    pub fn write_word(&mut self, address: MemoryAddress, value: Word) -> Result<(), MemoryError> {
        if (address as usize).saturating_add(3) >= self.data.len() {
            return Err(MemoryError::AddressOutOfRange(address));
        }
        
        let addr = address as usize;
        self.data[addr] = (value & 0xFF) as u8;
        self.data[addr + 1] = ((value >> 8) & 0xFF) as u8;
        self.data[addr + 2] = ((value >> 16) & 0xFF) as u8;
        self.data[addr + 3] = ((value >> 24) & 0xFF) as u8;
        Ok(())
    }

    /// メモリからワード（32ビット）を読み込む（リトルエンディアン）
    #[allow(dead_code)]
    pub fn read_word(&self, address: MemoryAddress) -> Result<Word, MemoryError> {
        if (address as usize).saturating_add(3) >= self.data.len() {
            return Err(MemoryError::AddressOutOfRange(address));
        }
        
        let addr = address as usize;
        let word = (self.data[addr] as Word)
            | ((self.data[addr + 1] as Word) << 8)
            | ((self.data[addr + 2] as Word) << 16)
            | ((self.data[addr + 3] as Word) << 24);
        Ok(word)
    }

    /// メモリに命令を書き込む
    pub fn write_instruction(&mut self, address: MemoryAddress, instruction: Instruction) -> Result<(), MemoryError> {
        self.write_word(address, instruction)
    }

    /// メモリから命令を読み込む
    #[allow(dead_code)]
    pub fn read_instruction(&self, address: MemoryAddress) -> Result<Instruction, MemoryError> {
        self.read_word(address)
    }

    /// メモリの指定範囲をクリア
    #[allow(dead_code)]
    pub fn clear_range(&mut self, start: MemoryAddress, end: MemoryAddress) -> Result<(), MemoryError> {
        if start as usize >= self.data.len() || end as usize >= self.data.len() || start > end {
            return Err(MemoryError::AddressOutOfRange(start));
        }
        
        for addr in start..=end {
            self.data[addr as usize] = 0;
        }
        Ok(())
    }

    /// メモリのサイズを取得
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// メモリの内容をダンプ（デバッグ用）
    #[allow(dead_code)]
    pub fn dump(&self, start: MemoryAddress, length: usize) -> String {
        let mut result = String::new();
        let end = std::cmp::min(start as usize + length, self.data.len());
        
        for i in (start as usize..end).step_by(16) {
            result.push_str(&format!("{:08X}: ", i));
            
            // 16バイト分の16進数表示
            for j in 0..16 {
                if i + j < end {
                    result.push_str(&format!("{:02X} ", self.data[i + j]));
                } else {
                    result.push_str("   ");
                }
            }
            
            result.push_str(" |");
            
            // ASCII文字表示
            for j in 0..16 {
                if i + j < end {
                    let byte = self.data[i + j];
                    if byte >= 32 && byte <= 126 {
                        result.push(byte as char);
                    } else {
                        result.push('.');
                    }
                } else {
                    result.push(' ');
                }
            }
            
            result.push_str("|\n");
        }
        
        result
    }
}

/// メモリエラー
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryError {
    AddressOutOfRange(MemoryAddress),
}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryError::AddressOutOfRange(addr) => {
                write!(f, "メモリアドレス 0x{:08X} が範囲外です", addr)
            }
        }
    }
}

impl std::error::Error for MemoryError {}

/// 32ビットのワード型
pub type Word = u32;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_read_write_byte() {
        let mut memory = Memory::new();
        
        // バイトの書き込みと読み込み
        memory.write_byte(0x1000, 0xAB).unwrap();
        assert_eq!(memory.read_byte(0x1000).unwrap(), 0xAB);
    }

    #[test]
    fn test_memory_read_write_word() {
        let mut memory = Memory::new();
        
        // ワードの書き込みと読み込み
        memory.write_word(0x1000, 0x12345678).unwrap();
        assert_eq!(memory.read_word(0x1000).unwrap(), 0x12345678);
    }

    #[test]
    fn test_memory_address_out_of_range() {
        let memory = Memory::new();
        
        // 範囲外のアドレスアクセス
        assert!(memory.read_byte(MEMORY_SIZE as u32).is_err());
    }

    #[test]
    fn test_memory_instruction_read_write() {
        let mut memory = Memory::new();
        
        // 命令の書き込みと読み込み
        let instruction = 0x00430820u32; // add $1, $2, $3
        memory.write_instruction(0x1000, instruction).unwrap();
        assert_eq!(memory.read_instruction(0x1000).unwrap(), instruction);
    }
}
