// mod sstable

use std::io::{Read, Write};

use crate::entry::{Entry, Seq, ValueKind};
use crate::error::{LsmError, Result};

pub fn write_record<W: Write>(writer: &mut W, key: &[u8], entry: &Entry) -> Result<()> {
    let key_len: u32 =
        u32::try_from(key.len()).map_err(|err| LsmError::Corrupt("key out bound of u32"))?;
    let (kind, seq, value): (u8, Seq, &[u8]) = match &entry.value {
        ValueKind::Put(value) => (0, entry.seq, value.as_slice()),
        ValueKind::Tombstone => (1, entry.seq, &[]),
    };
    let value_len: u32 =
        u32::try_from(value.len()).map_err(|_| LsmError::Corrupt("value out bound of u32"))?;

    if kind == 1 && value_len != 0 {
        return Err(LsmError::Corrupt("write error kind type"));
    }

    /*
       key_len: u32,
       key: &[u8],
       seq: u64,
       kind: u8,
       value_len: u32,
       value: &[u8],
    */
    writer.write_all(&key_len.to_le_bytes())?;
    writer.write_all(key)?;
    writer.write_all(&seq.to_le_bytes())?;
    writer.write_all(&[kind; 1])?;
    writer.write_all(&value_len.to_le_bytes())?;
    writer.write_all(value)?;

    Ok(())
}

/*
   return:
   1. Ok(None) 边界
   2. Ok(Some((key, Entry))) key+Entry
   3. Err()
*/
pub fn read_record<R: Read>(input: &mut R) -> Result<Option<(Vec<u8>, Entry)>> {
    let mut first = [0u8; 1];
    // 0. end of file
    if input.read(&mut first)? == 0 {
        return Ok(None);
    }
    // 1. key_len
    let mut key_len_arr = [0u8; 3];
    input.read_exact(&mut key_len_arr)?;
    let key_len = u32::from_le_bytes([first[0], key_len_arr[0], key_len_arr[1], key_len_arr[2]]);

    // 2. key
    let mut key = Vec::<u8>::new();
    key.reserve(
        usize::try_from(key_len)
            .map_err(|_| LsmError::Corrupt("key_len can't transfer to usize"))?,
    );
    key.resize(
        usize::try_from(key_len).map_err(|_| LsmError::Corrupt("key_len transfer error"))?,
        0u8,
    );
    input.read_exact(key.as_mut_slice())?;

    // 3. seq
    let mut seq_arr = [0u8; 8];
    input.read_exact(&mut seq_arr)?;
    let seq: Seq = u64::from_le_bytes(seq_arr);

    // 4. kind
    let mut kind_arr = [0u8; 1];
    input.read_exact(&mut kind_arr)?;
    let kind = kind_arr[0];

    // 5. value_len
    let mut value_len_arr = [0u8; 4];
    input.read_exact(&mut value_len_arr)?;
    let value_len = u32::from_le_bytes([
        value_len_arr[0],
        value_len_arr[1],
        value_len_arr[2],
        value_len_arr[3],
    ]);

    if kind == 1 && value_len != 0 {
        return Err(LsmError::Corrupt("read tombstone but value_len != 0"));
    }

    // 6. value
    let mut value = Vec::<u8>::new();
    value.reserve(
        usize::try_from(value_len)
            .map_err(|_| LsmError::Corrupt("value_len can't transfer to usize"))?,
    );
    value.resize(
        usize::try_from(value_len).map_err(|_| LsmError::Corrupt("value_len transfer error"))?,
        0u8,
    );
    input.read_exact(value.as_mut_slice())?;

    // 7. assemble return value
    let entry = match kind {
        0 => {
            // ValueKind::put
            Entry::put(seq, value)
        }
        1 if value_len == 0 => {
            // ValueKind::tombstone
            Entry::tombstone(seq)
        }
        _ => return Err(LsmError::Corrupt("unknowed kind error")),
    };

    Ok(Some((key, entry)))
}
