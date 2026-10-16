//! Zero-copy getters for the per-frame buffers the last [`Context::get_frame_data`] built.

use super::Context;
use crate::resource::StaticMeshBuffers;

impl<'a> Context<'a> {
    /// Pointer to the frame data built by the last [`get_frame_data`] call,
    /// without re-evaluating it. Null until the first call, and again after
    /// `reset` / `bind_resource`; valid until the next `get_frame_data`.
    ///
    /// [`get_frame_data`]: Self::get_frame_data
    pub fn get_frame_data_ptr(&self) -> *const u8 {
        if !self.eval.frame_data_ready {
            return ::core::ptr::null();
        }
        self.eval.fb.finished_data().as_ptr()
    }

    /// Byte length of the buffer [`get_frame_data_ptr`] points at.
    ///
    /// [`get_frame_data_ptr`]: Self::get_frame_data_ptr
    pub fn get_frame_data_len(&self) -> usize {
        if !self.eval.frame_data_ready {
            return 0;
        }
        self.eval.fb.finished_data().len()
    }

    pub fn get_world_matrices_ptr(&self) -> *const f32 {
        self.instance.globals.world_matrices.as_ptr() as *const f32
    }

    pub fn get_world_matrices_len(&self) -> usize {
        self.instance.globals.world_matrices.len() * 16
    }

    pub fn get_local_uvs_ptr(&self) -> *const f32 {
        self.instance.globals.local_uvs.as_ptr()
    }

    pub fn get_local_uvs_len(&self) -> usize {
        self.instance.globals.local_uvs.len()
    }

    pub fn get_shape_vertices_ptr(&self) -> *const f32 {
        self.instance.globals.shape_vertices.as_ptr()
    }

    pub fn get_shape_vertices_len(&self) -> usize {
        self.instance.globals.shape_vertices.len()
    }

    pub fn get_shape_vertex_box_coords_ptr(&self) -> *const f32 {
        self.instance.globals.shape_vertex_box_coords.as_ptr()
    }

    pub fn get_shape_vertex_box_coords_len(&self) -> usize {
        self.instance.globals.shape_vertex_box_coords.len()
    }

    pub fn get_shape_vertex_counts_ptr(&self) -> *const i32 {
        self.instance.globals.shape_vertex_counts.as_ptr()
    }

    pub fn get_mesh_vertices_x_ptr(&self) -> *const f32 {
        self.instance.globals.mesh_vertices_x.as_ptr()
    }

    pub fn get_mesh_vertices_x_len(&self) -> usize {
        self.instance.globals.mesh_vertices_x.len()
    }

    pub fn get_mesh_vertices_y_ptr(&self) -> *const f32 {
        self.instance.globals.mesh_vertices_y.as_ptr()
    }

    pub fn get_mesh_vertices_y_len(&self) -> usize {
        self.instance.globals.mesh_vertices_y.len()
    }

    pub fn get_mesh_vertex_offsets_ptr(&self) -> *const u32 {
        self.instance.globals.mesh_vertex_offsets.as_ptr()
    }

    pub fn get_mesh_vertex_offsets_len(&self) -> usize {
        self.instance.globals.mesh_vertex_offsets.len()
    }

    /// The static mesh buffers this instance draws with: none before the first
    /// Direct-mode `get_frame_data`, and none in SceneGraph mode.
    fn static_mesh(&self) -> Option<&StaticMeshBuffers> {
        self.eval.resource?.static_mesh(self.instance.globals.mesh_layout)
    }

    pub fn get_mesh_uvs_ptr(&self) -> *const f32 {
        self.static_mesh().map_or(::core::ptr::NonNull::dangling().as_ptr(), |m| m.uvs.as_ptr())
    }

    pub fn get_mesh_uvs_len(&self) -> usize {
        self.static_mesh().map_or(0, |m| m.uvs.len())
    }

    pub fn get_mesh_indices_ptr(&self) -> *const i32 {
        self.static_mesh().map_or(::core::ptr::NonNull::dangling().as_ptr(), |m| m.indices.as_ptr())
    }

    pub fn get_mesh_indices_len(&self) -> usize {
        self.static_mesh().map_or(0, |m| m.indices.len())
    }

    pub fn get_mesh_index_offsets_ptr(&self) -> *const u32 {
        self.static_mesh().map_or(::core::ptr::NonNull::dangling().as_ptr(), |m| m.index_offsets.as_ptr())
    }

    pub fn get_mesh_index_offsets_len(&self) -> usize {
        self.static_mesh().map_or(0, |m| m.index_offsets.len())
    }

    pub fn get_nines_vertices_ptr(&self) -> *const f32 {
        self.instance.globals.nines.vertices.as_ptr()
    }

    pub fn get_nines_vertices_len(&self) -> usize {
        self.instance.globals.nines.vertices.len()
    }

    pub fn get_nines_uvs_ptr(&self) -> *const f32 {
        self.instance.globals.nines.uvs.as_ptr()
    }

    pub fn get_nines_uvs_len(&self) -> usize {
        self.instance.globals.nines.uvs.len()
    }

    pub fn get_nines_box_coords_ptr(&self) -> *const f32 {
        self.instance.globals.nines.box_coords.as_ptr()
    }

    pub fn get_nines_box_coords_len(&self) -> usize {
        self.instance.globals.nines.box_coords.len()
    }

    pub fn get_nines_indices_ptr(&self) -> *const i32 {
        self.instance.globals.nines.indices.as_ptr()
    }

    pub fn get_nines_indices_len(&self) -> usize {
        self.instance.globals.nines.indices.len()
    }

    pub fn get_nines_vertex_offsets_ptr(&self) -> *const u32 {
        self.instance.globals.nines.vertex_offsets.as_ptr()
    }

    pub fn get_nines_vertex_offsets_len(&self) -> usize {
        self.instance.globals.nines.vertex_offsets.len()
    }

    pub fn get_nines_index_offsets_ptr(&self) -> *const u32 {
        self.instance.globals.nines.index_offsets.as_ptr()
    }

    pub fn get_nines_index_offsets_len(&self) -> usize {
        self.instance.globals.nines.index_offsets.len()
    }

    pub fn get_shape_vertex_counts_len(&self) -> usize {
        self.instance.globals.shape_vertex_counts.len()
    }

    pub fn get_cell_meta_ptr(&self) -> *const f32 {
        self.instance.resolved_layer.cell_meta_per_part.as_ptr()
    }

    pub fn get_cell_meta_len(&self) -> usize {
        self.instance.resolved_layer.cell_meta_per_part.len()
    }

    pub fn get_local_vertices_ptr(&self) -> *const f32 {
        self.instance.resolved_layer.local_vertices.as_ptr()
    }

    pub fn get_local_vertices_len(&self) -> usize {
        self.instance.resolved_layer.local_vertices.len()
    }
}
