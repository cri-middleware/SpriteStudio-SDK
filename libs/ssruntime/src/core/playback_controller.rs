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
}

impl PlaybackController {
    pub fn get_frame_no(&self) -> i32 {
        libm::floorf(self.current_frame) as i32
    }

    pub fn get_frame_no_decimal(&self) -> f32 {
        self.current_frame - libm::floorf(self.current_frame)
    }

    pub fn is_end_frame_reached(&self) -> bool {
        self.end_frame_reached
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
    }

    fn reset_state(&mut self) {
        self.next_frame_time = 0.0;
        self.current_frame = 0.0;
        self.end_frame_reached = false;
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
        self.loops = -1;

        self.is_skip_frames = true;
        self.playback_style = PlaybackStyle::Normal;
        self.playback_direction = PlaybackDirection::Forward;
        self.playback_forward = 1;
        self.update_interval = 1000.0 / (self.anime_data_fps as f32 * self.fps_rate);
    }

    pub fn set_skip_frames(&mut self, skip: bool) {
        self.is_skip_frames = skip;
    }

    /**
     * Set the number of loops.
     * -1: infinite loop
     * 0: no loop
     * n: loop n times
     */
    pub fn set_loops(&mut self, loops: i32) {
        self.loops = if loops >= 0 {loops} else {-1};
        self.remain_loops = loops;
    }

    pub fn set_frame_rate(&mut self, fps: i32) {
        self.fps = if fps > 0 {fps} else {self.anime_data_fps};
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
        self.fps_rate = if speed_rate > 0.0 {speed_rate} else {1.0};
        self.update_interval = 1000.0 / (self.fps as f32 * self.fps_rate);
    }

    pub fn set_animation_section(&mut self, start_frame: i32, end_frame: i32) {
        self.start_frame = if start_frame >= 0 && start_frame < self.anime_data_end_frame {start_frame} else {self.anime_data_start_frame};
        self.end_frame = if end_frame >= 0 && end_frame < self.anime_data_end_frame {end_frame} else {self.anime_data_end_frame};

        self.current_frame = if self.playback_direction == PlaybackDirection::Forward {self.start_frame as f32} else {self.end_frame as f32}
    }

    pub fn update(&mut self, delta: f32) -> (i32, f32) {
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
                let next_frame_no = libm::floorf(next) as i32;
                let mut next_frame_decimal = next - next_frame_no as f32;
                let mut current_frame_no = libm::floorf(self.current_frame) as i32;

                let v = (next_frame_no - current_frame_no).abs();
                for _ in 0 .. v {
                    let mut frame_no = current_frame_no + self.playback_forward;

                    let (start_limit, end_limit) = if self.playback_forward > 0 {
                        (self.start_frame, self.end_frame)
                    } else {
                        (self.end_frame, self.start_frame)
                    };

                    if frame_no == end_limit
                        && self.remain_loops != -1 {
                            match &self.playback_style {
                                PlaybackStyle::Normal => {
                                    self.end_frame_reached = true;
                                },
                                PlaybackStyle::PingPong => {
                                    let is_back_to_origin = match &self.playback_direction {
                                        PlaybackDirection::Forward => self.playback_forward == -1,
                                        PlaybackDirection::Backward => self.playback_forward == 1,
                                    };
                                    if is_back_to_origin {
                                        self.end_frame_reached = true;
                                    }
                                },
                            }
                        }

                    let is_reached = if self.playback_forward > 0 {
                        frame_no > end_limit
                    } else {
                        frame_no < end_limit
                    };

                    if is_reached {
                        match &self.playback_style {
                            PlaybackStyle::Normal => {
                                if self.remain_loops == -1 {
                                    // infinite loop
                                    frame_no = start_limit;
                                } else {
                                    self.remain_loops -= 1;
                                    if self.remain_loops == 0 {
                                        self.is_playing = false;
                                        self.end_frame_reached = true;
                                        // stop playing the animation
                                        frame_no = end_limit;
                                        current_frame_no = frame_no;
                                        next_frame_decimal = 0.0;
                                        break;
                                    } else {
                                        // continue to play the animation
                                        frame_no = start_limit;
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
                                            break;
                                        }
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
                }
                if self.is_playing {
                    self.current_frame = current_frame_no as f32 + next_frame_decimal;
                } else {
                    self.current_frame = current_frame_no as f32;
                }
            }

            (libm::floorf(self.current_frame) as i32, self.current_frame - libm::floorf(self.current_frame))
        } else {
            (libm::floorf(self.current_frame) as i32, self.current_frame - libm::floorf(self.current_frame))
        }
    }
}

impl PlaybackController {
    // playback control
    pub fn play(&mut self) {
        self.play_with_start_frame(-1)
    }

    pub fn play_with_start_frame(&mut self, frame_no: i32) {
        self.is_playing = true;
        self.is_pausing = false;

        let mut current_frame = match &self.playback_direction {
            PlaybackDirection::Forward => self.start_frame,
            PlaybackDirection::Backward => self.end_frame,
        };
        if frame_no >= 0 {
            current_frame = frame_no;
        }
        self.current_frame = current_frame as f32;
    }

    pub fn pause(&mut self) {
        self.is_pausing = !self.is_pausing;
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
    }

    pub fn set_frame_no(&mut self, frame: i32) {
        self.current_frame = frame as f32;
    }

    /**
     * Set the current frame to the next frame.
     * The function ignores direction and style.
     */
    pub fn next_frame(&mut self) {
        let current_frame = libm::floorf(self.current_frame) as i32;
        if current_frame == self.end_frame {
            return;
        }
        self.set_frame_no(current_frame + 1);
    }

    /**
     * Set the current frame to the previous frame.
     * The function ignores direction and style.
     */
    pub fn prev_frame(&mut self) {
        let current_frame = libm::floorf(self.current_frame) as i32;
        if current_frame == self.start_frame {
            return;
        }
        self.set_frame_no(current_frame - 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            let (frame, _) = t.update(interval);
            assert_eq!(frame, i, "Failed at step {} (Forward)", i);
            assert!(t.is_playing());
        }

        // At frame 10, end_frame_reached should be true
        assert!(t.is_end_frame_reached(), "end_frame_reached should be true at frame 10");

        // Next update should stop playback
        let (frame, _) = t.update(interval);
        assert_eq!(frame, 10, "Should remain at end frame when stopped");
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
        for _ in 1..=10 { t.update(interval); }
        assert!(t.is_end_frame_reached(), "end_frame_reached should be true after first loop");
        assert_eq!(t.get_frame_no(), 10);
        assert_eq!(t.get_remain_loops(), 2);

        // Transition to second loop (shows frame 0)
        let (frame, _) = t.update(interval);
        assert_eq!(frame, 0, "Should jump back to start frame");
        assert_eq!(t.get_remain_loops(), 1);
        assert!(!t.is_end_frame_reached(), "end_frame_reached should reset at start of new loop");

        // Second loop: 0 -> 10
        for _ in 1..=10 { t.update(interval); }
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
            let (frame, _) = t.update(interval);
            assert_eq!(frame, i, "Forward failed at step {}", i);
        }
        
        // At 10, it should NOT have reached end yet in PingPong (need to return to 0)
        assert!(!t.is_end_frame_reached(), "end_frame_reached should not be true at far end in PingPong");

        // Backward: 10 -> 0
        for i in (0..=9).rev() {
            let (frame, _) = t.update(interval);
            assert_eq!(frame, i, "Backward failed at step {}", i);
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
        
        let (frame, decimal) = t.update(delta);
        assert_eq!(frame, 0);
        assert!(decimal > 0.4 && decimal < 0.6, "Expected around 0.5, got {}", decimal);

        let (frame, decimal) = t.update(delta);
        assert_eq!(frame, 1);
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
        
        let (frame, _) = t.update(delta);
        
        // In Fixed mode, it should ONLY advance by 1 frame, no matter how large delta is.
        assert_eq!(frame, 1, "Fixed animation should only advance by 1 frame at a time");
        
        // The remaining time (500 - 33.33) should be stored in next_frame_time.
        // Calling update with 0 delta should continue to advance frames until the buffer is empty.
        let (frame, _) = t.update(0.0);
        assert_eq!(frame, 2, "Should use buffered time to advance even with 0 delta");
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
            let (frame, _) = t.update(interval);
            assert_eq!(frame, i, "Reverse failed at step {}", i);
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
        for _ in 0..10 { t.update(interval); }
        assert!(t.is_end_frame_reached());
        assert_eq!(t.get_frame_no(), 0);

        // Transition to second loop (jumps to 10)
        let (frame, _) = t.update(interval);
        assert_eq!(frame, 10);
        assert!(!t.is_end_frame_reached());

        // Second loop: 10 -> 0
        for _ in 0..10 { t.update(interval); }
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
            let (frame, _) = t.update(interval);
            assert_eq!(frame, i, "Initial backward failed at step {}", i);
        }
        
        assert!(!t.is_end_frame_reached());

        // Then Forward: 0 -> 10
        for i in 1..=10 {
            let (frame, _) = t.update(interval);
            assert_eq!(frame, i, "Return forward failed at step {}", i);
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
        
        let (frame, decimal) = t.update(delta);
        assert_eq!(frame, 2, "Should jump to frame 2");
        assert!(decimal > 0.4 && decimal < 0.6, "Decimal should be around 0.5, got {}", decimal);

        // Update another 1.0 frame
        let (frame, decimal) = t.update(1000.0 / 30.0);
        assert_eq!(frame, 3);
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
        
        let (frame, decimal) = t.update(delta);
        
        assert_eq!(frame, 0, "Should have looped back to start frame");
        assert!(decimal > 0.4 && decimal < 0.6, "Should have 0.5 decimal after loop, got {}", decimal);
        assert_eq!(t.get_remain_loops(), 1, "Should have 1 loop remaining");
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
        
        let (frame, decimal) = t.update(delta);
        assert_eq!(frame, 8, "Should have bounced back to frame 8");
        assert!((0.0..0.001).contains(&decimal), "Decimal should be exactly 0.0, got {}", decimal);
    }
}
