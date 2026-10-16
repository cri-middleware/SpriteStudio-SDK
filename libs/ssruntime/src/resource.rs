//! A parsed `.ssab`: its bytes, each animation's metadata, and the caches built from them on first use.

use alloc::vec::Vec;

use crate::error;

/// A read-only borrow of an external SSAB byte buffer.
///
/// The pointer/length pair is never observed publicly; it is only constructed
/// inside [`SsabData::load_borrow`] (whose `# Safety` clause owns the validity
/// invariant) and read internally via [`SsabStorage::as_slice`].
struct BorrowedBuffer {
    ptr: ::core::ptr::NonNull<u8>,
    len: usize,
}

// SAFETY: `BorrowedBuffer` represents a borrow of read-only memory whose
// validity is upheld by the caller of `SsabData::load_borrow`. Nothing in this
// crate mutates the pointed-to bytes, so transferring (`Send`) or sharing
// (`Sync`) the handle does not introduce data races as long as the caller's
// contract — that the memory remains valid and unmodified for the duration of
// the borrow, including across threads — is honored.
unsafe impl Send for BorrowedBuffer {}
unsafe impl Sync for BorrowedBuffer {}

enum SsabStorage {
    Owned(Vec<u8>),
    Borrowed(BorrowedBuffer),
}
impl SsabStorage {
    pub fn as_slice(&self) -> &[u8] {
        match self {
            SsabStorage::Owned(vec) => vec.as_slice(),
            // SAFETY: see the safety contract on `BorrowedBuffer` and
            // `SsabData::load_borrow`.
            SsabStorage::Borrowed(b) => unsafe { ::core::slice::from_raw_parts(b.ptr.as_ptr(), b.len) },
        }
    }
}

/// Which of an animation's eight per-coordinate-setup slots a setup uses.
fn coordinate_slot(coordinate_system: crate::core::SsCoordinateSystem, origin: crate::core::SsOrigin, unit: crate::core::SsUnit) -> usize {
    usize::from(coordinate_system == crate::core::SsCoordinateSystem::YDown) | usize::from(origin == crate::core::SsOrigin::TopLeft) << 1 | usize::from(unit == crate::core::SsUnit::Normalized) << 2
}

pub struct AnimationMetadata<'a> {
    pub name_hash: u32,
    pub start_frame: i32,
    pub end_frame: i32,
    pub fps: i32,
    pub canvas_width: f32,
    pub canvas_height: f32,
    /// The SSAE's setup pose, as stored. It is identical for every animation of
    /// an AnimationPack — the converter writes the same `isSetup` animation into
    /// each — and becomes the floor of the track stack once transformed into the
    /// instance's coordinate space (`AnimationMetadata::setup_pose`).
    pub initial_attributes: flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::InitialPartAttributes<'a>>>,
    pub events: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::EventsPerFrame<'a>>>>,
    pub initial_events: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::InitialEvents<'a>>>>,
    pub labels: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::Label<'a>>>>,
    animation: ssab::ssab::ss::format::AnimationData<'a>,
    pub(crate) source: alloc::sync::Arc<PartCacheSource<'a>>,
    cached_parts: once_cell::race::OnceBox<Vec<crate::core::framedata::CachedPartData>>,
    /// Which of the pack's mesh layouts this animation draws with
    /// (`PartCacheSource::mesh_buffers`).
    pub(crate) mesh_layout: usize,
    /// Every part's base (`PartBases::build`), one slot per coordinate setup.
    part_bases: [once_cell::race::OnceBox<crate::core::framedata::PartBases>; 8],
    /// The track stack's floor, one slot per coordinate setup like `part_bases`.
    setup_pose: [once_cell::race::OnceBox<alloc::vec::Vec<crate::runtime::framedata::ss::runtime::PartState>>; 8],
}

impl AnimationMetadata<'_> {
    /// Every part's cache for this animation, built the first time it is asked
    /// for — in practice by the first `setup_animation` of the animation — so a
    /// pack pays only for the animations it plays. Two first calls racing on
    /// different threads may both build it; one result is kept.
    pub fn cached_parts(&self) -> &[crate::core::framedata::CachedPartData] {
        self.cached_parts.get_or_init(|| alloc::boxed::Box::new(self.source.build(self.animation)))
    }

    /// Every part's base for this animation under one coordinate setup, built
    /// the first time an instance with that setup plays it and shared by all of
    /// them. Built only from attributes that never change during the animation.
    pub(crate) fn part_bases(&self, coordinate_system: crate::core::SsCoordinateSystem, origin: crate::core::SsOrigin, unit: crate::core::SsUnit) -> &crate::core::framedata::PartBases {
        let slot = coordinate_slot(coordinate_system, origin, unit);
        self.part_bases[slot].get_or_init(|| alloc::boxed::Box::new(crate::core::framedata::PartBases::build(self.cached_parts(), &self.source.keys, self.start_frame as f32, coordinate_system, origin, unit, self.canvas_width, self.canvas_height)))
    }

    /// The setup pose under one coordinate setup — the floor of the track stack —
    /// built the first time an instance with that setup mixes tracks and shared
    /// by all of them. A single track at full alpha never reads it, so an
    /// animation that is only ever played alone never builds it.
    pub(crate) fn setup_pose(&self, coordinate_system: crate::core::SsCoordinateSystem, origin: crate::core::SsOrigin, unit: crate::core::SsUnit) -> &[crate::runtime::framedata::ss::runtime::PartState] {
        self.setup_pose[coordinate_slot(coordinate_system, origin, unit)].get_or_init(|| {
            let parts = self.cached_parts();
            alloc::boxed::Box::new(
                self.initial_attributes.iter().enumerate().map(|(i, a)| crate::core::framedata::setup_pose_part(&a, parts.get(i).map_or(-1, |p| p.parent_index), coordinate_system, origin, unit, self.canvas_width, self.canvas_height)).collect(),
            )
        })
    }

    /// The static mesh buffers this animation draws with, built the first time
    /// any animation with the same mesh cells asks for them and shared by all of
    /// them.
    pub(crate) fn static_mesh(&self) -> &StaticMeshBuffers {
        self.source.mesh_buffers[self.mesh_layout].get_or_init(|| alloc::boxed::Box::new(StaticMeshBuffers::build(self.cached_parts())))
    }
}

/// What a Player draws a mesh part with besides its per-frame vertices: the
/// UVs (interleaved `[u, v, ...]`, texture-pixel space) and triangle indices of
/// each mesh part's cell, concatenated over the parts in CSR form. Fixed by
/// which cell each mesh part shows (its first key, by the SS mesh spec), so
/// every animation drawing its meshes with the same cells shares one.
pub(crate) struct StaticMeshBuffers {
    /// CSR prefix sum over vertices, length `num_parts + 1`; slices `uvs` (two
    /// floats a vertex) and the per-instance vertex buffers.
    pub(crate) vertex_offsets: Vec<u32>,
    pub(crate) uvs: Vec<f32>,
    pub(crate) indices: Vec<i32>,
    /// CSR prefix sum over `indices`, length `num_parts + 1`.
    pub(crate) index_offsets: Vec<u32>,
}

impl StaticMeshBuffers {
    fn build(cached_parts: &[crate::core::framedata::CachedPartData]) -> Self {
        let cell = |p: &crate::core::framedata::CachedPartData| p.mesh_cell.clone();
        // Non-mesh parts contribute an empty range. The Cell is the authoritative
        // geometry source for both skeletal and Deform-only meshes; the bind
        // cache only adds skinning.
        let mut vertex_offsets = Vec::with_capacity(cached_parts.len() + 1);
        let mut index_offsets = Vec::with_capacity(cached_parts.len() + 1);
        let (mut vertices, mut indices) = (0u32, 0u32);
        vertex_offsets.push(0);
        index_offsets.push(0);
        for p in cached_parts {
            if let Some(m) = cell(p) {
                vertices += m.num_vertices() as u32;
                indices += m.indices.len() as u32;
            }
            vertex_offsets.push(vertices);
            index_offsets.push(indices);
        }

        let mut uvs = alloc::vec![0.0; vertices as usize * 2];
        let mut all_indices = Vec::with_capacity(indices as usize);
        for (i, p) in cached_parts.iter().enumerate() {
            let Some(m) = cell(p) else { continue };
            all_indices.extend_from_slice(&m.indices);
            // UVs share the vertex CSR; copy only when the cell vertex count
            // matches the slot (true for well-formed meshes).
            let (start, end) = (vertex_offsets[i] as usize, vertex_offsets[i + 1] as usize);
            if m.uvs.len() == (end - start) * 2 {
                uvs[start * 2..end * 2].copy_from_slice(&m.uvs);
            }
        }
        Self { vertex_offsets, uvs, indices: all_indices, index_offsets }
    }
}

/// What building an animation's part cache needs from its pack, shared by all
/// of the pack's animations. A part's skin weights and a mesh cell's geometry
/// are the same whichever animation plays them, so they are decoded once, at
/// load, and every animation's `CachedPartData` shares them.
pub(crate) struct PartCacheSource<'a> {
    ssab: ssab::ssab::ss::format::SsAnimeBinary<'a>,
    pub(crate) keys: crate::core::keyframes::KeyColumns<'a>,
    mesh_bindings: Vec<Option<alloc::sync::Arc<crate::core::framedata::MeshBindingCache>>>,
    /// By cell, as an index for `KeyColumns::cell`.
    mesh_cells: alloc::collections::BTreeMap<u32, Option<alloc::sync::Arc<crate::core::framedata::MeshCellData>>>,
    /// One slot per distinct set of cells the pack's animations show on their
    /// mesh parts, filled the first time an animation using it plays.
    mesh_buffers: Vec<once_cell::race::OnceBox<StaticMeshBuffers>>,
    /// Each animation's slot in `mesh_buffers`, by animation index.
    mesh_layouts: Vec<usize>,
}

impl<'a> PartCacheSource<'a> {
    fn new(ssab: ssab::ssab::ss::format::SsAnimeBinary<'a>, keys: crate::core::keyframes::KeyColumns<'a>) -> Self {
        let fb_parts = ssab.parts();
        let mesh_bindings = fb_parts.iter().map(|part| crate::core::framedata::build_mesh_binding_cache(part.mesh_binding()).map(alloc::sync::Arc::new)).collect();
        let mut mesh_cells = alloc::collections::BTreeMap::new();
        // Animations that show the same cell on every mesh part draw with the
        // same static mesh buffers; number each distinct set as it appears.
        let mut layout_of = alloc::collections::BTreeMap::<Vec<(usize, u32)>, usize>::new();
        let mut mesh_layouts = Vec::with_capacity(ssab.animations().len());
        for anim in ssab.animations() {
            let mut layout = Vec::new();
            for (i, p) in anim.parts_animation_data().iter().enumerate() {
                if fb_parts.get(i).part_type_type() != ssab::ssab::ss::format::PartType::PartTypeMesh {
                    continue;
                }
                if let Some(cell) = crate::core::framedata::mesh_part_cell(&keys, keys.run(p.cell_keys())) {
                    layout.push((i, cell));
                    mesh_cells.entry(cell).or_insert_with(|| crate::core::framedata::build_mesh_cell_data(&ssab, &keys.cell(cell).cell).map(alloc::sync::Arc::new));
                }
            }
            let next = layout_of.len();
            mesh_layouts.push(*layout_of.entry(layout).or_insert(next));
        }
        let mesh_buffers = (0..layout_of.len()).map(|_| once_cell::race::OnceBox::new()).collect();
        Self { ssab, keys, mesh_bindings, mesh_cells, mesh_buffers, mesh_layouts }
    }

    fn build(&self, anim: ssab::ssab::ss::format::AnimationData<'a>) -> Vec<crate::core::framedata::CachedPartData> {
        let (ssab, keys) = (&self.ssab, &self.keys);
        let fb_parts = ssab.parts();
        let parts = anim.parts_animation_data();
        let mut cached_parts = Vec::with_capacity(parts.len());

        for (i, p) in parts.iter().enumerate() {
            let part_meta = fb_parts.get(i);
            let parent_index = part_meta.parent_index();
            let blend_type = part_meta.blend_type();
            let part_type_disc = part_meta.part_type_type();
            let mask_influence = part_meta.mask_influence();
            // PartTypeMask is always a pure mask; drawing parts use the unified draw_as_mask flag.
            let is_mask_contributor = part_type_disc == ssab::ssab::ss::format::PartType::PartTypeMask || part_meta.draw_as_mask();
            let shape_kind = part_meta.part_type_as_part_type_shape().and_then(|s| match s.shape_type() {
                ssab::ssab::ss::format::ShapeType::Rectangle => Some(crate::core::geometry::ShapeType::Rectangle),
                ssab::ssab::ss::format::ShapeType::Triangle => Some(crate::core::geometry::ShapeType::Triangle),
                ssab::ssab::ss::format::ShapeType::Star => Some(crate::core::geometry::ShapeType::Star),
                ssab::ssab::ss::format::ShapeType::Arrow => Some(crate::core::geometry::ShapeType::Arrow),
                _ => None,
            });
            let nines = crate::core::framedata::build_nines_part_data(&part_meta);
            let mesh_binding = self.mesh_bindings[i].clone();
            let mesh_cell = if part_type_disc == ssab::ssab::ss::format::PartType::PartTypeMesh { crate::core::framedata::mesh_part_cell(keys, keys.run(p.cell_keys())).and_then(|cell| self.mesh_cells.get(&cell).cloned().flatten()) } else { None };
            let name_hash = fnv1a_hash_str(part_meta.name());
            let mut part = crate::core::framedata::CachedPartData::new(p, keys, i as i16, parent_index, name_hash, shape_kind, nines, mesh_binding, mesh_cell, blend_type, part_type_disc, mask_influence, is_mask_contributor, anim.start_frame() as i32);
            part.cursor_base = crate::core::framedata::cursor_count(&cached_parts) as u32;
            cached_parts.push(part);
        }
        cached_parts
    }
}

pub struct SsabData {
    storage: SsabStorage,
}

impl SsabData {
    pub fn load_copy(ssab_data: &[u8]) -> Result<Self, error::SSRuntimeError> {
        let storage = SsabStorage::Owned(ssab_data.to_vec());
        Ok(Self { storage })
    }

    /// # Safety
    ///
    /// The caller must ensure:
    /// - `ptr` is non-null and points to a valid memory block of at least `len` bytes.
    /// - The memory remains valid and unmodified for the lifetime of the returned
    ///   `SsabData` (and any `SsabResource` derived from it).
    /// - If the resulting `SsabData` is sent to or shared with another thread,
    ///   no other party may mutate the memory for the duration of the borrow.
    ///
    /// Returns `SSRuntimeError::InvalidParameter` if `ptr` is null.
    pub unsafe fn load_borrow(ptr: *const u8, len: usize) -> Result<Self, error::SSRuntimeError> {
        let Some(ptr) = ::core::ptr::NonNull::new(ptr as *mut u8) else {
            return Err(error::SSRuntimeError::InvalidParameter);
        };
        Ok(Self { storage: SsabStorage::Borrowed(BorrowedBuffer { ptr, len }) })
    }

    pub fn as_slice(&self) -> &[u8] {
        self.storage.as_slice()
    }
}

pub struct AnimationResource<'a> {
    pub(crate) data: &'a [u8],
    pub(crate) animations: Vec<AnimationMetadata<'a>>,
    pub(crate) source: alloc::sync::Arc<PartCacheSource<'a>>,
}

/// A thread-safe, shared resource containing SSAB data and its parsed metadata.
///
/// This is a self-referential structure: `resource` holds borrows into `data`'s
/// buffer. The `'static` lifetime stored internally is fictitious — the actual
/// lifetime is bound to `&self`. The `resource()` accessor restores the correct
/// lifetime, so external code never observes the unsound `'static` form.
///
/// Invariants (must hold for soundness):
/// - Fields are private so `data` cannot be mutated or replaced after construction.
/// - Field declaration order ensures `resource` is dropped before `data`.
/// - `SsabData::Owned` keeps its `Vec<u8>` buffer at a stable heap address;
///   `SsabData::Borrowed` requires the caller to keep the memory valid (documented
///   on `SsabData::load_borrow`).
pub struct SsabResource {
    resource: Option<AnimationResource<'static>>,
    data: SsabData,
    /// Why `resource` is `None`, kept so the binary faces can say which of the
    /// several reasons it was. A version mismatch in particular names the two
    /// versions, and that is the whole of what the user needs to act.
    parse_error: Option<error::SSRuntimeError>,
}

impl SsabResource {
    /// Parses `data` into a shareable resource. On failure the resource holds no
    /// animation and [`Self::parse_error`] says why.
    ///
    /// # Safety
    ///
    /// The bytes must be a valid `.ssab`. They are read through FlatBuffers accessors that do not
    /// bounds-check in release builds, so any other bytes are undefined behaviour (out-of-bounds
    /// reads). A pack the converter wrote is valid; for any other, run the FlatBuffers verifier
    /// first (the `ssab` crate's checked `root_as_ss_anime_binary`).
    pub unsafe fn new(data: SsabData) -> alloc::sync::Arc<Self> {
        let res = Self { resource: None, data, parse_error: None };
        let mut arc = alloc::sync::Arc::new(res);
        if let Some(mut_res) = alloc::sync::Arc::get_mut(&mut arc) {
            // SAFETY: `mut_res.data` lives as long as `mut_res` (i.e. as long as the
            // Arc allocation). The `AnimationResource` we build holds borrows into
            // that buffer, but we store it as `'static` because Rust cannot express
            // a self-borrow directly. The `resource()` accessor downgrades the lie
            // back to a self-bound lifetime before exposing it.
            let data_slice: &'static [u8] = unsafe { &*(mut_res.data.as_slice() as *const [u8]) };
            // SAFETY: these are `data`'s bytes, which this function's caller guarantees are a
            // valid `.ssab` (`# Safety`).
            match unsafe { AnimationResource::new(data_slice) } {
                Ok(resource) => mut_res.resource = Some(resource),
                Err(e) => mut_res.parse_error = Some(e),
            }
        }
        arc
    }

    /// Why [`Self::resource`] is `None`, or `None` when the parse succeeded.
    #[inline]
    pub fn parse_error(&self) -> Option<&error::SSRuntimeError> {
        self.parse_error.as_ref()
    }

    /// Returns the parsed animation resource, with its lifetime tied to `&self`.
    #[inline]
    pub fn resource(&self) -> Option<&AnimationResource<'_>> {
        self.resource.as_ref()
    }

    /// Returns the underlying SSAB data.
    #[inline]
    pub fn data(&self) -> &SsabData {
        &self.data
    }

    /// Returns a raw pointer to the parsed resource (or null if parsing failed),
    /// detached from `self`'s borrow scope.
    ///
    /// This is intended for FFI/WASM bindings that need to extend the resource
    /// reference into a `'static` context while transferring ownership of the
    /// surrounding `Arc<SsabResource>` to a long-lived holder. Using a raw
    /// pointer breaks the borrow chain so the `Arc` can be moved into its
    /// holder before the pointer is dereferenced.
    ///
    /// Obtaining the pointer is safe; dereferencing it is `unsafe` and the
    /// caller must uphold the obligations below.
    ///
    /// # Caller obligations when dereferencing
    /// - Keep some `Arc<SsabResource>` pointing at this allocation alive for the
    ///   entire duration the pointer (or any borrow derived from it) is in use.
    /// - Do not dereference after the last `Arc` referencing this allocation
    ///   is dropped — doing so is undefined behaviour.
    #[inline]
    pub fn resource_static_ptr(&self) -> *const AnimationResource<'static> {
        match &self.resource {
            Some(r) => r as *const _,
            None => ::core::ptr::null(),
        }
    }
}

impl<'a> AnimationResource<'a> {
    /// Parses the animations' metadata out of `data`. Fails on a schema-version mismatch.
    ///
    /// # Safety
    ///
    /// The bytes must be a valid `.ssab`. They are read through FlatBuffers accessors that do not
    /// bounds-check in release builds, so any other bytes are undefined behaviour (out-of-bounds
    /// reads). A pack the converter wrote is valid; for any other, run the FlatBuffers verifier
    /// first (the `ssab` crate's checked `root_as_ss_anime_binary`).
    pub unsafe fn new(data: &'a [u8]) -> Result<Self, error::SSRuntimeError> {
        // SAFETY: the caller guarantees `data` is a valid `.ssab` (`# Safety`); only the version
        // is checked here.
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(data) };
        if ssab.version() != ssab::SSAB_SCHEMA_VERSION {
            return Err(error::SSRuntimeError::InvalidVersion { expected: ssab::SSAB_SCHEMA_VERSION, got: ssab.version() });
        }

        let keys = crate::core::keyframes::KeyColumns::new(ssab.keyframes(), ssab.cellmaps());
        let fb_animations = ssab.animations();
        let mut animations = Vec::with_capacity(fb_animations.len());
        let source = alloc::sync::Arc::new(PartCacheSource::new(ssab, keys));

        for (index, anim) in fb_animations.iter().enumerate() {
            let name_hash = anim.name_hash();
            let canvas_size = anim.canvas_size();
            let (canvas_width, canvas_height) = (canvas_size.v1(), canvas_size.v2());
            animations.push(AnimationMetadata {
                name_hash,
                start_frame: anim.start_frame() as i32,
                end_frame: anim.end_frame() as i32,
                fps: anim.fps() as i32,
                canvas_width,
                canvas_height,
                initial_attributes: anim.parts_initial_attributes(),
                events: anim.events(),
                initial_events: anim.initial_events(),
                labels: anim.labels(),
                animation: anim,
                source: source.clone(),
                cached_parts: once_cell::race::OnceBox::new(),
                mesh_layout: source.mesh_layouts[index],
                part_bases: Default::default(),
                setup_pose: Default::default(),
            });
        }

        Ok(Self { data, animations, source })
    }

    /// The static mesh buffers of mesh layout `layout`, if an animation using it
    /// has built them.
    pub(crate) fn static_mesh(&self, layout: usize) -> Option<&StaticMeshBuffers> {
        self.source.mesh_buffers.get(layout)?.get()
    }

    pub fn get_ssab_data(&self) -> &'a [u8] {
        self.data
    }

    pub fn animations(&self) -> &[AnimationMetadata<'a>] {
        &self.animations
    }

    pub fn get_animation_names(&self) -> Vec<alloc::string::String> {
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(self.data) };
        let mut names = Vec::new();
        let animations = ssab.animations();

        for anim in animations {
            names.push(alloc::string::String::from(anim.name()));
        }

        names
    }

    pub fn find_animation_index(&self, name: &str) -> Option<usize> {
        let hash = fnv1a_hash_str(name);
        self.find_animation_index_by_hash(hash)
    }

    pub fn find_animation_index_by_hash(&self, hash: u32) -> Option<usize> {
        self.animations.iter().position(|anim| anim.name_hash == hash)
    }
}

pub fn fnv1a_hash_str(text: &str) -> u32 {
    let mut hash: u32 = 2166136261;
    let prime: u32 = 16777619;

    for b in text.bytes() {
        hash ^= b as u32;
        hash = hash.wrapping_mul(prime);
    }
    hash
}
