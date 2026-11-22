/// Port of `intern/gpu_shader_log.cc` —
/// shader compile log parse (line:char err
/// extraction).
/// `parse_error_line` (verbatim —
/// `file:line:char: type: msg` split).
pub fn parse_error_line(line: &str) -> Option<(String, i32, i32, String)> {
    let mut it = line.splitn(4, ':');
    let file = it.next()?;
    let line_no: i32 = it.next()?.trim().parse().ok()?;
    let col: i32 = it.next()?.trim().parse().ok()?;
    let msg = it.next()?.to_string();
    Some((file.to_string(), line_no, col, msg))
}
