use std::{
    rc::Rc,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::source::Source;

/// https://github.com/rust-lang/rust/issues/72353#issuecomment-1093729062
pub struct AtomicF32 {
    storage: AtomicU32,
}
impl AtomicF32 {
    pub fn new(value: f32) -> Self {
        let as_u32 = value.to_bits();
        Self {
            storage: AtomicU32::new(as_u32),
        }
    }
    pub fn store(&self, value: f32, ordering: Ordering) {
        let as_u32 = value.to_bits();
        self.storage.store(as_u32, ordering)
    }
    pub fn load(&self, ordering: Ordering) -> f32 {
        let as_u32 = self.storage.load(ordering);
        f32::from_bits(as_u32)
    }
}

pub struct Volume {
    pub source: Box<dyn Source>,
    // rc so it can be tuned while owned by a sink
    pub volume: Rc<AtomicF32>,
}

impl Volume {
    pub fn new(source: Box<dyn Source>, volume: Rc<AtomicF32>) -> Self {
        Self { source, volume }
    }
}

impl Source for Volume {
    fn populate_mono(&mut self, offset: u64, sample_rate: u32, out: &mut [f32]) {
        self.source.populate_mono(offset, sample_rate, out);
        let volume = self.volume.load(Ordering::Relaxed);
        // samples are normalized in range from -1 to 1
        out.iter_mut().for_each(|v| *v *= volume);
    }

    fn populate_stereo(&mut self, offset: u64, sample_rate: u32, out: &mut [super::StereoOut]) {
        self.source.populate_stereo(offset, sample_rate, out);
        let volume = self.volume.load(Ordering::Relaxed);
        out.iter_mut().for_each(|v| {
            v.left *= volume;
            v.right *= volume;
        });
    }
}
