mod instructions;
mod memory;
mod cache;
mod processor;

use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

use instructions::{Instruction, InstructionType};
use memory::MemoryAddress;
use processor::{Processor, ProcessorError};

/// シミュレータの設定
//　ここで定義してdefault()で呼び出せるようにすることで、設定の変更が容易になり、拡張性が上がる
#[derive(Debug, Clone)]
pub struct SimulatorConfig {
    /// メモリサイズ（バイト）
    pub memory_size: usize,
    /// プログラムの開始アドレス
    pub program_start: MemoryAddress,
    /// デバッグモード
    pub debug_mode: bool,
    /// ステップ実行モード
    pub step_mode: bool,
}

impl Default for SimulatorConfig {
    fn default() -> Self {
        Self {
            memory_size: 16 * 1024 * 1024, // 16MB
            program_start: 0x00400000,
            debug_mode: false,
            step_mode: false,
        }
    }
}

/// MIPSシミュレータ
#[derive(Debug)]
pub struct MipsSimulator {
    processor: Processor,
    config: SimulatorConfig,
}

impl MipsSimulator {
    /// 新しいシミュレータを作成
    pub fn new(config: SimulatorConfig) -> Self {
        let processor = Processor::with_memory_size(config.memory_size);
        Self {
            processor,
            config,
        }
    }
    /// デフォルト設定でシミュレータを作成
    pub fn new_default() -> Self {
        Self::new(SimulatorConfig::default())
    }
    /// プログラムをファイルから読み込む
    // Pというジェネリック型を定義し、「PはPath（ファイルパス）として参照できる型なら何でも良い」という制約（AsRef<Path>）を付けています。
    // これにより、この関数を呼び出す側は、ファイルパスを様々な形式で渡せるようになり、利用者の使いやすさ（エルゴノミクス）を非常に高めます。
    // 成功すれば()（中身は空）、失敗すればSimulatorError（エラーの種類を示す列挙型）を返す
    pub fn load_program_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), SimulatorError> {
        // ?演算子は、Result型に対して以下の処理を自動で行います。
        // もし結果が成功 (Ok(値)) なら、Okを剥がして中の値だけを取り出す。
        // もし結果が失敗 (Err(エラー)) なら、その場でこの関数を終了し、中のエラーを関数の呼び出し元に返す
        let file = File::open(path).map_err(|e| SimulatorError::FileError(format!("ファイルを開けません: {}", e)))?;
        // Fileから直接1行ずつ読むと、そのたびにOSを呼び出す必要があり、パフォーマンスが低下します。
        // BufReaderは、最初にある程度の大きさの塊をまとめてメモリ上のバッファに読み込み、その後は高速なメモリから1行ずつ読み出します。
        // これにより、OSの呼び出し回数が劇的に減り、処理が高速になります。
        let reader = BufReader::new(file);

        // 命令を格納するベクターを宣言
        // let mut instructions = vec![];(マクロ呼び出し)と同じ
        let mut instructions = Vec::new();
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| SimulatorError::FileError(format!("inputfileの {} 行目を読み込めませんでした: {}", line_num + 1, e)))?;
            let line = line.trim();
            // 空行またはコメントをスキップ
            if line.is_empty() || line.starts_with('#') { continue };
            // 16進数部分のみを抽出（コメントの前まで）
            // if letで条件が成立した場合のみcomment_posが宣言され、そのスコープはif文内のみである
            let hex_part = if let Some(comment_pos) = line.find('#') {
                line[..comment_pos].trim()
            } else {
                line
            };
            // まだ文字列なので16進数に変換
            let instruction = u32::from_str_radix(hex_part, 16).map_err(|e| SimulatorError::ParseError(format!("inputfileの {} 行目: 文字列から16進数への変換に失敗しました: '{}': {}", line_num + 1, hex_part, e)))?;
            instructions.push(instruction);
        }
        self.load_program(&instructions)
    }
    /// プログラムをメモリにロード
    pub fn load_program(&mut self, program: &[Instruction]) -> Result<(), SimulatorError> {
        self.processor.load_program(program, self.config.program_start).map_err(|e| SimulatorError::MemoryError(e))?;
        Ok(())
    }
    /// シミュレータを実行
    pub fn run(&mut self) -> Result<(), SimulatorError> {
        if self.config.step_mode {
            self.run_step_mode()?
        } else {
            self.processor.run().map_err(|e| SimulatorError::ProcessorError(e))?;
        }
        Ok(())
    }
    /// ステップ実行モードで実行
    fn run_step_mode(&mut self) -> Result<(), SimulatorError> {
        let mut step_count = 0;
        loop {
            println!("\n=== ステップ {} ===", step_count);
            println!("PC: 0x{:08X}", self.processor.get_pc());

            // 現在の命令を表示
            let instruction = self.processor.fetch_instruction().map_err(|e| SimulatorError::MemoryError(e))?;
            let instruction_type = InstructionType::decode(instruction);
            println!("命令: 0x{:08X} ({})", instruction, instruction_type);
            
            // ユーザー入力を待つ
            print!("実行しますか？ (Enter: 実行, 'q': 終了, 's': 状態表示): ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            // 標準入力から一行読み込み、inputに格納。.unwrap() は、読み込み中にエラーが発生した場合にプログラムを停止させる
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();
            
            match input {
                "q" => break,
                "s" => {
                    println!("{}", self.processor.dump_state());
                    continue;
                }
                _ => {
                    // 命令を実行
                    match self.processor.step() {
                        Ok(branch_taken) => {
                            if branch_taken {
                                println!("分岐が発生しました");
                            }
                        }
                        Err(e) => {
                            println!("エラー: {}", e);
                            break;
                        }
                    }
                }
            }
            step_count += 1;
        }
        Ok(())
    }

    /// プロセッサの状態を取得
    pub fn get_processor_state(&self) -> String {
        self.processor.dump_state()
    }

    /// 統計情報を取得
    pub fn get_stats(&self) -> &processor::ProcessorStats {
        self.processor.get_stats()
    }

    /// キャッシュ統計を取得
    pub fn get_cache_stats(&self) -> &cache::CacheStats {
        self.processor.get_cache_stats()
    }

    /// 設定を取得
    pub fn get_config(&self) -> &SimulatorConfig {
        &self.config
    }

    /// 設定を更新
    pub fn set_config(&mut self, config: SimulatorConfig) {
        self.config = config;
    }
}

/// シミュレータエラー
//std::fmt::Display	{}	最終ユーザー向け。エラーの「ユーザーフレンドリーな簡潔な説明」を提供します。
//std::fmt::Debug	{:?}	開発者向け。デバッグ用の「構造的な詳細情報」を提供します。
#[derive(Debug, Clone)]
pub enum SimulatorError {
    FileError(String),
    ParseError(String),
    MemoryError(memory::MemoryError),
    ProcessorError(ProcessorError),
}

impl std::fmt::Display for SimulatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimulatorError::FileError(msg) => write!(f, "ファイルエラー: {}", msg),
            SimulatorError::ParseError(msg) => write!(f, "解析エラー: {}", msg),
            SimulatorError::MemoryError(e) => write!(f, "メモリエラー: {}", e),
            SimulatorError::ProcessorError(e) => write!(f, "プロセッサエラー: {}", e),
        }
    }
}

//Display トレイト（エラーをユーザーフレンドリーに表示する）の実装が既にあるため、std::error::Error トレイトの実装は形式的なものになっている
impl std::error::Error for SimulatorError {}

/// コマンドライン引数を解析
fn parse_args(args: &[String]) -> Result<(SimulatorConfig, Option<String>), String> {
    let mut config = SimulatorConfig::default();
    let mut i = 1; // ./mainをスキップ
    let mut program_file = None;

    while i < args.len() {
        match args[i].as_str() {
            "--memory-size" | "-m" => {
                if i + 1 >= args.len() {
                    return Err("--memory-size には値が必要です".to_string());
                }
                config.memory_size = args[i + 1].parse()
                    .map_err(|_| "無効なメモリサイズです".to_string())?;
                i += 2;
            }
            "--debug" | "-d" => {
                config.debug_mode = true;
                i += 1;
            }
            "--step" | "-s" => {
                config.step_mode = true;
                i += 1;
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            arg => {
                // オプションでない場合はプログラムファイルとして扱う
                if arg.starts_with('-') {
                    return Err(format!("このオプションは定義されていません: {}", arg));
                }
                // program_fileがOption型であることを明示するためにSome(arg.to_string())とする
                program_file = Some(arg.to_string());
                i += 1;
            }
        }
    }
    Ok((config, program_file))
}

/// 使用方法を表示
fn print_usage() {
    println!("MIPSプロセッサシミュレータ");
    println!();
    println!("使用方法: {} [オプション] <プログラムファイル>", std::env::args().next().unwrap_or("mips_simulator".to_string()));
    println!();
    println!("オプション:");
    println!("  -m, --memory-size <サイズ>  メモリサイズを指定（バイト単位）");
    println!("  -d, --debug                 デバッグモードで実行");
    println!("  -s, --step                  ステップ実行モードで実行");
    println!("  -h, --help                  このヘルプを表示");
    println!();
    println!("例:");
    println!("  {} fibonacci.hex", std::env::args().next().unwrap_or("mips_simulator".to_string()));
    println!("  {} -d -s fibonacci.hex", std::env::args().next().unwrap_or("mips_simulator".to_string()));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    let (config, program_file) = match parse_args(&args) {
        Ok((config, program_file)) => (config, program_file),
        Err(e) => {
            eprintln!("エラー: コマンドライン引数の解析に失敗しました: {}", e);
            print_usage();
            std::process::exit(1);
        }
    };
    
    let program_file = match program_file {
        Some(file) => file,
        None => {
            eprintln!("エラー: プログラムファイルがコマンドライン引数で指定されていません");
            print_usage();
            std::process::exit(1);
        }
    };
    
    let mut simulator = MipsSimulator::new(config);
    
    // プログラムを読み込み
    match simulator.load_program_from_file(&program_file) {
        Ok(()) => {
            if simulator.get_config().debug_mode {
                println!("プログラム '{}' を読み込みました", program_file);
            }
        }
        Err(e) => {
            eprintln!("エラー: プログラムの読み込みに失敗しました: {}", e);
            std::process::exit(1);
        }
    }
    
    // シミュレータを実行
    match simulator.run() {
        Ok(()) => {
            println!("{}", simulator.get_processor_state());
        }
        Err(e) => {
            eprintln!("エラー: シミュレーション中にエラーが発生しました: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulator_creation() {
        let config = SimulatorConfig::default();
        let simulator = MipsSimulator::new(config);
        assert_eq!(simulator.get_config().memory_size, 16 * 1024 * 1024);
    }

    #[test]
    fn test_load_program() {
        let mut simulator = MipsSimulator::new_default();
        let program = vec![0x00430820u32, 0x0000000Cu32]; // add $1, $2, $3; syscall
        simulator.load_program(&program).unwrap();
    }
}
