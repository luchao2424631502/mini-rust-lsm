use minilsm::{Entry, LsmError, read_record, write_record};

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
