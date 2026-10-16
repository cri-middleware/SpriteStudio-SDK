//! The promise of the Player Porting Guide's render loop page
//! (`docs/30_core/40_render_loop.md`): once an animation has played through
//! twice, `update`, `get_frame_data` and the effect and instance slot steps
//! allocate no memory.
//!
//! The allocator below serves the whole test binary, so it counts per thread.

mod common;

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ssab::ssab::ss::format as fbs;
use ssruntime::{AnimationResource, Context, EffectSlot, InstanceSlot, SsabData};

thread_local! {
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

fn count() {
    let _ = ALLOCATIONS.try_with(|n| n.set(n.get() + 1));
}

struct Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        count();
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        count();
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        count();
        unsafe { System.realloc(ptr, layout, new_size) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

fn allocations_during(f: impl FnOnce()) -> usize {
    let before = ALLOCATIONS.with(Cell::get);
    f();
    ALLOCATIONS.with(Cell::get) - before
}

const PROJECTS: [&str; 6] = ["Doll", "ParticleEffect", "Ringo", "allAttributeV7", "allPartsV7", "overall"];

/// Packs loaded for the rest of the run, so contexts and slots can borrow them
/// without threading lifetimes through the test.
#[derive(Default)]
struct Library(HashMap<PathBuf, (&'static [u8], &'static AnimationResource<'static>)>);

impl Library {
    fn load(&mut self, pack: &Path) -> (&'static [u8], &'static AnimationResource<'static>) {
        *self.0.entry(pack.to_path_buf()).or_insert_with(|| {
            let bytes = std::fs::read(pack).unwrap_or_else(|e| panic!("read {}: {e}", pack.display()));
            let data: &'static SsabData = Box::leak(Box::new(SsabData::load_copy(&bytes).expect("load pack")));
            let resource = unsafe { AnimationResource::new(data.as_slice()) }.expect("load resource");
            (data.as_slice(), Box::leak(Box::new(resource)))
        })
    }
}

/// What a Player keeps for one Context: a slot for each effect part, and a slot
/// and a child for each instance part.
struct Player {
    ctx: Context<'static>,
    effects: Vec<(u32, EffectSlot<'static>)>,
    instances: Vec<(u32, InstanceSlot, Player)>,
}

impl Player {
    fn new(library: &mut Library, pack: &Path, animation_hash: u32) -> Player {
        let (data, resource) = library.load(pack);
        let mut ctx = Context::new();
        ctx.bind_resource(resource);
        ctx.setup_animation_by_hash(animation_hash).expect("set up the animation");
        let (mut effects, mut instances) = (Vec::new(), Vec::new());
        for (i, part) in fbs::root_as_ss_anime_binary(data).expect("verify pack").parts().iter().enumerate() {
            let i = i as u32;
            if let Some(effect) = part.part_type_as_part_type_effect() {
                effects.push((i, EffectSlot::new(resource, effect.ref_effect_name_hash(), i).expect("effect slot")));
            }
            if let Some(instance) = part.part_type_as_part_type_instance() {
                let child_pack = pack.with_file_name(format!("{}.ssab", instance.ref_anime_pack()));
                instances.push((i, InstanceSlot::new(), Player::new(library, &child_pack, instance.ref_anime_name_hash())));
            }
        }
        Player { ctx, effects, instances }
    }

    /// One frame, the way the guide's render loop and instance page lay it out.
    fn tick(&mut self, delta_seconds: f32) {
        let frame = self.ctx.update(delta_seconds * 1000.0);
        let looped = self.ctx.just_looped();
        self.draw(frame, delta_seconds, looped);
    }

    fn draw(&mut self, frame: f32, delta_seconds: f32, looped: bool) {
        self.ctx.get_frame_data(frame).expect("frame data");
        let fps = self.ctx.get_frame_rate() as f32;
        for (part, slot) in &mut self.effects {
            slot.step(self.ctx.get_active_effect_event(*part), frame, delta_seconds, fps, looped, false, 2, true);
        }
        for (part, slot, child) in &mut self.instances {
            let r = slot.step(self.ctx.get_active_event_instance(*part), &mut child.ctx, frame, delta_seconds, looped);
            if r.transitioned {
                child.ctx.play();
            }
            if r.visible {
                child.draw(r.child_frame_no, delta_seconds, r.child_looped);
            }
        }
    }

    fn playthrough(&self) -> usize {
        (self.ctx.get_end_frame() - self.ctx.get_start_frame() + 1).max(1) as usize
    }

    /// Plays through twice, then counts the allocations of two more playthroughs.
    fn allocations_once_warm(&mut self) -> usize {
        let delta_seconds = 1.0 / self.ctx.get_frame_rate().max(1) as f32;
        let frames = self.playthrough() * 2;
        for _ in 0..frames {
            self.tick(delta_seconds);
        }
        allocations_during(|| {
            for _ in 0..frames {
                self.tick(delta_seconds);
            }
        })
    }
}

fn packs() -> Vec<PathBuf> {
    let mut packs: Vec<PathBuf> =
        PROJECTS.iter().flat_map(|project| std::fs::read_dir(common::converted(project)).expect("read the converted project")).map(|entry| entry.expect("directory entry").path()).filter(|path| path.extension().is_some_and(|ext| ext == "ssab")).collect();
    packs.sort();
    packs
}

#[test]
fn every_animation_plays_without_allocating_once_it_has_played_through_twice() {
    let mut library = Library::default();
    let (mut played, mut failures) = (0, Vec::new());
    for pack in packs() {
        let (_, resource) = library.load(&pack);
        for (animation, name) in resource.animations().iter().zip(resource.get_animation_names()) {
            let mut player = Player::new(&mut library, &pack, animation.name_hash);
            player.ctx.play();
            let n = player.allocations_once_warm();
            if n != 0 {
                failures.push(format!("{}#{name}: {n}", pack.display()));
            }
            played += 1;
        }
    }
    assert!(played > 100, "fixture sanity: only {played} animations were played");
    assert!(failures.is_empty(), "allocations after two playthroughs:\n{}", failures.join("\n"));
}

#[test]
fn a_mix_and_part_overrides_play_without_allocating_once_they_have_played_through_twice() {
    let mut library = Library::default();
    let pack = common::converted("Ringo").join("Ringo.ssab");
    let (_, resource) = library.load(&pack);
    let animations = resource.animations();
    assert!(animations.len() > 3, "fixture sanity: Ringo has several animations");
    let mut player = Player::new(&mut library, &pack, animations[0].name_hash);
    player.ctx.play();
    player.ctx.crossfade_by_hash(0, animations[3].name_hash, 1.0e9).unwrap();
    player.ctx.set_track_animation_by_hash(1, animations[1].name_hash).unwrap();
    player.ctx.set_track_alpha(1, 0.5).unwrap();
    player.ctx.set_part_color_override(3, 0, 0xff0000ff, 2).unwrap();
    player.ctx.set_part_visibility_override(5, 1, true).unwrap();
    assert_eq!(player.allocations_once_warm(), 0);
    assert!(player.ctx.get_track_fade_progress(0).is_some_and(|t| t < 1.0), "fixture sanity: the crossfade ran the whole time");
    assert_eq!(player.ctx.get_track_count(), 2);
}
