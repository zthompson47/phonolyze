#![allow(unused)]
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{mpsc, Arc, Mutex},
    time::{Duration, Instant},
};

use anyhow::{Error, Result};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, SizedSample,
};
use num::traits::Zero;
use symphonia::{
    core::{
        audio::SampleBuffer,
        codecs::{Decoder, DecoderOptions},
        errors::Error::DecodeError,
        formats::{FormatOptions, FormatReader},
        meta::MetadataOptions,
        probe::Hint,
    },
    default::{get_codecs, get_probe},
};

use crate::{file, Cli};

pub struct AudioFile {
    format: Box<dyn FormatReader>,
    pub decoder: Box<dyn Decoder>,
    default_track_id: u32,
}

impl AudioFile {
    pub fn info(&self) -> HashMap<String, String> {
        [
            ("sample_rate".to_string(), self.sample_rate().to_string()),
            ("channels".to_string(), self.channels().to_string()),
        ]
        .into()
    }

    pub fn sample_rate(&self) -> u32 {
        self.decoder.codec_params().sample_rate.unwrap()
    }

    pub fn channels(&self) -> usize {
        self.decoder.codec_params().channels.unwrap().count()
    }

    pub async fn open(path: &str) -> Result<Self> {
        let source = file::load_sound(path).await?;
        let hint = Hint::new();
        let format_opts: FormatOptions = Default::default();
        let metadata_opts: MetadataOptions = Default::default();
        let decoder_opts: DecoderOptions = Default::default();
        let format = get_probe()
            .format(&hint, source, &format_opts, &metadata_opts)
            .unwrap()
            .format;
        let track = format
            .default_track()
            .ok_or_else(|| Error::msg("No default track."))?;
        let decoder = get_codecs().make(&track.codec_params, &decoder_opts)?;
        let default_track_id = track.id;

        Ok(AudioFile {
            format,
            decoder,
            default_track_id,
        })
    }

    pub fn next_sample(&mut self, meth: CopyMethod) -> Result<Option<SampleBuffer<f32>>> {
        let packet = self.format.next_packet()?;
        if packet.track_id() != self.default_track_id {
            return Ok(None);
        }
        match self.decoder.decode(&packet) {
            Ok(audio_buf_ref) => {
                let spec = *audio_buf_ref.spec();
                let duration = audio_buf_ref.capacity() as u64;
                let mut buf = SampleBuffer::new(duration, spec);
                if let CopyMethod::Interleaved = meth {
                    buf.copy_interleaved_ref(audio_buf_ref);
                } else if let CopyMethod::Planar = meth {
                    buf.copy_planar_ref(audio_buf_ref);
                }
                Ok(Some(buf))
            }
            Err(DecodeError(_)) => Ok(None),
            Err(_) => Err(Error::msg("Decode error.")),
        }
    }

    pub fn dump(&mut self) -> (Vec<f32>, Vec<f32>) {
        let mut left = Vec::new();
        let mut right = Vec::new();
        while let Ok(buf) = self.next_sample(CopyMethod::Planar) {
            if let Some(buf) = buf {
                let s = buf.samples();
                left.append(&mut Vec::from(&s[..s.len() / 2]));
                right.append(&mut Vec::from(&s[s.len() / 2..]));
            }
        }
        (left, right)
    }

    pub fn dump_mono(&mut self, seconds: Option<f32>) -> Vec<f32> {
        let mut result = Vec::new();
        let sample_limit = seconds.map(|s| (s * self.sample_rate() as f32).round() as u32);
        let mut count = 0u32;

        // Just grab the left channel.
        while let Ok(buf) = self.next_sample(CopyMethod::Planar) {
            if let Some(buf) = buf {
                let mut samples = Vec::from(buf.samples());
                let len = samples.len();
                result.append(&mut Vec::from(&mut samples[0..len]));
                if let Some(limit) = sample_limit {
                    count += buf.len() as u32;
                    if count > limit {
                        break;
                    }
                }
            }
        }

        result
    }

    pub fn write_wav(&self, filename: &str, signal: &[f32], sample_rate: u32) {
        let mut writer = hound::WavWriter::create(
            Path::new(&filename),
            hound::WavSpec {
                channels: 1,
                sample_rate,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            },
        )
        .unwrap();

        signal.iter().step_by(16).for_each(|x| {
            let amplitude = std::i16::MAX as f32;
            writer.write_sample((x * amplitude) as i16).unwrap();
        });
        writer.finalize().unwrap();
    }
}

pub enum CopyMethod {
    Interleaved,
    Planar,
}

pub struct AudioPlayer {
    tx_play_song: mpsc::Sender<PathBuf>,
    //pub sample_count: Arc<Mutex<u32>>,
    pub progress: Arc<Mutex<PlaybackPosition>>,
    _stream: cpal::Stream,
}

#[derive(Debug)]
pub struct PlaybackPosition {
    pub instant: Instant,
    pub music_position: f64,
}

impl Default for PlaybackPosition {
    fn default() -> Self {
        Self {
            instant: Instant::now(),
            music_position: 0.0,
        }
    }
}

enum SampleData<S> {
    Silence,
    Signal(S),
}

impl AudioPlayer {
    pub async fn new<S>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        latency_ms: f32,
        chunk_size: usize,
    ) -> Result<Self>
    where
        S: SizedSample + FromSample<f32> + Zero + Send + 'static,
    {
        //let latency_ms = 0f32;

        let sample_rate = config.sample_rate.0 as f32;
        let channels = config.channels as u32;
        let latency_frames = (latency_ms * sample_rate / 1000.).round() as u32;
        let latency_samples = (latency_frames * channels) as usize;
        let (mut txrb_audio, mut rxrb_audio) = rtrb::RingBuffer::<S>::new(latency_samples * 2);

        for _ in 0..latency_samples {
            txrb_audio.push(S::zero());
        }

        let (tx_play_song, rx_play_song) = mpsc::channel::<PathBuf>();
        //let (tx_progress, rx_progress) = mpsc::channel::<u32>();
        //let sample_count = Arc::new(Mutex::new(0u32));
        //let sample_count_clone = sample_count.clone();

        std::thread::spawn(move || {
            pollster::block_on(async {
                while let Ok(song) = rx_play_song.recv() {
                    let mut audio = AudioFile::open(song.to_str().unwrap()).await.unwrap();

                    loop {
                        match audio.next_sample(CopyMethod::Interleaved) {
                            Ok(Some(signal)) => {
                                let samples = signal.samples();

                                //*sample_count_clone.lock().unwrap() += samples.len() as u32;

                                //let mut progress = progress_clone.lock().unwrap();
                                //*progress += samples.len() as u32;
                                //dbg!(*progress);
                                //drop(progress);

                                for sample in samples {
                                    loop {
                                        if txrb_audio.push(S::from_sample(*sample)).is_ok() {
                                            break;
                                        }
                                        //log::info!("sleep: {}", latency_ms);
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
                let instant = Instant::now()
                    + timestamp
                        .playback
                        .duration_since(&timestamp.callback)
                        .unwrap_or_else(|| Duration::from_secs(0));

                let new_sample_count = data.len() / channels as usize;
                let start_time = sample_count as f64 / sample_rate as f64;
                let end_time = (sample_count as f64 + new_sample_count as f64) / sample_rate as f64;

                if let Ok(mut pos) = progress_clone.lock() {
                    pos.instant = instant;
                    pos.music_position = start_time;
                }

                //let time_playback = info.timestamp().playback;
                //let time_callback = info.timestamp().callback;
                //dbg!(time_callback, time_playback);

                //let mut progress = progress_clone.lock().unwrap();
                //*progress += data.len() as u32;
                //dbg!(*progress);
                //drop(progress);

                //*sample_count_clone.lock().unwrap() += new_sample_count as u32;
                sample_count += new_sample_count as u32;

                for sample in data.chunks_mut(channels as usize) {
                    if let Ok(chunk) = rxrb_audio.read_chunk(2) {
                        let mut chunk = chunk.into_iter();
                        sample[0] = chunk.next().unwrap();
                        sample[1] = chunk.next().unwrap();
                    } else {
                        input_fell_behind = true;
                        sample[0] = S::zero();
                        sample[1] = S::zero();
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
