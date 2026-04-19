#[derive(Default, Clone, Debug)]
pub struct InstancePartState {
    pub accumulated_time: f32,  // Accumulated time for independent playback
    pub last_update_frame: i32, // Last updated parent frame to prevent multiple updates
}
