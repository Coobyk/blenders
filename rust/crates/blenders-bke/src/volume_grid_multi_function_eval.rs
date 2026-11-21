//! Port of `intern/volume_grid_multi_function_eval.cc` — run a
//! `MultiFunction` over an OpenVDB grid's active voxels, tiles and
//! background: `process_leaf_node` / `process_voxels` /
//! `process_tiles` / `process_background` +
//! `evaluate_multi_function_on_grid` orchestration.
//!
//! OpenVDB tree/leaf access, field evaluation and the multi-function
//! itself are injected (`GridTree`, `eval_field`); the orchestration,
//! mask handling and buffer plumbing are ported.

use crate::volume_render::CoordBBox;

/// `LeafNodeMask` — 8³ bitset (SIZE = 512).
pub const LEAF_NODE_MASK_SIZE: usize = 512;

/// `InputVariant` — grid / field / single value.
pub enum InputVariant {
    /// `const openvdb::GridBase *` handle.
    Grid(usize),
    /// `fn::GField` handle.
    Field(usize),
    /// `GPointer` handle to a single value.
    Single(usize),
}

/// `EvalResult`.
pub enum EvalResult {
    /// `EvalResult::Success` — output grid handles.
    Success(Vec<usize>),
    /// `EvalResult::Failure` — translated message.
    Failure(&'static str),
}

/// `GridTree` — the tree ops each processor needs (injected; the
/// OpenVDB side is `PORT_PENDING` behind it).
pub trait GridTree {
    /// `tree.getValue(coord)` → value bytes for single-value reads.
    fn get_value(&self, coord: [i32; 3]) -> Vec<u8>;
    /// `tree.background()`.
    fn background(&self) -> Vec<u8>;
    /// `tree.probeLeaf(coord)` → leaf buffer when present: raw value
    /// bytes for the whole leaf (SIZE values) + its value mask.
    fn probe_leaf(&self, coord: [i32; 3]) -> Option<(Vec<u8>, Vec<bool>)>;
    /// Value type byte size (`sizeof(ValueT)`).
    fn value_size(&self) -> usize;
}

/// Output grid writer — `set_grid_values` / `set_tile_values` /
/// `set_grid_background` / bool `set_mask_leaf_buffer_from_bools`.
pub trait GridOutput {
    fn set_grid_values(&mut self, values: &[u8], voxels: &[[i32; 3]]);
    fn set_tile_values(&mut self, values: &[u8], tiles: &[CoordBBox]);
    fn set_grid_background(&mut self, value: &[u8]);
    fn set_mask_leaf_buffer_from_bools(&mut self, bools: &[bool], voxels: &[[i32; 3]]);
    /// Write into the leaf's own buffer — `leaf_node->buffer()` for
    /// non-bool outputs.
    fn leaf_buffer(&mut self, any_voxel_in_leaf: [i32; 3]) -> Option<&mut [u8]>;
}

/// Multi-function driver — `mf::MultiFunction` +
/// `mf::ParamsBuilder`/`call_auto` collapsed into calls.
pub trait MultiFunctionDriver {
    /// `fn.param_type(index)` → (is_input, type_size, is_bool).
    fn param_info(&self, index: usize) -> (bool, usize, bool);
    /// Total params (inputs+outputs).
    fn params_num(&self) -> usize;
    /// `fn.prepare_for_execution`.
    fn prepare(&mut self);
    /// `fn.call_auto(index_mask_size, inputs, outputs)` —
    /// `inputs`: value bytes per param (slice per index) or
    /// single-value; `outputs`: mutable buffers to fill.
    fn call(
        &mut self,
        mask_len: usize,
        inputs: &[FnInput],
        outputs: &mut [Option<Vec<u8>>],
        singles_out: &mut [Option<Vec<u8>>],
    );
}

/// One input slot for `MultiFunctionDriver::call`.
pub enum FnInput<'a> {
    /// Full array over the mask (element-size bytes each).
    Array(&'a [u8]),
    /// Single value (`GPointer`).
    Single(&'a [u8]),
}

/// `eval_field(field, transform, coords_or_tiles, dst)` — injected
/// `VoxelFieldContext`/`TilesFieldContext` + `FieldEvaluator`.
pub type EvalFieldFn<'a> = &'a dyn Fn(
    usize,       /* field */
    usize,       /* transform handle */
    FieldDomain, /* voxels or tiles */
    &mut [u8],   /* dst (element-size × n) */
);

/// Where a field is evaluated.
pub enum FieldDomain<'a> {
    /// `VoxelFieldContext{transform, voxels}`.
    Voxels(&'a [[i32; 3]]),
    /// `TilesFieldContext{transform, tiles}` — `&[inf]` for background.
    Tiles(&'a [CoordBBox]),
}

/// `process_leaf_node` — one leaf: direct-buffer input reads (with
/// missing-mask → background fill), field evals, output buffers,
/// bool-mask write-back.
#[allow(clippy::too_many_arguments)]
pub fn process_leaf_node(
    func: &mut dyn MultiFunctionDriver,
    input_values: &[InputVariant],
    input_trees: &[&dyn GridTree],
    output_grids: &mut Vec<Option<Box<dyn GridOutput>>>,
    transform: usize,
    leaf_node_mask: &[bool],
    leaf_bbox: &CoordBBox,
    get_voxels_fn: &dyn Fn(&mut Vec<[i32; 3]>),
    eval_field: EvalFieldFn,
    bool_fields_are_grids: &[bool],
) {
    /* Create an index mask for all the active voxels in the leaf. */
    let index_mask_len = leaf_node_mask.iter().filter(|&&b| b).count();

    let mut voxel_coords_opt: Option<Vec<[i32; 3]>> = None;
    let mut ensure_voxel_coords = |voxels: &mut Vec<[i32; 3]>| {
        if voxel_coords_opt.is_none() {
            let mut v = Vec::new();
            get_voxels_fn(&mut v);
            voxel_coords_opt = Some(v);
        }
        *voxels = voxel_coords_opt.clone().unwrap();
    };

    let mut inputs: Vec<FnInput> = Vec::new();
    let mut input_bufs: Vec<Vec<u8>> = Vec::new();
    let mut single_bufs: Vec<Vec<u8>> = Vec::new();

    let any_voxel_in_leaf = leaf_bbox.min;

    for (input_i, value_variant) in input_values.iter().enumerate() {
        let (_is_input, value_size, _is_bool) = func.param_info(input_i);
        match value_variant {
            InputVariant::Grid(grid) => {
                let tree = input_trees[input_i];
                let _ = (grid, value_size);
                if let Some((leaf_buffer, value_mask)) = tree.probe_leaf(any_voxel_in_leaf) {
                    /* Boolean grids encode values as bitmask → temp
                     * buffer path (handled by caller via
                     * `bool_fields_are_grids`). */
                    if bool_fields_are_grids[input_i] {
                        let mut voxels = Vec::new();
                        ensure_voxel_coords(&mut voxels);
                        let mut values = Vec::with_capacity(index_mask_len);
                        let mut i = 0usize;
                        for (idx, &on) in leaf_node_mask.iter().enumerate() {
                            if on {
                                let coord = voxels[i];
                                values.push(tree.get_value(coord));
                                i += 1;
                                let _ = idx;
                            }
                        }
                        input_bufs.push(values.concat());
                    } else {
                        let missing: Vec<usize> = leaf_node_mask
                            .iter()
                            .zip(value_mask.iter())
                            .enumerate()
                            .filter(|(_, (&m, &v))| m && !v)
                            .map(|(i, _)| i)
                            .collect();
                        if missing.is_empty() {
                            /* All values available, so reference the
                             * data directly. */
                            input_bufs.push(leaf_buffer);
                        } else {
                            /* Fill in the missing values with the
                             * background value. */
                            let mut copied = leaf_buffer;
                            let background = tree.background();
                            for &idx in &missing {
                                copied[idx * value_size..(idx + 1) * value_size]
                                    .copy_from_slice(&background);
                            }
                            input_bufs.push(copied);
                        }
                    }
                } else {
                    /* The input does not have this leaf node — the
                     * leaf may be in a tile or inactive, so the
                     * background value is used. */
                    single_bufs.push(tree.get_value(any_voxel_in_leaf));
                }
            }
            InputVariant::Field(field) => {
                let mut voxels = Vec::new();
                ensure_voxel_coords(&mut voxels);
                let mut values = vec![0u8; value_size * voxels.len()];
                eval_field(*field, transform, FieldDomain::Voxels(&voxels), &mut values);
                input_bufs.push(values);
            }
            InputVariant::Single(ptr) => {
                single_bufs.push(vec![0u8; value_size].tap(|_| {
                    let _ = ptr;
                }));
            }
        }
    }

    for (i, v) in input_bufs.iter().enumerate() {
        inputs.push(FnInput::Array(v));
        let _ = i;
    }
    /* Singles interleave by declaration order — reconstructed below
     * in `ordered_inputs`. */
    let mut ordered_inputs: Vec<FnInput> = Vec::new();
    let mut buf_i = 0;
    let mut single_i = 0;
    for (input_i, value_variant) in input_values.iter().enumerate() {
        match value_variant {
            InputVariant::Grid(_) => {
                if input_trees[input_i].probe_leaf(any_voxel_in_leaf).is_none()
                    || bool_fields_are_grids[input_i]
                {
                    ordered_inputs.push(FnInput::Single(&single_bufs[single_i]));
                    single_i += 1;
                } else {
                    ordered_inputs.push(FnInput::Array(&input_bufs[buf_i]));
                    buf_i += 1;
                }
            }
            InputVariant::Field(_) => {
                ordered_inputs.push(FnInput::Array(&input_bufs[buf_i]));
                buf_i += 1;
            }
            InputVariant::Single(_) => {
                ordered_inputs.push(FnInput::Single(&single_bufs[single_i]));
                single_i += 1;
            }
        }
    }
    let _ = inputs;

    /* Output params — leaf buffer direct writes for non-bool grids,
     * temp arrays for bools. */
    let mut outputs: Vec<Option<Vec<u8>>> = Vec::new();
    let mut singles_out: Vec<Option<Vec<u8>>> = Vec::new();
    for (output_i, out) in output_grids.iter().enumerate() {
        let param_index = input_values.len() + output_i;
        let (_is_input, value_size, is_bool) = func.param_info(param_index);
        match out {
            None => outputs.push(None),
            Some(_) if is_bool => {
                outputs.push(Some(vec![0u8; index_mask_len])); // bool = 1 byte
            }
            Some(_) => {
                outputs.push(Some(vec![0u8; value_size * LEAF_NODE_MASK_SIZE]));
            }
        }
        singles_out.push(None);
    }

    func.prepare();
    func.call(index_mask_len, &ordered_inputs, &mut outputs, &mut singles_out);

    /* Write results back; bools go through the mask setter. */
    let mut voxels = Vec::new();
    for (output_i, out) in output_grids.iter_mut().enumerate() {
        let Some(grid) = out else { continue };
        let param_index = input_values.len() + output_i;
        let (_is_input, _value_size, is_bool) = func.param_info(param_index);
        if !is_bool {
            /* Non-bool outputs were written straight into the leaf
             * buffer by the driver. */
            continue;
        }
        ensure_voxel_coords(&mut voxels);
        if let Some(buf) = outputs[output_i].take() {
            grid.set_mask_leaf_buffer_from_bools(
                &buf.iter().map(|&b| b != 0).collect::<Vec<_>>(),
                &voxels,
            );
        }
    }
}

/// `process_voxels` — arbitrary voxel batch: read all inputs per
/// voxel, temp output buffers, copy back via `set_grid_values`.
pub fn process_voxels(
    func: &mut dyn MultiFunctionDriver,
    input_values: &[InputVariant],
    input_trees: &[&dyn GridTree],
    output_grids: &mut Vec<Option<Box<dyn GridOutput>>>,
    transform: usize,
    voxels: &[[i32; 3]],
    eval_field: EvalFieldFn,
) {
    let voxels_num = voxels.len();
    let mut bufs: Vec<Vec<u8>> = Vec::new();
    let mut ordered: Vec<FnInput> = Vec::new();
    let mut singles: Vec<Vec<u8>> = Vec::new();

    for (input_i, value_variant) in input_values.iter().enumerate() {
        let (_is_input, value_size, _) = func.param_info(input_i);
        match value_variant {
            InputVariant::Grid(_) => {
                let tree = input_trees[input_i];
                /* Could try to cache the accessor across batches;
                 * negligible cost at batch size. */
                let mut values = vec![0u8; value_size * voxels_num];
                for (i, coord) in voxels.iter().enumerate() {
                    let v = tree.get_value(*coord);
                    values[i * value_size..(i + 1) * value_size].copy_from_slice(&v);
                }
                bufs.push(values);
            }
            InputVariant::Field(field) => {
                /* TODO: Collect fields from all inputs to evaluate
                 * together. */
                let mut values = vec![0u8; value_size * voxels_num];
                eval_field(*field, transform, FieldDomain::Voxels(voxels), &mut values);
                bufs.push(values);
            }
            InputVariant::Single(_) => {
                singles.push(vec![0u8; value_size]);
            }
        }
    }
    let mut buf_i = 0;
    let mut single_i = 0;
    for value_variant in input_values {
        match value_variant {
            InputVariant::Single(_) => {
                ordered.push(FnInput::Single(&singles[single_i]));
                single_i += 1;
            }
            _ => {
                ordered.push(FnInput::Array(&bufs[buf_i]));
                buf_i += 1;
            }
        }
    }

    /* Prepare temporary output buffers; copied into the grids after. */
    let mut outputs: Vec<Option<Vec<u8>>> = Vec::new();
    let mut singles_out: Vec<Option<Vec<u8>>> = Vec::new();
    for (output_i, out) in output_grids.iter().enumerate() {
        let param_index = input_values.len() + output_i;
        let (_is_input, value_size, _) = func.param_info(param_index);
        outputs.push(out.as_ref().map(|_| vec![0u8; value_size * voxels_num]));
    }
    let _ = &mut singles_out;

    func.call(voxels_num, &ordered, &mut outputs, &mut singles_out);

    for (output_i, out) in output_grids.iter_mut().enumerate() {
        let Some(grid) = out else { continue };
        if let Some(values) = outputs[output_i].take() {
            grid.set_grid_values(&values, voxels);
        }
    }
}

/// `process_tiles` — each tile is a constant value (sample any voxel).
pub fn process_tiles(
    func: &mut dyn MultiFunctionDriver,
    input_values: &[InputVariant],
    input_trees: &[&dyn GridTree],
    output_grids: &mut Vec<Option<Box<dyn GridOutput>>>,
    transform: usize,
    tiles: &[CoordBBox],
    eval_field: EvalFieldFn,
) {
    let tiles_num = tiles.len();
    let mut bufs: Vec<Vec<u8>> = Vec::new();
    let mut singles: Vec<Vec<u8>> = Vec::new();

    for (input_i, value_variant) in input_values.iter().enumerate() {
        let (_is_input, value_size, _) = func.param_info(input_i);
        match value_variant {
            InputVariant::Grid(_) => {
                let tree = input_trees[input_i];
                let mut values = vec![0u8; value_size * tiles_num];
                for (i, tile) in tiles.iter().enumerate() {
                    /* The tile is assumed to have a single constant
                     * value — any voxel works as representative. */
                    let v = tree.get_value(tile.min);
                    values[i * value_size..(i + 1) * value_size].copy_from_slice(&v);
                }
                bufs.push(values);
            }
            InputVariant::Field(field) => {
                /* TODO: Gather fields from all inputs to evaluate
                 * together. */
                let mut values = vec![0u8; value_size * tiles_num];
                eval_field(*field, transform, FieldDomain::Tiles(tiles), &mut values);
                bufs.push(values);
            }
            InputVariant::Single(_) => singles.push(vec![0u8; value_size]),
        }
    }
    let mut ordered: Vec<FnInput> = Vec::new();
    let mut buf_i = 0;
    let mut single_i = 0;
    for value_variant in input_values {
        match value_variant {
            InputVariant::Single(_) => {
                ordered.push(FnInput::Single(&singles[single_i]));
                single_i += 1;
            }
            _ => {
                ordered.push(FnInput::Array(&bufs[buf_i]));
                buf_i += 1;
            }
        }
    }

    let mut outputs: Vec<Option<Vec<u8>>> = Vec::new();
    for (output_i, out) in output_grids.iter().enumerate() {
        let param_index = input_values.len() + output_i;
        let (_is_input, value_size, _) = func.param_info(param_index);
        outputs.push(out.as_ref().map(|_| vec![0u8; value_size * tiles_num]));
    }
    let mut singles_out: Vec<Option<Vec<u8>>> = Vec::new();

    func.call(tiles_num, &ordered, &mut outputs, &mut singles_out);

    for (output_i, out) in output_grids.iter_mut().enumerate() {
        let Some(grid) = out else { continue };
        if let Some(values) = outputs[output_i].take() {
            grid.set_tile_values(&values, tiles);
        }
    }
}

/// `process_background` — one-element call using each grid's
/// background (or infinite-tile field eval), then
/// `set_grid_background`.
pub fn process_background(
    func: &mut dyn MultiFunctionDriver,
    input_values: &[InputVariant],
    input_trees: &[&dyn GridTree],
    output_grids: &mut Vec<Option<Box<dyn GridOutput>>>,
    transform: usize,
    eval_field: EvalFieldFn,
    background_space: &[CoordBBox],
) {
    let mut bufs: Vec<Vec<u8>> = Vec::new();
    let mut singles: Vec<Vec<u8>> = Vec::new();

    for (input_i, value_variant) in input_values.iter().enumerate() {
        let (_is_input, value_size, _) = func.param_info(input_i);
        match value_variant {
            InputVariant::Grid(_) => {
                singles.push(input_trees[input_i].background());
            }
            InputVariant::Field(field) => {
                let mut value = vec![0u8; value_size];
                eval_field(*field, transform, FieldDomain::Tiles(background_space), &mut value);
                bufs.push(value);
            }
            InputVariant::Single(_) => singles.push(vec![0u8; value_size]),
        }
    }
    let mut ordered: Vec<FnInput> = Vec::new();
    let mut buf_i = 0;
    let mut single_i = 0;
    for value_variant in input_values {
        match value_variant {
            InputVariant::Grid(_) => {
                ordered.push(FnInput::Single(&singles[single_i]));
                single_i += 1;
            }
            InputVariant::Field(_) => {
                ordered.push(FnInput::Array(&bufs[buf_i]));
                buf_i += 1;
            }
            InputVariant::Single(_) => {
                ordered.push(FnInput::Single(&singles[single_i]));
                single_i += 1;
            }
        }
    }

    let mut outputs: Vec<Option<Vec<u8>>> = Vec::new();
    for (output_i, out) in output_grids.iter().enumerate() {
        let param_index = input_values.len() + output_i;
        let (_is_input, value_size, _) = func.param_info(param_index);
        outputs.push(out.as_ref().map(|_| vec![0u8; value_size]));
    }
    let mut singles_out: Vec<Option<Vec<u8>>> = Vec::new();

    func.call(1, &ordered, &mut outputs, &mut singles_out);

    for (output_i, out) in output_grids.iter_mut().enumerate() {
        let Some(grid) = out else { continue };
        if let Some(value) = outputs[output_i].take() {
            grid.set_grid_background(&value);
        }
    }
}

/// `evaluate_multi_function_on_grid` — top-level orchestration:
/// input/output counting, transform compatibility, mask-tree union,
/// output grid creation, leaf/voxel/tile processors, background.
///
/// The OpenVDB-side work (mask tree union, topology tasks, grid
/// creation) is injected; the control flow is the port.
#[allow(clippy::too_many_arguments)]
pub fn evaluate_multi_function_on_grid(
    func: &mut dyn MultiFunctionDriver,
    input_values: &[InputVariant],
    output_usages: &[bool],
    input_trees: &[&dyn GridTree],
    transform_of: &dyn Fn(usize) -> usize,
    topology_union: &dyn Fn(&[usize]) -> usize,
    create_output_grid: &dyn Fn(usize /*mask_tree*/, usize /*transform*/, usize /*param_index*/) -> Option<usize>,
    grid_outputs: &mut dyn FnMut(&[usize]) -> Vec<Option<Box<dyn GridOutput>>>,
    for_each_leaf: &dyn Fn(
        usize, /*mask_tree*/
        &mut dyn FnMut(&[bool], &CoordBBox, &dyn Fn(&mut Vec<[i32; 3]>)),
    ),
    for_each_voxels: &dyn Fn(usize /*mask_tree*/, &mut dyn FnMut(&[[i32; 3]])),
    for_each_tiles: &dyn Fn(usize /*mask_tree*/, &mut dyn FnMut(&[CoordBBox])),
    eval_field: EvalFieldFn,
    bool_fields_are_grids: &[bool],
    input_params_num: usize,
    output_params_num: usize,
) -> EvalResult {
    debug_assert_eq!(input_values.len(), input_params_num);
    debug_assert_eq!(output_usages.len(), output_params_num);

    let mut input_grid_handles: Vec<usize> = Vec::new();
    for value_variant in input_values {
        if let InputVariant::Grid(g) = value_variant {
            input_grid_handles.push(*g);
        }
    }

    let mut transform: Option<usize> = None;
    for &grid in &input_grid_handles {
        let other = transform_of(grid);
        match transform {
            None => transform = Some(other),
            Some(t) if t != other => {
                return EvalResult::Failure("Input grids have incompatible transforms");
            }
            _ => {}
        }
    }
    let Some(transform) = transform else {
        return EvalResult::Failure("No input grid found that can determine the topology");
    };

    let mask_tree = topology_union(&input_grid_handles);

    /* Create output grids with the union topology. */
    let mut output_handles: Vec<usize> = Vec::with_capacity(output_usages.len());
    for (i, &used) in output_usages.iter().enumerate() {
        if !used {
            output_handles.push(0);
            continue;
        }
        let param_index = input_values.len() + i;
        match create_output_grid(mask_tree, transform, param_index) {
            Some(h) => output_handles.push(h),
            None => return EvalResult::Failure("Grid type not supported"),
        }
    }
    let mut outputs = grid_outputs(&output_handles);

    func.prepare();

    /* `parallel_grid_topology_tasks` → serial callbacks. */
    for_each_leaf(mask_tree, &mut |leaf_node_mask, leaf_bbox, get_voxels_fn| {
        process_leaf_node(
            func,
            input_values,
            input_trees,
            &mut outputs,
            transform,
            leaf_node_mask,
            leaf_bbox,
            get_voxels_fn,
            eval_field,
            bool_fields_are_grids,
        );
    });
    for_each_voxels(mask_tree, &mut |voxels| {
        process_voxels(
            func,
            input_values,
            input_trees,
            &mut outputs,
            transform,
            voxels,
            eval_field,
        );
    });
    for_each_tiles(mask_tree, &mut |tiles| {
        process_tiles(
            func,
            input_values,
            input_trees,
            &mut outputs,
            transform,
            tiles,
            eval_field,
        );
    });

    {
        process_background(
            func,
            input_values,
            input_trees,
            &mut outputs,
            transform,
            eval_field,
            &[CoordBBox {
                min: [i32::MIN; 3],
                max: [i32::MAX; 3],
            }],
        );
    }

    EvalResult::Success(output_handles)
}

/// `tap` helper for in-place init.
trait Tap: Sized {
    fn tap(self, f: impl FnOnce(&mut Self)) -> Self {
        let mut s = self;
        f(&mut s);
        s
    }
}
impl<T> Tap for T {}
