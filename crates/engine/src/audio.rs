use wasm_bindgen::JsValue;
use web_sys::{AudioContext, AudioContextState, OscillatorType};

pub struct Audio {
    ctx: Option<AudioContext>,
    unlocked: bool,
}

impl Audio {
    pub fn new() -> Self {
        Self {
            ctx: None,
            unlocked: false,
        }
    }

    fn ensure_ctx(&mut self) -> Result<&AudioContext, JsValue> {
        if self.ctx.is_none() {
            let ctx = AudioContext::new()?;
            self.ctx = Some(ctx);
        }
        Ok(self.ctx.as_ref().unwrap())
    }

    pub fn resume(&mut self) {
        let Ok(ctx) = self.ensure_ctx() else {
            return;
        };
        if ctx.state() == AudioContextState::Suspended {
            let _ = ctx.resume();
        }
        self.unlocked = true;
    }

    pub fn beep(&mut self, freq_hz: f32, dur_s: f32) {
        self.resume();
        let Ok(ctx) = self.ensure_ctx() else {
            return;
        };
        let Ok(osc) = ctx.create_oscillator() else {
            return;
        };
        let Ok(gain) = ctx.create_gain() else {
            return;
        };

        osc.set_type(OscillatorType::Square);
        osc.frequency().set_value(freq_hz);

        let now = ctx.current_time();
        gain.gain().set_value(0.0001);
        let _ = gain.gain().linear_ramp_to_value_at_time(0.2, now + 0.005);
        let _ = gain
            .gain()
            .exponential_ramp_to_value_at_time(0.0001, now + f64::from(dur_s));

        let _ = osc.connect_with_audio_node(&gain);
        let _ = gain.connect_with_audio_node(&ctx.destination());
        let _ = osc.start_with_when(now);
        let _ = osc.stop_with_when(now + f64::from(dur_s) + 0.02);
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
