/// Port of `intern/gpu_vertex_format.cc` —
/// GPUVertFormat attribute layout.
/// `eGPUVertCompType` ordinals (verbatim).
pub const GPU_COMP_I8: i32 = 0;
pub const GPU_COMP_U8: i32 = 1;
pub const GPU_COMP_I16: i32 = 2;
pub const GPU_COMP_U16: i32 = 3;
pub const GPU_COMP_I32: i32 = 4;
pub const GPU_COMP_U32: i32 = 5;
pub const GPU_COMP_F32: i32 = 6;
pub const GPU_COMP_I10: i32 = 7;
/// `eGPUVertFetchMode` ordinals (verbatim).
pub const GPU_FETCH_FLOAT: i32 = 0;
pub const GPU_FETCH_INT: i32 = 1;
pub const GPU_FETCH_INT_TO_FLOAT_UNIT: i32 = 2;
/// `GPUVertAttr` (verbatim — one attribute
/// entry).
#[derive(Clone)]
pub struct GpuVertAttr {
    /// `name`.
    pub name: String,
    /// `comp_type` (eGPUVertCompType).
    pub comp_type: i32,
    /// `comp_len` (1-4).
    pub comp_len: i32,
    /// `fetch_mode`.
    pub fetch_mode: i32,
    /// `offset` in packed struct.
    pub offset: i32,
}
/// `GPUVertFormat` (verbatim — attribute list
/// + stride).
#[derive(Default)]
pub struct GpuVertFormat {
    /// `attrs`.
    pub attrs: Vec<GpuVertAttr>,
    /// `stride`.
    pub stride: i32,
    /// `packed`.
    pub packed: bool,
}
impl GpuVertFormat {
    /// `GPU_vertformat_attr_add` (verbatim —
    /// append + bump stride).
    pub fn attr_add(&mut self, name: &str, comp_type: i32, comp_len: i32, fetch_mode: i32) {
        let comp_size = match comp_type {
            GPU_COMP_I8 | GPU_COMP_U8 => 1,
            GPU_COMP_I16 | GPU_COMP_U16 => 2,
            GPU_COMP_I10 => 4,
            _ => 4,
        };
        self.attrs.push(GpuVertAttr {
            name: name.to_string(),
            comp_type,
            comp_len,
            fetch_mode,
            offset: self.stride,
        });
        self.stride += comp_len * comp_size;
    }
}
