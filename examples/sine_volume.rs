use std::{rc::Rc, sync::atomic::Ordering, thread::sleep, time::Duration};

use ferratone::{
    sink::{Sink, sdl3::SDL3MonoSink},
    source::{
        sine::Sine,
        volume::{AtomicF32, Volume},
    },
};
use sdl3_sys::hints::{SDL_HINT_AUDIO_DEVICE_SAMPLE_FRAMES, SDL_SetHint};

fn main() -> Result<(), String> {
    unsafe { SDL_SetHint(SDL_HINT_AUDIO_DEVICE_SAMPLE_FRAMES, c"2048".as_ptr()) };

    let sine_slider = Rc::new(AtomicF32::new(300.));
    let source = Sine::new(sine_slider.clone());

    let volume_slider = Rc::new(AtomicF32::new(0.));
    let source = Volume::new(Box::new(source), volume_slider.clone());

    let mut sink = SDL3MonoSink::new(Box::new(source))?;
    sink.play()?;

    sleep(Duration::from_millis(500));

    println!("fade in");
    for i in 1..=500 {
        volume_slider.store(i as f32 / 500., Ordering::Relaxed);
        sleep(Duration::from_millis(10));
    }
    println!("done");

    sleep(Duration::from_millis(500));

    println!("pitch up");
    for i in 0..=500 {
        sine_slider.store(300. + i as f32 / 10., std::sync::atomic::Ordering::Relaxed);
        sleep(Duration::from_millis(10));
    }
    println!("done");

    sleep(Duration::from_millis(500));

    println!("fade out");
    for i in (0..=500).rev() {
        volume_slider.store(i as f32 / 500., Ordering::Relaxed);
        sleep(Duration::from_millis(10));
    }
    println!("done");

    Ok(())
}
