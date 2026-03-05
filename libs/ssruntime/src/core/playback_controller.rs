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
        self.current_frame.floor() as i32
    }

    pub fn get_frame_no_decimal(&self) -> f32 {
        self.current_frame - self.current_frame.floor()
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

    pub fn get_skip_frames(&mut self) -> bool {
        self.is_skip_frames
    }

    pub fn get_playback_direction(&self) -> PlaybackDirection {
        self.playback_direction
    }

    pub fn get_playback_style(&self) -> PlaybackStyle {
        self.playback_style
    }

    pub fn get_loops(&mut self) -> i32 {
        self.loops
    }

    pub fn get_remain_loops(&mut self) -> i32 {
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
        self.loops = if loops > 0 {loops} else {-1};
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
            self.next_frame_time += delta;
            if self.next_frame_time >= self.update_interval {
                self.end_frame_reached = false;

                // skip frames
                let step = self.next_frame_time / self.update_interval;
                self.next_frame_time -= self.update_interval * step;
                let s = if self.is_skip_frames { step * self.playback_forward as f32 } else { self.playback_forward as f32 };
                let next: f32 = self.current_frame + s;
                let next_frame_no = next.floor() as i32;
                let mut next_frame_decimal = next - next_frame_no as f32;
                let mut current_frame_no = self.current_frame.floor() as i32;

                let v = (next_frame_no - current_frame_no).abs();
                for _ in 0 .. v {
                    let mut frame_no = current_frame_no + self.playback_forward;

                    let (start_limit, end_limit) = if self.playback_forward > 0 {
                        (self.start_frame, self.end_frame)
                    } else {
                        (self.end_frame, self.start_frame)
                    };

                    if frame_no == end_limit {
                        if self.remain_loops != -1 {
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
                                }
                            }
                        }
                    }
                    current_frame_no = frame_no;
                }
                self.current_frame = current_frame_no as f32 + next_frame_decimal;
            }

            (self.current_frame.floor() as i32, self.current_frame - self.current_frame.floor())
        } else {
            (self.current_frame.floor() as i32, self.current_frame - self.current_frame.floor())
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
        let current_frame = self.current_frame.floor() as i32;
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
        let current_frame = self.current_frame.floor() as i32;
        if current_frame == self.start_frame {
            return;
        }
        self.set_frame_no(current_frame - 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn play(t: &mut PlaybackController, delta: f32, loop_num: i32) {
        println!("INIT frame_no: {}, frame_no_decimal: {:}, remain_loops: {}, is_playing: {}, end_frame_reached: {}", t.get_frame_no(), t.get_frame_no_decimal(), t.get_remain_loops(), t.is_playing(), t.is_end_frame_reached());
        for _ in 0..loop_num {
            let (frame_no, decimal) = t.update(delta);
            println!("frame_no: {}, frame_no_decimal: {:}, remain_loops: {}, is_playing: {}, end_frame_reached: {}", frame_no, decimal, t.get_remain_loops(), t.is_playing(), t.is_end_frame_reached());
        }
    }

    #[test]
    fn infinitive_loops() {
        let t = &mut PlaybackController::new();
        t.setup(0, 99, 30);
        t.play();
        play(t, 16.7, 500);
    }

    #[test]
    fn infinitive_loops_reverse() {
        let t = &mut PlaybackController::new();
        t.setup(0, 99, 30);
        t.set_playback_direction(PlaybackDirection::Backward, PlaybackStyle::Normal);
        t.play();
        play(t, 16.7, 500);
    }

    #[test]
    fn no_loops() {
        let t = &mut PlaybackController::new();
        t.setup(0, 99, 30);
        t.set_loops(1);
        t.play();
        play(t, 16.7, 500);
    }

    #[test]
    fn no_loops_reverse() {
        let t = &mut PlaybackController::new();
        t.setup(0, 99, 30);
        t.set_playback_direction(PlaybackDirection::Backward, PlaybackStyle::Normal);
        t.set_loops(1);
        t.play();
        play(t, 16.7, 500);
    }

    #[test]
    fn loops() {
        let t = &mut PlaybackController::new();
        t.setup(0, 99, 30);
        t.set_loops(3);
        t.play();
        play(t, 16.7, 500);
    }

    #[test]
    fn loops_reverse() {
        let t = &mut PlaybackController::new();
        t.setup(0, 99, 30);
        t.set_playback_direction(PlaybackDirection::Backward, PlaybackStyle::Normal);
        t.set_loops(3);
        t.play();
        play(t, 16.7, 500);
    }


    #[test]
    fn infinitive_loops_pingpong() {
        let t = &mut PlaybackController::new();
        t.setup(0, 99, 30);
        t.set_playback_direction(PlaybackDirection::Forward, PlaybackStyle::PingPong);
        t.play();
        play(t, 16.7, 700);
    }

    #[test]
    fn infinitive_loops_pingpong_reverse() {
        let t = &mut PlaybackController::new();
        t.setup(0, 99, 30);
        t.set_playback_direction(PlaybackDirection::Backward, PlaybackStyle::PingPong);
        t.play();
        play(t, 16.7, 700);
    }

    #[test]
    fn loops_pingpong() {
        let t = &mut PlaybackController::new();
        t.setup(0, 99, 30);
        t.set_playback_direction(PlaybackDirection::Forward, PlaybackStyle::PingPong);
        t.set_loops(3);
        t.play();
        play(t, 16.7, 700);
    }

    #[test]
    fn loops_pingpong_reverse() {
        let t = &mut PlaybackController::new();
        t.setup(0, 99, 30);
        t.set_playback_direction(PlaybackDirection::Backward, PlaybackStyle::PingPong);
        t.set_loops(3);
        t.play();
        play(t, 16.7, 700);
    }

    #[test]
    fn no_loops_pingpong() {
        let t = &mut PlaybackController::new();
        t.setup(0, 99, 30);
        t.set_playback_direction(PlaybackDirection::Forward, PlaybackStyle::PingPong);
        t.set_loops(1);
        t.play();
        play(t, 16.7, 700);
    }

    #[test]
    fn no_loops_pingpong_reverse() {
        let t = &mut PlaybackController::new();
        t.setup(0, 99, 30);
        t.set_playback_direction(PlaybackDirection::Backward, PlaybackStyle::PingPong);
        t.set_loops(1);
        t.play();
        play(t, 16.7, 700);
    }
}
