//! Port of `intern/strip_connect.cc` — StripConnection
//! list ops (bidirectional linked refs → sets).

/// `StripConnection` verbatim.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct StripConnection {
    /// `strip_ref`.
    pub strip_ref: usize,
}
/// `connections_duplicate` (verbatim — copy list).
pub fn connections_duplicate(src: &[StripConnection]) -> Vec<StripConnection> {
    src.to_vec()
}
/// `disconnect(strip)` (verbatim — removes `strip`'s
/// outbound conns + the inbound conns pointing back).
pub fn disconnect(
    connections: &mut std::collections::HashMap<usize, Vec<StripConnection>>,
    strip: usize,
) -> bool {
    let conns = match connections.get(&strip) {
        Some(c) if !c.is_empty() => c.clone(),
        _ => return false,
    };
    for con in conns {
        let other = con.strip_ref;
        if let Some(other_conns) = connections.get_mut(&other) {
            other_conns.retain(|c| c.strip_ref != strip);
        }
    }
    connections.insert(strip, Vec::new());
    true
}
/// `disconnect(list)` (verbatim — or-fold changed).
pub fn disconnect_list(
    connections: &mut std::collections::HashMap<usize, Vec<StripConnection>>,
    strip_list: &[usize],
) -> bool {
    let mut changed = false;
    for &strip in strip_list {
        changed |= disconnect(connections, strip);
    }
    changed
}
/// `cut_one_way_connections` (verbatim — strip→other
/// without other→strip is one-way → drop).
pub fn cut_one_way_connections(
    connections: &mut std::collections::HashMap<usize, Vec<StripConnection>>,
    strip: usize,
) {
    let conns = connections.get(&strip).cloned().unwrap_or_default();
    for con in conns {
        let other = con.strip_ref;
        let is_one_way = connections
            .get(&other)
            .map(|l| !l.iter().any(|c| c.strip_ref == strip))
            .unwrap_or(true);
        if is_one_way {
            if let Some(list) = connections.get_mut(&strip) {
                list.retain(|c| c.strip_ref != other);
            }
        }
    }
}
/// `cut_outside_connections` (verbatim — drop conns
/// to strips not in the set, both directions).
pub fn cut_outside_connections(
    connections: &mut std::collections::HashMap<usize, Vec<StripConnection>>,
    strip_list: &[usize],
) {
    let set: std::collections::HashSet<usize> = strip_list.iter().copied().collect();
    for &strip in strip_list {
        let conns = connections.get(&strip).cloned().unwrap_or_default();
        for con in conns {
            if !set.contains(&con.strip_ref) {
                let other = con.strip_ref;
                if let Some(l) = connections.get_mut(&other) {
                    l.retain(|c| c.strip_ref != strip);
                }
                if let Some(l) = connections.get_mut(&strip) {
                    l.retain(|c| c.strip_ref != other);
                }
            }
        }
    }
}
/// `connect(list)` (verbatim — disconnect each then
/// link to every other).
pub fn connect(
    connections: &mut std::collections::HashMap<usize, Vec<StripConnection>>,
    strip_list: &[usize],
) {
    for &strip1 in strip_list {
        disconnect(connections, strip1);
    }
    for &strip1 in strip_list {
        let list: Vec<StripConnection> = strip_list
            .iter()
            .filter(|&&s2| s2 != strip1)
            .map(|&s2| StripConnection { strip_ref: s2 })
            .collect();
        connections.insert(strip1, list);
    }
}
/// `connected_strips_get` (verbatim — outbound set).
pub fn connected_strips_get(
    connections: &std::collections::HashMap<usize, Vec<StripConnection>>,
    strip: usize,
) -> Vec<usize> {
    connections
        .get(&strip)
        .map(|l| l.iter().map(|c| c.strip_ref).collect())
        .unwrap_or_default()
}
/// `is_strip_connected` (verbatim).
pub fn is_strip_connected(
    connections: &std::collections::HashMap<usize, Vec<StripConnection>>,
    strip: usize,
) -> bool {
    connections.get(&strip).map(|l| !l.is_empty()).unwrap_or(false)
}
/// `are_strips_connected_together` (verbatim —
/// each has exactly n-1 conns all inside set).
pub fn are_strips_connected_together(
    connections: &std::collections::HashMap<usize, Vec<StripConnection>>,
    strip_list: &[usize],
) -> bool {
    let expected = strip_list.len().saturating_sub(1);
    let set: std::collections::HashSet<usize> = strip_list.iter().copied().collect();
    for &strip1 in strip_list {
        let conns = connected_strips_get(connections, strip1);
        if conns.len() != expected {
            return false;
        }
        for strip2 in conns {
            if !set.contains(&strip2) {
                return false;
            }
        }
    }
    true
}
