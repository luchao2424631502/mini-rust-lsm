// mod memtable

use crate::{Entry, entry};
use skiplist::SkipMap;

pub struct MemTable {
    /*
       1. 插入: skipmap的语义是 不存在返回None, 存在返回Some(old_value)
       2. 读取: Option包装
    */
    entries: SkipMap<Vec<u8>, Entry>,
}

impl MemTable {
    pub fn new() -> Self {
        Self {
            entries: SkipMap::new(),
        }
    }

    pub fn put(&mut self, key: Vec<u8>, entry: Entry) {
        self.entries.insert(key, entry);
    }

    pub fn get(&self, key: &[u8]) -> Option<&Entry> {
        self.entries.get(key)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    // 迭代器
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a Vec<u8>, &'a Entry)> + 'a {
        self.entries.iter()
    }
}
