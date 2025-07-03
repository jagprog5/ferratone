pub mod sine;
pub mod volume;

#[repr(C)] // no need for packed
pub struct StereoOut {
    left: f32,
    right: f32,
}

pub trait Source {
    /// populate the output slice with the samples (normalized from -1 to 1)
    ///
    /// offset indicates the position of the first element in out relative to
    /// the beginning of the track
    ///
    /// sample_rate is the number of samples taken per second. each call to this
    /// instance should have the same sample_rate
    fn populate_mono(&mut self, offset: u64, sample_rate: u32, out: &mut [f32]);

    /// see populate_mono for details
    fn populate_stereo(&mut self, offset: u64, sample_rate: u32, out: &mut [StereoOut]);
}
