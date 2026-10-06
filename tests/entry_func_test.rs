use minilsm::{Entry, ValueKind};

#[test]
fn entry_put_keep_seq_and_values() {
    let entry = Entry::put(888, b"alice_and_bob".to_vec());

    assert_eq!(entry.seq, 888);
    assert_eq!(entry.value, ValueKind::Put(b"alice_and_bob".to_vec()));
    assert_ne!(entry.seq, 889);
    assert_ne!(entry.value, ValueKind::Put(b"alice_and_bob_error".to_vec()));

    let entry2 = Entry::put(1024, b"andriod");
    assert_eq!(entry2.seq, 1024);
    assert_eq!(entry2.value, ValueKind::Put(b"andriod".to_vec()));
}

#[test]
fn entry_tombstone_keep_seq() {
    let entry = Entry::tombstone(1024);
    assert_eq!(1024, entry.seq);
    // assert_eq!(1023, entry.seq);
    assert_eq!(entry.value, ValueKind::Tombstone);
}
