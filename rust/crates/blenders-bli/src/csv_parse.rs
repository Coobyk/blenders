//! Port of `csv_parse.cc` — `blender::csv_parse` buffered CSV parser.
//! Serial port of the parallel version: identical output semantics.

use crate::index_range::IndexRange;
use crate::offset_indices::OffsetIndices;

/// `CsvParseOptions`.
#[derive(Clone)]
pub struct CsvParseOptions {
    pub delimiter: char,
    pub quote: char,
    /// `quote_escape_chars` — chars that may escape the quote (e.g. `"` and `\`).
    pub quote_escape_chars: Vec<char>,
    pub chunk_size_bytes: usize,
}

impl Default for CsvParseOptions {
    fn default() -> Self {
        CsvParseOptions {
            delimiter: ',',
            quote: '"',
            quote_escape_chars: vec!['"', '\\'],
            chunk_size_bytes: 64 * 1024,
        }
    }
}

/// `CsvRecord` — the fields of one record (slices into the source buffer).
pub struct CsvRecord<'a> {
    pub fields: Vec<&'a [u8]>,
}

/// `CsvRecords` — record offsets + flat field list.
pub struct CsvRecords<'a> {
    pub offsets: Vec<i64>,
    pub fields: Vec<&'a [u8]>,
}

impl<'a> CsvRecords<'a> {
    /// Number of records.
    pub fn len(&self) -> usize {
        self.offsets.len().saturating_sub(1)
    }
    /// `records[record_i]` → fields.
    pub fn record(&self, i: usize) -> &[&'a [u8]] {
        let oi = OffsetIndices { data: unsafe { std::mem::transmute::<&[i64], &[i32]>(self.offsets.as_slice()) } };
        let _ = oi;
        let (a, b) = (self.offsets[i] as usize, self.offsets[i + 1] as usize);
        &self.fields[a..b]
    }
}

/// `detail::find_end_of_simple_field`.
pub fn find_end_of_simple_field(buffer: &[u8], start: usize, delimiter: u8) -> usize {
    let mut i = start;
    while i < buffer.len() {
        let c = buffer[i];
        if c == delimiter || c == b'\n' || c == b'\r' {
            return i;
        }
        i += 1;
    }
    buffer.len()
}

/// `detail::find_end_of_quoted_field` — `None` when unclosed.
pub fn find_end_of_quoted_field(
    buffer: &[u8],
    start: usize,
    quote: u8,
    escape_chars: &[u8],
) -> Option<usize> {
    let mut i = start;
    while i < buffer.len() {
        let c = buffer[i];
        if escape_chars.contains(&c) {
            if i + 1 < buffer.len() && buffer[i + 1] == quote {
                i += 2;
                continue;
            }
        }
        if c == quote {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// `detail::parse_record_fields` — parse one record starting at `start`;
/// pushes field slices to `r_fields`, returns next record start.
pub fn parse_record_fields<'a>(
    buffer: &'a [u8],
    start: usize,
    delimiter: u8,
    quote: u8,
    quote_escape_chars: &[u8],
    r_fields: &mut Vec<&'a [u8]>,
) -> Option<usize> {
    let mut i = start;
    while i < buffer.len() {
        let c = buffer[i];
        if c == b'\n' {
            return Some(i + 1);
        }
        if c == b'\r' {
            i += 1;
            continue;
        }
        if c == delimiter {
            r_fields.push(&buffer[i..i]);
            i += 1;
            if i <= buffer.len() {
                if i < buffer.len() {
                    if buffer[i] == b'\n' || buffer[i] == b'\r' {
                        r_fields.push(&buffer[i..i]);
                    }
                } else {
                    r_fields.push(&buffer[i..i]);
                }
            }
            continue;
        }
        if c == quote {
            i += 1;
            let end_of_field = find_end_of_quoted_field(buffer, i, quote, quote_escape_chars)?;
            r_fields.push(&buffer[i..end_of_field]);
            i = end_of_field;
            while i < buffer.len() {
                let inner_c = buffer[i];
                if inner_c == quote {
                    i += 1;
                    continue;
                }
                if inner_c == delimiter {
                    i += 1;
                    if i <= buffer.len() {
                        if i < buffer.len() {
                            if buffer[i] == b'\n' || buffer[i] == b'\r' {
                                r_fields.push(&buffer[i..i]);
                            }
                        } else {
                            r_fields.push(&buffer[i..i]);
                        }
                    }
                    break;
                }
                if inner_c == b'\n' || inner_c == b'\r' {
                    break;
                }
                i += 1;
            }
            continue;
        }
        let end_of_field = find_end_of_simple_field(buffer, i, delimiter);
        r_fields.push(&buffer[i..end_of_field]);
        i = end_of_field;
        while i < buffer.len() {
            let inner_c = buffer[i];
            if inner_c == delimiter {
                i += 1;
                if i <= buffer.len() {
                    if i < buffer.len() {
                        if buffer[i] == b'\n' || buffer[i] == b'\r' {
                            r_fields.push(&buffer[i..i]);
                        }
                    } else {
                        r_fields.push(&buffer[i..i]);
                    }
                }
                break;
            }
            if inner_c == b'\n' || inner_c == b'\r' {
                break;
            }
            debug_assert!(false, "unreachable");
            break;
        }
    }
    Some(buffer.len())
}

/// `guess_next_record_start`.
fn guess_next_record_start(buffer: &[u8], start: usize) -> usize {
    let mut i = start;
    while i < buffer.len() {
        if buffer[i] == b'\n' {
            return i + 1;
        }
        i += 1;
    }
    buffer.len()
}

/// `split_into_aligned_chunks`.
fn split_into_aligned_chunks(buffer: &[u8], approximate_chunk_size: usize) -> Vec<&[u8]> {
    let cs = approximate_chunk_size.max(1);
    let mut chunks = Vec::new();
    let mut start = 0;
    while start < buffer.len() {
        let mut end = (start + cs).min(buffer.len());
        end = guess_next_record_start(buffer, end);
        chunks.push(&buffer[start..end]);
        start = end;
    }
    chunks
}

/// `parse_records` — parse all records of `buffer`.
pub fn parse_records<'a>(
    buffer: &'a [u8],
    options: &CsvParseOptions,
) -> Option<CsvRecords<'a>> {
    let mut offsets: Vec<i64> = vec![0];
    let mut fields: Vec<&'a [u8]> = Vec::new();
    let mut start = 0usize;
    let escapes: Vec<u8> = options.quote_escape_chars.iter().map(|&c| c as u8).collect();
    while start < buffer.len() {
        let next = parse_record_fields(
            buffer,
            start,
            options.delimiter as u8,
            options.quote as u8,
            &escapes,
            &mut fields,
        )?;
        if fields.len() as i64 > *offsets.last().unwrap() {
            offsets.push(fields.len() as i64);
        }
        start = next;
    }
    Some(CsvRecords { offsets, fields })
}

/// `parse_csv_in_chunks` — parse header, then chunked records (serial).
/// `process_records` is called once per chunk, collecting results.
pub fn parse_csv_in_chunks<'a, T>(
    buffer: &'a [u8],
    options: &CsvParseOptions,
    mut process_header: impl FnMut(CsvRecord<'a>),
    mut process_records: impl FnMut(&CsvRecords<'a>) -> T,
) -> Option<Vec<T>> {
    let escapes: Vec<u8> = options.quote_escape_chars.iter().map(|&c| c as u8).collect();
    let mut header_fields: Vec<&'a [u8]> = Vec::new();
    let first_data_record_start = parse_record_fields(
        buffer,
        0,
        options.delimiter as u8,
        options.quote as u8,
        &escapes,
        &mut header_fields,
    )?;
    process_header(CsvRecord {
        fields: header_fields,
    });
    let data_buffer = &buffer[first_data_record_start..];
    let chunks = split_into_aligned_chunks(data_buffer, options.chunk_size_bytes);
    let mut results = Vec::new();
    let mut found_malformed_chunk = false;
    for chunk in chunks {
        match parse_records(chunk, options) {
            Some(records) => results.push(process_records(&records)),
            None => {
                found_malformed_chunk = true;
                break;
            }
        }
    }
    if found_malformed_chunk {
        results.clear();
        let records = parse_records(data_buffer, options)?;
        results.push(process_records(&records));
    }
    Some(results)
}

/// `unescape_field` — `""`/`\"` → `"` inside a field.
pub fn unescape_field<'a>(str_: &'a [u8], options: &CsvParseOptions) -> Vec<u8> {
    let escape_chars: Vec<u8> = options.quote_escape_chars.iter().map(|&c| c as u8).collect();
    if !str_.iter().any(|c| escape_chars.contains(c)) {
        return str_.to_vec();
    }
    let mut out = Vec::with_capacity(str_.len());
    let mut i = 0;
    let quote = options.quote as u8;
    while i < str_.len() {
        let c = str_[i];
        if escape_chars.contains(&c) && i + 1 < str_.len() && str_[i + 1] == quote {
            out.push(quote);
            i += 2;
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

/// Re-export for parity with the C usage pattern.
pub use crate::index_range::IndexRange as _IndexRange;
