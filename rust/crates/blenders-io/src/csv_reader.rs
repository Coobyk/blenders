/// Port of `common/intern/csv_reader.cc` —
/// CSV row/field reader.
/// `read_csv` (verbatim — rows of comma-split
/// fields).
pub fn read_csv(data: &str) -> Vec<Vec<String>> {
    data.lines()
        .map(|l| l.split(',').map(|f| f.trim().to_string()).collect())
        .collect()
}
