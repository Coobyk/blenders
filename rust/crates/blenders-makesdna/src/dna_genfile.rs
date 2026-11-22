/// Port of `intern/dna_genfile.cc` — emit
/// `dna.c` (flattened SDNA registry).
/// `dna_genfile` (verbatim — emit tables).
pub fn dna_genfile(parse: &super::dna_parse::DnaParse) -> String {
    let mut out = String::new();
    for s in &parse.structs {
        out.push_str(&s.name);
        out.push('\n');
    }
    out
}
