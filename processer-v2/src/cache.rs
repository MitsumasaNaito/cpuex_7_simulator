//! キャッシュシステムの実装

use std::fmt;

use crate::memory::{Memory, MemoryAddress, Word, MemoryError};

/// キャッシュラインのサイズ（バイト単位）
pub const CACHE_LINE_SIZE: usize = 32;

/// キャッシュのセット数
pub const CACHE_SETS: usize = 64;

/// キャッシュの連想度（ウェイ数）
pub const CACHE_WAYS: usize = 4;

/// キャッシュライン
#[derive(Debug, Clone)]
struct CacheLine {
    /// 有効ビット
    valid: bool,
    /// ダーティビット（書き込み済みかどうか）
    dirty: bool,
    /// タグ
    tag: u32,
    /// データ
    data: [u8; CACHE_LINE_SIZE],
    /// アクセス時刻（LRU用）
    access_time: u64,
}

impl CacheLine {
    fn new() -> Self {
        Self {
            valid: false,
            dirty: false,
            tag: 0,
            data: [0; CACHE_LINE_SIZE],
            access_time: 0,
        }
    }
}

/// キャッシュセット
#[derive(Debug, Clone)]
struct CacheSet {
    lines: [CacheLine; CACHE_WAYS],
}

impl CacheSet {
    fn new() -> Self {
        Self {
            lines: [
                CacheLine::new(),
                CacheLine::new(),
                CacheLine::new(),
                CacheLine::new(),
            ],
        }
    }
}

/// キャッシュ統計情報
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// ヒット数
    pub hits: u64,
    /// ミス数
    pub misses: u64,
    /// 書き込みバック数
    pub writebacks: u64,
}

impl CacheStats {
    /// ヒット率を計算
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// キャッシュシステム
#[derive(Debug)]
pub struct Cache {
    /// キャッシュセット
    sets: [CacheSet; CACHE_SETS],
    /// 統計情報
    stats: CacheStats,
    /// アクセス時刻カウンタ
    access_counter: u64,
}

impl Cache {
    /// 新しいキャッシュシステムを作成
    pub fn new() -> Self {
        Self {
            sets: [(); CACHE_SETS].map(|_| CacheSet::new()),
            stats: CacheStats::default(),
            access_counter: 0,
        }
    }

    /// アドレスからセットインデックスを計算
    fn get_set_index(&self, address: MemoryAddress) -> usize {
        ((address as usize) / CACHE_LINE_SIZE) % CACHE_SETS
    }

    /// アドレスからタグを計算
    fn get_tag(&self, address: MemoryAddress) -> u32 {
        ((address as usize) / CACHE_LINE_SIZE) as u32 / CACHE_SETS as u32
    }

    /// アドレスからキャッシュライン内オフセットを計算
    fn get_offset(&self, address: MemoryAddress) -> usize {
        (address as usize) % CACHE_LINE_SIZE
    }

    /// 指定されたタグのキャッシュラインを検索
    fn find_line(&mut self, set_index: usize, tag: u32) -> Option<usize> {
        let set = &mut self.sets[set_index];
        for (i, line) in set.lines.iter_mut().enumerate() {
            if line.valid && line.tag == tag {
                line.access_time = self.access_counter;
                self.access_counter += 1;
                return Some(i);
            }
        }
        None
    }

    /// LRUでキャッシュラインを選択
    fn select_lru_line(&mut self, set_index: usize) -> usize {
        let set = &mut self.sets[set_index];
        let mut lru_index = 0;
        let mut oldest_time = set.lines[0].access_time;
        
        for (i, line) in set.lines.iter().enumerate() {
            if !line.valid {
                return i; // 無効なラインがあればそれを使用
            }
            if line.access_time < oldest_time {
                oldest_time = line.access_time;
                lru_index = i;
            }
        }
        
        lru_index
    }

    /// キャッシュラインをメモリに書き戻す
    #[allow(dead_code)]
    fn writeback_line(&mut self, memory: &mut Memory, set_index: usize, way_index: usize) -> Result<(), MemoryError> {
        let line = &self.sets[set_index].lines[way_index];
        if !line.dirty {
            return Ok(());
        }

        let base_address = ((line.tag * CACHE_SETS as u32 + set_index as u32) * CACHE_LINE_SIZE as u32) as MemoryAddress;
        
        for i in 0..CACHE_LINE_SIZE {
            memory.write_byte(base_address + i as u32, line.data[i])?;
        }
        
        self.stats.writebacks += 1;
        Ok(())
    }

    /// メモリからキャッシュラインを読み込む
    fn load_line(&mut self, memory: &Memory, set_index: usize, way_index: usize, tag: u32) -> Result<(), MemoryError> {
        let base_address = ((tag * CACHE_SETS as u32 + set_index as u32) * CACHE_LINE_SIZE as u32) as MemoryAddress;
        let line = &mut self.sets[set_index].lines[way_index];
        
        for i in 0..CACHE_LINE_SIZE {
            line.data[i] = memory.read_byte(base_address + i as u32)?;
        }
        
        line.valid = true;
        line.dirty = false;
        line.tag = tag;
        line.access_time = self.access_counter;
        self.access_counter += 1;
        
        Ok(())
    }

    /// バイトを読み込む
    pub fn read_byte(&mut self, memory: &mut Memory, address: MemoryAddress) -> Result<u8, MemoryError> {
        let set_index = self.get_set_index(address);
        let tag = self.get_tag(address);
        let offset = self.get_offset(address);

        if let Some(way_index) = self.find_line(set_index, tag) {
            // キャッシュヒット
            self.stats.hits += 1;
            Ok(self.sets[set_index].lines[way_index].data[offset])
        } else {
            // キャッシュミス
            self.stats.misses += 1;
            let way_index = self.select_lru_line(set_index);
            
            // 既存のラインがダーティなら書き戻し
            if self.sets[set_index].lines[way_index].valid && self.sets[set_index].lines[way_index].dirty {
                self.writeback_line(memory, set_index, way_index)?;
            }
            
            // メモリからラインを読み込み
            self.load_line(memory, set_index, way_index, tag)?;
            Ok(self.sets[set_index].lines[way_index].data[offset])
        }
    }

    /// バイトを書き込む
    pub fn write_byte(&mut self, memory: &mut Memory, address: MemoryAddress, value: u8) -> Result<(), MemoryError> {
        let set_index = self.get_set_index(address);
        let tag = self.get_tag(address);
        let offset = self.get_offset(address);

        if let Some(way_index) = self.find_line(set_index, tag) {
            // キャッシュヒット
            self.stats.hits += 1;
            self.sets[set_index].lines[way_index].data[offset] = value;
            self.sets[set_index].lines[way_index].dirty = true;
        } else {
            // キャッシュミス
            self.stats.misses += 1;
            let way_index = self.select_lru_line(set_index);
            
            // 既存のラインがダーティなら書き戻し
            if self.sets[set_index].lines[way_index].valid && self.sets[set_index].lines[way_index].dirty {
                self.writeback_line(memory, set_index, way_index)?;
            }
            
            // 新しいラインを初期化
            let line = &mut self.sets[set_index].lines[way_index];
            line.valid = true;
            line.dirty = true;
            line.tag = tag;
            line.access_time = self.access_counter;
            self.access_counter += 1;
            
            // データを書き込み
            line.data[offset] = value;
        }
        
        Ok(())
    }

    /// ワードを読み込む
    pub fn read_word(&mut self, memory: &mut Memory, address: MemoryAddress) -> Result<Word, MemoryError> {
        // 4バイトの境界チェック
        if (address as usize).saturating_add(3) >= memory.size() {
            return Err(MemoryError::AddressOutOfRange(address));
        }
        
        // 4バイトを個別に読み込んでワードを構築
        let b0 = self.read_byte(memory, address)? as u32;
        let b1 = self.read_byte(memory, address + 1)? as u32;
        let b2 = self.read_byte(memory, address + 2)? as u32;
        let b3 = self.read_byte(memory, address + 3)? as u32;
        
        Ok((b3 << 24) | (b2 << 16) | (b1 << 8) | b0)
    }

    /// ワードを書き込む
    pub fn write_word(&mut self, memory: &mut Memory, address: MemoryAddress, value: Word) -> Result<(), MemoryError> {
        // 4バイトの境界チェック
        if (address as usize).saturating_add(3) >= memory.size() {
            return Err(MemoryError::AddressOutOfRange(address));
        }
        
        // ワードを4バイトに分解して個別に書き込み
        self.write_byte(memory, address, (value & 0xFF) as u8)?;
        self.write_byte(memory, address + 1, ((value >> 8) & 0xFF) as u8)?;
        self.write_byte(memory, address + 2, ((value >> 16) & 0xFF) as u8)?;
        self.write_byte(memory, address + 3, ((value >> 24) & 0xFF) as u8)?;
        
        Ok(())
    }

    /// 統計情報を取得
    pub fn get_stats(&self) -> &CacheStats {
        &self.stats
    }

    /// 統計情報をリセット
    #[allow(dead_code)]
    pub fn reset_stats(&mut self) {
        self.stats = CacheStats::default();
    }

    /// キャッシュをフラッシュ（全てのダーティラインをメモリに書き戻し）
    #[allow(dead_code)]
    pub fn flush(&mut self, memory: &mut Memory) -> Result<(), MemoryError> {
        // set_indexを使用し、self.setsの再借用を避ける
        for (set_index, set) in self.sets.iter_mut().enumerate() {
            // set_indexをu32にキャスト
            let set_index_u32 = set_index as u32; 

            for line in &mut set.lines {
                if line.valid && line.dirty {
                    // set_index_u32 を使用して base_address を計算
                    let base_address = ((line.tag * CACHE_SETS as u32 + 
                        set_index_u32) // <-- ここを修正
                        * CACHE_LINE_SIZE as u32) as MemoryAddress;
                    
                    for i in 0..CACHE_LINE_SIZE {
                        memory.write_byte(base_address + i as u32, line.data[i])?;
                    }
                    
                    line.dirty = false;
                    self.stats.writebacks += 1;
                }
            }
        }
        Ok(())
    }
}

impl fmt::Display for CacheStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "キャッシュ統計:\n")?;
        write!(f, "  ヒット数: {}\n", self.hits)?;
        write!(f, "  ミス数: {}\n", self.misses)?;
        write!(f, "  ヒット率: {:.2}%\n", self.hit_rate() * 100.0)?;
        write!(f, "  書き込みバック数: {}", self.writebacks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // テスト環境で Memory::new() や write_byte などが利用可能である前提
    // Memory型がプロジェクト内のどこかで定義されている必要があります。
    // 仮の Memory 構造体を定義します (もしメインのコードに含まれていない場合)
    
    // NOTE: `Memory` は `crate::memory` でインポートされているため、
    // テストが実行される環境に依存しますが、ここでは省略します。
    // ただし、`read_word`内で `memory.size()` を呼び出しているので、
    // `Memory`にそのメソッドが必要なはずです。

    // テスト用のMemory構造体は既にcrate::memoryで定義されているため削除


    #[test]
    fn test_cache_read_write() {
        let mut memory = Memory::new();
        let mut cache = Cache::new();
        
        // メモリにデータを書き込み
        memory.write_byte(0x1000, 0xAB).unwrap();
        
        // キャッシュ経由で読み込み
        let value = cache.read_byte(&mut memory, 0x1000).unwrap();
        assert_eq!(value, 0xAB);
        
        // キャッシュ経由で書き込み
        cache.write_byte(&mut memory, 0x1000, 0xCD).unwrap();
        
        // キャッシュから読み込み
        let value = cache.read_byte(&mut memory, 0x1000).unwrap();
        assert_eq!(value, 0xCD);
    }

    #[test]
    fn test_cache_stats() {
        let mut memory = Memory::new();
        let mut cache = Cache::new();
        
        // いくつかのアクセスを実行
        // 1回目: ミス (0x1000)
        cache.read_byte(&mut memory, 0x1000).unwrap();
        // 2回目: ヒット (0x1000)
        cache.read_byte(&mut memory, 0x1000).unwrap();
        // 3回目: ヒット (0x1000)
        cache.read_byte(&mut memory, 0x1001).unwrap(); // 同じライン内ならヒット
        
        let stats = cache.get_stats();
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hits, 2);
    }
}
