//! Chiptune pattern sequencer: 2 pulse + triangle + noise, WebAudio lookahead.

use web_sys::{AudioContext, GainNode, OscillatorType};

use super::Audio;

const LOOKAHEAD_S: f64 = 0.25;
const MUSIC_GAIN: f32 = 0.35;

#[derive(Clone, Copy, Debug)]
pub struct Note {
    /// Beat position within the loop.
    pub start: f32,
    pub len: f32,
    pub midi: u8,
    pub vol: f32,
}

#[derive(Clone, Debug)]
pub struct MusicTrackParams {
    pub bpm: f32,
    pub loop_len: f32,
    pub channels: [Vec<Note>; 4],
}

struct ActiveMusic {
    params: MusicTrackParams,
    /// AudioContext time of absolute beat 0.
    epoch: f64,
    /// Next absolute beat cursor (grows forever).
    next_beat: f64,
}

pub(super) struct MusicEngine {
    gain: Option<GainNode>,
    active: Option<ActiveMusic>,
    muted: bool,
}

impl MusicEngine {
    pub fn new() -> Self {
        Self {
            gain: None,
            active: None,
            muted: false,
        }
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
        if let Some(g) = &self.gain {
            g.gain().set_value(if muted { 0.0001 } else { MUSIC_GAIN });
        }
    }

    fn ensure_gain(&mut self, ctx: &AudioContext) {
        if self.gain.is_none() {
            if let Ok(g) = ctx.create_gain() {
                let level = if self.muted { 0.0001 } else { MUSIC_GAIN };
                g.gain().set_value(level);
                let _ = g.connect_with_audio_node(&ctx.destination());
                self.gain = Some(g);
            }
        }
    }

    pub fn stop(&mut self) {
        self.active = None;
    }

    pub fn set_track(&mut self, ctx: &AudioContext, track: Option<MusicTrackParams>) {
        let Some(track) = track else {
            self.stop();
            return;
        };
        self.ensure_gain(ctx);
        let now = ctx.current_time();
        if let Some(g) = &self.gain {
            let _ = g.gain().cancel_scheduled_values(now);
            let peak = if self.muted { 0.0001 } else { MUSIC_GAIN };
            g.gain().set_value(0.0001);
            let _ = g.gain().linear_ramp_to_value_at_time(peak, now + 0.2);
        }
        self.active = Some(ActiveMusic {
            params: track,
            epoch: now + 0.05,
            next_beat: 0.0,
        });
    }

    pub fn tick(&mut self, ctx: &AudioContext) {
        if self.muted {
            return;
        }
        self.ensure_gain(ctx);
        let Some(gain) = self.gain.clone() else {
            return;
        };
        let now = ctx.current_time();
        let horizon = now + LOOKAHEAD_S;
        let Some(active) = self.active.as_mut() else {
            return;
        };

        let bpm = active.params.bpm.max(1.0);
        let beat_s = 60.0 / f64::from(bpm);
        let loop_len = f64::from(active.params.loop_len.max(1.0));
        let epoch = active.epoch;

        let mut guard = 0u32;
        while epoch + active.next_beat * beat_s < horizon {
            let abs_beat = active.next_beat;
            let loop_index = (abs_beat / loop_len).floor();
            let beat_in_loop = abs_beat - loop_index * loop_len;
            let window_end = beat_in_loop + 1.0;

            for ch in 0..4 {
                for &n in &active.params.channels[ch] {
                    let s = f64::from(n.start);
                    let hit = if window_end <= loop_len + 0.001 {
                        s >= beat_in_loop && s < window_end
                    } else {
                        s >= beat_in_loop || s < (window_end - loop_len)
                    };
                    if !hit || n.midi == 0 {
                        continue;
                    }
                    let note_loop = if s >= beat_in_loop {
                        loop_index
                    } else {
                        loop_index + 1.0
                    };
                    let when = epoch + (note_loop * loop_len + s) * beat_s;
                    if when < now - 0.02 {
                        continue;
                    }
                    schedule_voice(ctx, &gain, ch, n, when, beat_s);
                }
            }

            active.next_beat += 1.0;
            guard += 1;
            if guard > 64 {
                break;
            }
        }
    }
}

fn schedule_voice(
    ctx: &AudioContext,
    bus: &GainNode,
    channel: usize,
    n: Note,
    when: f64,
    beat_s: f64,
) {
    let dur = (f64::from(n.len) * beat_s).max(0.04);
    let peak = (n.vol * 0.22).clamp(0.01, 0.35);
    let Ok(gain) = ctx.create_gain() else {
        return;
    };
    gain.gain().set_value(0.0001);
    let _ = gain
        .gain()
        .linear_ramp_to_value_at_time(peak, when + 0.008);
    let _ = gain
        .gain()
        .exponential_ramp_to_value_at_time(0.0001, when + dur);
    let _ = gain.connect_with_audio_node(bus);

    let Ok(osc) = ctx.create_oscillator() else {
        return;
    };
    if channel == 3 {
        osc.set_type(OscillatorType::Square);
        osc.frequency()
            .set_value(90.0 + f32::from(n.midi.saturating_mul(3)));
        let _ = osc.connect_with_audio_node(&gain);
        let _ = osc.start_with_when(when);
        let _ = osc.stop_with_when(when + dur.min(0.1) + 0.01);
        return;
    }

    osc.set_type(match channel {
        0 | 1 => OscillatorType::Square,
        _ => OscillatorType::Triangle,
    });
    osc.frequency().set_value(midi_hz(n.midi));
    let _ = osc.connect_with_audio_node(&gain);
    let _ = osc.start_with_when(when);
    let _ = osc.stop_with_when(when + dur + 0.02);
}

fn midi_hz(midi: u8) -> f32 {
    440.0 * 2f32.powf((f32::from(midi) - 69.0) / 12.0)
}

impl Audio {
    pub fn set_music(&mut self, track: Option<MusicTrackParams>) {
        self.resume();
        let Some(ctx) = self.ctx.clone() else {
            return;
        };
        self.music.set_track(&ctx, track);
    }

    pub fn tick_music(&mut self) {
        if !self.unlocked {
            return;
        }
        let Some(ctx) = self.ctx.clone() else {
            return;
        };
        self.music.tick(&ctx);
    }
}
