//! The playhead, the playback state and settings, and the controls that drive the character's clock.

use super::{CharacterClock, Context};
use crate::eval;
use crate::mix::TrackFade;

impl<'a> Context<'a> {
    // Playhead getters

    pub fn get_frame_no(&self) -> f32 {
        self.instance.primary_source().controller.get_frame_no()
    }

    /// `true` once the configured loops have completed, and sticky until the
    /// next `play` / `setup_animation`. Never `true` under an infinite loop
    /// count, and never raised by `stop`.
    ///
    /// This is a *state*, not a pulse: to notify once, take its rising edge.
    pub fn is_finished(&self) -> bool {
        self.instance.primary_source().controller.is_finished()
    }

    pub fn just_looped(&self) -> bool {
        self.instance.primary_source().controller.just_looped()
    }

    // Playback state and setting getters

    /// `true` while the clock runs and track 0 has not finished. A held
    /// character is still playing.
    pub fn is_playing(&self) -> bool {
        self.instance.clock.playing && self.instance.primary_source().controller.is_playing()
    }

    pub fn is_playing_forward(&self) -> bool {
        self.instance.primary_source().controller.is_playing_forward()
    }

    /// `true` while a playing character is held. Implies [`Self::is_playing`],
    /// so the pair reads as three states. Once track 0 has finished, a pause
    /// still holds the other tracks and the fades, but this reads `false`
    /// along with `is_playing`.
    pub fn is_pausing(&self) -> bool {
        self.instance.clock.pausing && self.is_playing()
    }

    /// `true` from `play` until `stop` or the next setup: the character's
    /// clock, which [`Self::is_playing`] reads only while track 0 has not
    /// finished. A host that stops ticking a character with nothing to move
    /// reads this, since the other tracks and the fades run on after track 0
    /// ends. A held clock is still playing.
    pub fn is_clock_playing(&self) -> bool {
        self.instance.clock.playing
    }

    /// `true` while [`Self::pause`] holds a playing clock. Implies
    /// [`Self::is_clock_playing`], so the pair reads as three states, as
    /// [`Self::is_playing`] and [`Self::is_pausing`] do; unlike `is_pausing`,
    /// it still reads the hold after track 0 has finished.
    pub fn is_clock_pausing(&self) -> bool {
        self.instance.clock.pausing
    }

    /// `true` while the track stack would move the pose on a running clock for a
    /// reason other than track 0's own animation stepping: a fade on any track,
    /// or a source — other than track 0's main one — that plays, at a speed above
    /// 0 and a weight above 0, on a track that is heard. A cleared, finished,
    /// silenced or scrubbed track does not count, nor does a source a crossfade
    /// has retired; one a cancelled crossfade left behind does.
    ///
    /// It does not read the clock: a Player gating its redraw combines it with
    /// [`Self::is_clock_playing`] and [`Self::is_clock_pausing`], and compares
    /// track 0's frame itself. With this, a Player keeps no copy of which tracks
    /// and sources it has laid, which goes wrong the moment one is released
    /// inside the runtime.
    pub fn is_stack_in_motion(&self) -> bool {
        let instance = &self.instance;
        if instance.tracks.iter().any(|t| t.fade.is_some()) {
            return true;
        }
        let primary = instance.primary_idx();
        instance
            .sources
            .iter()
            .enumerate()
            .any(|(i, source)| i != primary && source.has_real_animation() && source.weight > 0.0 && source.controller.is_playing() && instance.tracks.get(source.track as usize).is_some_and(|t| t.is_audible() && t.policy.speed_scale > 0.0))
    }

    // The track 0 settings below read the track, which their setters write:
    // set before an animation is laid, there is no controller holding them yet.

    /// Track 0's [`Self::get_track_skip_frames`].
    pub fn get_skip_frames(&self) -> bool {
        self.instance.tracks[0].policy.is_skip_frames
    }

    /// Track 0's [`Self::get_track_playback_direction`].
    pub fn get_playback_direction(&self) -> eval::playback_controller::PlaybackDirection {
        self.instance.tracks[0].policy.playback_direction
    }

    /// Track 0's [`Self::get_track_playback_style`].
    pub fn get_playback_style(&self) -> eval::playback_controller::PlaybackStyle {
        self.instance.tracks[0].policy.playback_style
    }

    pub fn get_loop_count(&self) -> i32 {
        self.instance.primary_source().controller.get_loop_count()
    }

    pub fn get_remaining_loops(&self) -> i32 {
        self.instance.primary_source().controller.get_remaining_loops()
    }

    pub fn get_start_frame(&self) -> i32 {
        self.instance.primary_source().controller.get_start_frame()
    }

    pub fn get_end_frame(&self) -> i32 {
        self.instance.primary_source().controller.get_end_frame()
    }

    pub fn get_anime_data_start_frame(&self) -> i32 {
        self.instance.primary_source().controller.get_anime_data_start_frame()
    }

    pub fn get_anime_data_end_frame(&self) -> i32 {
        self.instance.primary_source().controller.get_anime_data_end_frame()
    }

    /// Track 0's [`Self::get_track_frame_rate`].
    pub fn get_frame_rate(&self) -> i32 {
        let policy = &self.instance.tracks[0].policy;
        self.instance.track_main_controller(0).map_or(policy.frame_rate_override.unwrap_or(0), |c| c.get_frame_rate())
    }

    /// Track 0's [`Self::get_track_speed`].
    pub fn get_speed_scale(&self) -> f32 {
        self.instance.tracks[0].policy.speed_scale
    }

    // Playback setup

    /// Track 0's [`Self::set_track_skip_frames`].
    pub fn set_skip_frames(&mut self, skip: bool) {
        self.instance.set_track_policy(0, |p| p.is_skip_frames = skip, |c| c.set_skip_frames(skip));
    }

    pub fn set_loop_count(&mut self, loop_count: i32) {
        self.instance.primary_source_mut().controller.set_loop_count(loop_count);
    }

    /// Track 0's [`Self::set_track_frame_rate`].
    pub fn set_frame_rate(&mut self, frame_rate: i32) {
        self.instance.set_track_policy(0, |p| p.set_frame_rate(frame_rate), |c| c.set_frame_rate(frame_rate));
    }

    /// Track 0's [`Self::set_track_playback_direction`].
    pub fn set_playback_direction(&mut self, direction: eval::playback_controller::PlaybackDirection, style: eval::playback_controller::PlaybackStyle) {
        self.instance.set_track_policy(
            0,
            |p| {
                p.playback_direction = direction;
                p.playback_style = style;
            },
            |c| c.set_playback_direction(direction, style),
        );
    }

    /// Track 0's [`Self::set_track_speed`].
    pub fn set_speed_scale(&mut self, speed_scale: f32) {
        self.instance.set_track_policy(0, |p| p.set_speed_scale(speed_scale), |c| c.set_speed_scale(speed_scale));
    }

    pub fn set_animation_section(&mut self, start_frame: i32, end_frame: i32) {
        self.instance.primary_source_mut().controller.set_animation_section(start_frame, end_frame);
    }

    pub fn set_coordinate_system(&mut self, coordinate_system: crate::eval::SsCoordinateSystem) {
        self.instance.coordinate_system = coordinate_system;
        // The part bases live in this space, so they have to move with it.
        self.prepare_part_bases();
    }

    pub fn get_coordinate_system(&self) -> crate::eval::SsCoordinateSystem {
        self.instance.coordinate_system
    }

    pub fn set_origin(&mut self, origin: crate::eval::SsOrigin) {
        self.instance.origin = origin;
        // The part bases live in this space, so they have to move with it.
        self.prepare_part_bases();
    }

    pub fn get_origin(&self) -> crate::eval::SsOrigin {
        self.instance.origin
    }

    pub fn set_unit(&mut self, unit: crate::eval::SsUnit) {
        self.instance.unit = unit;
        // The part bases live in this space, so they have to move with it.
        self.prepare_part_bases();
    }

    pub fn get_unit(&self) -> crate::eval::SsUnit {
        self.instance.unit
    }

    // Playback control

    /// Advance every source's controller by `delta` milliseconds and return the
    /// frame to hand to [`Self::get_frame_data`]: that of track 0's main
    /// animation, which `get_frame_data` validates against. A crossfade on
    /// track 0 hands that role over when it begins, so across one this is the
    /// incoming animation's frame.
    ///
    /// Nothing moves while the character's clock is stopped or held, fades
    /// included. The controllers are still updated with no elapsed time, which
    /// is what reports a frame a seek has just landed on.
    pub fn update(&mut self, delta: f32) -> f32 {
        self.instance.updates += 1;
        let running = self.instance.clock.is_running();
        let elapsed = if running { delta } else { 0.0 };
        for source in self.instance.sources.iter_mut() {
            if !source.has_real_animation() {
                continue;
            }
            source.controller.update(elapsed);
        }

        if running {
            self.advance_track_fades(delta);
        }
        self.collect_events();

        self.get_frame_no()
    }

    /// Advance every track's fade by `delta`.
    ///
    /// A crossfade raises the incoming source's weight linearly while the
    /// outgoing side sinks by the same amount, and on completion releases every
    /// source on the track but the incoming one, which has been its main one
    /// since the fade began. An alpha fade drives the track's `alpha` to its
    /// target. A crossfade whose incoming source was released elsewhere cancels
    /// itself.
    fn advance_track_fades(&mut self, delta: f32) {
        for track in 0..self.instance.tracks.len() {
            let Some(fade) = self.instance.tracks[track].fade else {
                continue;
            };
            match fade {
                TrackFade::Cross { to, mut elapsed, duration } => {
                    if self.instance.sources.get(to as usize).is_none_or(|s| !s.has_real_animation()) {
                        self.instance.tracks[track].fade = None;
                        continue;
                    }
                    elapsed += delta;
                    let t = (elapsed / duration).clamp(0.0, 1.0);
                    for (i, source) in self.instance.sources.iter_mut().enumerate() {
                        // A source added to the track after the fade began has no
                        // share, keeps the weight it was given, and is released with
                        // the rest on completion.
                        if i as u32 != to && source.track as usize == track && source.has_real_animation() && source.outgoing_share > 0.0 {
                            source.weight = source.outgoing_share * (1.0 - t);
                        }
                    }
                    self.instance.sources[to as usize].weight = t;
                    if elapsed >= duration {
                        self.replace_track_main(track as u32, to);
                    } else {
                        self.instance.tracks[track].fade = Some(TrackFade::Cross { to, elapsed, duration });
                    }
                }
                TrackFade::Alpha { start, target, mut elapsed, duration } => {
                    elapsed += delta;
                    let t = (elapsed / duration).clamp(0.0, 1.0);
                    self.instance.tracks[track].alpha = start + (target - start) * t;
                    if elapsed >= duration {
                        self.instance.tracks[track].alpha = target;
                        self.instance.tracks[track].fade = None;
                    } else {
                        self.instance.tracks[track].fade = Some(TrackFade::Alpha { start, target, elapsed, duration });
                    }
                }
            }
        }
    }

    /// Heads track 0 out — back to its section start, loop count re-armed — and
    /// starts the character's clock. The other tracks keep their playheads, as
    /// they do when track 0 loops or ends on its own.
    pub fn play(&mut self) {
        self.instance.primary_source_mut().controller.play();
        self.start_clock();
    }

    pub fn play_with_start_frame(&mut self, frame_no: f32) {
        self.instance.primary_source_mut().controller.play_with_start_frame(frame_no);
        self.start_clock();
    }

    fn start_clock(&mut self) {
        self.instance.clock = CharacterClock { playing: true, pausing: false };
    }

    /// Holds the whole character in place: every track and every fade.
    /// Idempotent, and ignored while the clock is stopped; `is_playing` stays
    /// `true`.
    pub fn pause(&mut self) {
        if self.instance.clock.playing {
            self.instance.clock.pausing = true;
        }
    }

    /// Lifts a [`Self::pause`]. Idempotent, and not a way to start a stopped
    /// animation — `play` rewinds, so it cannot serve as a resume.
    pub fn resume(&mut self) {
        self.instance.clock.pausing = false;
    }

    /// Stops the whole character where it stands: every track and every fade.
    /// Nothing rewinds; only `play` starts it again.
    pub fn stop(&mut self) {
        self.instance.clock = CharacterClock::default();
    }

    pub fn set_frame_no(&mut self, frame: f32) {
        self.instance.primary_source_mut().controller.set_frame_no(frame);
    }

    /// Sets the playback position based on `diff` (frames elapsed since
    /// playback started, with per-instance speed already applied), applying
    /// the configured section / loops / style / direction. Used by parent
    /// Players to drive child Instance Players deterministically
    /// across reverse, seek, and frame-skip operations.
    pub fn set_frame_relative(&mut self, diff: f32) {
        self.instance.primary_source_mut().controller.set_frame_relative(diff);
    }

    pub fn next_frame(&mut self) {
        self.instance.primary_source_mut().controller.next_frame();
    }

    pub fn prev_frame(&mut self) {
        self.instance.primary_source_mut().controller.prev_frame();
    }
}
