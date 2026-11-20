//! Port of `hash_tables.cc` — `HashTableStats::print`.

/// `HashTableStats` — diagnostics snapshot produced by hash table types.
#[derive(Default)]
pub struct HashTableStats {
    pub address: usize,
    pub capacity: i64,
    pub size: i64,
    pub size_in_bytes: i64,
    pub load_factor: f32,
    pub removed_load_factor: f32,
    pub removed_amount: i64,
    pub size_per_element: i64,
    pub average_collisions: f32,
    pub keys_by_collision_count: Vec<i64>,
}

impl HashTableStats {
    /// `HashTableStats::print` — prints via `println!` instead of `std::cout`.
    /// `BLI_str_format_byte_unit` is inlined as a minimal IEC formatter.
    pub fn print(&self, name: &str) {
        println!("Hash Table Stats: {}", name);
        println!("  Address: {}", self.address);
        println!("  Total Slots: {}", self.capacity);
        println!(
            "  Occupied Slots:  {} ({} %)",
            self.size,
            self.load_factor * 100.0
        );
        println!(
            "  Removed Slots: {} ({} %)",
            self.removed_amount,
            self.removed_load_factor * 100.0
        );
        println!("  Size: ~{}", format_byte_unit(self.size_in_bytes));
        println!("  Size per Slot: {} bytes", self.size_per_element);
        println!("  Average Collisions: {}", self.average_collisions);
        for (i, count) in self.keys_by_collision_count.iter().enumerate() {
            println!("  {} Collisions: {}", i, count);
        }
    }
}

/// Minimal `BLI_str_format_byte_unit` for the stats printer.
fn format_byte_unit(bytes: i64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut v = bytes as f64;
    let mut u = 0;
    while v >= 1024.0 && u < UNITS.len() - 1 {
        v /= 1024.0;
        u += 1;
    }
    format!("{:.1}{}", v, UNITS[u])
}
