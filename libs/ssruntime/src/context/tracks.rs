//! The track API: laying animations on tracks, crossfades, and each track's and source's settings.

use super::Context;
use crate::mix::{LayerState, NO_SOURCE, SourceState, TrackBlend, TrackFade, TrackState, TrackSync, count_attribute_capacities};
use crate::resource::fnv1a_hash_str;
use crate::{error, eval};

impl<'a> Context<'a> {
    /// Put `name` on `track`, replacing whatever it was playing.
    ///
    /// Tracks are addressed by index and grow on demand: setting track 3 creates
    /// 0..=3, the new ones silent. Track 0 is the structure track — its animation
    /// supplies every discrete attribute and drives events — so it must be set up
    /// first (`setup_animation` does exactly that), and this errors with
    /// `AnimationNotFound` until it has been.
    ///
    /// The animation must live in the currently bound `AnimationResource`
    /// (same-SSAB constraint). Cancels any fade in flight on the track.
    pub fn set_track_animation(&mut self, track: u32, name: &str) -> Result<(), error::SSRuntimeError> {
        self.set_track_animation_by_hash(track, fnv1a_hash_str(name))
    }

    pub fn set_track_animation_by_hash(&mut self, track: u32, hash: u32) -> Result<(), error::SSRuntimeError> {
        let index = self.resolve_animation(hash)?;
        self.ensure_track(track)?;
        let handle = self.attach_source(track, index, 1.0)?;
        self.replace_track_main(track, handle);
        Ok(())
    }

    /// The strength of a track, i.e. how much of it survives onto the pose the
    /// tracks below produced. **This is the knob that always means something**: a
    /// source's `weight` only decides proportions among the sources sharing one
    /// track, and is silent when a source is the only one animating an attribute.
    ///
    /// Alpha is not clamped — values above 1 extrapolate, which is occasionally
    /// what an overlay wants — but `alpha <= 0` makes the track contribute
    /// nothing. Track 0 at alpha 0 resolves to the setup pose, which is how an
    /// animation fades out to rest.
    pub fn set_track_alpha(&mut self, track: u32, alpha: f32) -> Result<(), error::SSRuntimeError> {
        let t = self.instance.tracks.get_mut(track as usize).ok_or(error::SSRuntimeError::InvalidParameter)?;
        if alpha.is_nan() {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        t.alpha = alpha;
        Ok(())
    }

    pub fn get_track_alpha(&self, track: u32) -> Option<f32> {
        self.instance.tracks.get(track as usize).map(|t| t.alpha)
    }

    /// How the track folds onto what is below: [`TrackBlend::Mix`] lerps toward
    /// the track's pose, [`TrackBlend::Additive`] adds the track's offset from the
    /// setup pose. Additive is for overlays that displace a performance rather
    /// than replace it.
    pub fn set_track_blend(&mut self, track: u32, blend: TrackBlend) -> Result<(), error::SSRuntimeError> {
        let t = self.instance.tracks.get_mut(track as usize).ok_or(error::SSRuntimeError::InvalidParameter)?;
        t.blend = blend;
        Ok(())
    }

    pub fn get_track_blend(&self, track: u32) -> Option<TrackBlend> {
        self.instance.tracks.get(track as usize).map(|t| t.blend)
    }

    /// Where the animations laid on `track` start: [`TrackSync::Free`] from the
    /// head of their section, [`TrackSync::Start`] at the phase track 0 stands
    /// at. It applies to everything laid on the track from then on —
    /// `set_track_animation`, `crossfade` and `add_track_source` — so set it
    /// before laying, and a track with nothing on it yet is created for it.
    /// Like the rest of a track it goes with the stack on the next setup.
    pub fn set_track_sync(&mut self, track: u32, sync: TrackSync) -> Result<(), error::SSRuntimeError> {
        self.ensure_track(track)?;
        self.instance.tracks[track as usize].sync = sync;
        Ok(())
    }

    pub fn get_track_sync(&self, track: u32) -> Option<TrackSync> {
        self.instance.tracks.get(track as usize).map(|t| t.sync)
    }

    /// Number of tracks that exist, including silent ones. Always at least 1.
    pub fn get_track_count(&self) -> u32 {
        self.instance.tracks.len() as u32
    }

    /// Silence a track: every source on it is deactivated and its fade cancelled.
    /// Track 0 cannot be cleared — it carries the structure, and an instance with
    /// no structure has no pose to emit. Fade it to alpha 0 instead.
    pub fn clear_track(&mut self, track: u32) -> Result<(), error::SSRuntimeError> {
        if track == 0 {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        let t = self.instance.tracks.get_mut(track as usize).ok_or(error::SSRuntimeError::InvalidParameter)?;
        t.main = NO_SOURCE;
        t.fade = None;
        for source in self.instance.sources.iter_mut() {
            if source.track as u32 == track {
                source.deactivate();
            }
        }
        Ok(())
    }

    /// Fade `track` to `name` over `duration_ms` (the same time unit as
    /// [`Context::update`]'s `delta`). The fade starts from whatever the track is
    /// already playing, so there are no handles to pair up and nothing to clean up
    /// afterwards: on completion the outgoing sources are deactivated.
    ///
    /// **The incoming animation becomes the track's main one the moment the fade
    /// begins**, at weight 0 — on track 0 that makes it the primary, so the
    /// frame, `is_playing` / `is_finished` / `just_looped`, the playback settings,
    /// the discrete attributes and the events all speak for it from the call on.
    /// The host has said where the character is going, and the outgoing side is
    /// only its pose fading away: it neither reports state nor fires events
    /// (unless its event threshold is lowered). Handing over on completion
    /// instead let an outgoing animation that ran out mid-fade report the
    /// character finished, and then playing again once the fade completed.
    ///
    /// **Everything the track carries fades out together**, in the proportions it
    /// carries them: the main animation, any `add_track_source` sources, and
    /// whatever a cancelled fade left behind. Their weights are normalised to sum
    /// to 1 when the fade begins and scaled down by `1 - t` from there, so the
    /// pose does not jump when the fade begins or when it completes.
    ///
    /// Starting a crossfade while one is running on the same track folds the one
    /// in flight into the new outgoing side, at the weights it has reached. There
    /// is never more than one outgoing side, so fades restarted faster than they
    /// run do not pile up, and every outgoing source is released on completion.
    pub fn crossfade(&mut self, track: u32, name: &str, duration_ms: f32) -> Result<(), error::SSRuntimeError> {
        self.crossfade_by_hash(track, fnv1a_hash_str(name), duration_ms)
    }

    pub fn crossfade_by_hash(&mut self, track: u32, hash: u32, duration_ms: f32) -> Result<(), error::SSRuntimeError> {
        if duration_ms.is_nan() || duration_ms <= 0.0 {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        let index = self.resolve_animation(hash)?;
        self.ensure_track(track)?;

        let to = self.attach_source(track, index, 0.0)?;
        let outgoing: f32 = self.instance.sources.iter().enumerate().filter(|&(i, s)| i as u32 != to && s.track as u32 == track && s.has_real_animation()).map(|(_, s)| s.weight.max(0.0)).sum();
        if outgoing <= 0.0 {
            // Nothing audible to fade from — this is just "start playing".
            self.instance.sources[to as usize].weight = 1.0;
            self.replace_track_main(track, to);
            return Ok(());
        }

        // Normalise the outgoing side once, here, so the fade only has to scale it.
        // Left as they were, a cancelled fade's leftovers or a host's weights of 2
        // would change their share of the pose the moment the incoming one rose.
        for (i, source) in self.instance.sources.iter_mut().enumerate() {
            if i as u32 != to && source.track as u32 == track && source.has_real_animation() {
                source.outgoing_share = source.weight.max(0.0) / outgoing;
                source.weight = source.outgoing_share;
            }
        }
        let t = &mut self.instance.tracks[track as usize];
        t.main = to;
        t.fade = Some(TrackFade::Cross { to, elapsed: 0.0, duration: duration_ms });
        if track == 0 {
            self.instance.primary = to;
        }
        Ok(())
    }

    /// Drive a track's `alpha` to `target` over `duration_ms`. Fading track 0 to 0
    /// resolves the instance to its setup pose, which is what "fade out to rest"
    /// means here — no placeholder animation required.
    pub fn fade_track(&mut self, track: u32, target: f32, duration_ms: f32) -> Result<(), error::SSRuntimeError> {
        if duration_ms.is_nan() || duration_ms <= 0.0 || target.is_nan() {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        let t = self.instance.tracks.get_mut(track as usize).ok_or(error::SSRuntimeError::InvalidParameter)?;
        t.fade = Some(TrackFade::Alpha { start: t.alpha, target, elapsed: 0.0, duration: duration_ms });
        Ok(())
    }

    /// Progress of the fade in flight on `track`, in `[0.0, 1.0]`; `None` when the
    /// track is not fading.
    pub fn get_track_fade_progress(&self, track: u32) -> Option<f32> {
        self.instance.tracks.get(track as usize)?.fade.as_ref().map(TrackFade::progress)
    }

    /// Cancel the fade on `track`, leaving alpha and weights where they stand.
    pub fn cancel_track_fade(&mut self, track: u32) -> Result<(), error::SSRuntimeError> {
        let t = self.instance.tracks.get_mut(track as usize).ok_or(error::SSRuntimeError::InvalidParameter)?;
        t.fade = None;
        Ok(())
    }

    /// Seek the track's current animation. This is how a modulation axis is
    /// driven: author the axis as an animation and scrub it with the tracked
    /// value.
    pub fn set_track_frame(&mut self, track: u32, frame: f32) -> Result<(), error::SSRuntimeError> {
        let handle = self.track_main(track)?;
        self.instance.sources[handle as usize].controller.set_frame_no(frame);
        Ok(())
    }

    pub fn get_track_frame(&self, track: u32) -> Option<f32> {
        let handle = self.instance.tracks.get(track as usize)?.main;
        let source = self.instance.sources.get(handle as usize)?;
        (source.animation_idx >= 0).then(|| source.controller.get_frame_no())
    }

    /// The speed of `track`, as a multiple of each animation's frame rate. Kept
    /// by the track: it reaches every source on it now and every one laid on it
    /// later, so a crossfade or a `set_track_animation` does not reset it.
    /// Track 0's is [`Self::set_speed_scale`]. Like every track setting it goes
    /// with the track when `setup_animation` collapses the stack, except on
    /// track 0.
    pub fn set_track_speed(&mut self, track: u32, speed: f32) -> Result<(), error::SSRuntimeError> {
        self.ensure_track(track)?;
        self.instance.set_track_policy(track, |p| p.set_speed_scale(speed), |c| c.set_speed_scale(speed));
        Ok(())
    }

    /// What [`Self::set_track_speed`] stored: a negative speed reads back as 0.
    pub fn get_track_speed(&self, track: u32) -> Option<f32> {
        Some(self.instance.tracks.get(track as usize)?.policy.speed_scale)
    }

    /// The frame rate `track` plays its animations at instead of the rate they
    /// were authored at; `0` or a negative rate clears it. Kept by the track,
    /// as [`Self::set_track_speed`] is.
    pub fn set_track_frame_rate(&mut self, track: u32, frame_rate: i32) -> Result<(), error::SSRuntimeError> {
        self.ensure_track(track)?;
        self.instance.set_track_policy(track, |p| p.set_frame_rate(frame_rate), |c| c.set_frame_rate(frame_rate));
        Ok(())
    }

    /// The rate `track`'s animation plays at: the override when one is set,
    /// otherwise the rate it was authored at. `None` for a track that does not
    /// exist; `0` for one with no animation and no override.
    pub fn get_track_frame_rate(&self, track: u32) -> Option<i32> {
        let policy = self.instance.tracks.get(track as usize)?.policy;
        Some(self.instance.track_main_controller(track).map_or(policy.frame_rate_override.unwrap_or(0), |c| c.get_frame_rate()))
    }

    /// Whether `track` skips frames to keep up. Kept by the track, as
    /// [`Self::set_track_speed`] is.
    pub fn set_track_skip_frames(&mut self, track: u32, skip: bool) -> Result<(), error::SSRuntimeError> {
        self.ensure_track(track)?;
        self.instance.set_track_policy(track, |p| p.is_skip_frames = skip, |c| c.set_skip_frames(skip));
        Ok(())
    }

    pub fn get_track_skip_frames(&self, track: u32) -> Option<bool> {
        Some(self.instance.tracks.get(track as usize)?.policy.is_skip_frames)
    }

    /// The direction and style `track` plays its animations in. Kept by the
    /// track, as [`Self::set_track_speed`] is.
    pub fn set_track_playback_direction(&mut self, track: u32, direction: eval::playback_controller::PlaybackDirection, style: eval::playback_controller::PlaybackStyle) -> Result<(), error::SSRuntimeError> {
        self.ensure_track(track)?;
        self.instance.set_track_policy(
            track,
            |p| {
                p.playback_direction = direction;
                p.playback_style = style;
            },
            |c| c.set_playback_direction(direction, style),
        );
        Ok(())
    }

    pub fn get_track_playback_direction(&self, track: u32) -> Option<eval::playback_controller::PlaybackDirection> {
        Some(self.instance.tracks.get(track as usize)?.policy.playback_direction)
    }

    pub fn get_track_playback_style(&self, track: u32) -> Option<eval::playback_controller::PlaybackStyle> {
        Some(self.instance.tracks.get(track as usize)?.policy.playback_style)
    }

    /// How many times `track`'s animation plays before it holds its last frame;
    /// `0` or a negative count loops forever. Unlike the speed this belongs to
    /// the animation on the track, not the track: laying another animation
    /// there starts it looping forever again, so state the count after laying
    /// it. Track 0's is [`Self::set_loop_count`].
    pub fn set_track_loop_count(&mut self, track: u32, loop_count: i32) -> Result<(), error::SSRuntimeError> {
        let handle = self.track_main(track)?;
        self.instance.sources[handle as usize].controller.set_loop_count(loop_count);
        Ok(())
    }

    /// `-1` for a track that loops forever; `None` for one with no animation.
    pub fn get_track_loop_count(&self, track: u32) -> Option<i32> {
        Some(self.instance.track_main_controller(track)?.get_loop_count())
    }

    /// `true` once `track`'s animation has played its loop count out and holds
    /// its last frame, as [`Self::is_finished`] is for track 0. A finished track
    /// keeps mixing that frame in until the host fades or clears it. `false` for
    /// a track with no animation.
    pub fn is_track_finished(&self, track: u32) -> bool {
        self.instance.track_main_controller(track).is_some_and(|c| c.is_finished())
    }

    /// `true` for the update in which `track`'s animation wrapped back to its
    /// start, as [`Self::just_looped`] is for track 0.
    pub fn track_just_looped(&self, track: u32) -> bool {
        self.instance.track_main_controller(track).is_some_and(|c| c.just_looped())
    }

    /// Handle of the source carrying `track`'s animation. Needed to give that
    /// animation a weight of its own once a track holds several — the animation
    /// put there by `set_track_animation` starts at weight 1.
    pub fn get_track_main_source(&self, track: u32) -> Option<u32> {
        let main = self.instance.tracks.get(track as usize)?.main;
        (main != NO_SOURCE).then_some(main)
    }

    /// Add another weighted source to `track`, for the case a track needs several
    /// animations blended by proportion (locomotion: walk + run). Returns a stable
    /// handle for [`Self::set_source_weight`].
    ///
    /// Most tracks never need this — one animation plus the track's `alpha`
    /// covers layering, and `crossfade` covers transitions.
    pub fn add_track_source(&mut self, track: u32, name: &str, weight: f32) -> Result<u32, error::SSRuntimeError> {
        self.add_track_source_by_hash(track, fnv1a_hash_str(name), weight)
    }

    pub fn add_track_source_by_hash(&mut self, track: u32, hash: u32, weight: f32) -> Result<u32, error::SSRuntimeError> {
        let index = self.resolve_animation(hash)?;
        self.ensure_track(track)?;
        let handle = self.attach_source(track, index, weight)?;
        // The first animation to land on a track becomes its main one, so callers
        // may build a track entirely out of `add_track_source` calls.
        if self.instance.tracks[track as usize].main == NO_SOURCE {
            self.replace_track_main(track, handle);
        }
        Ok(handle)
    }

    /// Proportion of one source **within its track**. See [`Self::set_track_alpha`]
    /// for the strength of the track as a whole.
    pub fn set_source_weight(&mut self, handle: u32, weight: f32) -> Result<(), error::SSRuntimeError> {
        let source = self.instance.sources.get_mut(handle as usize).ok_or(error::SSRuntimeError::InvalidParameter)?;
        if source.animation_idx < 0 {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        source.weight = weight;
        Ok(())
    }

    pub fn get_source_weight(&self, handle: u32) -> Option<f32> {
        let source = self.instance.sources.get(handle as usize)?;
        (source.animation_idx >= 0).then_some(source.weight)
    }

    pub fn set_source_frame(&mut self, handle: u32, frame: f32) -> Result<(), error::SSRuntimeError> {
        let source = self.instance.sources.get_mut(handle as usize).ok_or(error::SSRuntimeError::InvalidParameter)?;
        if source.animation_idx < 0 {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        source.controller.set_frame_no(frame);
        Ok(())
    }

    /// Which animation `handle` plays, as an index into the pack's animations:
    /// the order of [`AnimationResource::get_animation_names`] and of the
    /// ssab's `animations()`. `None` for a handle that names no source.
    ///
    /// An index from [`Self::get_passed_event_indices`] addresses the `events()`
    /// of the animation of the source that fired it, so this is how a Player
    /// decodes an event. Track 0's included: a crossfade on track 0 hands it to
    /// another animation, and a table kept from `setup_animation` is then the
    /// wrong one.
    ///
    /// [`AnimationResource::get_animation_names`]: crate::AnimationResource::get_animation_names
    pub fn get_source_animation_index(&self, handle: u32) -> Option<u32> {
        let source = self.instance.sources.get(handle as usize)?;
        source.has_real_animation().then_some(source.animation_idx as u32)
    }

    /// Which animation track 0's main source plays, as an index into the pack's
    /// animations: the one `setup_animation` set up, until a `crossfade` or a
    /// `set_track_animation` on track 0 hands the track to another, from the
    /// call on. `None` before anything is set up.
    pub fn get_current_animation_index(&self) -> Option<u32> {
        self.get_source_animation_index(self.instance.primary)
    }

    pub fn get_source_frame(&self, handle: u32) -> Option<f32> {
        let source = self.instance.sources.get(handle as usize)?;
        (source.animation_idx >= 0).then(|| source.controller.get_frame_no())
    }

    /// Per-source event firing threshold. A source fires events once its weight
    /// reaches the threshold. The default is `+∞`, so by default no source
    /// fires but track 0's main source, which always does regardless of this
    /// setting. Distinguish sources with
    /// [`Self::get_passed_event_source_handles`].
    pub fn set_source_event_threshold(&mut self, handle: u32, threshold: f32) -> Result<(), error::SSRuntimeError> {
        let source = self.instance.sources.get_mut(handle as usize).ok_or(error::SSRuntimeError::InvalidParameter)?;
        if !source.has_real_animation() {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        source.event_threshold = threshold;
        Ok(())
    }

    pub fn get_source_event_threshold(&self, handle: u32) -> Option<f32> {
        let source = self.instance.sources.get(handle as usize)?;
        source.has_real_animation().then_some(source.event_threshold)
    }

    /// How far into a crossfade on `track` the outgoing side keeps firing
    /// events. It fires while its share of the track, `1 - t`, is at least
    /// `threshold`, however many sources it holds: `0.5` keeps it firing for the
    /// first half of the fade. The default, `+∞`, keeps it silent from the moment
    /// the fade begins, which is where the incoming animation starts firing.
    /// The outgoing sources' own thresholds are not consulted while they fade.
    ///
    /// A setting of the track rather than of a source, since the outgoing side
    /// can be several sources, released when the fade completes; it applies to
    /// every crossfade on the track until it is changed. Track 0 keeps it
    /// across `setup_animation`, as it keeps its playback settings.
    pub fn set_track_outgoing_event_threshold(&mut self, track: u32, threshold: f32) -> Result<(), error::SSRuntimeError> {
        self.ensure_track(track)?;
        self.instance.tracks[track as usize].outgoing_event_threshold = threshold;
        Ok(())
    }

    pub fn get_track_outgoing_event_threshold(&self, track: u32) -> Option<f32> {
        Some(self.instance.tracks.get(track as usize)?.outgoing_event_threshold)
    }

    // ---- internals shared by the track API ---------------------------------

    /// Build the part bases for this instance's coordinate setup now, not on the
    /// first frame. Runs when the animation or a coordinate setting changes.
    pub(super) fn prepare_part_bases(&mut self) {
        let Some(resource) = self.eval.resource else {
            return;
        };
        let anim_idx = self.instance.primary_source().animation_idx;
        if anim_idx < 0 {
            return;
        }
        resource.animations[anim_idx as usize].part_bases(self.instance.coordinate_system, self.instance.origin, self.instance.unit);
    }

    fn resolve_animation(&self, hash: u32) -> Result<usize, error::SSRuntimeError> {
        let Some(resource) = self.eval.resource else {
            return Err(error::SSRuntimeError::NotLoaded);
        };
        resource.find_animation_index_by_hash(hash).ok_or(error::SSRuntimeError::AnimationNotFound)
    }

    /// Grow the stack so `track` exists. Every track but 0 may be created this
    /// way; track 0 exists from construction, but until `setup_animation` has run
    /// there is no structure to layer onto, so this reports `AnimationNotFound`.
    fn ensure_track(&mut self, track: u32) -> Result<(), error::SSRuntimeError> {
        if track != 0 && !self.instance.sources[self.instance.primary_idx()].has_real_animation() {
            return Err(error::SSRuntimeError::AnimationNotFound);
        }
        if track as usize >= self.instance.tracks.len() {
            self.instance.tracks.resize(track as usize + 1, TrackState::new());
        }
        Ok(())
    }

    /// Park a new source on `track`, reusing a slot left behind by a finished
    /// fade when one is free so a long-lived instance does not grow a slot per
    /// transition. Handles stay stable: a slot is only reused once inactive.
    fn attach_source(&mut self, track: u32, animation_idx: usize, weight: f32) -> Result<u32, error::SSRuntimeError> {
        let Some(resource) = self.eval.resource else {
            return Err(error::SSRuntimeError::NotLoaded);
        };
        let metadata = &resource.animations[animation_idx];
        let num_parts = metadata.cached_parts().len();
        let (cell_c, color_c, shader_c, deform_c) = count_attribute_capacities(metadata);
        let cursor_c = crate::eval::framedata::cursor_count(metadata.cached_parts());

        metadata.part_bases(self.instance.coordinate_system, self.instance.origin, self.instance.unit);
        let mut source = SourceState::new();
        // The track's policy, not the defaults: a laid animation is played the
        // way the host plays that track. Before `setup`, which derives the
        // heading and the frame interval from it.
        source.controller.adopt_policy(&self.instance.tracks[track as usize].policy);
        source.controller.setup(metadata.start_frame, metadata.end_frame, metadata.fps);
        // A source laid on a track plays from the moment it is attached; the
        // character's clock decides whether it actually moves. Left stopped, an
        // overlay sat on its first frame and a completed crossfade froze the
        // character, since nothing but `play` ever started a controller.
        source.controller.play();
        if self.instance.tracks[track as usize].sync == TrackSync::Start
            && let Some(phase) = self.primary_phase()
        {
            let c = &mut source.controller;
            let frame = c.get_start_frame() as f32 + phase * (c.get_end_frame() - c.get_start_frame() + 1) as f32;
            c.set_frame_no(frame);
        }
        source.animation_idx = animation_idx as i32;
        source.weight = weight;
        source.track = track as u16;

        let free = self.instance.sources.iter().position(|s| s.animation_idx < 0);
        let handle = match free {
            Some(i) => {
                self.instance.sources[i] = source;
                self.instance.layers[i].resize_for_parts(num_parts, cursor_c, cell_c, color_c, shader_c, deform_c);
                i
            }
            None => {
                self.instance.sources.push(source);
                let mut layer = LayerState::new();
                layer.resize_for_parts(num_parts, cursor_c, cell_c, color_c, shader_c, deform_c);
                self.instance.layers.push(layer);
                self.instance.sources.len() - 1
            }
        };
        Ok(handle as u32)
    }

    /// How far through its section track 0's main animation stands, in
    /// `[0.0, 1.0)`; `None` with nothing set up. Measured by position, not by
    /// progress, so a backward or ping-pong return leg reads where it stands.
    /// During a crossfade on track 0 this is the incoming animation, which takes
    /// the main role when the fade begins; the source a crossfade is laying
    /// is attached before that, so it reads the outgoing one it is replacing.
    fn primary_phase(&self) -> Option<f32> {
        let source = self.instance.primary_source();
        if !source.has_real_animation() {
            return None;
        }
        let c = &source.controller;
        let length = (c.get_end_frame() - c.get_start_frame() + 1) as f32;
        Some(((c.get_frame_no() - c.get_start_frame() as f32) / length).clamp(0.0, 1.0))
    }

    /// Make `handle` the track's animation, retiring whatever held that role.
    pub(super) fn replace_track_main(&mut self, track: u32, handle: u32) {
        let previous = self.instance.tracks[track as usize].main;
        if previous != NO_SOURCE && previous != handle {
            self.instance.sources[previous as usize].deactivate();
        }
        // Any other source on this track belonged to the animation being replaced.
        for (i, source) in self.instance.sources.iter_mut().enumerate() {
            if i as u32 != handle && source.track as u32 == track {
                source.deactivate();
            }
        }
        let t = &mut self.instance.tracks[track as usize];
        t.main = handle;
        t.fade = None;
        if track == 0 {
            self.instance.primary = handle;
        }
    }

    fn track_main(&self, track: u32) -> Result<u32, error::SSRuntimeError> {
        let t = self.instance.tracks.get(track as usize).ok_or(error::SSRuntimeError::InvalidParameter)?;
        if t.main == NO_SOURCE {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        Ok(t.main)
    }
}
