pub const SEED_MAGIC: i32 = 7573;
pub const LIFE_EXTEND_SCALE: i32 = 8;
pub const LIFE_EXTEND_MIN: i32 = 64;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Color {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(a: u8, r: u8, g: u8, b: u8) -> Self {
        Self { a, r, g, b }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EmitterParameter {
    pub life: i32,
    pub interval: i32,
    pub emitnum: i32,
    pub emitmax: i32,
    pub particle_life: i32,
    pub particle_life2: i32,
    pub is_infinite: bool,
    pub loop_start: i32,
    pub loop_end: i32,
    pub loop_len: i32,
    pub loop_gen: i32,
}

impl Default for EmitterParameter {
    fn default() -> Self {
        Self {
            life: 15,
            interval: 1,
            emitnum: 2,
            emitmax: 32,
            particle_life: 15,
            particle_life2: 15,
            is_infinite: false,
            loop_start: 0,
            loop_end: 0,
            loop_len: 0,
            loop_gen: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParticleParameter {
    pub scale: Vector2,
    pub startcolor: Color,
    pub endcolor: Color,
    pub speed: f32,
    pub speed2: f32,
    pub angle: f32,
    pub angle_variance: f32,
    pub use_gravity: bool,
    pub gravity: Vector2,
    pub use_offset: bool,
    pub offset: Vector2,
    pub offset2: Vector2,
    pub use_rotation: bool,
    pub rotation: f32,
    pub rotation2: f32,
    pub rotation_add: f32,
    pub rotation_add2: f32,
    pub use_rotation_trans: bool,
    pub rotation_factor: f32,
    pub end_life_time_per: f32,
    pub use_tan_accel: bool,
    pub tangential_accel: f32,
    pub tangential_accel2: f32,
    pub use_color: bool,
    pub init_color: Color,
    pub init_color2: Color,
    pub use_trans_color: bool,
    pub trans_color: Color,
    pub trans_color2: Color,
    pub use_init_scale: bool,
    pub scale_range: Vector2,
    pub scale_factor: f32,
    pub scale_factor2: f32,
    pub use_trans_scale: bool,
    pub transscale: Vector2,
    pub scale_range2: Vector2,
    pub transscale_factor: f32,
    pub transscale_factor2: f32,
    pub delay: f32,
    pub use_p_gravity: bool,
    pub gravity_pos: Vector2,
    pub gravity_power: f32,
    pub use_alpha_fade: bool,
    pub alpha_fade: f32,
    pub alpha_fade2: f32,
    pub use_trans_speed: bool,
    pub trans_speed: f32,
    pub trans_speed2: f32,
    pub use_turn_direc: bool,
    pub direc_rot_add: f32,
    pub user_override_r_seed: bool,
    pub override_r_seed: i32,
}

impl Default for ParticleParameter {
    fn default() -> Self {
        Self {
            scale: Vector2::new(1.0, 1.0),
            startcolor: Color::new(255, 255, 255, 255),
            endcolor: Color::new(255, 255, 255, 255),
            speed: 0.0,
            speed2: 0.0,
            angle: 0.0,
            angle_variance: 0.0,
            use_gravity: false,
            gravity: Vector2::default(),
            use_offset: false,
            offset: Vector2::default(),
            offset2: Vector2::default(),
            use_rotation: false,
            rotation: 0.0,
            rotation2: 0.0,
            rotation_add: 0.0,
            rotation_add2: 0.0,
            use_rotation_trans: false,
            rotation_factor: 0.0,
            end_life_time_per: 0.0,
            use_tan_accel: false,
            tangential_accel: 0.0,
            tangential_accel2: 0.0,
            use_color: false,
            init_color: Color::default(),
            init_color2: Color::default(),
            use_trans_color: false,
            trans_color: Color::default(),
            trans_color2: Color::default(),
            use_init_scale: false,
            scale_range: Vector2::default(),
            scale_factor: 1.0,
            scale_factor2: 0.0,
            use_trans_scale: false,
            transscale: Vector2::new(1.0, 1.0),
            scale_range2: Vector2::default(),
            transscale_factor: 1.0,
            transscale_factor2: 0.0,
            delay: 0.0,
            use_p_gravity: false,
            gravity_pos: Vector2::default(),
            gravity_power: 0.0,
            use_alpha_fade: false,
            alpha_fade: 0.0,
            alpha_fade2: 0.0,
            use_trans_speed: false,
            trans_speed: 0.0,
            trans_speed2: 0.0,
            use_turn_direc: false,
            direc_rot_add: 0.0,
            user_override_r_seed: false,
            override_r_seed: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct EmitPattern {
    pub uid: i32,
    pub life: i32,
    pub cycle: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleDrawState {
    pub x: f32,
    pub y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation: f32,
    pub direction: f32,
    pub color: Color,
}

impl Default for ParticleDrawState {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
            direction: 0.0,
            color: Color::new(255, 255, 255, 255),
        }
    }
}
