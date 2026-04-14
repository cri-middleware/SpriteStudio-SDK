use super::types::*;

#[derive(Clone, Debug, Default)]
pub struct ParticleStateSoA {
    pub id: Vec<i32>,
    pub cycle: Vec<i32>,
    pub exist: Vec<bool>,
    pub born: Vec<bool>,
    pub stime: Vec<i32>,
    pub endtime: Vec<i32>,
}

impl ParticleStateSoA {
    pub fn resize(&mut self, size: usize) {
        self.id.resize(size, 0);
        self.cycle.resize(size, 0);
        self.exist.resize(size, false);
        self.born.resize(size, false);
        self.stime.resize(size, 0);
        self.endtime.resize(size, 0);
    }

    pub fn cleanup(&mut self, index: usize) {
        self.id[index] = 0;
        self.cycle[index] = 0;
        self.exist[index] = false;
        self.born[index] = false;
        self.stime[index] = 0;
        self.endtime[index] = 0;
    }
}

#[derive(Clone, Debug, Default)]
pub struct EmitterState {
    pub seed_offset: i32,
    pub position: Vector2,
    pub particle_state: ParticleStateSoA,
}

impl EmitterState {
    pub fn new(emitmax: i32) -> Self {
        let mut state = Self::default();
        state.particle_state.resize(emitmax as usize);
        state
    }
}

#[derive(Clone, Debug, Default)]
pub struct RenderState {
    pub current_frame: f32,
    pub target_frame: f32,
    pub seed_offset: i32,
    pub is_playing: bool,
    pub is_pausing: bool,
    pub is_loop: bool,
    pub emitter_states: Vec<EmitterState>,
}
