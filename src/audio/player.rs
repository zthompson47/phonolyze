use std::{
    cell::Cell,
    path::PathBuf,
    sync::{mpsc, Arc, Mutex},
};

use anyhow::{Error, Result};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, SizedSample,
};
use num::traits::Zero;

use crate::Cli;

use super::{AudioFile, CopyMethod, PlaybackPosition, Sample};

pub struct AudioPlayer {
    tx_play_song: mpsc::Sender<PathBuf>,
    pub progress: Arc<Mutex<PlaybackPosition>>,
    _stream: cpal::Stream,
}

impl std::fmt::Debug for AudioPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioPlayer")
            .field("tx_play_song", &self.tx_play_song)
            .field("progress", &self.progress)
            .field("_stream", &"n/a")
            .finish()
    }
}

impl AudioPlayer {
    pub async fn new<S>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        latency_ms: f32,
        _chunk_size: usize,
    ) -> Result<Self>
    where
        S: SizedSample + FromSample<f32> + Zero + Send + 'static,
    {
        let sample_rate = config.sample_rate.0 as f32;
        let channels = config.channels;
        let latency_frames = (latency_ms * sample_rate / 1000.).round() as u32;
        let latency_samples = (latency_frames * channels as u32) as usize;
        let (mut txrb_audio, mut rxrb_audio) =
            rtrb::RingBuffer::<Sample<S>>::new(latency_samples * 2);
        let (tx_play_song, rx_play_song) = mpsc::channel::<PathBuf>();
        //let (tx_stop_song, rx_stop_song) = mpsc::sync_channel::<()>(1);
        dbg!(sample_rate, channels, latency_frames, latency_samples);

        let audio_channels: Cell<usize> = Cell::new(channels as usize);
        for _ in 0..latency_samples {
            txrb_audio.push(Sample::Silence).unwrap();
        }

        std::thread::spawn(move || {
            pollster::block_on(async {
                while let Ok(song) = rx_play_song.recv() {
                    let mut audio = AudioFile::open(song.to_str().unwrap()).await.unwrap();
                    txrb_audio
                        .push(Sample::SetChannels(audio.channels()))
                        .unwrap();
                    loop {
                        match audio.next_sample(CopyMethod::Interleaved) {
                            Ok(Some(signal)) => {
                                let samples = signal.samples();

                                for sample in samples {
                                    loop {
                                        if txrb_audio
                                            .push(Sample::Signal(S::from_sample(*sample)))
                                            .is_ok()
                                        {
                                            break;
                                        }
                                        std::thread::sleep(instant::Duration::from_millis(
                                            latency_ms as u64 / 2,
                                        ));
                                    }
                                }
                            }
                            Ok(None) => break,
                            Err(e) => {
                                log::error!("{e:?}");
                                break;
                            }
                        }
                    }
                }
            });
        });

        let mut sample_count = 0u32;
        let progress = Arc::new(Mutex::new(PlaybackPosition::default()));
        let progress_clone = progress.clone();

        let _stream = device.build_output_stream(
            config,
            move |data: &mut [S], info: &cpal::OutputCallbackInfo| {
                let mut input_fell_behind = false;
                let timestamp = info.timestamp();
                let instant = instant::Instant::now()
                    + timestamp
                        .playback
                        .duration_since(&timestamp.callback)
                        .unwrap_or_else(|| instant::Duration::from_secs(0));
                let new_sample_count = data.len() / audio_channels.get();
                let samples_per_channel = sample_count as f64 / audio_channels.get() as f64;
                let start_time = samples_per_channel / sample_rate as f64;
                let _end_time =
                    (samples_per_channel + new_sample_count as f64) / sample_rate as f64;

                if let Ok(mut pos) = progress_clone.lock() {
                    pos.instant = instant;
                    pos.music_position = start_time;
                }

                let mut next_sample = |s: &Sample<S>| -> Option<S> {
                    match s {
                        Sample::Silence => Some(S::zero()),
                        Sample::Signal(s) => {
                            sample_count += 1;
                            Some(*s)
                        }
                        Sample::SetChannels(num) => {
                            audio_channels.set(*num);
                            None
                        }
                    }
                };

                fn silence<S: Zero>(samples: &mut [S]) {
                    for sample in samples.iter_mut() {
                        *sample = S::zero();
                    }
                }

                for samples in data.chunks_mut(channels as usize) {
                    if let Ok(audio_sample) = rxrb_audio.pop() {
                        let mut next = next_sample(&audio_sample);
                        while next.is_none() {
                            if let Ok(audio_sample) = rxrb_audio.pop() {
                                next = next_sample(&audio_sample);
                            } else {
                                input_fell_behind = true;
                                silence(samples);
                                continue;
                            }
                        }

                        let next = next.unwrap();

                        for (i, sample) in samples.iter_mut().enumerate() {
                            if i == 0 || (audio_channels.get() == 1 && i == 1) {
                                *sample = next;
                            } else if audio_channels.get() > i {
                                *sample = next_sample(&rxrb_audio.pop().unwrap()).unwrap();
                            } else {
                                *sample = S::zero();
                            }
                        }
                    } else {
                        input_fell_behind = true;
                        silence(samples);
                    }
                }

                if input_fell_behind {
                    //log::warn!("input fell behind");
                }
            },
            move |err| {
                log::error!("{err}");
            },
            None,
        )?;

        _stream.play()?;

        Ok(AudioPlayer {
            _stream,
            tx_play_song,
            progress,
            //sample_count,
        })
    }

    pub fn play(&self, song: std::path::PathBuf) {
        self.tx_play_song.send(song).unwrap();
    }
}

impl From<&Cli> for AudioPlayer {
    fn from(cli: &Cli) -> Self {
        let device = cpal::default_host()
            .default_output_device()
            .ok_or(Error::msg("No audio device found"))
            .unwrap();
        let config = device.default_output_config().unwrap();
        let audio_player = {
            match config.sample_format() {
                cpal::SampleFormat::I8 => pollster::block_on(AudioPlayer::new::<i8>(
                    &device,
                    &config.into(),
                    cli.latency_ms,
                    cli.chunk_size,
                )),
                cpal::SampleFormat::F32 => pollster::block_on(AudioPlayer::new::<f32>(
                    &device,
                    &config.into(),
                    cli.latency_ms,
                    cli.chunk_size,
                )),
                _ => panic!("unsupported format"),
            }
            .unwrap()
        };

        if cli.play_audio {
            audio_player.play(cli.audio_file.clone().into());
        };

        audio_player
    }
}
