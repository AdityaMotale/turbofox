///
/// Entry for the memory table
///
pub struct MemoryTableEntry {
    pub key: Vec<u8>,
    pub value: Option<Vec<u8>>,
    pub timestamp: u128,
    pub deleted: bool,
}

///
/// MemoryTable holds sorted list of records in the db
///
/// Entries are sorted in a Vector instead of HashMap to support scans
///
pub struct MemoryTable {
    entries: Vec<MemoryTableEntry>,
    size: usize,
}

impl MemoryTable {
    ///
    /// create a new instance of MemoryTable
    ///
    pub fn new() -> MemoryTable {
        MemoryTable {
            entries: Vec::new(),
            size: 0,
        }
    }

    ///
    /// Sets a Key-Value pair in the MemoryTable.
    ///
    pub fn set(&mut self, key: &[u8], value: &[u8], timestamp: u128) {
        let entry = MemoryTableEntry {
            key: key.to_owned(),
            value: Some(value.to_owned()),
            timestamp,
            deleted: false,
        };

        match self.get_index(key) {
            Ok(idx) => {
                // If a Value existed on the deleted record, then add the difference of the new and old Value to the MemoryTable's size.
                if let Some(current_value) = self.entries[idx].value.as_ref() {
                    if value.len() < current_value.len() {
                        self.size -= current_value.len() - value.len();
                    } else {
                        self.size += value.len() - current_value.len();
                    }
                } else {
                    // if value never existed
                    self.size += value.len();
                }

                self.entries[idx] = entry;
            }
            Err(idx) => {
                // Increase the size of the MemoryTable by the Key size, Value size, Timestamp size (16 bytes), Tombstone size (1 byte).
                self.size += key.len() + value.len() + 16 + 1;

                self.entries.insert(idx, entry);
            }
        }
    }

    ///
    /// Gets a Key-Value pair from the [MemoryTable]
    ///
    /// If no record with the same key exists in the [MemoryTable], return None.
    ///
    pub fn get(&self, key: &[u8]) -> Option<&MemoryTableEntry> {
        if let Ok(idx) = self.get_index(key) {
            return Some(&self.entries[idx]);
        }
        None
    }

    /// Deletes a Key-Value pair in the MemoryTable.
    ///
    /// This is achieved using tombstones.
    pub fn delete(&mut self, key: &[u8], timestamp: u128) {
        let entry = MemoryTableEntry {
            key: key.to_owned(),
            value: None,
            timestamp,
            deleted: true,
        };

        match self.get_index(key) {
            Ok(idx) => {
                // If a Value existed on the deleted record, then subtract the size of the Value from the MemoryTable.
                if let Some(value) = self.entries[idx].value.as_ref() {
                    self.size -= value.len();
                }
                self.entries[idx] = entry;
            }
            Err(idx) => {
                // Increase the size of the MemoryTable by the Key size, Timestamp size (16 bytes), Tombstone size (1 byte).
                self.size += key.len() + 16 + 1;
                self.entries.insert(idx, entry);
            }
        }
    }

    ///
    /// Gets a number of record from the [MemoryTable]
    ///
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    ///
    /// Gets all of the records from [MemoryTable]
    ///
    pub fn entries(&self) -> &[MemoryTableEntry] {
        &self.entries
    }

    ///
    /// Gets a total size of records from the [MemoryTable]
    ///
    pub fn size(&self) -> usize {
        self.size
    }

    ///
    /// Performs a binary search to find a record in the memory table
    ///
    /// If the record is found `[Result::Ok]` is returned, with the index of record. If the record is not
    /// found then `[Result::Err]` is returned, with the index to insert the record at.
    ///
    fn get_index(&self, key: &[u8]) -> Result<usize, usize> {
        self.entries
            .binary_search_by_key(&key, |e| e.key.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use crate::memory_table::MemoryTable;

    #[test]
    fn test_mem_table_put_start() {
        let mut table = MemoryTable::new();
        table.set(b"Lime", b"Lime Smoothie", 0); // 17 + 16 + 1
        table.set(b"Orange", b"Orange Smoothie", 10); // 21 + 16 + 1

        table.set(b"Apple", b"Apple Smoothie", 20); // 19 + 16 + 1

        assert_eq!(table.entries[0].key, b"Apple");
        assert_eq!(table.entries[0].value.as_ref().unwrap(), b"Apple Smoothie");
        assert_eq!(table.entries[0].timestamp, 20);
        assert_eq!(table.entries[0].deleted, false);
        assert_eq!(table.entries[1].key, b"Lime");
        assert_eq!(table.entries[1].value.as_ref().unwrap(), b"Lime Smoothie");
        assert_eq!(table.entries[1].timestamp, 0);
        assert_eq!(table.entries[1].deleted, false);
        assert_eq!(table.entries[2].key, b"Orange");
        assert_eq!(table.entries[2].value.as_ref().unwrap(), b"Orange Smoothie");
        assert_eq!(table.entries[2].timestamp, 10);
        assert_eq!(table.entries[2].deleted, false);

        assert_eq!(table.size, 108);
    }

    #[test]
    fn test_mem_table_put_middle() {
        let mut table = MemoryTable::new();
        table.set(b"Apple", b"Apple Smoothie", 0);
        table.set(b"Orange", b"Orange Smoothie", 10);

        table.set(b"Lime", b"Lime Smoothie", 20);

        assert_eq!(table.entries[0].key, b"Apple");
        assert_eq!(table.entries[0].value.as_ref().unwrap(), b"Apple Smoothie");
        assert_eq!(table.entries[0].timestamp, 0);
        assert_eq!(table.entries[0].deleted, false);
        assert_eq!(table.entries[1].key, b"Lime");
        assert_eq!(table.entries[1].value.as_ref().unwrap(), b"Lime Smoothie");
        assert_eq!(table.entries[1].timestamp, 20);
        assert_eq!(table.entries[1].deleted, false);
        assert_eq!(table.entries[2].key, b"Orange");
        assert_eq!(table.entries[2].value.as_ref().unwrap(), b"Orange Smoothie");
        assert_eq!(table.entries[2].timestamp, 10);
        assert_eq!(table.entries[2].deleted, false);

        assert_eq!(table.size, 108);
    }

    #[test]
    fn test_mem_table_put_end() {
        let mut table = MemoryTable::new();
        table.set(b"Apple", b"Apple Smoothie", 0);
        table.set(b"Lime", b"Lime Smoothie", 10);

        table.set(b"Orange", b"Orange Smoothie", 20);

        assert_eq!(table.entries[0].key, b"Apple");
        assert_eq!(table.entries[0].value.as_ref().unwrap(), b"Apple Smoothie");
        assert_eq!(table.entries[0].timestamp, 0);
        assert_eq!(table.entries[0].deleted, false);
        assert_eq!(table.entries[1].key, b"Lime");
        assert_eq!(table.entries[1].value.as_ref().unwrap(), b"Lime Smoothie");
        assert_eq!(table.entries[1].timestamp, 10);
        assert_eq!(table.entries[1].deleted, false);
        assert_eq!(table.entries[2].key, b"Orange");
        assert_eq!(table.entries[2].value.as_ref().unwrap(), b"Orange Smoothie");
        assert_eq!(table.entries[2].timestamp, 20);
        assert_eq!(table.entries[2].deleted, false);

        assert_eq!(table.size, 108);
    }

    #[test]
    fn test_mem_table_put_overwrite() {
        let mut table = MemoryTable::new();
        table.set(b"Apple", b"Apple Smoothie", 0);
        table.set(b"Lime", b"Lime Smoothie", 10);
        table.set(b"Orange", b"Orange Smoothie", 20);

        table.set(b"Lime", b"A sour fruit", 30);

        assert_eq!(table.entries[0].key, b"Apple");
        assert_eq!(table.entries[0].value.as_ref().unwrap(), b"Apple Smoothie");
        assert_eq!(table.entries[0].timestamp, 0);
        assert_eq!(table.entries[0].deleted, false);
        assert_eq!(table.entries[1].key, b"Lime");
        assert_eq!(table.entries[1].value.as_ref().unwrap(), b"A sour fruit");
        assert_eq!(table.entries[1].timestamp, 30);
        assert_eq!(table.entries[1].deleted, false);
        assert_eq!(table.entries[2].key, b"Orange");
        assert_eq!(table.entries[2].value.as_ref().unwrap(), b"Orange Smoothie");
        assert_eq!(table.entries[2].timestamp, 20);
        assert_eq!(table.entries[2].deleted, false);

        assert_eq!(table.size, 107);
    }

    #[test]
    fn test_mem_table_get_exists() {
        let mut table = MemoryTable::new();
        table.set(b"Apple", b"Apple Smoothie", 0);
        table.set(b"Lime", b"Lime Smoothie", 10);
        table.set(b"Orange", b"Orange Smoothie", 20);

        let entry = table.get(b"Orange").unwrap();

        assert_eq!(entry.key, b"Orange");
        assert_eq!(entry.value.as_ref().unwrap(), b"Orange Smoothie");
        assert_eq!(entry.timestamp, 20);
    }

    #[test]
    fn test_mem_table_get_not_exists() {
        let mut table = MemoryTable::new();
        table.set(b"Apple", b"Apple Smoothie", 0);
        table.set(b"Lime", b"Lime Smoothie", 0);
        table.set(b"Orange", b"Orange Smoothie", 0);

        let res = table.get(b"Potato");
        assert_eq!(res.is_some(), false);
    }

    #[test]
    fn test_mem_table_delete_exists() {
        let mut table = MemoryTable::new();
        table.set(b"Apple", b"Apple Smoothie", 0);

        table.delete(b"Apple", 10);

        let res = table.get(b"Apple").unwrap();
        assert_eq!(res.key, b"Apple");
        assert_eq!(res.value, None);
        assert_eq!(res.timestamp, 10);
        assert_eq!(res.deleted, true);

        assert_eq!(table.entries[0].key, b"Apple");
        assert_eq!(table.entries[0].value, None);
        assert_eq!(table.entries[0].timestamp, 10);
        assert_eq!(table.entries[0].deleted, true);

        assert_eq!(table.size, 22);
    }

    #[test]
    fn test_mem_table_delete_empty() {
        let mut table = MemoryTable::new();

        table.delete(b"Apple", 10);

        let res = table.get(b"Apple").unwrap();
        assert_eq!(res.key, b"Apple");
        assert_eq!(res.value, None);
        assert_eq!(res.timestamp, 10);
        assert_eq!(res.deleted, true);

        assert_eq!(table.entries[0].key, b"Apple");
        assert_eq!(table.entries[0].value, None);
        assert_eq!(table.entries[0].timestamp, 10);
        assert_eq!(table.entries[0].deleted, true);

        assert_eq!(table.size, 22);
    }
}
