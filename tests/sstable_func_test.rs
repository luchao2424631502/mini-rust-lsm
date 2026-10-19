use std::fs;

use minilsm::{
    Entry, LsmError, MemTable, ValueKind, find_in_sstable, read_record, write_record, write_sstable,
};

#[test]
fn test_tombstone_round_trip() {
    // tombstone
    let mut record = Vec::<u8>::new();
    let key = b"alice_and_bob";
    let entry = Entry::tombstone(1024);

    write_record(&mut record, key, &entry).unwrap();

    let mut input = &record[..];
    let (read_key, read_entry) = read_record(&mut input).unwrap().unwrap();

    assert_eq!(key, read_key.as_slice());
    assert_eq!(entry.seq, read_entry.seq);
    assert_eq!(entry.value, read_entry.value);
}

#[test]
fn test_put_round_trip() {
    // put
    let mut bytes = Vec::<u8>::new();

    let key = b"alice_and_dave";
    let entry = Entry::put(1024, b"who knows");

    write_record(&mut bytes, key, &entry).unwrap();

    let mut input = &bytes[..];
    let (read_key, read_entry) = read_record(&mut input).unwrap().unwrap();

    assert_eq!(key, read_key.as_slice());
    assert_eq!(entry.seq, read_entry.seq);
    assert_eq!(entry.value, read_entry.value);
}

#[test]
// 截断数据
fn test_truncate_data() {
    // put
    let mut bytes = Vec::<u8>::new();

    let key = b"alice_and_loius";
    let entry = Entry::put(999, b"prade_truncate_data");

    write_record(&mut bytes, key, &entry).unwrap();

    // truncate data
    bytes.truncate(bytes.len() - 1);

    let mut input = &bytes[..];
    let err = read_record(&mut input).unwrap_err();

    match err {
        LsmError::Io(real_err) => {
            assert_eq!(real_err.kind(), std::io::ErrorKind::UnexpectedEof);
        }
        _ => {
            panic!("wrong error");
        }
    };
}

// kind = 2
#[test]
fn test_wrong_kind_data() {
    let mut bytes = Vec::<u8>::new();
    let key = b"wrong_key";
    let entry = Entry::put(1024, b"put_value");

    write_record(&mut bytes, key, &entry).unwrap();
    // 修改kind = 2 [4+9+8]
    bytes[21] = 2;

    let mut input = &bytes[..];
    let err = read_record(&mut input).unwrap_err();

    match err {
        LsmError::Corrupt(real_error) => {
            assert_eq!(real_error, "unknowed kind error");
        }
        _ => {
            panic!("wrong error");
        }
    }
}

// 非法tombstone长度
#[test]
fn test_wrong_tombstone_length() {
    let mut bytes = Vec::<u8>::new();
    let key = b"wrong_tombstone_len";
    let entry = Entry::tombstone(1024);

    write_record(&mut bytes, key, &entry).unwrap();
    // 4+19+8+1 = 20+12 = 32
    bytes[32] = 0x78u8;
    bytes[33] = 0x56u8;
    bytes[34] = 0x34u8;
    bytes[35] = 0x12u8;

    let mut input = &bytes[..];
    let err = read_record(&mut input).unwrap_err();
    match err {
        LsmError::Corrupt(msg) => {
            assert_eq!(msg, "read tombstone but value_len != 0");
        }
        _ => {
            panic!("wrong error");
        }
    }
}

// 检测读取到文件末尾能否正确返回
#[test]
fn test_read_normal_end_of_file() {
    let mut bytes = Vec::<u8>::new();
    let key1 = b"key_key";
    let entry1 = Entry::tombstone(24);

    write_record(&mut bytes, key1, &entry1).unwrap();

    let mut input = &bytes[..];
    read_record(&mut input).unwrap();
    assert_eq!(true, read_record(&mut input).unwrap().is_none());
}

#[test]
fn test_two_read_return_results() {
    let mut bytes = Vec::<u8>::new();
    let key1 = b"testkey1";
    let entry1 = Entry::tombstone(1);

    let key2 = b"testkey2";
    let entry2 = Entry::put(2, b"value2");

    write_record(&mut bytes, key1, &entry1).unwrap();
    write_record(&mut bytes, key2, &entry2).unwrap();

    let mut input = &bytes[..];
    let (readkey1, readentry1) = read_record(&mut input).unwrap().unwrap();
    let (readkey2, readentry2) = read_record(&mut input).unwrap().unwrap();

    assert_eq!(readkey1.as_slice(), b"testkey1");
    assert_eq!(entry1, readentry1);

    assert_eq!(readkey2.as_slice(), b"testkey2");
    assert_eq!(entry2, readentry2);

    assert_eq!(read_record(&mut input).unwrap().is_none(), true);
}

//
#[test]
fn single_file_write_read() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sst_000001.sst");

    let mut mem = MemTable::new();
    mem.put(b"c".to_vec(), Entry::put(1024, b"b_value"));
    mem.put(b"a".to_vec(), Entry::tombstone(1023));
    write_sstable(&path, &mem).unwrap();

    assert_eq!(find_in_sstable(&path, b"a").unwrap().unwrap().seq, 1023);
}

#[test]
fn single_file_write_multi_read() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sst_000001.sst");

    let mut mem = MemTable::new();
    mem.put(b"d".to_vec(), Entry::put(4, b"d_value"));
    mem.put(b"b".to_vec(), Entry::tombstone(2));
    mem.put(b"a".to_vec(), Entry::put(1, b"a_value"));
    mem.put(b"c".to_vec(), Entry::tombstone(3));
    write_sstable(&path, &mem).unwrap();

    let mut input = fs::OpenOptions::new().read(true).open(path).unwrap();
    let (key_a, entry_a) = read_record(&mut input).unwrap().unwrap();
    assert_eq!(key_a, b"a".to_vec());
    assert_eq!(entry_a.seq, 1);
    assert_eq!(entry_a.value, ValueKind::Put(b"a_value".to_vec()));

    let (key_b, entry_b) = read_record(&mut input).unwrap().unwrap();
    assert_eq!(key_b, b"b".to_vec());
    assert_eq!(entry_b.seq, 2);
    assert_eq!(entry_b.value, ValueKind::Tombstone);

    let (key_c, entry_c) = read_record(&mut input).unwrap().unwrap();
    assert_eq!(key_c, b"c".to_vec());
    assert_eq!(entry_c.seq, 3);
    assert_eq!(entry_c.value, ValueKind::Tombstone);

    let (key_d, entry_d) = read_record(&mut input).unwrap().unwrap();
    assert_eq!(key_d, b"d".to_vec());
    assert_eq!(entry_d.seq, 4);
    assert_eq!(entry_d.value, ValueKind::Put(b"d_value".to_vec()));
}

#[test]
fn none_read_in_single_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tmp.sst");

    let mut mem = MemTable::new();
    mem.put(b"1".to_vec(), Entry::put(1, b"1v"));
    mem.put(b"9".to_vec(), Entry::put(2, b"2v"));
    mem.put(b"124".to_vec(), Entry::tombstone(0124));

    write_sstable(&path, &mem).unwrap();

    let found_entry = find_in_sstable(&path, b"125").unwrap();

    assert_eq!(found_entry, None);
}
