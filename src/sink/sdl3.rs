use std::{ffi::CStr, sync::atomic::AtomicU64, time::Duration};

use sdl3_sys::{
    audio::{
        SDL_AUDIO_DEVICE_DEFAULT_PLAYBACK, SDL_AUDIO_F32, SDL_AudioStream, SDL_DestroyAudioStream,
        SDL_OpenAudioDeviceStream, SDL_PauseAudioStreamDevice, SDL_ResumeAudioStreamDevice,
    },
    error::SDL_GetError,
    init::{SDL_INIT_AUDIO, SDL_InitSubSystem, SDL_QuitSubSystem},
};

use crate::sink::Sink;
use crate::source::Source;

pub fn get_error() -> String {
    unsafe {
        let err = SDL_GetError();
        CStr::from_ptr(err).to_str().unwrap().to_owned()
    }
}

/// allocated on heap - userdata pointer must point to fixed data
///
/// guarded by SDL_LockAudioStream - lock required before access not from within
/// callback
struct SDL3MonoSinkInternals {
    sample_offset: AtomicU64,
    source: Box<dyn Source>,
    // const
    sample_rate: ::core::ffi::c_int,
}

/// SDL3 backend for audio sink
pub struct SDL3MonoSink {
    internal: Box<SDL3MonoSinkInternals>,
    stream_handle: *mut SDL_AudioStream,
}

impl SDL3MonoSink {
    pub fn new(source: Box<dyn Source>) -> Result<Self, String> {
        SDL3MonoSink::new_with_freq(source, 48000)
    }

    pub fn new_with_freq(source: Box<dyn Source>, sample_rate: u32) -> Result<Self, String> {
        // internally increments ref count - multiple instances ok
        if !unsafe { SDL_InitSubSystem(SDL_INIT_AUDIO) } {
            return Err(get_error());
        }

        let sample_rate: ::core::ffi::c_int = sample_rate
            .try_into()
            .map_err(|_| "sample_rate out of range for c_int")?;

        let spec = sdl3_sys::audio::SDL_AudioSpec {
            format: SDL_AUDIO_F32,
            channels: 1,
            freq: sample_rate,
        };

        let mut internal = Box::new(SDL3MonoSinkInternals {
            sample_offset: AtomicU64::new(0),
            source,
            sample_rate,
        });

        let internal_ptr = &mut *internal as *mut SDL3MonoSinkInternals as *mut ::core::ffi::c_void;
        let stream_handle = unsafe {
            SDL_OpenAudioDeviceStream(
                SDL_AUDIO_DEVICE_DEFAULT_PLAYBACK,
                &spec,
                Some(Self::callback),
                internal_ptr,
            )
        };

        if stream_handle.is_null() {
            // clean up before exit
            unsafe { SDL_QuitSubSystem(SDL_INIT_AUDIO) };
            return Err(get_error());
        }

        Ok(SDL3MonoSink {
            internal,
            stream_handle,
        })
    }

    extern "C" fn callback(
        userdata: *mut ::core::ffi::c_void,
        stream: *mut SDL_AudioStream,
        _additional_amount: ::core::ffi::c_int,
        total_amount: ::core::ffi::c_int,
    ) {
        // Cast userdata pointer
        let internals = unsafe { &mut *(userdata as *mut SDL3MonoSinkInternals) };

        // fill the largest amount of data possible all the time - helps prevent
        // audio artifacts. and, fewer dyn dispatch calls to source overall
        let mut samples_left = (total_amount as usize) / std::mem::size_of::<f32>();

        const CHUNK_SIZE: usize = 2048;
        let mut buffer = [0f32; CHUNK_SIZE];

        let mut sample_offset = internals
            .sample_offset
            .load(std::sync::atomic::Ordering::Acquire);

        while samples_left > 0 {
            let chunk_len = samples_left.min(CHUNK_SIZE);

            internals.source.populate_mono(
                sample_offset,
                internals.sample_rate as u32,
                &mut buffer[..chunk_len],
            );

            let bytes = (chunk_len * std::mem::size_of::<f32>()) as i32;
            unsafe {
                // ignore error
                sdl3_sys::audio::SDL_PutAudioStreamData(stream, buffer.as_ptr() as *const _, bytes)
            };

            // todo fix. - sample offset logic not right. needs full mutex
            sample_offset = sample_offset.wrapping_add(chunk_len as u64);
            samples_left -= chunk_len;
        }

        internals
            .sample_offset
            .store(sample_offset, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Sink for SDL3MonoSink {
    fn play(&mut self) -> Result<(), String> {
        if unsafe { !SDL_ResumeAudioStreamDevice(self.stream_handle) } {
            return Err(get_error());
        }
        Ok(())
    }

    fn play_at(&mut self, position: Duration) -> Result<(), String> {
        let seconds = position.as_secs() as f64 + position.subsec_nanos() as f64 * 1e-9;
        let sample_offset = (seconds * self.internal.sample_rate as f64).round() as u64;
        self.internal
            .sample_offset
            .store(sample_offset as u64, std::sync::atomic::Ordering::Relaxed);
        self.play()
    }

    fn pause(&mut self) -> Result<(), String> {
        if unsafe { !SDL_PauseAudioStreamDevice(self.stream_handle) } {
            return Err(get_error());
        }
        Ok(())
    }
}

impl Drop for SDL3MonoSink {
    fn drop(&mut self) {
        unsafe {
            SDL_DestroyAudioStream(self.stream_handle);
            SDL_QuitSubSystem(SDL_INIT_AUDIO);
        }
    }
}
