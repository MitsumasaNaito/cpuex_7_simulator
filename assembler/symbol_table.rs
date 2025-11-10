use std::collections::HashMap;
use std::io::Error;

pub fn make_symbol_table(symbol_table: &mut HashMap<String, i32>, line: &str, line_number: i32) -> Result<(), Error> {
    let label = match line.split_once(':') {
        Some((left, _)) => left,
        None => "",
    };
    if label != "" {
        symbol_table.insert(String::from(label.trim()), line_number);
    }
    Ok(())
}