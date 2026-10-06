// mod entry;

pub type Seq = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueKind {
    Put(Vec<u8>),
    Tombstone,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub seq: Seq,
    pub value: ValueKind,
}

impl Entry {
    /* value */
    pub fn put(seq: Seq, value: impl Into<Vec<u8>>) -> Self {
        Self {
            seq: seq,
            value: ValueKind::Put(value.into()),
        }
    }

    /* delete */
    pub fn tombstone(seq: Seq) -> Self {
        Self {
            seq: seq,
            value: ValueKind::Tombstone,
        }
    }
}
