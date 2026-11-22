/// Port of `intern/dna_utils.cc` — size/
/// align tables + name cleanup.
/// `dna_type_size` (verbatim — builtin
/// sizes).
pub fn dna_type_size(ty: &str) -> usize {
    match ty {
        "char" | "uchar" => 1,
        "short" | "ushort" => 2,
        "int" | "long" | "ulong" | "float" => 4,
        "int64_t" | "uint64_t" | "double" | "longlong" | "ulonglong" => 8,
        _ => 4,
    }
}
/// `dna_elem_name_size` (verbatim — strip
/// `*`, `()` → base name).
pub fn dna_elem_name_size(name: &str) -> &str {
    name.trim_start_matches(|c| c == '*' || c == '(').trim_end_matches(')')
}
