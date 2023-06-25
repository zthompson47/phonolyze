mod file;
mod player;

pub use file::AudioFile;
pub use player::AudioPlayer;

enum Sample<S> {
    Silence,
    Signal(S),
    SetChannels(usize),
}

pub enum CopyMethod {
    Interleaved,
    Planar,
}

#[derive(Debug)]
pub struct PlaybackPosition {
    pub instant: instant::Instant,
    pub music_position: f64,
    pub music_length: f64,
}

impl Default for PlaybackPosition {
    fn default() -> Self {
        Self {
            instant: instant::Instant::now(),
            music_position: 0.0,
            music_length: 0.0,
        }
    }
}
