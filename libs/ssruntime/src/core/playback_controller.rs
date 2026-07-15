use crate::core::math::*;
use alloc::vec::Vec;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum PlaybackStyle {
    Normal = 0,
    PingPong = 1,
}

#[derive(PartialEq, Eq, Copy, Clone)]
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

    (frame, false)
}

pub struct PlaybackController {
    is_pausing: bool,
    is_playing: bool,

    anime_data_start_frame: i32,
    anime_data_end_frame: i32,
    anime_data_fps: i32,

    start_frame: i32,
    end_frame: i32,
    fps: i32,
    fps_rate: f32,
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
    is_looped: bool,

    passed_frames: Vec<i32>,
    needs_evaluation: bool,
}

impl PlaybackController {
    pub fn get_frame_no(&self) -> f32 {
        self.current_frame
    }

    /// `true` while the last frame of a cycle is being shown. With a finite
    /// loop count this is raised once per cycle, not only on the last one, and
    /// with an infinite loop count it is never raised. Use [`is_finished`] to
    /// detect the end of playback.
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

    /// `true` for the single `update` that wrapped the playhead back to the
    /// start frame. Cleared on entry to every `update`, so it must be polled
    /// after each one or the pulse is lost. Never raised on the final cycle of
    /// a finite loop count — that cycle finishes instead of looping.
    pub fn is_looped(&self) -> bool {
        self.is_looped
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

    pub fn get_loops(&self) -> i32 {
        self.loops
    }

    pub fn get_remain_loops(&self) -> i32 {
        self.remain_loops
    }

    pub fn get_start_frame(&self) -> i32 {
        self.start_frame
    }

    pub fn get_end_frame(&self) -> i32 {
        self.end_frame
    }

    pub fn get_fps(&self) -> i32 {
        self.fps
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
            is_pausing: true,
            is_playing: false,

            anime_data_start_frame: 0,
            anime_data_end_frame: 0,
            anime_data_fps: 0,

            start_frame: 0,
            end_frame: 0,
            fps: 0,
            fps_rate: 1.0,

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
            is_looped: false,

            passed_frames: Vec::new(),
            needs_evaluation: false,
        }
    }

    pub fn reset_setup(&mut self) {
        self.reset_state();

        self.is_pausing = true;
        self.is_playing = false;
        self.anime_data_start_frame = 0;
        self.anime_data_end_frame = 0;
        self.anime_data_fps = 0;

        self.start_frame = 0;
        self.end_frame = 0;
        self.fps = 0;
        self.fps_rate = 1.0;

        self.update_interval = 0.0;
        self.passed_frames.clear();
    }

    fn reset_state(&mut self) {
        self.next_frame_time = 0.0;
        self.current_frame = 0.0;
        self.end_frame_reached = false;
        self.is_looped = false;
        self.passed_frames.clear();
        self.needs_evaluation = false;
    }

    pub fn setup(&mut self, start_frame: i32, end_frame: i32, fps: i32) {
        self.reset_state();

        self.is_pausing = true;
        self.is_playing = false;

        self.anime_data_start_frame = start_frame;
        self.anime_data_end_frame = end_frame;
        self.anime_data_fps = fps;
        self.start_frame = self.anime_data_start_frame;
        self.end_frame = self.anime_data_end_frame;
        self.fps = self.anime_data_fps;
        self.fps_rate = 1.0;

        self.current_frame = start_frame as f32;
        // Resets `remain_loops` too: a stale count from a previous animation
        // would otherwise raise `end_frame_reached` at the end of this one's
        // first cycle.
        self.set_loops(-1);

        self.is_skip_frames = true;
        self.playback_style = PlaybackStyle::Normal;
        self.playback_direction = PlaybackDirection::Forward;
        self.playback_forward = 1;
        self.update_interval = 1000.0 / (self.anime_data_fps as f32 * self.fps_rate);
        self.passed_frames.clear();
        self.needs_evaluation = false;
    }

    pub fn set_skip_frames(&mut self, skip: bool) {
        self.is_skip_frames = skip;
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
    pub fn set_loops(&mut self, loops: i32) {
        self.loops = if loops > 0 { loops } else { -1 };
        self.remain_loops = self.loops;
    }

    pub fn set_frame_rate(&mut self, fps: i32) {
        self.fps = if fps > 0 { fps } else { self.anime_data_fps };
        self.update_interval = 1000.0 / (self.fps as f32 * self.fps_rate);
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

    pub fn set_animation_speed(&mut self, speed_rate: f32) {
        self.fps_rate = if speed_rate > 0.0 { speed_rate } else { 1.0 };
        self.update_interval = 1000.0 / (self.fps as f32 * self.fps_rate);
    }

    pub fn set_animation_section(&mut self, start_frame: i32, end_frame: i32) {
        self.start_frame = if start_frame >= 0 && start_frame < self.anime_data_end_frame { start_frame } else { self.anime_data_start_frame };
        self.end_frame = if end_frame >= 0 && end_frame < self.anime_data_end_frame { end_frame } else { self.anime_data_end_frame };

        self.current_frame = if self.playback_direction == PlaybackDirection::Forward { self.start_frame as f32 } else { self.end_frame as f32 }
    }

    pub fn update(&mut self, delta: f32) -> f32 {
        self.passed_frames.clear();
        self.is_looped = false;

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

                let next: f32 = self.current_frame + s;
                let next_frame_no = floorf(next) as i32;
                let mut next_frame_decimal = next - next_frame_no as f32;
                let mut current_frame_no = floorf(self.current_frame) as i32;

                let v = (next_frame_no - current_frame_no).abs();
                for _ in 0..v {
                    let mut frame_no = current_frame_no + self.playback_forward;

                    let (start_limit, end_limit) = if self.playback_forward > 0 { (self.start_frame, self.end_frame) } else { (self.end_frame, self.start_frame) };

                    if frame_no == end_limit && self.remain_loops != -1 {
                        match &self.playback_style {
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
                        match &self.playback_style {
                            PlaybackStyle::Normal => {
                                if self.remain_loops == -1 {
                                    // infinite loop
                                    frame_no = start_limit;
                                    self.is_looped = true;
                                } else {
                                    self.remain_loops -= 1;
                                    if self.remain_loops == 0 {
                                        self.is_playing = false;
                                        self.end_frame_reached = true;
                                        // stop playing the animation
                                        frame_no = end_limit;
                                        current_frame_no = frame_no;
                                        next_frame_decimal = 0.0;
                                        self.passed_frames.push(frame_no);
                                        break;
                                    } else {
                                        // continue to play the animation
                                        frame_no = start_limit;
                                        self.is_looped = true;
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
                                            // stop playing the animation
                                            frame_no = if self.playback_direction == PlaybackDirection::Forward { self.start_frame } else { self.end_frame };
                                            current_frame_no = frame_no;
                                            next_frame_decimal = 0.0;
                                            self.passed_frames.push(frame_no);
                                            break;
                                        } else {
                                            self.is_looped = true;
                                        }
                                    } else {
                                        // infinite loop
                                        self.is_looped = true;
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
        self.is_looped = false;

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

    pub fn pause(&mut self) {
        self.is_pausing = !self.is_pausing;
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
    }

    pub fn set_frame_no(&mut self, frame: f32) {
        self.current_frame = frame;
        self.needs_evaluation = true;
    }

    /// Sets `current_frame` to the position reached after `diff` frames have
    /// elapsed since playback started, with the configured `start_frame` /
    /// `end_frame` / `loops` / `playback_style` / `playback_direction`
    /// applied. `diff` should already incorporate per-instance `speed`
    /// (i.e. `diff = (parent_frame - event_frame) * speed`).
    ///
    /// Unlike `update(delta)` this is a deterministic, non-incremental
    /// computation — the same `diff` always produces the same frame, which is
    /// what allows parent-driven seek / reverse / frame-skip on Instance and
    /// Effect parts to render correctly.
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
        t.set_loops(-1);
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
            if t.is_looped() {
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
    fn set_loops_3_emits_looped_looped_finished() {
        let mut t = PlaybackController::new();
        t.setup(0, 4, 30);
        t.set_loops(3);
        t.set_skip_frames(false);
        t.play();

        let kinds: Vec<_> = collect_events(&mut t, 30).into_iter().map(|(_, k)| k).collect();
        assert_eq!(kinds, ["looped", "looped", "finished"]);
        assert_eq!(t.get_frame_no(), 4.0, "Should rest on the end frame");
        assert!(!t.is_playing());
    }

    #[test]
    fn infinite_loop_emits_looped_only_and_never_finishes() {
        let mut t = PlaybackController::new();
        t.setup(0, 4, 30);
        t.set_loops(-1);
        t.set_skip_frames(false);
        t.play();

        // 30 steps over an inclusive 5-frame cycle wraps 6 times.
        let kinds: Vec<_> = collect_events(&mut t, 30).into_iter().map(|(_, k)| k).collect();
        assert_eq!(kinds, ["looped"; 6]);
        assert!(!t.is_finished());
        assert!(t.is_playing());
    }

    #[test]
    fn set_loops_0_is_infinite() {
        let mut t = PlaybackController::new();
        t.setup(0, 4, 30);
        t.set_loops(0);
        assert_eq!(t.get_loops(), -1, "0 normalizes to the infinite sentinel");
        assert_eq!(t.get_remain_loops(), -1);
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
        t.set_loops(3);
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
        t.set_loops(3);
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
        t.set_loops(1);
        t.set_skip_frames(false);
        t.play();
        for _ in 0..6 {
            t.update(1000.0 / 30.0);
        }
        assert!(t.is_finished());
        assert_eq!(t.get_remain_loops(), 0);

        // A stale remain_loops == 0 would raise end_frame_reached at the end of
        // the next animation's first cycle even though it loops forever.
        t.setup(0, 4, 30);
        assert_eq!(t.get_loops(), -1);
        assert_eq!(t.get_remain_loops(), -1, "remain_loops must not leak across setup()");
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
        t.set_loops(2);
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

    #[test]
    fn test_normal_playback_no_loop() {
        let mut t = PlaybackController::new();
        t.setup(0, 10, 30);
        t.set_loops(1);
        t.set_skip_frames(false);
        t.play();

        let interval = 1000.0 / 30.0;

        // Frames 0 to 9
        for i in 1..=10 {
            let frame = t.update(interval);
            assert_eq!(frame, i as f32, "Failed at step {} (Forward)", i);
            assert!(t.is_playing());
        }

        // At frame 10, end_frame_reached should be true
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
        t.set_loops(2);
        t.set_skip_frames(false);
        t.play();

        let interval = 1000.0 / 30.0;

        // First loop: 0 -> 10
        for _ in 1..=10 {
            t.update(interval);
        }
        assert!(t.is_end_frame_reached(), "end_frame_reached should be true after first loop");
        assert_eq!(t.get_frame_no(), 10.0);
        assert_eq!(t.get_remain_loops(), 2);

        // Transition to second loop (shows frame 0)
        let frame = t.update(interval);
        assert_eq!(frame, 0.0, "Should jump back to start frame");
        assert_eq!(t.get_remain_loops(), 1);
        assert!(!t.is_end_frame_reached(), "end_frame_reached should reset at start of new loop");

        // Second loop: 0 -> 10
        for _ in 1..=10 {
            t.update(interval);
        }
        assert!(t.is_end_frame_reached());

        // Final update stops
        t.update(interval);
        assert!(!t.is_playing(), "Should stop playing after 2 loops");
        assert_eq!(t.get_remain_loops(), 0);
    }

    #[test]
    fn test_pingpong_playback() {
        let mut t = PlaybackController::new();
        t.setup(0, 10, 30);
        t.set_playback_direction(PlaybackDirection::Forward, PlaybackStyle::PingPong);
        t.set_loops(1);
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

        // Back at 0, now end_frame_reached should be true
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
        t.set_loops(-1);
        t.play();

        let interval = 1000.0 / 30.0;

        for _ in 0..100 {
            t.update(interval);
        }

        assert!(t.is_playing(), "Infinite loop should never stop");
        assert_eq!(t.get_remain_loops(), -1);
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

        // In Fixed mode, it should ONLY advance by 1 frame, no matter how large delta is.
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
        t.set_loops(1);
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
        t.set_loops(2);
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
        t.set_loops(1);
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
        t.set_loops(2);
        t.set_skip_frames(true);
        t.play();

        // Advance by 11.5 frames.
        // 0.0 -> 10.0 (end frame) -> 0.0 (start of next loop) -> 0.5 (decimal)
        let delta = (1000.0 / 30.0) * 11.5;

        let frame = t.update(delta);

        assert_eq!(floorf(frame), 0.0, "Should have looped back to start frame");
        let decimal = frame - floorf(frame);
        assert!(decimal > 0.4 && decimal < 0.6, "Should have 0.5 decimal after loop, got {}", decimal);
        assert_eq!(t.get_remain_loops(), 1, "Should have 1 loop remaining");
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
        t.set_loops(2);
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
        t.set_loops(-1); // Infinite
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
        t.set_loops(-1); // Infinite
        t.set_playback_direction(PlaybackDirection::Forward, PlaybackStyle::Normal);
        t.set_skip_frames(false);
        t.play();

        let interval = 1000.0 / 30.0;

        // 0..4
        for _ in 0..5 {
            t.update(interval);
            assert!(!t.is_looped(), "Should not be looped yet");
        }

        // Frame 5 -> 0 (Loop)
        t.update(interval);
        assert!(t.is_looped(), "Should be looped at boundary");
        assert!(!t.is_end_frame_reached(), "Should not reach end frame in infinite loop");

        // Next frame, flag should reset
        t.update(interval);
        assert!(!t.is_looped(), "is_looped should reset on next update");
    }

    #[test]
    fn test_is_looped_flag_pingpong() {
        let mut t = PlaybackController::new();
        t.setup(0, 5, 30); // 0 to 5
        t.set_loops(2); // 2 loops
        t.set_playback_direction(PlaybackDirection::Forward, PlaybackStyle::PingPong);
        t.set_skip_frames(false);
        t.play();

        let interval = 1000.0 / 30.0;

        // Forward 0..4
        for _ in 0..5 {
            t.update(interval);
            assert!(!t.is_looped(), "Should not be looped during forward");
        }

        // Frame 5 -> Start backward
        t.update(interval);
        // In ping-pong, hitting the far end doesn't count as a "loop" completion until we return to origin
        assert!(!t.is_looped(), "Should not loop at far end of pingpong");

        // Backward 4..1
        for _ in 0..4 {
            t.update(interval);
            assert!(!t.is_looped(), "Should not be looped during backward");
        }

        // Frame 0 -> 1 (Loop 1 complete)
        t.update(interval);
        assert!(t.is_looped(), "Should be looped when returning to origin in pingpong");
        assert!(!t.is_end_frame_reached());

        // Forward 1..4 again
        for _ in 0..4 {
            t.update(interval);
            assert!(!t.is_looped());
        }

        // Frame 5 -> Start backward again
        t.update(interval);
        assert!(!t.is_looped());

        // Backward 4..1
        for _ in 0..4 {
            t.update(interval);
            assert!(!t.is_looped());
        }

        // Frame 0 -> End
        t.update(interval);
        assert!(!t.is_looped(), "Should not loop on final end");
        assert!(t.is_end_frame_reached(), "Should reach end frame on last loop");
    }
}
