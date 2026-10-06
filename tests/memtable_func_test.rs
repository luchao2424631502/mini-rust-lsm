use minilsm::{Entry, MemTable, ValueKind};

#[test]
fn test_memtable_put_get() {
    let mut memtable = MemTable::new();

    // 不重复key 写入 value + tombstone
    memtable.put(b"alice1".to_vec(), Entry::put(1011, b"value1"));
    memtable.put(b"alice2".to_vec(), Entry::put(1012, b"value2"));
    memtable.put(b"alice3".to_vec(), Entry::put(1013, b"value3"));
    memtable.put(b"alice4".to_vec(), Entry::put(1014, b"value4"));
    memtable.put(b"bob1".to_vec(), Entry::tombstone(1024));
    memtable.put(b"bob2".to_vec(), Entry::tombstone(1025));

    assert_eq!(memtable.get(b"alice1"), Some(&Entry::put(1011, b"value1")));
    assert_eq!(memtable.get(b"alice2"), Some(&Entry::put(1012, b"value2")));
    assert_eq!(memtable.get(b"alice3"), Some(&Entry::put(1013, b"value3")));
    assert_eq!(memtable.get(b"alice4"), Some(&Entry::put(1014, b"value4")));
    assert_eq!(memtable.get(b"bob1"), Some(&Entry::tombstone(1024)));
    assert_eq!(memtable.get(b"bob2"), Some(&Entry::tombstone(1025)));

    // 覆盖写
    memtable.put(b"alice1".to_vec(), Entry::put(101, b"tef7e1"));
    memtable.put(b"alice2".to_vec(), Entry::put(102, b"tef7e2"));
    memtable.put(b"alice3".to_vec(), Entry::tombstone(103));
    memtable.put(b"alice4".to_vec(), Entry::tombstone(104));
    memtable.put(b"bob1".to_vec(), Entry::put(99, b"tef7e3"));
    memtable.put(b"bob2".to_vec(), Entry::put(98, b"tef7e4"));

    assert_eq!(memtable.get(b"alice1"), Some(&Entry::put(101, b"tef7e1")));
    assert_eq!(memtable.get(b"alice2"), Some(&Entry::put(102, b"tef7e2")));
    assert_eq!(memtable.get(b"alice3"), Some(&Entry::tombstone(103)));
    assert_eq!(memtable.get(b"alice4"), Some(&Entry::tombstone(104)));
    assert_eq!(memtable.get(b"bob1"), Some(&Entry::put(99, b"tef7e3")));
    assert_eq!(memtable.get(b"bob2"), Some(&Entry::put(98, b"tef7e4")));

    // len长度
    assert_eq!(memtable.len(), 6);

    // 清除
    memtable.clear();
    // len长度
    assert_eq!(memtable.len(), 0);
}

/* 测试迭代器 */
#[test]
fn test_memtable_iter() {
    let mut memtable = MemTable::new();

    memtable.put(b"bob2".to_vec(), Entry::tombstone(1025));
    memtable.put(b"bob1".to_vec(), Entry::tombstone(1024));
    memtable.put(b"alice1".to_vec(), Entry::put(1011, b"value1"));
    memtable.put(b"alice4".to_vec(), Entry::put(1014, b"value4"));
    memtable.put(b"alice2".to_vec(), Entry::put(1012, b"value2"));
    memtable.put(b"alice3".to_vec(), Entry::put(1013, b"value3"));

    let mut iter = memtable.iter();

    let ans_key: Vec<Vec<u8>> = vec![
        b"alice1".to_vec(),
        b"alice2".to_vec(),
        b"alice3".to_vec(),
        b"alice4".to_vec(),
        b"bob1".to_vec(),
        b"bob2".to_vec(),
    ];

    let ans_value = vec![
        Entry::put(1011, b"value1"),
        Entry::put(1012, b"value2"),
        Entry::put(1013, b"value3"),
        Entry::put(1014, b"value4"),
        Entry::tombstone(1024),
        Entry::tombstone(1025),
    ];

    let mut index = 0;
    loop {
        let Some(entry) = iter.next() else {
            break;
        };

        assert_eq!(*entry.0, ans_key[index]);
        assert_eq!(*entry.1, ans_value[index]);

        index += 1;
    }

    assert_eq!(index, 6);
}
