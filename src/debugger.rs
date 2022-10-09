use crate::chip8::QuirksMode;


#[derive(Clone)]
pub struct DebuggerState {
    pub run_speed: f32,
    pub paused: bool,
    pub register_scroll: i32,
    pub quirks: QuirksMode
}

impl Default for DebuggerState {
    fn default() -> Self {
        Self {
            run_speed: 1.0,
            paused: false,
            register_scroll: 0,
            quirks: QuirksMode::default()
        }
    }
}

pub enum DebugInstructions {
    Step,
    Frame,
    Reset,
    Reload(String)
}
