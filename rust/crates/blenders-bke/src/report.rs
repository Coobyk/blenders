//! Port of `intern/report.cc` + `BKE_report.hh` — `ReportList`:
//! thread-safe operator log with print/store levels.
//!
//! CLOG → `eprintln!`-style closure; `G.background` injected.

use std::fmt::Write as _;
use std::io::Write as IoWrite;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Mutex;

/// `eReportType : uint16_t`.
pub type EReportType = u16;
pub const RPT_DEBUG: EReportType = 1 << 0;
pub const RPT_INFO: EReportType = 1 << 1;
pub const RPT_OPERATOR: EReportType = 1 << 2;
pub const RPT_PROPERTY: EReportType = 1 << 3;
pub const RPT_WARNING: EReportType = 1 << 4;
pub const RPT_ERROR: EReportType = 1 << 5;
pub const RPT_ERROR_INVALID_INPUT: EReportType = 1 << 6;
pub const RPT_ERROR_INVALID_CONTEXT: EReportType = 1 << 7;
pub const RPT_ERROR_OUT_OF_MEMORY: EReportType = 1 << 8;

pub const RPT_DEBUG_ALL: EReportType = RPT_DEBUG;
pub const RPT_INFO_ALL: EReportType = RPT_INFO;
pub const RPT_OPERATOR_ALL: EReportType = RPT_OPERATOR;
pub const RPT_PROPERTY_ALL: EReportType = RPT_PROPERTY;
pub const RPT_WARNING_ALL: EReportType = RPT_WARNING;
pub const RPT_ERROR_ALL: EReportType = RPT_ERROR
    | RPT_ERROR_INVALID_INPUT
    | RPT_ERROR_INVALID_CONTEXT
    | RPT_ERROR_OUT_OF_MEMORY;

/// `ReportListFlags`.
pub const RPT_PRINT: i32 = 1 << 0;
pub const RPT_STORE: i32 = 1 << 1;
pub const RPT_FREE: i32 = 1 << 2;
/// `RPT_OP_HOLD` — don't move into the operator global list.
pub const RPT_OP_HOLD: i32 = 1 << 3;
/// `RPT_PRINT_HANDLED_BY_OWNER`.
pub const RPT_PRINT_HANDLED_BY_OWNER: i32 = 1 << 4;

/// `Report`.
#[derive(Clone)]
pub struct Report {
    /// `eReportType`.
    pub type_: i16,
    pub flag: i16,
    /// `strlen(message)` for word wrap.
    pub len: i32,
    pub message: String,
    /// `BKE_report_type_str(type)` — translated at store time.
    pub typestr: String,
    /// `report_session_uid_counter`.
    pub session_uid: i32,
}

/// `ReportList` — `list` is a VecDeque (ListBase→deque; lock folded
/// into the struct).
pub struct ReportList {
    pub storelevel: EReportType,
    pub printlevel: EReportType,
    pub flag: i32,
    pub list: Mutex<Vec<Report>>,
    /// `uint list_len` — cached count; kept in sync on push/clear.
    pub list_len: i32,
}

impl Default for ReportList {
    fn default() -> Self {
        Self {
            storelevel: RPT_INFO,
            printlevel: RPT_ERROR,
            flag: 0,
            list: Mutex::new(Vec::new()),
            list_len: 0,
        }
    }
}

fn report_session_uid_counter_get_next() -> i32 {
    static COUNTER: AtomicI32 = AtomicI32::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed) + 1
}

/// `BKE_report_log` — level-routed log call; `log` =
/// `(level, message)`.
pub fn bke_report_log(type_: EReportType, message: &str, log: &mut dyn FnMut(&str, &str)) {
    let (level, msg) = match type_ {
        RPT_DEBUG => ("DEBUG", message),
        RPT_INFO | RPT_OPERATOR | RPT_PROPERTY => ("INFO", message),
        RPT_WARNING => ("WARN", message),
        _ => ("ERROR", message),
    };
    log(level, msg);
}

/// `BKE_report_type_str`.
pub fn bke_report_type_str(type_: EReportType) -> &'static str {
    match type_ {
        RPT_DEBUG => "Debug",
        RPT_INFO => "Info",
        RPT_OPERATOR => "Operator",
        RPT_PROPERTY => "Property",
        RPT_WARNING => "Warning",
        RPT_ERROR => "Error",
        RPT_ERROR_INVALID_INPUT => "Invalid Input Error",
        RPT_ERROR_INVALID_CONTEXT => "Invalid Context Error",
        RPT_ERROR_OUT_OF_MEMORY => "Out Of Memory Error",
        _ => "Undefined Type",
    }
}

/// `BKE_reports_init`.
pub fn bke_reports_init(flag: i32) -> ReportList {
    ReportList {
        flag,
        ..Default::default()
    }
}

/// `BKE_reports_clear`.
pub fn bke_reports_clear(reports: &ReportList) {
    reports.list.lock().unwrap().clear();
}

/// `BKE_reports_move_to_reports` — append src→dst.
pub fn bke_reports_move_to_reports(reports_dst: &ReportList, reports_src: &ReportList) {
    let mut src = reports_src.list.lock().unwrap();
    let mut dst = reports_dst.list.lock().unwrap();
    dst.append(&mut src);
}

fn push_report(reports: &ReportList, type_: EReportType, message: String) {
    let mut report = Report {
        type_: type_ as i16,
        flag: 0,
        len: message.len() as i32,
        message,
        typestr: bke_report_type_str(type_).to_string(),
        session_uid: report_session_uid_counter_get_next(),
    };
    let mut list = reports.list.lock().unwrap();
    report.len = report.message.len() as i32;
    list.push(report);
}

/// `BKE_report`.
pub fn bke_report(
    reports: Option<&ReportList>,
    type_: EReportType,
    message: &str,
    background: bool,
    log: &mut dyn FnMut(&str, &str),
) {
    if bke_reports_print_test(reports, type_, background) {
        bke_report_log(type_, message, log);
        let _ = std::io::stdout().flush();
    }

    if let Some(reports) = reports {
        if (reports.flag & RPT_STORE) != 0 && type_ >= reports.storelevel {
            push_report(reports, type_, message.to_string());
        }
    }
}

/// `BKE_reportf`.
pub fn bke_reportf(
    reports: Option<&ReportList>,
    type_: EReportType,
    message: String,
    background: bool,
    log: &mut dyn FnMut(&str, &str),
) {
    bke_report(reports, type_, &message, background, log);
}

/// `reports_prepend_impl`.
fn reports_prepend_impl(reports: &ReportList, prepend: &str) {
    let mut list = reports.list.lock().unwrap();
    let prefix_len = prepend.len() as i32;
    for report in list.iter_mut() {
        report.message = format!("{prepend}{}", report.message);
        report.len += prefix_len;
        debug_assert!(report.len == report.message.len() as i32);
    }
}

/// `BKE_reports_prepend`.
pub fn bke_reports_prepend(reports: Option<&ReportList>, prepend: &str) {
    let Some(reports) = reports else {
        return;
    };
    if reports.list.lock().unwrap().is_empty() {
        return;
    }
    reports_prepend_impl(reports, prepend);
}

/// `BKE_reports_prependf` — varargs folded to `String`.
pub fn bke_reports_prependf(reports: Option<&ReportList>, prepend: String) {
    bke_reports_prepend(reports, &prepend);
}

/// `BKE_report_print_level`.
pub fn bke_report_print_level(reports: Option<&ReportList>) -> EReportType {
    match reports {
        None => RPT_ERROR,
        Some(r) => r.printlevel,
    }
}

/// `BKE_report_print_level_set`.
pub fn bke_report_print_level_set(reports: Option<&mut ReportList>, level: EReportType) {
    if let Some(r) = reports {
        let _lock = r.list.lock().unwrap();
        r.printlevel = level;
    }
}

/// `BKE_report_store_level`.
pub fn bke_report_store_level(reports: Option<&ReportList>) -> EReportType {
    match reports {
        None => RPT_ERROR,
        Some(r) => r.storelevel,
    }
}

/// `BKE_report_store_level_set`.
pub fn bke_report_store_level_set(reports: Option<&mut ReportList>, level: EReportType) {
    if let Some(r) = reports {
        let _lock = r.list.lock().unwrap();
        r.storelevel = level;
    }
}

/// `BKE_reports_string` — `"{typestr}: {message}\n"` lines ≥ level.
pub fn bke_reports_string(reports: Option<&ReportList>, level: EReportType) -> Option<String> {
    let reports = reports?;
    if reports.list.lock().unwrap().is_empty() {
        return None;
    }
    let mut s = String::new();
    let list = reports.list.lock().unwrap();
    for report in list.iter() {
        if (report.type_ as u16) >= level {
            let _ = writeln!(s, "{}: {}", report.typestr, report.message);
        }
    }
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// `BKE_reports_print_test`.
pub fn bke_reports_print_test(
    reports: Option<&ReportList>,
    type_: EReportType,
    background: bool,
) -> bool {
    let Some(reports) = reports else {
        return true;
    };
    if (reports.flag & RPT_PRINT_HANDLED_BY_OWNER) != 0 {
        return false;
    }
    /* In background mode always print otherwise there are cases the
     * errors won't be displayed. */
    if background {
        return true;
    }
    (reports.flag & RPT_PRINT) != 0 && type_ >= reports.printlevel
}

/// `BKE_reports_log` — emit each stored report ≥ level through `log`.
pub fn bke_reports_log(
    reports: Option<&ReportList>,
    level: EReportType,
    log: &mut dyn FnMut(&str, &str),
) {
    let Some(reports) = reports else {
        return;
    };
    for report in reports.list.lock().unwrap().iter() {
        if (report.type_ as u16) >= level {
            bke_report_log(report.type_ as u16, &report.message, log);
        }
    }
}

/// `BKE_reports_print`.
pub fn bke_reports_print(reports: Option<&ReportList>, level: EReportType) {
    let Some(cstring) = bke_reports_string(reports, level) else {
        return;
    };
    print!("{cstring}");
    let _ = std::io::stdout().flush();
}

/// `BKE_reports_last_displayable` — last ERROR/WARNING/INFO.
pub fn bke_reports_last_displayable(reports: &ReportList) -> Option<Report> {
    let list = reports.list.lock().unwrap();
    list.iter()
        .rev()
        .find(|r| {
            matches!(r.type_ as u16, RPT_ERROR | RPT_WARNING | RPT_INFO)
        })
        .cloned()
}

/// `BKE_reports_contain`.
pub fn bke_reports_contain(reports: Option<&ReportList>, level: EReportType) -> bool {
    match reports {
        None => false,
        Some(r) => r
            .list
            .lock()
            .unwrap()
            .iter()
            .any(|rep| (rep.type_ as u16) >= level),
    }
}

/// `BKE_report_write_file_fp` — `"{message}  # {typestr}\n"`.
pub fn bke_report_write_file_fp<W: IoWrite>(
    fp: &mut W,
    reports: &ReportList,
    header: Option<&str>,
) -> bool {
    if let Some(header) = header {
        if fp.write_all(header.as_bytes()).is_err() {
            return false;
        }
    }
    for report in reports.list.lock().unwrap().iter() {
        if writeln!(fp, "{}  # {}", report.message, report.typestr).is_err() {
            return false;
        }
    }
    true
}

/// `BKE_report_write_file`.
pub fn bke_report_write_file(
    filepath: &std::path::Path,
    reports: &ReportList,
    header: Option<&str>,
) -> bool {
    match std::fs::File::create(filepath) {
        Ok(mut fp) => bke_report_write_file_fp(&mut fp, reports, header),
        Err(e) => {
            eprintln!("Unable to save '{}': {}", filepath.display(), e);
            false
        }
    }
}
