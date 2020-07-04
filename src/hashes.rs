use std::collections::HashMap;
use std::hash::Hasher;
use std::io::Cursor;
use twox_hash::XxHash32;

static HASHES_U: &'static str = include_str!("../data/wiiu_hashes.json");
static HASHES_NX: &'static str = include_str!("../data/switch_hashes.json");
type HashTable = HashMap<&'static str, Vec<u32>>;

/// Platform enum for Wii U or Switch copy of BOTW
#[derive(Debug, Eq, PartialEq)]
pub enum Platform {
    WiiU,
    Switch,
}

/// Gets a hash table of stock BOTW 1.5.0 (for Wii U) or stock 1.6.0 (for Switch) game files and
/// possible hashes for them. These include, where applicable, the original hash and variants
/// created by processing unmodified files with common libraries and tools.
pub fn get_hash_table(platform: &Platform) -> HashTable {
    match platform {
        Platform::WiiU => serde_json::from_str(HASHES_U).unwrap(),
        Platform::Switch => serde_json::from_str(HASHES_NX).unwrap(),
    }
}

/// A struct wrapping a hash table for stock BOTW files with a few convenience methods
#[derive(Debug, Eq, PartialEq)]
pub struct StockHashTable {
    table: HashTable,
}

impl StockHashTable {
    /// Constructs StockHashTable instance for the specified platform
    ///
    /// # Arguments
    ///
    /// * `platform` - Specifies whether to use a Wii U 1.5.0 or Switch 1.6.0 hash table
    pub fn new(platform: &Platform) -> StockHashTable {
        StockHashTable {
            table: get_hash_table(platform),
        }
    }

    /// Iterates the files in the stock hash table by their canonical resource paths.
    pub fn get_stock_files(&self) -> impl Iterator<Item = &&str> {
        self.table.keys()
    }

    /// Gets an owend list of the canonical resource paths for all files in the stock hash table.
    pub fn list_stock_files(&self) -> Vec<String> {
        self.table.keys().map(|x| x.to_string()).collect()
    }

    /// Checks a file to see if it has been modified. Automatically decompresses yaz0 data.
    ///
    /// # Arguments
    ///
    /// * `file_name` - The canonical resource name of the file to check as a string slice
    /// * `data` - The binary data for the file, as a binary data slice (`&[u8]`)
    /// * `flag_new` - Whether to count files not present in stock BOTW as modified
    pub fn is_file_modded<S: AsRef<str>, D: AsRef<[u8]>>(
        &self,
        file_name: S,
        data: D,
        flag_new: bool,
    ) -> bool {
        match self.table.contains_key(file_name.as_ref()) {
            true => {
                let data = data.as_ref();
                let mut hasher = XxHash32::with_seed(0);
                if &data[0..4] == b"Yaz0" {
                    hasher.write(
                        &yaz0::Yaz0Archive::new(Cursor::new(data))
                            .unwrap()
                            .decompress()
                            .unwrap(),
                    );
                } else {
                    hasher.write(data);
                }
                let hash: u32 = hasher.finish() as u32;
                !self.table[file_name.as_ref()].contains(&hash)
            }
            false => flag_new,
        }
    }

    /// Checks if a file is present in the unmodded game.
    ///
    /// # Arguments
    ///
    /// * `file_name` - The canonical resource name of the file to check as a string slice
    pub fn is_file_new<S: AsRef<str>>(&self, file_name: S) -> bool {
        !self.table.contains_key(file_name.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cst_hash_table() {
        get_hash_table(&Platform::WiiU);
        get_hash_table(&Platform::Switch);
    }

    #[test]
    fn check_val() {
        let table = get_hash_table(&Platform::WiiU);
        assert_eq!(
            table
                .get("Actor/ModelList/DgnMrgPrt_Dungeon023.bmodellist")
                .unwrap(),
            &vec![2304172004u32, 1028910535u32]
        )
    }

    #[test]
    fn is_file_modded() {
        let tbl = StockHashTable::new(&Platform::Switch);
        assert_eq!(
            tbl.is_file_modded(
                "Actor/Physics/FldObj_MountainSheikerWall_A_06.bphysics",
                b"Random data",
                true
            ),
            true
        )
    }

    #[test]
    fn print_files() {
        let tbl = StockHashTable::new(&Platform::WiiU);
        for file in tbl.get_stock_files() {
            println!("{}", file)
        }
    }
}
