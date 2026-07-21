mod music;

pub use music::{MusicTrackParams, Note};

use wasm_bindgen::JsValue;
use web_sys::{AudioContext, AudioContextState, OscillatorType};

use music::MusicEngine;

#[derive(Clone, Copy, Debug)]
pub enum OscKind {
    Square,
    Triangle,
    Saw,
    Sine,
    Noise,
}

#[derive(Clone, Copy, Debug)]
pub struct SfxParams {
    pub osc: OscKind,
    pub freq_start: f32,
    pub freq_end: f32,
    pub attack_s: f32,
    pub decay_s: f32,
    pub gain: f32,
    pub noise_mix: f32,
}

pub struct Audio {
    ctx: Option<AudioContext>,
    unlocked: bool,
    muted: bool,
    noise_buffer: Option<web_sys::AudioBuffer>,
    music: MusicEngine,
}

impl Audio {
    pub fn new() -> Self {
        Self {
            ctx: None,
            unlocked: false,
            muted: false,
            noise_buffer: None,
            music: MusicEngine::new(),
        }
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
        self.music.set_muted(muted);
    }

    pub fn muted(&self) -> bool {
        self.muted
    }

    fn ensure_ctx(&mut self) -> Result<&AudioContext, JsValue> {
        if self.ctx.is_none() {
            let ctx = AudioContext::new()?;
            self.ctx = Some(ctx);
        }
        Ok(self.ctx.as_ref().unwrap())
    }

    fn ensure_noise(&mut self) -> Result<(), JsValue> {
        if self.noise_buffer.is_some() {
            return Ok(());
        }
        let ctx = self.ensure_ctx()?;
        let sample_rate = ctx.sample_rate();
        let len = (sample_rate * 0.25) as usize;
        let buf = ctx.create_buffer(1, len as u32, sample_rate)?;
        let mut samples = vec![0.0_f32; len];
        for (i, sample) in samples.iter_mut().enumerate() {
            let n = (i as u32)
                .wrapping_mul(1664525)
                .wrapping_add(1013904223);
            *sample = (n as f32 / u32::MAX as f32) * 2.0 - 1.0;
        }
        buf.copy_to_channel(&samples, 0)?;
        self.noise_buffer = Some(buf);
        Ok(())
    }

    pub fn resume(&mut self) {
        let Ok(ctx) = self.ensure_ctx() else {
            return;
        };
        if ctx.state() == AudioContextState::Suspended {
            let _ = ctx.resume();
        }
        self.unlocked = true;
        let _ = self.ensure_noise();
    }

    pub fn beep(&mut self, freq_hz: f32, dur_s: f32) {
        self.play(&SfxParams {
            osc: OscKind::Square,
            freq_start: freq_hz,
            freq_end: freq_hz,
            attack_s: 0.005,
            decay_s: dur_s,
            gain: 0.2,
            noise_mix: 0.0,
        });
    }

    pub fn play(&mut self, p: &SfxParams) {
        if self.muted {
            return;
        }
        self.resume();
        let Ok(_) = self.ensure_noise() else {
            return;
        };
        let Some(ctx) = self.ctx.as_ref() else {
            return;
        };
        let Ok(gain) = ctx.create_gain() else {
            return;
        };
        let now = ctx.current_time();
        let dur = f64::from(p.attack_s + p.decay_s);
        gain.gain().set_value(0.0001);
        let peak = p.gain.max(0.0001);
        let _ = gain
            .gain()
            .linear_ramp_to_value_at_time(peak, now + f64::from(p.attack_s));
        let _ = gain
            .gain()
            .exponential_ramp_to_value_at_time(0.0001, now + dur.max(0.03));

        let use_noise = matches!(p.osc, OscKind::Noise) || p.noise_mix > 0.5;
        if use_noise {
            if let Some(buf) = &self.noise_buffer {
                let Ok(src) = ctx.create_buffer_source() else {
                    return;
                };
                src.set_buffer(Some(buf));
                let _ = src.connect_with_audio_node(&gain);
                let _ = gain.connect_with_audio_node(&ctx.destination());
                let _ = src.start_with_when(now);
                #[allow(deprecated)]
                let _ = src.stop_with_when(now + dur + 0.02);
            }
        } else {
            let Ok(osc) = ctx.create_oscillator() else {
                return;
            };
            osc.set_type(match p.osc {
                OscKind::Square => OscillatorType::Square,
                OscKind::Triangle => OscillatorType::Triangle,
                OscKind::Saw => OscillatorType::Sawtooth,
                OscKind::Sine => OscillatorType::Sine,
                OscKind::Noise => OscillatorType::Square,
            });
            osc.frequency().set_value(p.freq_start);
            let _ = osc
                .frequency()
                .linear_ramp_to_value_at_time(p.freq_end, now + dur);
            let _ = osc.connect_with_audio_node(&gain);
            let _ = gain.connect_with_audio_node(&ctx.destination());
            let _ = osc.start_with_when(now);
            let _ = osc.stop_with_when(now + dur + 0.02);
        }
    }

    pub fn unlocked(&self) -> bool {
        self.unlocked
    }
}

impl Default for Audio {
    fn default() -> Self {
        Self::new()
    }
}
