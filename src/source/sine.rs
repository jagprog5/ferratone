use crate::source::{Source, StereoOut, volume::AtomicF32};
use std::{f32::consts::PI, rc::Rc};

pub struct Sine {
    // rc so it can be tuned while owned by a sink
    pub freq: Rc<AtomicF32>,
    /// Phase accumulator in [0, 1)
    phase: f32,
}

impl Sine {
    pub fn new(freq: Rc<AtomicF32>) -> Self {
        Self { freq, phase: 0.0 }
    }

    fn advance_phase(&mut self, delta: f32) {
        self.phase = (self.phase + delta) % 1.0;
    }
}

impl Source for Sine {
    fn populate_mono(&mut self, _offset: u64, frequency: u32, out: &mut [f32]) {
        let sample_rate = frequency as f32;

        for sample in out.iter_mut() {
            let freq = self.freq.load(std::sync::atomic::Ordering::Relaxed);
            let phase_inc = freq / sample_rate;
            *sample = (2.0 * PI * self.phase).sin();
            self.advance_phase(phase_inc);
        }
    }

    fn populate_stereo(&mut self, _offset: u64, frequency: u32, out: &mut [StereoOut]) {
        let sample_rate = frequency as f32;

        for sample in out.iter_mut() {
            let freq = self.freq.load(std::sync::atomic::Ordering::Relaxed);
            let phase_inc = freq / sample_rate;
            let s = (2.0 * PI * self.phase).sin();
            sample.left = s;
            sample.right = s;
            self.advance_phase(phase_inc);
        }
    }
}
