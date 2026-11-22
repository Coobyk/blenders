//! Port of `intern/blt_translation.cc` — pgettext +
//! translate gates.

/// `BLT_I18NCONTEXT_DEFAULT`/`_BPYRNA` verbatim.
pub const BLT_I18NCONTEXT_DEFAULT: &str = "";
pub const BLT_I18NCONTEXT_DEFAULT_BPYRNA: &str = "bpy.";
/// `USER_TR_*` transopts verbatim.
pub const USER_TR_IFACE: i32 = 1 << 0;
pub const USER_TR_TOOLTIPS: i32 = 1 << 1;
pub const USER_TR_REPORTS: i32 = 1 << 2;
pub const USER_TR_NEWDATANAME: i32 = 1 << 3;

/// `BLT_is_default_context` (verbatim — empty or
/// bpy-prefix first char).
pub fn blt_is_default_context(msgctxt: &str) -> bool {
    msgctxt.is_empty() || msgctxt.as_bytes()[0] == BLT_I18NCONTEXT_DEFAULT_BPYRNA.as_bytes()[0]
}

/// `BLT_translate_*` gates (verbatim — WITH_INTERNATIONAL
/// + main thread + transopts flag).
pub fn blt_translate(is_main_thread: bool) -> bool {
    is_main_thread
}
pub fn blt_translate_iface(transopts: i32, is_main_thread: bool) -> bool {
    blt_translate(is_main_thread) && (transopts & USER_TR_IFACE) != 0
}
pub fn blt_translate_tooltips(transopts: i32, is_main_thread: bool) -> bool {
    blt_translate(is_main_thread) && (transopts & USER_TR_TOOLTIPS) != 0
}
pub fn blt_translate_reports(transopts: i32, is_main_thread: bool) -> bool {
    blt_translate(is_main_thread) && (transopts & USER_TR_REPORTS) != 0
}
pub fn blt_translate_new_dataname(transopts: i32, is_main_thread: bool) -> bool {
    blt_translate(is_main_thread) && (transopts & USER_TR_NEWDATANAME) != 0
}

/// `BLT_pgettext` (verbatim — default-ctx swap, lookup,
/// fallback msgid).
pub fn blt_pgettext<'a>(
    msgctxt: &'a str,
    msgid: &'a str,
    locale_translate: &dyn Fn(&str, &str) -> Option<String>,
) -> String {
    if msgid.is_empty() {
        return msgid.to_string();
    }
    let ctx = if blt_is_default_context(msgctxt) {
        BLT_I18NCONTEXT_DEFAULT
    } else {
        msgctxt
    };
    (locale_translate)(ctx, msgid).unwrap_or_else(|| msgid.to_string())
}
