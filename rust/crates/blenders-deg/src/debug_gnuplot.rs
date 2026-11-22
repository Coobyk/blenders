//! Port of `intern/debug/deg_debug_stats_gnuplot.cc` —
/// gnuplot stats-script writer.

use std::fmt::Write;

/// One `StatsEntry` (verbatim).
pub struct StatsEntry {
    /// `id_node->id_orig->name`.
    pub id_name: String,
    /// `node->stats.current_time`.
    pub time: f64,
}

/// `gnuplotify_id_code` (verbatim — first two chars).
pub fn gnuplotify_id_code(name: &str) -> String {
    name.chars().take(2).collect()
}

/// `gnuplotify_name` (verbatim — strips leading id-code).
pub fn gnuplotify_name(name: &str) -> String {
    name.chars().skip(2).collect()
}

/// `write_stats_data` (verbatim — sort desc by time, keep
/// top 32, reverse, emit `$data` block).
pub fn write_stats_data(entries: Vec<StatsEntry>, out: &mut String) {
    let mut stats: Vec<StatsEntry> = entries.into_iter().filter(|e| e.time != 0.0).collect();
    stats.sort_by(|a, b| b.time.partial_cmp(&a.time).unwrap());
    stats.truncate(32);
    stats.reverse();
    let _ = writeln!(out, "$data << EOD");
    for e in &stats {
        let _ = writeln!(
            out,
            "\"[{}] {}\",{}",
            gnuplotify_id_code(&e.id_name),
            gnuplotify_name(&e.id_name),
            e.time
        );
    }
    let _ = writeln!(out, "EOD");
}

/// `deg_debug_stats_gnuplot` (verbatim — data + label +
/// terminal/output commands).
pub fn deg_debug_stats_gnuplot(
    entries: Vec<StatsEntry>,
    label: &str,
    output_filename: &str,
) -> String {
    let mut out = String::new();
    write_stats_data(entries, &mut out);
    if !label.is_empty() {
        let _ = writeln!(out, "set title \"{}\"", label);
    }
    let _ = writeln!(out, "set terminal pngcairo size 1920,1080");
    let _ = writeln!(out, "set output \"{}\"", output_filename);
    let _ = writeln!(out, "set grid");
    let _ = writeln!(out, "set datafile separator ','");
    let _ = writeln!(out, "set style fill solid");
    let _ = writeln!(
        out,
        "plot \"$data\" using ($2*0.5):0:($2*0.5):(0.2):yticlabels(1) \
         with boxxyerrorbars t '' lt rgb \"#406090\""
    );
    out
}
