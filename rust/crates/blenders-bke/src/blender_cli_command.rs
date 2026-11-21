//! Port of `intern/blender_cli_command.cc` — `BKE_blender_cli_command_*`
//! registry for `--command` handlers.

use std::sync::Mutex;

/// `CommandHandler` (BKE_blender_cli_command.hh).
pub struct CommandHandler {
    /// Unique command id (`--command <id>`).
    pub id: String,
    /// `exec` — returns the process exit code.
    pub exec: Box<dyn FnMut(&[String]) -> i32 + Send>,
    /// Help lines printed under `--command help`.
    pub help: Vec<String>,
    /// `is_duplicate` — set when another handler shares `id`.
    pub is_duplicate: bool,
}

fn cli_command_handlers() -> &'static Mutex<Vec<CommandHandler>> {
    static HANDLERS: Mutex<Vec<CommandHandler>> = Mutex::new(Vec::new());
    &HANDLERS
}

/// `blender_cli_command_lookup` — first handler with `id`.
fn blender_cli_command_lookup<'a>(handlers: &'a [CommandHandler], id: &str) -> Option<usize> {
    handlers.iter().position(|c| c.id == id)
}

/// `BKE_blender_cli_command_register` — dup IDs stay registered but become
/// inaccessible (verbatim).
pub fn bke_blender_cli_command_register(mut cmd: CommandHandler) {
    let mut handlers = cli_command_handlers().lock().unwrap();
    let mut is_duplicate = false;
    if let Some(idx) = blender_cli_command_lookup(&handlers, &cmd.id) {
        eprintln!("warning: registered duplicate command {:?}, this will be inaccessible", cmd.id);
        handlers[idx].is_duplicate = true;
        is_duplicate = true;
    }
    cmd.is_duplicate = is_duplicate;
    handlers.push(cmd);
}

/// `BKE_blender_cli_command_unregister` — by index (the Vec replaces the
/// pointer identity C used).
pub fn bke_blender_cli_command_unregister(cmd_index: usize) -> bool {
    let mut handlers = cli_command_handlers().lock().unwrap();
    if cmd_index >= handlers.len() {
        eprintln!("failed to unregister command handler");
        return false;
    }
    if handlers[cmd_index].is_duplicate {
        let id = handlers[cmd_index].id.clone();
        /* Clear the flag on another dup of the same id (C: cmd_other). */
        let mut other: Option<usize> = None;
        for (i, c) in handlers.iter().enumerate() {
            if i == cmd_index {
                continue;
            }
            if c.is_duplicate && c.id == id {
                if other.is_some() {
                    other = None;
                    break;
                }
                other = Some(i);
            }
        }
        if let Some(i) = other {
            handlers[i].is_duplicate = false;
        }
    }
    handlers.swap_remove(cmd_index); /* `remove_and_reorder` */
    true
}

/// `BKE_blender_cli_command_exec`.
pub fn bke_blender_cli_command_exec(id: &str, argv: &[String]) -> i32 {
    let mut handlers = cli_command_handlers().lock().unwrap();
    let Some(idx) = blender_cli_command_lookup(&handlers, id) else {
        eprintln!("Unrecognized command: {:?} (see 'help')", id);
        return 1; /* EXIT_FAILURE */
    };
    if handlers[idx].is_duplicate {
        eprintln!("Command: {:?} was registered multiple times, must be resolved, aborting!", id);
        return 1;
    }
    (handlers[idx].exec)(argv)
}

/// `BKE_blender_cli_command_print_help` — sorted listing, then per-command
/// help lines.
pub fn bke_blender_cli_command_print_help() -> String {
    let mut handlers = cli_command_handlers().lock().unwrap();
    handlers.sort_by(|a, b| a.id.cmp(&b.id));
    let mut out = String::new();
    out.push_str("convenience arguments:\n");
    for cmd in handlers.iter() {
        out.push_str(&format!("\t{}\n", cmd.id));
    }
    for cmd in handlers.iter() {
        for line in &cmd.help {
            out.push_str(&format!("{}\n", line));
        }
    }
    out
}
