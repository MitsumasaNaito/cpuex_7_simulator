use std::io::{Error, ErrorKind};
use crate::Instruction;

pub fn parse_mips_line(line: &str) -> Result<Instruction, Error>{
    // ラベルを取得
    let label = match line.split_once(':') {
        Some((left, _)) => left,
        None => "",
    };
    let line = match line.split_once(':') {
        Some((_, right)) => right,
        None => line,
    };

    // コメントを削除
    let line = line.split('#').next().unwrap_or("").trim();
    // 空白とカンマとカッコで分割
    let parts: Vec<&str> = line.split(|c| c == ' ' || c == ',' || c == '(' || c == ')').filter(|s| !s.is_empty()).collect();
    if parts.len() < 2 && parts[0] != "syscall" {
        return Err(Error::new(ErrorKind::InvalidInput, "Invalid MIPS line"));
    }
    let opecode = parts[0];
    let operands: Vec<String> = parts[1..].iter().map(|&s| s.to_string()).collect();
    Ok(Instruction { label: label.to_string(), opecode: opecode.to_string(), operands: operands })
}

