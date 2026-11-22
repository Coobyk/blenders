//! Port of `message_bus/intern/wm_message_bus.cc`,
//! `wm_message_bus_static.cc`, `wm_message_bus_remote_io.cc`,
//! `wm_message_bus_rna.cc` — typed pub/sub bus.

/// `WM_MSG_TYPE_*` verbatim.
pub const WM_MSG_TYPE_RNA: usize = 0;
pub const WM_MSG_TYPE_STATIC: usize = 1;
pub const WM_MSG_TYPE_REMOTE_IO: usize = 2;
pub const WM_MSG_TYPE_NUM: usize = 3;

/// `wmMsgSubscribeValue` verbatim.
#[derive(Default)]
pub struct WmMsgSubscribeValue {
    /// `owner`.
    pub owner: usize,
    /// `user_data`.
    pub user_data: usize,
    /// `notify` handle.
    pub notify: usize,
    /// `free_data` handle.
    pub free_data: usize,
    /// `tag`.
    pub tag: bool,
}

/// `wmMsgSubscribeValueLink` (params + list link).
#[derive(Default)]
pub struct WmMsgSubscribeValueLink {
    /// `params`.
    pub params: WmMsgSubscribeValue,
}

/// `wmMsgSubscribeKey` verbatim.
#[derive(Default)]
pub struct WmMsgSubscribeKey {
    /// `values`.
    pub values: Vec<WmMsgSubscribeValueLink>,
    /// `msg.type`.
    pub msg_type: usize,
    /// `msg.head.id`.
    pub msg_id: usize,
    /// `params` hash key bytes.
    pub params_key: u64,
}

/// `wmMsgBus` verbatim.
#[derive(Default)]
pub struct WmMsgBus {
    /// `messages` list.
    pub messages: Vec<WmMsgSubscribeKey>,
    /// `messages_gset[type]` → key indices.
    pub messages_gset: [std::collections::HashMap<u64, usize>; WM_MSG_TYPE_NUM],
    /// `messages_tag_count`.
    pub messages_tag_count: i32,
}

/// `wm_msg_static_gset_hash`/`cmp` (verbatim — event key).
pub fn wm_msg_static_gset_hash(event: i32) -> u64 {
    event as u64
}

/// `WM_msgbus_create` (verbatim — reserve 512 gsets).
pub fn wm_msgbus_create() -> WmMsgBus {
    WmMsgBus {
        messages: Vec::with_capacity(512),
        ..Default::default()
    }
}

/// `WM_msgbus_clear_by_owner` (verbatim — drop values
/// owned by `owner`, remove empty keys).
pub fn wm_msgbus_clear_by_owner(mbus: &mut WmMsgBus, owner: usize) {
    mbus.messages.retain_mut(|msg_key| {
        msg_key.values.retain(|msg_lnk| {
            if msg_lnk.params.owner == owner {
                if msg_lnk.params.tag {
                    mbus.messages_tag_count -= 1;
                }
                false
            } else {
                true
            }
        });
        !msg_key.values.is_empty()
    });
}

/// `WM_msgbus_handle` (verbatim — notify all tagged,
/// clear tags).
pub fn wm_msgbus_handle(
    mbus: &mut WmMsgBus,
    notify: &mut dyn FnMut(usize, usize),
) {
    if mbus.messages_tag_count == 0 {
        return;
    }
    for key in mbus.messages.iter_mut() {
        for msg_lnk in key.values.iter_mut() {
            if msg_lnk.params.tag {
                (notify)(key.msg_id, msg_lnk.params.user_data);
                msg_lnk.params.tag = false;
                mbus.messages_tag_count -= 1;
            }
        }
    }
    debug_assert_eq!(mbus.messages_tag_count, 0);
    mbus.messages_tag_count = 0;
}

/// `WM_msg_publish_with_key` (verbatim — notify_now or
/// tag for later).
pub fn wm_msg_publish_with_key(
    mbus: &mut WmMsgBus,
    msg_key: &mut WmMsgSubscribeKey,
    notify_now: bool,
    notify: &mut dyn FnMut(usize, usize),
) {
    for msg_lnk in msg_key.values.iter_mut() {
        if notify_now {
            (notify)(msg_key.msg_id, msg_lnk.params.user_data);
        } else if !msg_lnk.params.tag {
            msg_lnk.params.tag = true;
            mbus.messages_tag_count += 1;
        }
    }
}

/// `WM_msg_lookup_static` (verbatim — event-keyed key).
pub fn wm_msg_lookup_static<'a>(
    mbus: &'a mut WmMsgBus,
    event: i32,
) -> Option<&'a mut WmMsgSubscribeKey> {
    let idx = *mbus.messages_gset[WM_MSG_TYPE_STATIC].get(&wm_msg_static_gset_hash(event))?;
    mbus.messages.get_mut(idx)
}
