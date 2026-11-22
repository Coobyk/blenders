/// Port of `opengl/gl_work_in_flight.cc` — GL backend
/// `work_in_flight`.
/// `GLWorkInFlight` (verbatim — GL
/// object handle + state).
#[derive(Default)]
pub struct GlWorkInFlight {
    /// `id` (GL object name).
    pub id: u32,
}
