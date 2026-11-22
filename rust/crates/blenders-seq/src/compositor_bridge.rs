//! Port of `intern/compositor.cc` — strip →
//! compositor node-tree bridge (per-type input
//! population).

/// `seq_compositor` props (verbatim — one input
/// per strip input socket).
#[derive(Default)]
pub struct SequencerCompositorProps {
    /// `scene`/`strip`/`frame` handles.
    pub scene: usize,
    pub strip: usize,
    /// `props` name→value.
    pub props: Vec<(String, f32)>,
}
