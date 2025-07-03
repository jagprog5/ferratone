// enable a backend
#[cfg(feature = "sdl3")]
pub mod sdl3;

use std::time::Duration;

pub trait Sink {
    /// non blocking. play from previous position, or beginning if there is no
    /// previous position. if already playing, doesn't have an effect
    fn play(&mut self) -> Result<(), String>;

    /// non blocking. play at position (relative to epoch)
    fn play_at(&mut self, position: Duration) -> Result<(), String>;

    /// non blocking. stop playback at current position. if already paused,
    /// doesn't have an effect
    fn pause(&mut self) -> Result<(), String>;
}
