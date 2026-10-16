use crate::core::math::*;
use alloc::vec::Vec;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum PlaybackStyle {
    Normal = 0,
    PingPong = 1,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum PlaybackDirection {
    Forward = 1,
    Backward = -1,
}

/// Pure, deterministic mapping from "frames elapsed since playback started"
/// (`diff`, with per-instance `speed` already applied) to the resolved
/// playback frame within the inclusive `[start, end]` range, applying loop
/// count and playback style / direction. Matches SpriteStudio 6 SDK's
/// `SsAnimeDecoder::updateInstance` semantics, including the inclusive
/// `inst_scale = (end - start) + 1` cycle length and the duplicated peak /
/// trough frames at PingPong reversal boundaries.
///
/// Returns `(frame, finished)` where `finished` is `true` once the
/// configured `loops` have completed. For `loops == -1` (infinite) and
/// `loops == 0` (treated as infinite to match `PlaybackController::update`)
/// `finished` is always `false`. `diff < 0` is clamped to `0`.
///
/// `loops > 0` defines total cycles. A "cycle" is `(end - start) + 1`
/// frames for `Normal` and `2 * ((end - start) + 1)` frames for `PingPong`
/// (one full there-and-back, with the turn-around frame appearing in both
/// halves to match SS6).
pub fn resolve_frame(diff: f32, start: i32, end: i32, loops: i32, style: PlaybackStyle, direction: PlaybackDirection) -> (f32, bool) {
    let length = (end - start + 1) as f32;
    if length <= 0.0 {
        let f = match direction {
            PlaybackDirection::Forward => start as f32,
            PlaybackDirection::Backward => end as f32,
        };
        return (f, true);
    }

    let cycle_len = match style {
        PlaybackStyle::Normal => length,
        PlaybackStyle::PingPong => length * 2.0,
    };

    let total_len = if loops > 0 { Some((loops as f32) * cycle_len) } else { None };

    let elapsed = diff.max(0.0);

    let finished = match total_len {
        Some(total) => elapsed >= total,
        None => false,
    };

    if finished {
        let final_frame = match (style, direction) {
            (PlaybackStyle::Normal, PlaybackDirection::Forward) => end as f32,
            (PlaybackStyle::Normal, PlaybackDirection::Backward) => start as f32,
            (PlaybackStyle::PingPong, PlaybackDirection::Forward) => start as f32,
            (PlaybackStyle::PingPong, PlaybackDirection::Backward) => end as f32,
        };
        return (final_frame, true);
    }

    let phase = elapsed - floorf(elapsed / cycle_len) * cycle_len;
    let local_offset = match style {
        PlaybackStyle::Normal => phase,
        PlaybackStyle::PingPong => {
            // Phase [0, length) is the forward half (start → end), and
            // [length, 2*length) is the reverse half (end → start). The
            // turn-around frame (end) appears at both `phase = length - 1`
            // and `phase = length`, matching SS6's nowloop transition where
            // reftime=length plays end again at the start of the reverse
            // loop.
            if phase < length { phase } else { cycle_len - 1.0 - phase }
        }
    };

    let frame = match direction {
        PlaybackDirection::Forward => start as f32 + local_offset,
        PlaybackDirection::Backward => end as f32 - local_offset,
    };

    let frame = if frame < start as f32 { start as f32 } else { frame };

    (frame, false)
}

/// The settings that describe how a host plays an animation rather than the
/// animation itself, so they outlive a change of animation: the speed, the
/// frame-rate override, the frame skip, and the direction and style. A track
/// keeps one and hands it to every source laid on it.
///
/// The section and the loop count are not here: a section is frame numbers of
/// one animation, and a loop count is restated with each animation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlaybackPolicy {
    pub(crate) speed_scale: f32,
    pub(crate) frame_rate_override: Option<i32>,
    pub(crate) is_skip_frames: bool,
    pub(crate) playback_direction: PlaybackDirection,
    pub(crate) playback_style: PlaybackStyle,
}

impl PlaybackPolicy {
    /// What a controller starts with before a host sets anything.
    pub(crate) const DEFAULT: Self = Self { speed_scale: 1.0, frame_rate_override: None, is_skip_frames: true, playback_direction: PlaybackDirection::Forward, playback_style: PlaybackStyle::Normal };

    /// Stores a speed the way [`PlaybackController::set_speed_scale`] does.
    pub(crate) fn set_speed_scale(&mut self, speed_scale: f32) {
        self.speed_scale = normalise_speed_scale(speed_scale);
    }

    /// Stores a rate the way [`PlaybackController::set_frame_rate`] does.
    pub(crate) fn set_frame_rate(&mut self, frame_rate: i32) {
        self.frame_rate_override = normalise_frame_rate(frame_rate);
    }
}

/// A negative speed clamps to `0.0`, which stops the playhead.
fn normalise_speed_scale(speed_scale: f32) -> f32 {
    if speed_scale > 0.0 { speed_scale } else { 0.0 }
}

/// `0` or a negative rate is no override at all.
fn normalise_frame_rate(frame_rate: i32) -> Option<i32> {
    if frame_rate > 0 { Some(frame_rate) } else { None }
}

pub struct PlaybackController {
    is_pausing: bool,
    is_playing: bool,

    anime_data_start_frame: i32,
    anime_data_end_frame: i32,
    anime_data_fps: i32,

    start_frame: i32,
    end_frame: i32,
    /// The rate the host asked for, when it asked for one at all. Kept apart
    /// from `anime_data_fps` so `setup` can re-read the clip's own rate without
    /// having to guess whether the current value was a host's choice or just
    /// what the previous clip happened to be authored at.
    frame_rate_override: Option<i32>,
    speed_scale: f32,
    loops: i32,

    update_interval: f32,
    playback_style: PlaybackStyle,
    playback_direction: PlaybackDirection,
    is_skip_frames: bool,

    playback_forward: i32,
    current_frame: f32,
    next_frame_time: f32,
    remain_loops: i32,
    end_frame_reached: bool,
    just_looped: bool,

    passed_frames: Vec<i32>,
    needs_evaluation: bool,
}

impl PlaybackController {
    pub fn get_frame_no(&self) -> f32 {
        self.current_frame
    }

    /// `true` while the last frame of a cycle is being shown. With a finite
    /// loop count this is raised once per cycle, not only on the last one, and
    /// with an infinite loop count it is never raised.
    ///
    /// Internal playback state, deliberately not part of the Player-facing
    /// surface — it is absent from the C-API and the WASM bindings because
    /// every past attempt to build a `finished` notification on it produced a
    /// bug. Use [`is_finished`] for that.
    ///
    /// [`is_finished`]: Self::is_finished
    pub fn is_end_frame_reached(&self) -> bool {
        self.end_frame_reached
    }

    /// `true` once the configured loops have completed, and sticky until the
    /// next `play` / `setup`. Never becomes `true` under an infinite loop
    /// count, nor after a bare `stop()` part-way through playback.
    ///
    /// `remain_loops` only reaches `0` on the branch that ends playback, so
    /// this cannot be confused with `stop()` being called while the last frame
    /// of a non-final cycle happens to be showing.
    pub fn is_finished(&self) -> bool {
        self.remain_loops == 0 && !self.is_playing
    }

    /// `true` for the single `update` that took the playhead back to the start
    /// of its cycle. Cleared on entry to every `update`, so it must be polled
    /// after each one or the pulse is lost. Never raised on the final cycle of
    /// a finite loop count — that cycle finishes instead of looping.
    pub fn just_looped(&self) -> bool {
        self.just_looped
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn is_pausing(&self) -> bool {
        self.is_pausing
    }

    pub fn get_skip_frames(&self) -> bool {
        self.is_skip_frames
    }

    pub fn is_playing_forward(&self) -> bool {
        self.playback_forward == 1
    }

    pub fn get_playback_direction(&self) -> PlaybackDirection {
        self.playback_direction
    }

    pub fn get_playback_style(&self) -> PlaybackStyle {
        self.playback_style
    }

    pub fn get_loop_count(&self) -> i32 {
        self.loops
    }

    pub fn get_remaining_loops(&self) -> i32 {
        self.remain_loops
    }

    pub fn get_start_frame(&self) -> i32 {
        self.start_frame
    }

    pub fn get_end_frame(&self) -> i32 {
        self.end_frame
    }

    /// The bound animation's own range, which `set_animation_section` never
    /// touches. `get_start_frame` / `get_end_frame` report the section, so a
    /// caller that computes a section from them feeds its own last narrowing
    /// back in.
    pub fn get_anime_data_start_frame(&self) -> i32 {
        self.anime_data_start_frame
    }

    pub fn get_anime_data_end_frame(&self) -> i32 {
        self.anime_data_end_frame
    }

    pub fn get_frame_rate(&self) -> i32 {
        self.effective_frame_rate()
    }

    /// The playback speed in effect, as a multiple of [`Self::get_frame_rate`].
    ///
    /// This is what [`Self::set_speed_scale`] *stored*, not what it was
    /// handed: a negative rate clamps to `0.0` on the way in, so a host that
    /// reads this back sees the stop it actually got.
    pub fn get_speed_scale(&self) -> f32 {
        self.speed_scale
    }

    pub fn get_passed_frames(&self) -> &[i32] {
        &self.passed_frames
    }
}

impl Default for PlaybackController {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaybackController {
    pub fn new() -> Self {
        Self {
            // Not paused and not playing are different things: a controller
            // that has never played must not report itself as held, or every
            // Player's `is_pausing` inherits a meaningless `true`.
            is_pausing: false,
            is_playing: false,

            anime_data_start_frame: 0,
            anime_data_end_frame: 0,
            anime_data_fps: 0,

            start_frame: 0,
            end_frame: 0,
            frame_rate_override: None,
            speed_scale: 1.0,

            update_interval: 0.0,
            playback_style: PlaybackStyle::Normal,
            playback_direction: PlaybackDirection::Forward,
            is_skip_frames: true,

            playback_forward: 1,
            current_frame: 0.0,
            next_frame_time: 0.0,
            loops: -1,
            remain_loops: -1,
            end_frame_reached: false,
            just_looped: false,

            passed_frames: Vec::new(),
            needs_evaluation: false,
        }
    }

    pub fn reset_setup(&mut self) {
        self.reset_state();

        self.is_pausing = false;
        self.is_playing = false;
        self.anime_data_start_frame = 0;
        self.anime_data_end_frame = 0;
        self.anime_data_fps = 0;

        self.start_frame = 0;
        self.end_frame = 0;
        self.frame_rate_override = None;
        self.speed_scale = 1.0;
        self.is_skip_frames = true;
        self.playback_direction = PlaybackDirection::Forward;
        self.playback_style = PlaybackStyle::Normal;
        self.playback_forward = 1;

        self.update_interval = 0.0;
        self.passed_frames.clear();
    }

    fn reset_state(&mut self) {
        self.next_frame_time = 0.0;
        self.current_frame = 0.0;
        self.end_frame_reached = false;
        self.just_looped = false;
        self.passed_frames.clear();
        self.needs_evaluation = false;
    }

    /// Binds a new animation to this source.
    ///
    /// What resets and what survives is deliberate, and the line is **who sets
    /// it**. A Player's state machine calls this on every transition, so
    /// anything the transition itself specifies is reset — otherwise state N's
    /// behaviour would depend on state N-1, and a `loops = 1` meant for an
    /// attack would quietly follow the character into its walk. That covers the
    /// section (frame indices mean nothing against a different clip) and the
    /// loop count.
    ///
    /// The speed, the frame-rate override, the frame skip and the direction /
    /// style survive, because a *different* system sets them: a slow-motion
    /// effect, a haste buff, a "show every frame" capture mode. Resetting those
    /// here would let an animation transition clobber a setting the state
    /// machine never knew about. The Players' sub-frame flag is the same kind of
    /// setting and already behaves this way — it simply lives on their side of
    /// the FFI, so this never touched it.
    ///
    /// [`Self::reset_setup`] does reset them: it releases the slot for a
    /// different owner, rather than handing the same one a new clip.
    pub fn setup(&mut self, start_frame: i32, end_frame: i32, fps: i32) {
        self.reset_state();

        self.is_pausing = false;
        self.is_playing = false;

        self.anime_data_start_frame = start_frame;
        self.anime_data_end_frame = end_frame;
        self.anime_data_fps = fps;
        self.start_frame = self.anime_data_start_frame;
        self.end_frame = self.anime_data_end_frame;

        self.current_frame = start_frame as f32;
        // Resets `remain_loops` too: a stale count from a previous animation
        // would otherwise raise `end_frame_reached` at the end of this one's
        // first cycle.
        self.set_loop_count(-1);

        // The direction and style are the host's, like the speed and the frame
        // skip: "play this backwards" describes the playback, not the clip that
        // happens to be loaded. Only the *heading* is re-derived -- ping-pong
        // may have flipped it mid-cycle, and a new animation starts on the
        // outward leg of whatever direction is in force.
        self.playback_forward = if self.playback_direction == PlaybackDirection::Forward { 1 } else { -1 };
        // Carries the retained rate and any retained override onto the new
        // animation. Reading `anime_data_fps` directly here would ignore the
        // override, which used to be invisible only because `setup` overwrote it.
        self.recompute_update_interval();
        self.passed_frames.clear();
        self.needs_evaluation = false;
    }

    pub fn set_skip_frames(&mut self, skip: bool) {
        self.is_skip_frames = skip;
    }

    /// Recomputes the per-frame interval from the effective frame rate and the
    /// speed multiplier.
    ///
    /// A rate of zero — a speed of `0.0`, or no animation set up yet — leaves no
    /// interval to advance by, and is stored as `0.0`: the sentinel `update`
    /// already reads as "do not advance". Dividing by it instead would put an
    /// infinity in the field and rely on the arithmetic to work out.
    fn recompute_update_interval(&mut self) {
        let rate = self.effective_frame_rate() as f32 * self.speed_scale;
        self.update_interval = if rate > 0.0 { 1000.0 / rate } else { 0.0 };
    }

    /// Puts the host's playback policy on a controller that is about to be set
    /// up, so neither a rebuilt slot nor a freshly laid source reads as a reset
    /// of settings the host owns.
    ///
    /// Call it before `setup`, which derives the heading from the direction and
    /// the frame interval from the rate; until then there is no animation to
    /// advance.
    pub(crate) fn adopt_policy(&mut self, policy: &PlaybackPolicy) {
        self.speed_scale = policy.speed_scale;
        self.frame_rate_override = policy.frame_rate_override;
        self.is_skip_frames = policy.is_skip_frames;
        self.playback_direction = policy.playback_direction;
        self.playback_style = policy.playback_style;
    }

    /**
     * Set the number of loops.
     * n > 0: play n cycles, then stop
     * -1: infinite loop
     * 0: infinite loop (accepted as a synonym for -1)
     *
     * `0` and every negative value normalize to `-1` so that `update` and
     * `resolve_frame` agree on the infinite sentinel.
     */
    pub fn set_loop_count(&mut self, loops: i32) {
        self.loops = if loops > 0 { loops } else { -1 };
        self.remain_loops = self.loops;
    }

    /// Overrides the rate the clip was authored at. `0` or any negative value
    /// clears the override, handing playback back to the authored rate.
    ///
    /// This is an *absolute* rate — "this player runs at n fps" — where
    /// [`Self::set_speed_scale`] is a multiple of whatever the clip was
    /// authored at. Both are the host's, so both survive `setup`.
    pub fn set_frame_rate(&mut self, frame_rate: i32) {
        self.frame_rate_override = normalise_frame_rate(frame_rate);
        self.recompute_update_interval();
    }

    /// The rate playback actually runs at: the host's override when it set one,
    /// otherwise the rate the current clip was authored at.
    fn effective_frame_rate(&self) -> i32 {
        match self.frame_rate_override {
            Some(fps) => fps,
            None => self.anime_data_fps,
        }
    }

    /**
     * Set playback direction and playback style.
     * Execute the function before calling play().
     */
    pub fn set_playback_direction(&mut self, direction: PlaybackDirection, style: PlaybackStyle) {
        self.playback_direction = direction;
        self.playback_style = style;

        self.playback_forward = if self.playback_direction == PlaybackDirection::Forward { 1 } else { -1 };
    }

    /// Sets the playback speed as a multiple of the effective frame rate.
    ///
    /// `0.0` **stops the playhead** where it stands, rather than meaning "no
    /// setting": time passes, `is_playing` stays true, and nothing advances.
    /// That matches what a host means by a zero time scale, and what engine
    /// animation players do with one.
    ///
    /// A negative rate is **not** reverse playback — direction is
    /// [`Self::set_playback_direction`]'s business — and is clamped to a stop
    /// rather than silently running forward at full speed, which is what
    /// normalizing it to `1.0` used to do.
    pub fn set_speed_scale(&mut self, speed_scale: f32) {
        self.speed_scale = normalise_speed_scale(speed_scale);
        self.recompute_update_interval();
    }

    pub fn set_animation_section(&mut self, start_frame: i32, end_frame: i32) {
        self.start_frame = if start_frame >= 0 && start_frame < self.anime_data_end_frame { start_frame } else { self.anime_data_start_frame };
        self.end_frame = if end_frame >= 0 && end_frame < self.anime_data_end_frame { end_frame } else { self.anime_data_end_frame };

        self.current_frame = if self.playback_direction == PlaybackDirection::Forward { self.start_frame as f32 } else { self.end_frame as f32 };
        self.needs_evaluation = true;
    }

    pub fn update(&mut self, delta: f32) -> f32 {
        self.passed_frames.clear();
        self.just_looped = false;

        if self.needs_evaluation {
            self.passed_frames.push(floorf(self.current_frame) as i32);
            self.needs_evaluation = false;
        }

        let to_next_frame = self.is_playing && !self.is_pausing;
        if to_next_frame && self.update_interval != 0.0 {
            let s = if self.is_skip_frames {
                // Timed Animation: Advance by the exact elapsed time (can skip frames)
                (delta / self.update_interval) * self.playback_forward as f32
            } else {
                // Fixed Animation: Accumulate time and advance by exactly one frame at a time
                self.next_frame_time += delta;
                if self.next_frame_time >= self.update_interval {
                    self.next_frame_time -= self.update_interval;
                    self.playback_forward as f32
                } else {
                    0.0
                }
            };

            if s != 0.0 {
                self.end_frame_reached = false;

                let playback_style = if self.start_frame == self.end_frame { PlaybackStyle::Normal } else { self.playback_style };

                let next: f32 = self.current_frame + s;
                let next_frame_no = floorf(next) as i32;
                let mut next_frame_decimal = next - next_frame_no as f32;
                let mut current_frame_no = floorf(self.current_frame) as i32;

                let v = (next_frame_no - current_frame_no).abs();
                for _ in 0..v {
                    let mut frame_no = current_frame_no + self.playback_forward;

                    let (start_limit, end_limit) = if self.playback_forward > 0 { (self.start_frame, self.end_frame) } else { (self.end_frame, self.start_frame) };

                    if frame_no == end_limit && self.remain_loops != -1 {
                        match &playback_style {
                            PlaybackStyle::Normal => {
                                self.end_frame_reached = true;
                            }
                            PlaybackStyle::PingPong => {
                                let is_back_to_origin = match &self.playback_direction {
                                    PlaybackDirection::Forward => self.playback_forward == -1,
                                    PlaybackDirection::Backward => self.playback_forward == 1,
                                };
                                if is_back_to_origin {
                                    self.end_frame_reached = true;
                                }
                            }
                        }
                    }

                    let is_reached = if self.playback_forward > 0 { frame_no > end_limit } else { frame_no < end_limit };

                    if is_reached {
                        match &playback_style {
                            PlaybackStyle::Normal => {
                                if self.remain_loops == -1 {
                                    // infinite loop
                                    frame_no = start_limit;
                                    self.just_looped = true;
                                } else {
                                    self.remain_loops -= 1;
                                    if self.remain_loops == 0 {
                                        self.is_playing = false;
                                        self.end_frame_reached = true;
                                        // Park on the end frame and stop. It is not pushed to
                                        // `passed_frames`: reaching it is what let this branch be
                                        // entered, so it was already reported -- either earlier in
                                        // this same loop, or by the update that landed on it.
                                        // Pushing again re-fires its events.
                                        frame_no = end_limit;
                                        current_frame_no = frame_no;
                                        next_frame_decimal = 0.0;
                                        break;
                                    } else {
                                        // continue to play the animation
                                        frame_no = start_limit;
                                        self.just_looped = true;
                                    }
                                }
                            }
                            PlaybackStyle::PingPong => {
                                self.playback_forward *= -1;
                                frame_no = current_frame_no + self.playback_forward;

                                let is_back_to_origin = match &self.playback_direction {
                                    PlaybackDirection::Forward => self.playback_forward == 1,
                                    PlaybackDirection::Backward => self.playback_forward == -1,
                                };
                                if is_back_to_origin {
                                    if self.remain_loops != -1 {
                                        self.remain_loops -= 1;
                                        if self.remain_loops == 0 {
                                            self.is_playing = false;
                                            self.end_frame_reached = true;
                                            // Park on the turn-around frame and stop. Not pushed --
                                            // the update that landed back on it already reported it
                                            // (see the Normal arm above).
                                            frame_no = if self.playback_direction == PlaybackDirection::Forward { self.start_frame } else { self.end_frame };
                                            current_frame_no = frame_no;
                                            next_frame_decimal = 0.0;
                                            break;
                                        } else {
                                            self.just_looped = true;
                                        }
                                    } else {
                                        // infinite loop
                                        self.just_looped = true;
                                    }
                                } else {
                                    next_frame_decimal = 1.0 - next_frame_decimal;
                                    if next_frame_decimal >= 1.0 {
                                        next_frame_decimal = 0.0;
                                    }
                                }
                            }
                        }
                    }
                    current_frame_no = frame_no;
                    self.passed_frames.push(current_frame_no);
                }
                if self.is_playing {
                    self.current_frame = current_frame_no as f32 + next_frame_decimal;
                } else {
                    self.current_frame = current_frame_no as f32;
                }
            }
        }
        self.current_frame
    }
}

impl PlaybackController {
    // playback control
    pub fn play(&mut self) {
        self.play_with_start_frame(-1.0)
    }

    pub fn play_with_start_frame(&mut self, frame_no: f32) {
        self.is_playing = true;
        self.is_pausing = false;

        // Re-arm the loop counter and drop the terminal flags. Replaying a
        // finished animation would otherwise inherit `remain_loops == 0`,
        // which decrements straight to the infinite sentinel on the first
        // wrap, and expose a stale `end_frame_reached` until the first
        // advancing update.
        self.remain_loops = self.loops;
        self.end_frame_reached = false;
        self.just_looped = false;

        let mut current_frame = match &self.playback_direction {
            PlaybackDirection::Forward => self.start_frame as f32,
            PlaybackDirection::Backward => self.end_frame as f32,
        };
        if frame_no >= 0.0 {
            current_frame = frame_no;
        }
        self.current_frame = current_frame;
        self.needs_evaluation = true;
    }

    /// Holds playback where it stands. Idempotent: pausing an already-paused
    /// animation leaves it paused. `is_playing` stays `true` — a pause is a
    /// hold, not a stop — so a caller telling the two apart reads
    /// [`Self::is_pausing`]. Use [`Self::resume`] to lift the hold; `play`
    /// rewinds and is not a resume.
    ///
    /// Ignored when nothing is playing. Holding a stopped animation would mean
    /// nothing — `play` clears the hold on its way out anyway — and it is the
    /// one way left to reach "stopped, and also paused", the combination the
    /// seed value and [`Self::stop`] were both cleaned up to avoid. Keeping it
    /// unreachable is what lets `is_pausing` be read on its own.
    pub fn pause(&mut self) {
        if !self.is_playing {
            return;
        }
        self.is_pausing = true;
    }

    /// Lifts the hold set by [`Self::pause`], carrying on from the frame it
    /// stopped at. Idempotent, and a no-op on an animation that was never
    /// paused. Does not start a stopped animation — that is `play`'s job.
    pub fn resume(&mut self) {
        self.is_pausing = false;
    }

    /// Ends playback. The playhead stays where it stands — nothing rewinds —
    /// so the host keeps drawing the frame it stopped on. Seek, or `play`,
    /// to move it.
    pub fn stop(&mut self) {
        self.is_playing = false;
        // A stopped animation is not a held one. Leaving the hold set would
        // surface through every Player as "stopped, and also paused", the same
        // meaningless state the seed value used to produce. Nothing internal
        // reads it while `is_playing` is false, so this only cleans up what the
        // getter reports.
        self.is_pausing = false;
    }

    pub fn set_frame_no(&mut self, frame: f32) {
        self.current_frame = self.clamp_to_section(frame);
        self.needs_evaluation = true;
    }

    fn clamp_to_section(&self, frame: f32) -> f32 {
        let low = self.start_frame as f32;
        let high = self.end_frame as f32;
        if frame < low {
            low
        } else if frame >= high + 1.0 {
            high
        } else {
            frame
        }
    }

    /// Sets `current_frame` to the position reached after `diff` frames have
    /// elapsed since playback started, with the configured `start_frame` /
    /// `end_frame` / `loops` / `playback_style` / `playback_direction`
    /// applied. `diff` should already incorporate per-instance `speed`
    /// (i.e. `diff = (parent_frame - event_frame) * speed`).
    ///
    /// Unlike `update(delta)` this is a deterministic, non-incremental
    /// computation — the same `diff` always produces the same frame, which is
    /// what allows parent-driven seek / reverse / frame-skip on Instance parts
    /// to render correctly.
    pub fn set_frame_relative(&mut self, diff: f32) {
        let (frame, _finished) = resolve_frame(diff, self.start_frame, self.end_frame, self.loops, self.playback_style, self.playback_direction);
        self.current_frame = frame;
        self.needs_evaluation = true;
    }

    /**
     * Set the current frame to the next frame.
     * The function ignores direction and style.
     */
    pub fn next_frame(&mut self) {
        let current_frame = floorf(self.current_frame) as i32;
        if current_frame == self.end_frame {
            return;
        }
        self.set_frame_no((current_frame + 1) as f32);
    }

    /**
     * Set the current frame to the previous frame.
     * The function ignores direction and style.
     */
    pub fn prev_frame(&mut self) {
        let current_frame = floorf(self.current_frame) as i32;
        if current_frame == self.start_frame {
            return;
        }
        self.set_frame_no((current_frame - 1) as f32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rf(diff: f32, start: i32, end: i32, loops: i32, style: PlaybackStyle, dir: PlaybackDirection) -> (f32, bool) {
        resolve_frame(diff, start, end, loops, style, dir)
    }

    #[test]
    fn resolve_frame_normal_forward_finite() {
        // start=0 end=10 inclusive → 11 frames per cycle (matches SS6 inst_scale)
        let (start, end) = (0, 10);
        let s = PlaybackStyle::Normal;
        let d = PlaybackDirection::Forward;

        assert_eq!(rf(-5.0, start, end, 1, s, d), (0.0, false));
        assert_eq!(rf(0.0, start, end, 1, s, d), (0.0, false));
        assert_eq!(rf(5.0, start, end, 1, s, d), (5.0, false));
        assert_eq!(rf(10.0, start, end, 1, s, d), (10.0, false));
        assert_eq!(rf(11.0, start, end, 1, s, d), (10.0, true));
        assert_eq!(rf(50.0, start, end, 1, s, d), (10.0, true));

        // 2 cycles = 22 frames
        assert_eq!(rf(11.0, start, end, 2, s, d), (0.0, false));
        assert_eq!(rf(15.0, start, end, 2, s, d), (4.0, false));
        assert_eq!(rf(21.0, start, end, 2, s, d), (10.0, false));
        assert_eq!(rf(22.0, start, end, 2, s, d), (10.0, true));
        assert_eq!(rf(100.0, start, end, 2, s, d), (10.0, true));
    }

    #[test]
    fn resolve_frame_normal_forward_infinite() {
        let (start, end) = (0, 10);
        let s = PlaybackStyle::Normal;
        let d = PlaybackDirection::Forward;

        assert_eq!(rf(0.0, start, end, -1, s, d), (0.0, false));
        assert_eq!(rf(10.0, start, end, -1, s, d), (10.0, false));
        assert_eq!(rf(11.0, start, end, -1, s, d), (0.0, false));
        assert_eq!(rf(15.0, start, end, -1, s, d), (4.0, false));
        assert_eq!(rf(22.0, start, end, -1, s, d), (0.0, false));
        assert!(!rf(0.0, start, end, 0, s, d).1);
        assert!(!rf(100.0, start, end, 0, s, d).1);
    }

    #[test]
    fn resolve_frame_normal_backward() {
        let (start, end) = (0, 10);
        let s = PlaybackStyle::Normal;
        let d = PlaybackDirection::Backward;

        assert_eq!(rf(0.0, start, end, 1, s, d), (10.0, false));
        assert_eq!(rf(3.0, start, end, 1, s, d), (7.0, false));
        assert_eq!(rf(10.0, start, end, 1, s, d), (0.0, false));
        assert_eq!(rf(11.0, start, end, 1, s, d), (0.0, true));
        assert_eq!(rf(50.0, start, end, 1, s, d), (0.0, true));
        // Infinite: cycle wraps; phase=15 → phase mod 11 = 4 → frame = end - 4 = 6.
        assert_eq!(rf(15.0, start, end, -1, s, d), (6.0, false));
    }

    #[test]
    fn resolve_frame_pingpong_forward() {
        // start=0 end=10. cycle = 22 (forward 0..10 then reverse 10..0,
        // peak 10 appears at phase 10 and phase 11 to match SS6).
        let (start, end) = (0, 10);
        let s = PlaybackStyle::PingPong;
        let d = PlaybackDirection::Forward;

        assert_eq!(rf(0.0, start, end, 1, s, d), (0.0, false));
        assert_eq!(rf(5.0, start, end, 1, s, d), (5.0, false));
        assert_eq!(rf(10.0, start, end, 1, s, d), (10.0, false));
        assert_eq!(rf(11.0, start, end, 1, s, d), (10.0, false)); // peak duplicated
        assert_eq!(rf(12.0, start, end, 1, s, d), (9.0, false));
        assert_eq!(rf(21.0, start, end, 1, s, d), (0.0, false));
        assert_eq!(rf(22.0, start, end, 1, s, d), (0.0, true));

        // 2 cycles = 44
        assert_eq!(rf(22.0, start, end, 2, s, d), (0.0, false));
        assert_eq!(rf(33.0, start, end, 2, s, d), (10.0, false));
        assert_eq!(rf(44.0, start, end, 2, s, d), (0.0, true));

        // Infinite
        assert_eq!(rf(22.0, start, end, -1, s, d), (0.0, false));
        assert_eq!(rf(33.0, start, end, -1, s, d), (10.0, false));
    }

    #[test]
    fn resolve_frame_pingpong_backward() {
        let (start, end) = (0, 10);
        let s = PlaybackStyle::PingPong;
        let d = PlaybackDirection::Backward;

        assert_eq!(rf(0.0, start, end, 1, s, d), (10.0, false));
        assert_eq!(rf(5.0, start, end, 1, s, d), (5.0, false));
        assert_eq!(rf(10.0, start, end, 1, s, d), (0.0, false));
        assert_eq!(rf(11.0, start, end, 1, s, d), (0.0, false)); // trough duplicated
        assert_eq!(rf(21.0, start, end, 1, s, d), (10.0, false));
        assert_eq!(rf(22.0, start, end, 1, s, d), (10.0, true));
    }

    #[test]
    fn resolve_frame_never_leaves_the_section_on_fractional_diffs() {
        for &(start, end) in &[(0, 4), (5, 9), (5, 5)] {
            for &style in &[PlaybackStyle::Normal, PlaybackStyle::PingPong] {
                for &dir in &[PlaybackDirection::Forward, PlaybackDirection::Backward] {
                    for i in 0..400 {
                        let diff = i as f32 * 0.25;
                        let (frame, _) = rf(diff, start, end, 3, style, dir);
                        let reported = floorf(frame) as i32;
                        assert!((start..=end).contains(&reported), "diff={diff} start={start} end={end} style={} dir={} resolved {frame} -> reports frame {reported}", style as i32, dir as i32);
                    }
                }
            }
        }
    }

    #[test]
    fn resolve_frame_backward_holds_the_start_frame_through_its_slot() {
        let (start, end) = (5, 9);
        let s = PlaybackStyle::Normal;
        let d = PlaybackDirection::Backward;

        assert_eq!(rf(4.0, start, end, 3, s, d), (5.0, false));
        assert_eq!(rf(4.5, start, end, 3, s, d), (5.0, false));
        assert_eq!(rf(5.0, start, end, 3, s, d), (9.0, false));
    }

    #[test]
    fn resolve_frame_pingpong_holds_the_trough_through_its_slot() {
        let (start, end) = (5, 9);
        let s = PlaybackStyle::PingPong;
        let d = PlaybackDirection::Forward;

        assert_eq!(rf(9.0, start, end, 3, s, d), (5.0, false));
        assert_eq!(rf(9.5, start, end, 3, s, d), (5.0, false));
        assert_eq!(rf(10.0, start, end, 3, s, d), (5.0, false));
    }

    #[test]
    fn resolve_frame_zero_length_section_clamps() {
        // end == start → single-frame section, length=1, valid.
        let s = PlaybackStyle::Normal;
        let d = PlaybackDirection::Forward;
        assert_eq!(rf(0.0, 5, 5, 1, s, d), (5.0, false));
        assert_eq!(rf(1.0, 5, 5, 1, s, d), (5.0, true));
        assert_eq!(rf(100.0, 5, 5, -1, s, d), (5.0, false));
    }

    #[test]
    fn set_frame_relative_applies_section_and_style() {
        let mut t = PlaybackController::new();
        t.setup(0, 100, 30);
        // section [10, 20] inclusive → 11-frame cycle
        t.set_animation_section(10, 20);
        t.set_loop_count(-1);
        t.set_playback_direction(PlaybackDirection::Forward, PlaybackStyle::Normal);

        t.set_frame_relative(0.0);
        assert_eq!(t.get_frame_no(), 10.0);
        t.set_frame_relative(5.0);
        assert_eq!(t.get_frame_no(), 15.0);
        t.set_frame_relative(10.0);
        assert_eq!(t.get_frame_no(), 20.0);
        t.set_frame_relative(11.0);
        assert_eq!(t.get_frame_no(), 10.0);
        t.set_frame_relative(15.0);
        assert_eq!(t.get_frame_no(), 14.0);
    }

    /// Drives `t` one frame at a time and records the loop / finish events a
    /// Player would observe by polling after every `update`.
    fn collect_events(t: &mut PlaybackController, steps: usize) -> Vec<(usize, &'static str)> {
        let interval = 1000.0 / 30.0;
        let mut events = Vec::new();
        let mut was_finished = false;
        for i in 0..steps {
            t.update(interval);
            if t.just_looped() {
                events.push((i, "looped"));
            }
            if t.is_finished() && !was_finished {
                events.push((i, "finished"));
            }
            was_finished = t.is_finished();
        }
        events
    }

    #[test]
    fn set_loop_count_3_emits_looped_looped_finished() {
        let mut t = PlaybackController::new();
        t.setup(0, 4, 30);
        t.set_loop_count(3);
        t.set_skip_frames(false);
        t.play();

        let kinds: Vec<_> = collect_events(&mut t, 30).into_iter().map(|(_, k)| k).collect();
        assert_eq!(kinds, ["looped", "looped", "finished"]);
        assert_eq!(t.get_frame_no(), 4.0, "Should rest on the end frame");
        assert!(!t.is_playing());
    }

    #[test]
    fn set_frame_no_keeps_sub_frame_progress_inside_the_last_frame() {
        let mut t = PlaybackController::new();
        t.setup(0, 10, 60);

        t.set_frame_no(10.25);
        assert_eq!(t.get_frame_no(), 10.25, "frame 10 spans [10, 11); 10.25 is inside the section");

        t.set_frame_no(11.0);
        assert_eq!(t.get_frame_no(), 10.0, "frame 11 is past the section");

        t.set_frame_no(15.0);
        assert_eq!(t.get_frame_no(), 10.0);

        t.set_frame_no(-1.0);
        assert_eq!(t.get_frame_no(), 0.0);
    }

    #[test]
    fn writing_the_reported_frame_back_does_not_freeze_sub_frame_playback() {
        // The host loop Unity runs for an independent Instance child: advance the
        // controller, then hand the reported frame straight back through
        // `set_frame_no`. Clamping that write-back to `end_frame` truncates the
        // last frame's sub-frame progress every tick, so the head can never reach
        // the wrap and the child freezes on the section end.
        let mut t = PlaybackController::new();
        t.setup(0, 10, 60);
        t.set_loop_count(-1);
        t.play();

        let quarter_frame = 1000.0 / 60.0 / 4.0;
        let mut looped = 0;
        for _ in 0..(11 * 4 * 2) {
            t.update(quarter_frame);
            let reported = t.get_frame_no();
            t.set_frame_no(reported);
            if t.just_looped() {
                looped += 1;
            }
        }
        assert_eq!(looped, 2, "infinite loop froze at frame {}", t.get_frame_no());
    }

    #[test]
    fn infinite_loop_emits_looped_only_and_never_finishes() {
        let mut t = PlaybackController::new();
        t.setup(0, 4, 30);
        t.set_loop_count(-1);
        t.set_skip_frames(false);
        t.play();

        // 30 steps over an inclusive 5-frame cycle wraps 6 times.
        let kinds: Vec<_> = collect_events(&mut t, 30).into_iter().map(|(_, k)| k).collect();
        assert_eq!(kinds, ["looped"; 6]);
        assert!(!t.is_finished());
        assert!(t.is_playing());
    }

    #[test]
    fn set_loop_count_0_is_infinite() {
        let mut t = PlaybackController::new();
        t.setup(0, 4, 30);
        t.set_loop_count(0);
        assert_eq!(t.get_loop_count(), -1, "0 normalizes to the infinite sentinel");
        assert_eq!(t.get_remaining_loops(), -1);
        t.set_skip_frames(false);
        t.play();

        let kinds: Vec<_> = collect_events(&mut t, 30).into_iter().map(|(_, k)| k).collect();
        assert_eq!(kinds, ["looped"; 6], "0 must behave exactly like -1");
        assert!(!t.is_end_frame_reached(), "Infinite loop must never raise end_frame_reached");
        assert!(!t.is_finished());
    }

    #[test]
    fn is_finished_distinguishes_stop_from_completion() {
        // stop() part-way through must not read as finished...
        let mut t = PlaybackController::new();
        t.setup(0, 4, 30);
        t.set_loop_count(3);
        t.set_skip_frames(false);
        t.play();
        t.update(1000.0 / 30.0);
        t.stop();
        assert!(!t.is_playing());
        assert!(!t.is_finished(), "stop() mid-playback is not 'finished'");

        // ...not even when the playhead is resting on the last frame of a
        // non-final cycle, where end_frame_reached is legitimately set.
        let mut t = PlaybackController::new();
        t.setup(0, 4, 30);
        t.set_loop_count(3);
        t.set_skip_frames(false);
        t.play();
        for _ in 0..4 {
            t.update(1000.0 / 30.0);
        }
        assert!(t.is_end_frame_reached(), "Should be showing the end frame of cycle 1");
        t.stop();
        assert!(!t.is_finished(), "end_frame_reached + stopped is not 'finished'");
    }

    #[test]
    fn setup_rearms_loop_count_for_the_next_animation() {
        let mut t = PlaybackController::new();
        t.setup(0, 4, 30);
        t.set_loop_count(1);
        t.set_skip_frames(false);
        t.play();
        for _ in 0..6 {
            t.update(1000.0 / 30.0);
        }
        assert!(t.is_finished());
        assert_eq!(t.get_remaining_loops(), 0);

        // A stale remain_loops == 0 would raise end_frame_reached at the end of
        // the next animation's first cycle even though it loops forever.
        t.setup(0, 4, 30);
        assert_eq!(t.get_loop_count(), -1);
        assert_eq!(t.get_remaining_loops(), -1, "remain_loops must not leak across setup()");
        t.set_skip_frames(false);
        t.play();
        let kinds: Vec<_> = collect_events(&mut t, 12).into_iter().map(|(_, k)| k).collect();
        assert_eq!(kinds, ["looped", "looped"]);
        assert!(!t.is_end_frame_reached());
    }

    #[test]
    fn replaying_a_finished_animation_rearms_the_loop_count() {
        let mut t = PlaybackController::new();
        t.setup(0, 4, 30);
        t.set_loop_count(2);
        t.set_skip_frames(false);
        t.play();
        let kinds: Vec<_> = collect_events(&mut t, 30).into_iter().map(|(_, k)| k).collect();
        assert_eq!(kinds, ["looped", "finished"]);

        // Without re-arming, remain_loops == 0 would decrement to the infinite
        // sentinel on the first wrap and the replay would never finish.
        t.play();
        assert!(!t.is_finished(), "play() must drop the stale terminal state");
        assert!(!t.is_end_frame_reached());
        let kinds: Vec<_> = collect_events(&mut t, 30).into_iter().map(|(_, k)| k).collect();
        assert_eq!(kinds, ["looped", "finished"], "Replay must honour the same loop count");
    }

    /// Every frame `t` reports as passed, across `steps` updates.
    fn collect_passed(t: &mut PlaybackController, steps: usize) -> Vec<i32> {
        let interval = 1000.0 / 30.0;
        let mut all = Vec::new();
        for _ in 0..steps {
            t.update(interval);
            all.extend_from_slice(t.get_passed_frames());
        }
        all
    }

    #[test]
    fn end_frame_is_reported_once_when_playback_stops() {
        // The update that stops playback parks on the end frame, which the
        // previous update already reported. Reporting it again re-fires any
        // user-data / signal / audio key authored on the last frame.
        let mut t = PlaybackController::new();
        t.setup(0, 4, 30);
        t.set_loop_count(1);
        t.set_skip_frames(false);
        t.play();
        assert_eq!(collect_passed(&mut t, 8), [0, 1, 2, 3, 4]);
    }

    #[test]
    fn end_frame_is_reported_once_when_a_single_update_overruns_the_end() {
        // Frame-skipping with a delta big enough to walk the whole clip in one
        // update: the stop branch is then entered by the same inner loop that
        // just reported the end frame.
        let mut t = PlaybackController::new();
        t.setup(0, 4, 30);
        t.set_loop_count(1);
        t.set_skip_frames(true);
        t.play();
        t.update((1000.0 / 30.0) * 10.0);
        assert_eq!(t.get_passed_frames(), [0, 1, 2, 3, 4]);
        assert!(!t.is_playing());
    }

    #[test]
    fn single_frame_clip_reports_its_only_frame_once() {
        let mut t = PlaybackController::new();
        t.setup(5, 5, 30);
        t.set_loop_count(1);
        t.set_skip_frames(false);
        t.play();
        assert_eq!(collect_passed(&mut t, 6), [5]);
    }

    #[test]
    fn single_frame_pingpong_clip_stays_inside_the_section() {
        let mut t = PlaybackController::new();
        t.setup(5, 5, 30);
        t.set_playback_direction(PlaybackDirection::Forward, PlaybackStyle::PingPong);
        t.set_loop_count(3);
        t.set_skip_frames(false);
        t.play();
        assert_eq!(collect_passed(&mut t, 8), [5, 5, 5]);
    }

    #[test]
    fn single_frame_pingpong_clip_stays_inside_the_section_when_backward() {
        let mut t = PlaybackController::new();
        t.setup(5, 5, 30);
        t.set_playback_direction(PlaybackDirection::Backward, PlaybackStyle::PingPong);
        t.set_loop_count(3);
        t.set_skip_frames(false);
        t.play();
        assert_eq!(collect_passed(&mut t, 8), [5, 5, 5]);
    }

    #[test]
    fn narrowing_the_section_to_one_frame_reports_it() {
        let mut t = PlaybackController::new();
        t.setup(0, 9, 30);
        t.set_loop_count(1);
        t.set_skip_frames(false);
        t.play();
        collect_passed(&mut t, 1);
        t.set_animation_section(3, 3);
        assert_eq!(collect_passed(&mut t, 5), [3]);
    }

    #[test]
    fn a_new_section_reports_its_own_start_frame() {
        let mut t = PlaybackController::new();
        t.setup(0, 9, 30);
        t.set_loop_count(1);
        t.set_skip_frames(false);
        t.play();
        collect_passed(&mut t, 1);
        t.set_animation_section(2, 5);
        assert_eq!(collect_passed(&mut t, 8), [2, 3, 4, 5]);
    }

    #[test]
    fn seeking_outside_the_section_is_confined_to_it() {
        let mut t = PlaybackController::new();
        t.setup(0, 9, 30);
        t.set_animation_section(2, 5);
        t.set_loop_count(1);
        t.set_skip_frames(false);
        t.play();

        t.set_frame_no(9.0);
        assert_eq!(t.get_frame_no(), 5.0);
        assert_eq!(collect_passed(&mut t, 3), [5]);

        t.set_frame_no(-4.0);
        assert_eq!(t.get_frame_no(), 2.0);
    }

    #[test]
    fn pingpong_reports_the_turnaround_frame_once_when_playback_stops() {
        let mut t = PlaybackController::new();
        t.setup(0, 3, 30);
        t.set_playback_direction(PlaybackDirection::Forward, PlaybackStyle::PingPong);
        t.set_loop_count(1);
        t.set_skip_frames(false);
        t.play();
        // Out 0..3, back 3..0. The trailing 0 -- where playback stops -- must be
        // reported once. (The peak is not doubled here: the duplicated
        // turn-around frame is a `resolve_frame` property, not an `update` one.)
        assert_eq!(collect_passed(&mut t, 10), [0, 1, 2, 3, 2, 1, 0]);
    }

    #[test]
    fn looping_still_reports_every_frame_of_every_cycle() {
        let mut t = PlaybackController::new();
        t.setup(0, 2, 30);
        t.set_loop_count(2);
        t.set_skip_frames(false);
        t.play();
        assert_eq!(collect_passed(&mut t, 8), [0, 1, 2, 0, 1, 2]);
    }

    #[test]
    fn test_normal_playback_no_loop() {
        let mut t = PlaybackController::new();
        t.setup(0, 10, 30);
        t.set_loop_count(1);
        t.set_skip_frames(false);
        t.play();

        let interval = 1000.0 / 30.0;

        // Frames 0 to 9
        for i in 1..=10 {
            let frame = t.update(interval);
            assert_eq!(frame, i as f32, "Failed at step {} (Forward)", i);
            assert!(t.is_playing());
        }

        assert!(t.is_end_frame_reached(), "end_frame_reached should be true at frame 10");

        // Next update should stop playback
        let frame = t.update(interval);
        assert_eq!(frame, 10.0, "Should remain at end frame when stopped");
        assert!(!t.is_playing(), "Should stop playing after 1 loop");
    }

    #[test]
    fn test_normal_playback_loops() {
        let mut t = PlaybackController::new();
        t.setup(0, 10, 30);
        t.set_loop_count(2);
        t.set_skip_frames(false);
        t.play();

        let interval = 1000.0 / 30.0;

        // First loop: 0 -> 10
        for _ in 1..=10 {
            t.update(interval);
        }
        assert!(t.is_end_frame_reached(), "end_frame_reached should be true after first loop");
        assert_eq!(t.get_frame_no(), 10.0);
        assert_eq!(t.get_remaining_loops(), 2);

        // Transition to second loop (shows frame 0)
        let frame = t.update(interval);
        assert_eq!(frame, 0.0, "Should jump back to start frame");
        assert_eq!(t.get_remaining_loops(), 1);
        assert!(!t.is_end_frame_reached(), "end_frame_reached should reset at start of new loop");

        // Second loop: 0 -> 10
        for _ in 1..=10 {
            t.update(interval);
        }
        assert!(t.is_end_frame_reached());

        // Final update stops
        t.update(interval);
        assert!(!t.is_playing(), "Should stop playing after 2 loops");
        assert_eq!(t.get_remaining_loops(), 0);
    }

    #[test]
    fn test_pingpong_playback() {
        let mut t = PlaybackController::new();
        t.setup(0, 10, 30);
        t.set_playback_direction(PlaybackDirection::Forward, PlaybackStyle::PingPong);
        t.set_loop_count(1);
        t.set_skip_frames(false);
        t.play();

        let interval = 1000.0 / 30.0;

        // Forward: 0 -> 10
        for i in 1..=10 {
            let frame = t.update(interval);
            assert_eq!(frame, i as f32, "Forward failed at step {}", i);
        }

        // At 10, it should NOT have reached end yet in PingPong (need to return to 0)
        assert!(!t.is_end_frame_reached(), "end_frame_reached should not be true at far end in PingPong");

        // Backward: 10 -> 0
        for i in (0..=9).rev() {
            let frame = t.update(interval);
            assert_eq!(frame, i as f32, "Backward failed at step {}", i);
        }

        assert!(t.is_end_frame_reached(), "end_frame_reached should be true after full PingPong cycle");

        // Final update stops
        t.update(interval);
        assert!(!t.is_playing(), "Should stop playing after 1 PingPong loop");
    }

    #[test]
    fn test_smooth_interpolation() {
        let mut t = PlaybackController::new();
        t.setup(0, 10, 30);
        t.set_skip_frames(true);
        t.play();

        let delta = 1000.0 / 60.0; // Half a frame

        let frame = t.update(delta);
        assert_eq!(floorf(frame), 0.0);
        let decimal = frame - floorf(frame);
        assert!(decimal > 0.4 && decimal < 0.6, "Expected around 0.5, got {}", decimal);

        let frame = t.update(delta);
        assert_eq!(floorf(frame), 1.0);
        let decimal = frame - floorf(frame);
        assert!((0.0..0.1).contains(&decimal), "Expected around 0.0, got {}", decimal);
    }

    #[test]
    fn test_infinite_loop() {
        let mut t = PlaybackController::new();
        t.setup(0, 10, 30);
        t.set_loop_count(-1);
        t.play();

        let interval = 1000.0 / 30.0;

        for _ in 0..100 {
            t.update(interval);
        }

        assert!(t.is_playing(), "Infinite loop should never stop");
        assert_eq!(t.get_remaining_loops(), -1);
    }

    #[test]
    fn test_fixed_animation_no_skip() {
        let mut t = PlaybackController::new();
        t.setup(0, 10, 30);
        t.set_skip_frames(false); // Fixed Animation mode
        t.play();

        // 30 FPS, interval = 33.33ms
        // Supply a very large delta (e.g., 500ms = 15 frames worth of time)
        let delta = 500.0;

        let frame = t.update(delta);

        assert_eq!(frame, 1.0, "Fixed animation should only advance by 1 frame at a time");

        // The remaining time (500 - 33.33) should be stored in next_frame_time.
        // Calling update with 0 delta should continue to advance frames until the buffer is empty.
        let frame = t.update(0.0);
        assert_eq!(frame, 2.0, "Should use buffered time to advance even with 0 delta");
    }

    #[test]
    fn test_reverse_playback_no_loop() {
        let mut t = PlaybackController::new();
        t.setup(0, 10, 30);
        t.set_playback_direction(PlaybackDirection::Backward, PlaybackStyle::Normal);
        t.set_loop_count(1);
        t.set_skip_frames(false);
        t.play();

        let interval = 1000.0 / 30.0;

        // Starts at 10, goes down to 1
        for i in (0..=9).rev() {
            let frame = t.update(interval);
            assert_eq!(frame, i as f32, "Reverse failed at step {}", i);
        }

        assert!(t.is_end_frame_reached(), "end_frame_reached should be true at frame 0 (start_frame for reverse)");
        t.update(interval);
        assert!(!t.is_playing(), "Should stop after 1 reverse loop");
    }

    #[test]
    fn test_reverse_playback_loops() {
        let mut t = PlaybackController::new();
        t.setup(0, 10, 30);
        t.set_playback_direction(PlaybackDirection::Backward, PlaybackStyle::Normal);
        t.set_loop_count(2);
        t.set_skip_frames(false);
        t.play();

        let interval = 1000.0 / 30.0;

        // First loop: 10 -> 0
        for _ in 0..10 {
            t.update(interval);
        }
        assert!(t.is_end_frame_reached());
        assert_eq!(t.get_frame_no(), 0.0);

        // Transition to second loop (jumps to 10)
        let frame = t.update(interval);
        assert_eq!(frame, 10.0);
        assert!(!t.is_end_frame_reached());

        // Second loop: 10 -> 0
        for _ in 0..10 {
            t.update(interval);
        }
        assert!(t.is_end_frame_reached());

        t.update(interval);
        assert!(!t.is_playing());
    }

    #[test]
    fn test_reverse_pingpong_playback() {
        let mut t = PlaybackController::new();
        t.setup(0, 10, 30);
        t.set_playback_direction(PlaybackDirection::Backward, PlaybackStyle::PingPong);
        t.set_loop_count(1);
        t.set_skip_frames(false);
        t.play();

        let interval = 1000.0 / 30.0;

        // Backward first: 10 -> 0
        for i in (0..=9).rev() {
            let frame = t.update(interval);
            assert_eq!(frame, i as f32, "Initial backward failed at step {}", i);
        }

        assert!(!t.is_end_frame_reached());

        // Then Forward: 0 -> 10
        for i in 1..=10 {
            let frame = t.update(interval);
            assert_eq!(frame, i as f32, "Return forward failed at step {}", i);
        }

        assert!(t.is_end_frame_reached());
        t.update(interval);
        assert!(!t.is_playing());
    }

    #[test]
    fn test_skip_frames_multi_step() {
        let mut t = PlaybackController::new();
        t.setup(0, 10, 30);
        t.set_skip_frames(true);
        t.play();

        // 30 FPS, interval = 33.33ms
        // Skip 2.5 frames at once (33.33 * 2.5 = 83.33ms)
        let delta = (1000.0 / 30.0) * 2.5;

        let frame = t.update(delta);
        assert_eq!(floorf(frame), 2.0, "Should jump to frame 2");
        let decimal = frame - floorf(frame);
        assert!(decimal > 0.4 && decimal < 0.6, "Decimal should be around 0.5, got {}", decimal);

        // Update another 1.0 frame
        let frame = t.update(1000.0 / 30.0);
        assert_eq!(floorf(frame), 3.0);
        let decimal = frame - floorf(frame);
        assert!(decimal > 0.4 && decimal < 0.6, "Decimal should stay around 0.5, got {}", decimal);
    }

    #[test]
    fn test_skip_frames_loop_boundary() {
        let mut t = PlaybackController::new();
        t.setup(0, 10, 30);
        t.set_loop_count(2);
        t.set_skip_frames(true);
        t.play();

        // Advance by 11.5 frames.
        // 0.0 -> 10.0 (end frame) -> 0.0 (start of next loop) -> 0.5 (decimal)
        let delta = (1000.0 / 30.0) * 11.5;

        let frame = t.update(delta);

        assert_eq!(floorf(frame), 0.0, "Should have looped back to start frame");
        let decimal = frame - floorf(frame);
        assert!(decimal > 0.4 && decimal < 0.6, "Should have 0.5 decimal after loop, got {}", decimal);
        assert_eq!(t.get_remaining_loops(), 1, "Should have 1 loop remaining");
    }

    #[test]
    fn test_passed_frames_recording() {
        let mut t = PlaybackController::new();
        t.setup(0, 10, 30);
        t.set_skip_frames(true);
        t.play();

        let interval = 1000.0 / 30.0;

        // 1. Advance by 2.5 frames (0.0 -> 2.5)
        // Should pass frames 0, 1, and 2. (0 is the start frame, handled by needs_evaluation)
        t.update(interval * 2.5);
        assert_eq!(t.get_passed_frames(), &[0, 1, 2]);

        // 2. Advance by another 1.0 frame (2.5 -> 3.5)
        // Should pass frame 3.
        t.update(interval);
        assert_eq!(t.get_passed_frames(), &[3]);

        // 3. Large skip (3.5 -> 8.5)
        // Should pass 4, 5, 6, 7, 8.
        t.update(interval * 5.0);
        assert_eq!(t.get_passed_frames(), &[4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_passed_frames_looping() {
        let mut t = PlaybackController::new();
        t.setup(0, 5, 30); // 0 to 5
        t.set_loop_count(2);
        t.set_skip_frames(true);
        t.play();

        let interval = 1000.0 / 30.0;

        // Currently at 0.0. Skip to 2.5.
        // Should pass 0 (start), 1, 2
        t.update(interval * 2.5);
        assert_eq!(t.get_passed_frames(), &[0, 1, 2]);

        // Skip across loop boundary: 2.5 -> 7.5
        // Frame sequence: 3, 4, 5 (hit end), 0 (loop), 1
        t.update(interval * 5.0);
        assert_eq!(t.get_passed_frames(), &[3, 4, 5, 0, 1]);
    }

    #[test]
    fn test_passed_frames_start_frame() {
        let mut t = PlaybackController::new();
        t.setup(0, 10, 30);
        t.play();

        let interval = 1000.0 / 30.0;

        // Advance by 0.5 frames. We are still on frame 0, but since this is the very
        // first update after play(), frame 0 should be considered "passed/reached".
        t.update(interval * 0.5);
        assert_eq!(t.get_passed_frames(), &[0], "Start frame must be collected on first update");

        // Advance by another 0.5 frames. We reach frame 1.
        t.update(interval * 0.5);
        assert_eq!(t.get_passed_frames(), &[1], "Should collect frame 1");
    }

    #[test]
    fn test_skip_frames_pingpong_boundary() {
        let mut t = PlaybackController::new();
        t.setup(0, 10, 30);
        t.set_playback_direction(PlaybackDirection::Forward, PlaybackStyle::PingPong);
        t.set_skip_frames(true);
        t.play();

        // Currently at 0.0.
        // Advance by 12.0 frames.
        // 0.0 -> 10.0 (hit end) -> (12.0 - 10.0 = 2.0 frames backwards) -> 8.0
        let delta = (1000.0 / 30.0) * 12.0;

        let frame = t.update(delta);
        assert_eq!(floorf(frame), 8.0, "Should have bounced back to frame 8");
        let decimal = frame - floorf(frame);
        assert!((0.0..0.001).contains(&decimal), "Decimal should be exactly 0.0, got {}", decimal);

        // Check passed_frames for pingpong boundary
        assert_eq!(t.get_passed_frames(), &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 9, 8]);
    }

    #[test]
    fn test_passed_frames_high_speed() {
        let mut t = PlaybackController::new();
        t.setup(0, 20, 30);
        t.set_skip_frames(true);
        t.play();

        let interval = 1000.0 / 30.0;

        // Advance 15.5 frames at once (0.0 -> 15.5)
        t.update(interval * 15.5);
        let passed = t.get_passed_frames();
        // Expect 0 to 15
        let expected: Vec<i32> = (0..=15).collect();
        assert_eq!(passed, expected.as_slice());
    }

    #[test]
    fn test_passed_frames_multiple_loops() {
        let mut t = PlaybackController::new();
        t.setup(0, 5, 30); // Total 6 frames (0,1,2,3,4,5)
        t.set_loop_count(-1); // Infinite
        t.set_skip_frames(true);
        t.play();

        let interval = 1000.0 / 30.0;

        // Current at 0.0. Advance 14.5 frames.
        // 0, 1, 2, 3, 4, 5 (end) -> 0, 1, 2, 3, 4, 5 (end) -> 0, 1, 2 (current 2.5)
        t.update(interval * 14.5);
        let passed = t.get_passed_frames();
        let expected = vec![0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2];
        assert_eq!(passed, expected.as_slice());
    }

    #[test]
    fn test_passed_frames_backward_skip() {
        let mut t = PlaybackController::new();
        t.setup(0, 10, 30);
        t.set_playback_direction(PlaybackDirection::Backward, PlaybackStyle::Normal);
        t.set_skip_frames(true);
        t.play(); // Starts at 10.0

        let interval = 1000.0 / 30.0;

        // 10.0 -> 2.5 (Skip 7.5 frames backward)
        t.update(interval * 7.5);
        let passed = t.get_passed_frames();
        // Expected: 10 (start), 9, 8, 7, 6, 5, 4, 3, 2 (final floor(2.5))
        let expected = vec![10, 9, 8, 7, 6, 5, 4, 3, 2];
        assert_eq!(passed, expected.as_slice());
    }

    #[test]
    fn test_passed_frames_pingpong_multi_skip() {
        let mut t = PlaybackController::new();
        t.setup(0, 5, 30);
        t.set_playback_direction(PlaybackDirection::Forward, PlaybackStyle::PingPong);
        t.set_skip_frames(true);
        t.play(); // Starts at 0.0

        let interval = 1000.0 / 30.0;

        // Advance 12.5 frames.
        // 0 -> 5 (Forward end) : 0,1,2,3,4,5
        // 5 -> 0 (Backward end) : 4,3,2,1,0
        // 0 -> 2.5 (Forward) : 1, 2
        t.update(interval * 12.5);
        let passed = t.get_passed_frames();
        let expected = vec![0, 1, 2, 3, 4, 5, 4, 3, 2, 1, 0, 1, 2];
        assert_eq!(passed, expected.as_slice());
    }

    #[test]
    fn test_is_looped_flag_normal_infinite() {
        let mut t = PlaybackController::new();
        t.setup(0, 5, 30); // 0 to 5 (6 frames)
        t.set_loop_count(-1); // Infinite
        t.set_playback_direction(PlaybackDirection::Forward, PlaybackStyle::Normal);
        t.set_skip_frames(false);
        t.play();

        let interval = 1000.0 / 30.0;

        // 0..4
        for _ in 0..5 {
            t.update(interval);
            assert!(!t.just_looped(), "Should not be looped yet");
        }

        // Frame 5 -> 0 (Loop)
        t.update(interval);
        assert!(t.just_looped(), "Should be looped at boundary");
        assert!(!t.is_end_frame_reached(), "Should not reach end frame in infinite loop");

        // Next frame, flag should reset
        t.update(interval);
        assert!(!t.just_looped(), "just_looped should reset on next update");
    }

    #[test]
    fn test_is_looped_flag_pingpong() {
        let mut t = PlaybackController::new();
        t.setup(0, 5, 30); // 0 to 5
        t.set_loop_count(2);
        t.set_playback_direction(PlaybackDirection::Forward, PlaybackStyle::PingPong);
        t.set_skip_frames(false);
        t.play();

        let interval = 1000.0 / 30.0;

        // Forward 0..4
        for _ in 0..5 {
            t.update(interval);
            assert!(!t.just_looped(), "Should not be looped during forward");
        }

        // Frame 5 -> Start backward
        t.update(interval);
        // In ping-pong, hitting the far end doesn't count as a "loop" completion until we return to origin
        assert!(!t.just_looped(), "Should not loop at far end of pingpong");

        // Backward 4..1
        for _ in 0..4 {
            t.update(interval);
            assert!(!t.just_looped(), "Should not be looped during backward");
        }

        // Frame 0 -> 1 (Loop 1 complete)
        t.update(interval);
        assert!(t.just_looped(), "Should be looped when returning to origin in pingpong");
        assert!(!t.is_end_frame_reached());

        // Forward 1..4 again
        for _ in 0..4 {
            t.update(interval);
            assert!(!t.just_looped());
        }

        // Frame 5 -> Start backward again
        t.update(interval);
        assert!(!t.just_looped());

        // Backward 4..1
        for _ in 0..4 {
            t.update(interval);
            assert!(!t.just_looped());
        }

        // Frame 0 -> End
        t.update(interval);
        assert!(!t.just_looped(), "Should not loop on final end");
        assert!(t.is_end_frame_reached(), "Should reach end frame on last loop");
    }

    // ---- pause / resume / stop contract ------------------------------------
    //
    // The four transport verbs are what every Player publishes, so their
    // semantics are pinned here rather than left to each Player to rediscover.

    fn primed(loops: i32) -> PlaybackController {
        let mut t = PlaybackController::new();
        t.setup(0, 4, 30);
        t.set_loop_count(loops);
        t.set_skip_frames(false);
        t
    }

    #[test]
    fn a_controller_that_never_played_is_neither_playing_nor_paused() {
        // `is_pausing` used to be seeded `true`, which surfaced through every
        // Player as "paused" on an animation that had never run.
        let t = PlaybackController::new();
        assert!(!t.is_playing());
        assert!(!t.is_pausing());

        let t = primed(-1);
        assert!(!t.is_playing(), "setup() alone must not start playback");
        assert!(!t.is_pausing(), "setup() alone must not report a hold");
    }

    #[test]
    fn update_does_not_advance_until_play_is_called() {
        // `is_pausing` no longer starts life `true`, so the guard that keeps a
        // never-played controller still is `is_playing` alone. Pin that: the
        // playhead must not move, and no frame must be reported as passed, for
        // a controller that was only set up.
        let mut t = primed(-1);
        assert!(!t.is_playing());
        assert!(!t.is_pausing());

        for _ in 0..20 {
            t.update(1000.0 / 30.0);
            assert_eq!(t.get_frame_no(), 0.0, "setup() alone must not advance the playhead");
            assert!(!t.is_playing(), "update() must not start playback on its own");
            assert!(!t.just_looped());
            assert!(!t.is_finished());
        }

        // ...and the same after a stop, which also leaves is_pausing false.
        t.play();
        t.update(1000.0 / 30.0);
        t.stop();
        let stopped_at = t.get_frame_no();
        assert!(!t.is_pausing());
        for _ in 0..20 {
            t.update(1000.0 / 30.0);
            assert_eq!(t.get_frame_no(), stopped_at, "a stopped controller must not advance");
        }
    }

    #[test]
    fn pause_is_idempotent_and_never_resumes() {
        let mut t = primed(-1);
        t.play();
        t.update(1000.0 / 30.0);
        let held_at = t.get_frame_no();

        t.pause();
        assert!(t.is_pausing());
        t.pause();
        assert!(t.is_pausing(), "a second pause() must not lift the hold");

        // Held: time passes, the playhead does not.
        for _ in 0..4 {
            t.update(1000.0 / 30.0);
        }
        assert_eq!(t.get_frame_no(), held_at, "a paused animation must not advance");
        assert!(t.is_playing(), "a pause is a hold, not a stop");
    }

    #[test]
    fn resume_is_idempotent_and_carries_on_in_place() {
        let mut t = primed(-1);
        t.play();
        t.update(1000.0 / 30.0);
        t.pause();
        let held_at = t.get_frame_no();

        t.resume();
        assert!(!t.is_pausing());
        t.resume();
        assert!(!t.is_pausing(), "a second resume() must not re-pause");
        assert_eq!(t.get_frame_no(), held_at, "resume() must not move the playhead");

        t.update(1000.0 / 30.0);
        assert!(t.get_frame_no() > held_at, "resume() must let the playhead advance again");
    }

    #[test]
    fn resume_does_not_start_a_stopped_animation() {
        let mut t = primed(-1);
        t.play();
        t.update(1000.0 / 30.0);
        t.stop();
        assert!(!t.is_playing());

        t.resume();
        assert!(!t.is_playing(), "resume() lifts a hold; starting is play()'s job");
    }

    #[test]
    fn play_clears_a_hold_and_rewinds_to_the_section_start() {
        let mut t = primed(-1);
        t.play();
        for _ in 0..3 {
            t.update(1000.0 / 30.0);
        }
        t.pause();
        assert!(t.is_pausing());
        assert!(t.get_frame_no() > 0.0);

        // play() is a head-out, not a resume: it drops the hold *and* rewinds.
        t.play();
        assert!(!t.is_pausing(), "play() must clear the hold");
        assert_eq!(t.get_frame_no(), 0.0, "play() must rewind to the section start");
    }

    #[test]
    fn stop_clears_the_hold_and_holds_the_playhead() {
        let mut t = primed(-1);
        t.play();
        for _ in 0..3 {
            t.update(1000.0 / 30.0);
        }
        t.pause();
        let stopped_at = t.get_frame_no();

        t.stop();
        assert!(!t.is_playing());
        assert!(!t.is_pausing(), "a stopped animation must not report itself held");
        assert_eq!(t.get_frame_no(), stopped_at, "stop() does not rewind");
    }

    #[test]
    fn setup_clears_a_hold_left_by_the_previous_animation() {
        let mut t = primed(-1);
        t.play();
        t.update(1000.0 / 30.0);
        t.pause();
        assert!(t.is_pausing());

        // Switching animations must not carry the hold across, or the next
        // play() would look like it did nothing.
        t.setup(0, 9, 30);
        assert!(!t.is_pausing());
        assert!(!t.is_playing());
    }

    #[test]
    fn pausing_while_stopped_is_ignored() {
        // `is_pausing` implies `is_playing`: "stopped, and also paused" is not a
        // state this controller can be put into, so a host may read is_pausing
        // on its own without also testing is_playing.
        let mut t = primed(-1);
        t.pause();
        assert!(!t.is_pausing(), "pausing a never-played animation must do nothing");

        t.play();
        t.update(1000.0 / 30.0);
        t.stop();
        t.pause();
        assert!(!t.is_pausing(), "pausing a stopped animation must do nothing");
        assert!(!t.is_playing());

        // The hold is still available the moment something is actually playing.
        t.play();
        t.pause();
        assert!(t.is_pausing());
        assert!(t.is_playing());
    }

    // ---- what survives setup ----------------------------------------------

    #[test]
    fn setup_keeps_the_speed_and_frame_skip_the_host_chose() {
        // The settings a *different* system owns — a slow-motion effect, a
        // "show every frame" capture mode, a rewind mode — must not be clobbered
        // by an animation transition. What the transition itself specifies does
        // reset, so transitions stay independent of the one before them.
        let mut t = PlaybackController::new();
        t.setup(0, 4, 30);

        t.set_speed_scale(2.0);
        t.set_skip_frames(false);
        t.set_loop_count(3);
        t.set_animation_section(1, 3);
        t.set_playback_direction(PlaybackDirection::Backward, PlaybackStyle::PingPong);

        // A different clip, at a different frame rate.
        t.setup(0, 9, 60);

        assert_eq!(t.speed_scale, 2.0, "speed is the host's, not the clip's");
        assert!(!t.get_skip_frames(), "frame skip is the host's, not the clip's");

        assert!(t.playback_direction == PlaybackDirection::Backward, "direction is the host's, not the clip's");
        assert!(t.playback_style == PlaybackStyle::PingPong, "style is the host's, not the clip's");
        assert_eq!(t.playback_forward, -1, "and the heading is re-derived from it");

        assert_eq!(t.get_loop_count(), -1, "loop count is per-transition");
        assert_eq!((t.start_frame, t.end_frame), (0, 9), "the section is the new clip's own range");
    }

    #[test]
    fn setup_restarts_a_ping_pong_on_the_outward_leg() {
        // The direction survives, but the *heading* does not: a ping-pong that
        // was on its way back when the animation changed would otherwise start
        // the new one walking the wrong way out of the gate.
        let mut t = PlaybackController::new();
        t.setup(0, 9, 30);
        t.set_playback_direction(PlaybackDirection::Forward, PlaybackStyle::PingPong);
        t.playback_forward = -1; // mid-cycle: the return leg

        t.setup(0, 9, 30);

        assert!(t.playback_style == PlaybackStyle::PingPong, "the mode is kept");
        assert_eq!(t.playback_forward, 1, "but the new animation starts outbound");
    }

    #[test]
    fn releasing_a_slot_resets_the_direction() {
        // reset_setup hands the slot to a different owner, so unlike setup it
        // must not leak the previous one's direction.
        let mut t = PlaybackController::new();
        t.setup(0, 4, 30);
        t.set_playback_direction(PlaybackDirection::Backward, PlaybackStyle::PingPong);
        t.reset_setup();
        assert!(t.playback_direction == PlaybackDirection::Forward);
        assert!(t.playback_style == PlaybackStyle::Normal);
        assert_eq!(t.playback_forward, 1);
    }

    #[test]
    fn a_retained_speed_applies_to_the_new_animations_frame_rate() {
        // Observable rather than field-level: the retained rate has to be folded
        // into `update_interval` against the *new* clip's fps, not the old one's.
        // The clip is long enough that one second cannot wrap the loop and hide
        // the result behind a modulo.
        let mut t = PlaybackController::new();
        t.setup(0, 1000, 30);
        t.set_skip_frames(true);
        t.set_speed_scale(2.0);

        t.setup(0, 1000, 60); // same host, new clip at 60fps
        t.play();
        t.update(1000.0); // one second

        // 60fps x2 = 120 frames in a second; losing the speed, or folding it into
        // the old clip's 30fps, gives 60.
        assert!((t.get_frame_no() - 120.0).abs() < 1.0, "expected ~120 frames, got {}", t.get_frame_no());
    }

    #[test]
    fn the_speed_reads_back_as_it_was_stored() {
        // The getter exists so a host does not have to keep its own copy of the
        // rate — the copy is what drifts. What it returns is the value that was
        // *stored*, so a clamped negative reads back as the stop it became
        // rather than as the reverse playback it was mistaken for.
        let mut t = PlaybackController::new();
        t.setup(0, 4, 30);
        assert_eq!(t.get_speed_scale(), 1.0, "the default is 1x");

        t.set_speed_scale(2.5);
        assert_eq!(t.get_speed_scale(), 2.5);

        t.set_speed_scale(0.0);
        assert_eq!(t.get_speed_scale(), 0.0, "zero is a setting, not 'unset'");

        t.set_speed_scale(-1.0);
        assert_eq!(t.get_speed_scale(), 0.0, "a negative rate clamps to a stop, and reads back as one");
    }

    #[test]
    fn releasing_a_slot_does_reset_the_speed() {
        // reset_setup hands the slot to a different owner, so unlike setup it
        // must not leak the previous one's settings.
        let mut t = PlaybackController::new();
        t.setup(0, 4, 30);
        t.set_speed_scale(2.0);
        t.reset_setup();
        assert_eq!(t.speed_scale, 1.0);
    }

    #[test]
    fn an_fps_override_survives_setup_and_the_authored_rate_does_not() {
        // `set_frame_rate` is the host saying "this player runs at n fps", so it
        // outlives a transition the way the speed multiplier does. The clip's own
        // rate is data and is re-read every time.
        let mut t = PlaybackController::new();
        t.setup(0, 1000, 30);
        assert_eq!(t.get_frame_rate(), 30, "no override yet: the clip's own rate");

        t.set_frame_rate(60);
        assert_eq!(t.get_frame_rate(), 60);

        t.setup(0, 1000, 24); // a different clip, authored at 24
        assert_eq!(t.get_frame_rate(), 60, "the host's rate outlives the transition");

        // ...and clearing it hands playback back to the *new* clip's rate, not
        // the one that was current when the override was set.
        t.set_frame_rate(0);
        assert_eq!(t.get_frame_rate(), 24);
    }

    #[test]
    fn setup_folds_the_override_into_the_update_interval() {
        // Regression: `setup` used to compute the interval from `anime_data_fps`
        // while every other writer used the effective rate. That was invisible
        // while `setup` also cleared the override, and would advance at the
        // authored rate the moment it stopped.
        let mut t = PlaybackController::new();
        t.setup(0, 1000, 30);
        t.set_skip_frames(true);
        t.set_frame_rate(60);

        t.setup(0, 1000, 24); // authored 24, override 60
        t.play();
        t.update(1000.0);

        assert!((t.get_frame_no() - 60.0).abs() < 1.0, "expected ~60 frames (the override), got {} (24 would mean setup ignored it)", t.get_frame_no());
    }

    #[test]
    fn the_override_and_the_speed_multiply() {
        // They compose: 60fps at x0.5 is 30 frames a second, whatever the clip
        // was authored at.
        let mut t = PlaybackController::new();
        t.setup(0, 1000, 24);
        t.set_skip_frames(true);
        t.set_frame_rate(60);
        t.set_speed_scale(0.5);
        t.play();
        t.update(1000.0);

        assert!((t.get_frame_no() - 30.0).abs() < 1.0, "expected ~30 frames, got {}", t.get_frame_no());
    }

    #[test]
    fn releasing_a_slot_clears_the_override_too() {
        let mut t = PlaybackController::new();
        t.setup(0, 4, 30);
        t.set_frame_rate(60);
        t.reset_setup();
        t.setup(0, 4, 30);
        assert_eq!(t.get_frame_rate(), 30, "a recycled slot must not inherit the previous owner's rate");
    }

    #[test]
    fn a_zero_speed_freezes_the_playhead() {
        // It used to normalize to 1.0, so dragging a speed slider to its minimum
        // played at full speed — the opposite of what the value says.
        let mut t = PlaybackController::new();
        t.setup(0, 1000, 30);
        t.set_skip_frames(true);
        t.play();
        t.update(1000.0 / 30.0);
        let held_at = t.get_frame_no();
        assert!(held_at > 0.0, "sanity: it was advancing before the speed changed");

        t.set_speed_scale(0.0);
        for _ in 0..10 {
            t.update(1000.0);
        }
        assert_eq!(t.get_frame_no(), held_at, "a zero speed must not advance the playhead");
        assert!(t.is_playing(), "frozen is not stopped");
        assert!(!t.is_pausing(), "nor is it the pause flag");

        // ...and it is not a one-way door.
        t.set_speed_scale(1.0);
        t.update(1000.0 / 30.0);
        assert!(t.get_frame_no() > held_at, "restoring a positive speed resumes advancing");
    }

    #[test]
    fn a_zero_speed_freezes_fixed_stepping_too() {
        // The other advance path: with frame skip off, `update` accumulates time
        // against the interval instead of dividing by it.
        let mut t = PlaybackController::new();
        t.setup(0, 1000, 30);
        t.set_skip_frames(false);
        t.set_speed_scale(0.0);
        t.play();
        let held_at = t.get_frame_no();
        for _ in 0..10 {
            t.update(1000.0);
        }
        assert_eq!(t.get_frame_no(), held_at);
    }

    #[test]
    fn a_negative_speed_stops_rather_than_running_forward() {
        // Reverse is `set_playback_direction`'s job. Clamping to a stop makes an
        // unsupported request visible; normalizing to 1.0 hid it behind
        // full-speed forward playback.
        let mut t = PlaybackController::new();
        t.setup(0, 1000, 30);
        t.set_skip_frames(true);
        t.set_speed_scale(-1.0);
        t.play();
        let held_at = t.get_frame_no();
        t.update(1000.0);
        assert_eq!(t.get_frame_no(), held_at);
    }
}
