//! Clean-room multi-track audio bus routing and spectral bioacoustic inspector (PA-05).
//!
//! Provides multi-track gain/pan bus mixing, peak/RMS metering, clipping detection,
//! and Radix-2 Cooley-Tukey FFT Hann-windowed STFT spectrogram inspection.

/// Audio bus specification for routing and mastering.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioBusSpec {
    pub bus_id: u32,
    pub name: String,
    pub channels: u16,
    pub gain: f32,
    pub pan: f32,
}

/// A track within the spectral audio session.
#[derive(Debug, Clone, PartialEq)]
pub struct SpectralTrack {
    pub track_id: u32,
    pub name: String,
    pub sample_rate_hz: u32,
    pub channels: u16,
    pub gain: f32,
    pub pan: f32,
    pub samples: Vec<f32>,
    pub muted: bool,
    pub solo: bool,
}

/// Output frame of STFT bioacoustic spectral analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct SpectrogramFrame {
    pub timestamp_seconds: f64,
    pub bin_step_hz: f32,
    pub magnitudes_db: Vec<f32>,
    pub dominant_freq_hz: f32,
    pub dominant_magnitude_db: f32,
}

/// Metering metrics for an audio bus.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BusMeterMetrics {
    pub peak_linear: f32,
    pub rms_linear: f32,
    pub is_clipping: bool,
}

/// Output of a bus mixdown operation.
#[derive(Debug, Clone, PartialEq)]
pub struct MixdownOutput {
    pub channels: u16,
    pub frame_count: usize,
    pub samples: Vec<f32>,
    pub metrics: BusMeterMetrics,
}

/// Errors occurring during audio session or spectral processing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpectralSessionError {
    InvalidSampleRate { rate: u32 },
    SampleRateMismatch { session_rate: u32, track_rate: u32 },
    InvalidChannelCount { channels: u16 },
    InvalidFftSize { size: usize },
    InvalidHopSize { hop_size: usize },
    EmptySamples,
    BufferTooSmall { required: usize, provided: usize },
    TrackNotFound { track_id: u32 },
}

impl std::fmt::Display for SpectralSessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSampleRate { rate } => write!(f, "invalid sample rate: {rate} Hz (must be > 0)"),
            Self::SampleRateMismatch { session_rate, track_rate } => {
                write!(f, "sample rate mismatch: session {session_rate} Hz, track {track_rate} Hz")
            }
            Self::InvalidChannelCount { channels } => {
                write!(f, "invalid channels: {channels} (must be in 1..=16)")
            }
            Self::InvalidFftSize { size } => {
                write!(f, "invalid FFT size: {size} (must be power of two >= 16)")
            }
            Self::InvalidHopSize { hop_size } => write!(f, "invalid hop size: {hop_size}"),
            Self::EmptySamples => write!(f, "sample buffer is empty"),
            Self::BufferTooSmall { required, provided } => {
                write!(f, "buffer too small: required {required}, provided {provided}")
            }
            Self::TrackNotFound { track_id } => write!(f, "track {track_id} not found"),
        }
    }
}
impl std::error::Error for SpectralSessionError {}

#[derive(Debug, Clone, Copy)]
struct Complex {
    re: f32,
    im: f32,
}

impl Complex {
    #[inline]
    fn new(re: f32, im: f32) -> Self { Self { re, im } }
    #[inline]
    fn add(self, o: Self) -> Self { Self::new(self.re + o.re, self.im + o.im) }
    #[inline]
    fn sub(self, o: Self) -> Self { Self::new(self.re - o.re, self.im - o.im) }
    #[inline]
    fn mul(self, o: Self) -> Self {
        Self::new(self.re * o.re - self.im * o.im, self.re * o.im + self.im * o.re)
    }
}

fn cooley_tukey_radix2_fft(buf: &mut [Complex]) {
    let n = buf.len();
    assert!(n.is_power_of_two());
    let mut j = 0;
    for i in 0..(n - 1) {
        if i < j { buf.swap(i, j); }
        let mut k = n >> 1;
        while k <= j { j -= k; k >>= 1; }
        j += k;
    }
    let mut len = 2;
    while len <= n {
        let half = len >> 1;
        let angle = -std::f32::consts::TAU / len as f32;
        let w_step = Complex::new(angle.cos(), angle.sin());
        for i in (0..n).step_by(len) {
            let mut w = Complex::new(1.0, 0.0);
            for k in 0..half {
                let u = buf[i + k];
                let v = buf[i + k + half].mul(w);
                buf[i + k] = u.add(v);
                buf[i + k + half] = u.sub(v);
                w = w.mul(w_step);
            }
        }
        len <<= 1;
    }
}

/// Multi-track audio session engine with bus mixing and spectral analysis.
#[derive(Debug, Clone)]
pub struct SpectralSessionEngine {
    pub sample_rate_hz: u32,
    pub master_bus: AudioBusSpec,
    pub tracks: Vec<SpectralTrack>,
}

impl SpectralSessionEngine {
    pub fn new(sample_rate_hz: u32, bus_channels: u16) -> Result<Self, SpectralSessionError> {
        if sample_rate_hz == 0 {
            return Err(SpectralSessionError::InvalidSampleRate { rate: 0 });
        }
        if bus_channels == 0 || bus_channels > 16 {
            return Err(SpectralSessionError::InvalidChannelCount { channels: bus_channels });
        }
        Ok(Self {
            sample_rate_hz,
            master_bus: AudioBusSpec {
                bus_id: 0,
                name: "Master".to_string(),
                channels: bus_channels,
                gain: 1.0,
                pan: 0.0,
            },
            tracks: Vec::new(),
        })
    }

    pub fn add_track(&mut self, track: SpectralTrack) -> Result<(), SpectralSessionError> {
        if track.sample_rate_hz != self.sample_rate_hz {
            return Err(SpectralSessionError::SampleRateMismatch {
                session_rate: self.sample_rate_hz,
                track_rate: track.sample_rate_hz,
            });
        }
        if track.channels == 0 || track.channels > 16 {
            return Err(SpectralSessionError::InvalidChannelCount { channels: track.channels });
        }
        self.tracks.push(track);
        Ok(())
    }

    /// Mixes down all active tracks into the master bus with gain and pan.
    pub fn mix_down(&self, max_frames: usize) -> Result<MixdownOutput, SpectralSessionError> {
        let bus_ch = self.master_bus.channels as usize;
        let mut total_frames = 0;
        for t in &self.tracks {
            let ch = t.channels.max(1) as usize;
            let frames = t.samples.len() / ch;
            if frames > total_frames { total_frames = frames; }
        }
        let render_frames = if max_frames > 0 { total_frames.min(max_frames) } else { total_frames };
        let mut mix_buf = vec![0.0f32; render_frames * bus_ch];

        let has_solo = self.tracks.iter().any(|t| t.solo && !t.muted);

        for track in &self.tracks {
            if track.muted || (has_solo && !track.solo) {
                continue;
            }
            let tch = track.channels as usize;
            let track_frames = (track.samples.len() / tch).min(render_frames);

            let pan_norm = ((track.pan.clamp(-1.0, 1.0) + 1.0) * 0.5).clamp(0.0, 1.0);
            let (l_gain, r_gain) = if bus_ch >= 2 {
                let angle = pan_norm * std::f32::consts::FRAC_PI_2;
                (angle.cos() * track.gain, angle.sin() * track.gain)
            } else {
                (track.gain, track.gain)
            };

            for f in 0..track_frames {
                let mono_sample = if tch == 1 {
                    track.samples[f]
                } else {
                    let mut s = 0.0f32;
                    for c in 0..tch { s += track.samples[f * tch + c]; }
                    s / tch as f32
                };

                let out_idx = f * bus_ch;
                if bus_ch >= 2 {
                    mix_buf[out_idx] += mono_sample * l_gain * self.master_bus.gain;
                    mix_buf[out_idx + 1] += mono_sample * r_gain * self.master_bus.gain;
                } else {
                    mix_buf[out_idx] += mono_sample * track.gain * self.master_bus.gain;
                }
            }
        }

        let mut peak = 0.0f32;
        let mut sum_sq = 0.0f32;
        for &s in &mix_buf {
            let abs_s = s.abs();
            if abs_s > peak { peak = abs_s; }
            sum_sq += s * s;
        }
        let rms = if !mix_buf.is_empty() { (sum_sq / mix_buf.len() as f32).sqrt() } else { 0.0 };

        Ok(MixdownOutput {
            channels: self.master_bus.channels,
            frame_count: render_frames,
            samples: mix_buf,
            metrics: BusMeterMetrics {
                peak_linear: peak,
                rms_linear: rms,
                is_clipping: peak > 1.0,
            },
        })
    }

    /// Computes STFT spectrogram frames with Hann window and extracts dominant frequencies.
    pub fn analyze_spectrogram(
        &self,
        samples: &[f32],
        fft_size: usize,
        hop_size: usize,
    ) -> Result<Vec<SpectrogramFrame>, SpectralSessionError> {
        if samples.is_empty() {
            return Err(SpectralSessionError::EmptySamples);
        }
        if fft_size < 16 || !fft_size.is_power_of_two() {
            return Err(SpectralSessionError::InvalidFftSize { size: fft_size });
        }
        if hop_size == 0 {
            return Err(SpectralSessionError::InvalidHopSize { hop_size });
        }

        let num_bins = fft_size / 2 + 1;
        let bin_step = self.sample_rate_hz as f32 / fft_size as f32;

        let mut window = Vec::with_capacity(fft_size);
        for i in 0..fft_size {
            let w = 0.5 * (1.0 - (std::f32::consts::TAU * i as f32 / (fft_size as f32 - 1.0)).cos());
            window.push(w);
        }

        let mut frames = Vec::new();
        let mut offset = 0;
        let mut fft_buf = vec![Complex::new(0.0, 0.0); fft_size];

        while offset + fft_size <= samples.len() {
            for i in 0..fft_size {
                fft_buf[i] = Complex::new(samples[offset + i] * window[i], 0.0);
            }
            cooley_tukey_radix2_fft(&mut fft_buf);

            let mut mags_db = Vec::with_capacity(num_bins);
            let mut dom_bin = 1;
            let mut dom_mag = -120.0f32;

            for k in 0..num_bins {
                let mag = (fft_buf[k].re * fft_buf[k].re + fft_buf[k].im * fft_buf[k].im).sqrt()
                    / (fft_size as f32 * 0.5);
                let db = 20.0 * (mag.max(1e-7)).log10();
                mags_db.push(db);

                if k > 0 && db > dom_mag {
                    dom_mag = db;
                    dom_bin = k;
                }
            }

            let dom_freq = dom_bin as f32 * bin_step;
            frames.push(SpectrogramFrame {
                timestamp_seconds: offset as f64 / self.sample_rate_hz as f64,
                bin_step_hz: bin_step,
                magnitudes_db: mags_db,
                dominant_freq_hz: dom_freq,
                dominant_magnitude_db: dom_mag,
            });

            offset += hop_size;
        }

        Ok(frames)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fft_sine_dominant_frequency() {
        let sr = 44100u32;
        let engine = SpectralSessionEngine::new(sr, 2).expect("engine");
        let target_freq = 440.0f32;
        let num_samples = 4096;
        let mut sine = Vec::with_capacity(num_samples);
        for i in 0..num_samples {
            let t = i as f32 / sr as f32;
            sine.push((std::f32::consts::TAU * target_freq * t).sin());
        }

        let frames = engine.analyze_spectrogram(&sine, 1024, 512).expect("spectrogram");
        assert!(!frames.is_empty());
        let dom = frames[0].dominant_freq_hz;
        assert!((dom - target_freq).abs() <= frames[0].bin_step_hz * 1.5);
    }

    #[test]
    fn test_bus_mixing_and_clipping() {
        let sr = 48000u32;
        let mut engine = SpectralSessionEngine::new(sr, 2).expect("engine");

        let t1 = SpectralTrack {
            track_id: 1,
            name: "Lead".into(),
            sample_rate_hz: sr,
            channels: 1,
            gain: 0.8,
            pan: -0.5,
            samples: vec![0.9; 480],
            muted: false,
            solo: false,
        };

        let t2 = SpectralTrack {
            track_id: 2,
            name: "Harmony".into(),
            sample_rate_hz: sr,
            channels: 1,
            gain: 0.8,
            pan: 0.5,
            samples: vec![0.9; 480],
            muted: false,
            solo: false,
        };

        engine.add_track(t1).expect("add t1");
        engine.add_track(t2).expect("add t2");

        let mix = engine.mix_down(480).expect("mixdown");
        assert_eq!(mix.channels, 2);
        assert_eq!(mix.frame_count, 480);
        assert!(mix.metrics.peak_linear > 0.5);
        assert!(mix.metrics.rms_linear > 0.0);
    }

    #[test]
    fn test_negative_cases() {
        // Sample rate 0
        let err1 = SpectralSessionEngine::new(0, 2);
        assert!(matches!(err1, Err(SpectralSessionError::InvalidSampleRate { rate: 0 })));

        // Channel count 0 or > 16
        let err2 = SpectralSessionEngine::new(48000, 0);
        assert!(matches!(err2, Err(SpectralSessionError::InvalidChannelCount { channels: 0 })));
        let err3 = SpectralSessionEngine::new(48000, 17);
        assert!(matches!(err3, Err(SpectralSessionError::InvalidChannelCount { channels: 17 })));

        // Non-power-of-two FFT size
        let engine = SpectralSessionEngine::new(44100, 2).expect("engine");
        let err4 = engine.analyze_spectrogram(&[0.1; 1000], 500, 250);
        assert!(matches!(err4, Err(SpectralSessionError::InvalidFftSize { size: 500 })));

        // Track sample rate mismatch
        let mut eng = SpectralSessionEngine::new(48000, 2).expect("engine");
        let err5 = eng.add_track(SpectralTrack {
            track_id: 1,
            name: "BadSR".into(),
            sample_rate_hz: 44100,
            channels: 1,
            gain: 1.0,
            pan: 0.0,
            samples: vec![0.0; 100],
            muted: false,
            solo: false,
        });
        assert!(matches!(err5, Err(SpectralSessionError::SampleRateMismatch { .. })));
    }
}
