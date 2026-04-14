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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_state_soa_resize() {
        let mut state = ParticleStateSoA::default();
        state.resize(5);
        assert_eq!(state.id.len(), 5);
        assert_eq!(state.cycle.len(), 5);
        assert_eq!(state.exist.len(), 5);
        assert_eq!(state.born.len(), 5);
        assert_eq!(state.stime.len(), 5);
        assert_eq!(state.endtime.len(), 5);
    }

    #[test]
    fn test_particle_state_soa_cleanup() {
        let mut state = ParticleStateSoA::default();
        state.resize(2);
        
        // Modify index 1
        state.id[1] = 42;
        state.cycle[1] = 1;
        state.exist[1] = true;
        state.born[1] = true;
        state.stime[1] = 10;
        state.endtime[1] = 20;

        // Cleanup index 1
        state.cleanup(1);

        // Verify index 1 is reset
        assert_eq!(state.id[1], 0);
        assert_eq!(state.cycle[1], 0);
        assert_eq!(state.exist[1], false);
        assert_eq!(state.born[1], false);
        assert_eq!(state.stime[1], 0);
        assert_eq!(state.endtime[1], 0);

        // Ensure index 0 is unaffected
        assert_eq!(state.id[0], 0);
        assert_eq!(state.exist[0], false);
    }

    #[test]
    fn test_emitter_state_new() {
        let state = EmitterState::new(10);
        assert_eq!(state.particle_state.id.len(), 10);
        assert_eq!(state.seed_offset, 0);
    }
}