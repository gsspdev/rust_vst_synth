use vst::prelude::*;
use vst::plugin_main;
use vst::util::AtomicFloat;
use std::f32::consts::PI;
use std::sync::{Arc, Mutex};

struct RustSynth {
    sample_rate: f32,
    time: f32,
    note: u8,
    note_on: bool,
    params: Arc<RustSynthParameters>,
}

struct RustSynthParameters {
    volume: AtomicFloat,
    attack: AtomicFloat,
    decay: AtomicFloat,
    sustain: AtomicFloat,
    release: AtomicFloat,
}

impl Default for RustSynth {
    fn default() -> RustSynth {
        RustSynth {
            sample_rate: 44100.0,
            time: 0.0,
            note: 0,
            note_on: false,
            params: Arc::new(RustSynthParameters {
                volume: AtomicFloat::new(0.5),
                attack: AtomicFloat::new(0.01),
                decay: AtomicFloat::new(0.1),
                sustain: AtomicFloat::new(0.5),
                release: AtomicFloat::new(0.1),
            }),
        }
    }
}

impl Plugin for RustSynth {
    fn new(_host: HostCallback) -> Self {
        Default::default()
    }

    fn get_info(&self) -> Info {
        Info {
            name: "Rust Synth".to_string(),
            vendor: "Your Name".to_string(),
            unique_id: 1234,
            category: Category::Synth,
            inputs: 0,
            outputs: 2,
            parameters: 5,
            initial_delay: 0,
            ..Default::default()
        }
    }

    fn init(&mut self) {
        self.sample_rate = 44100.0;
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let samples = buffer.samples();
        let (_, mut outputs) = buffer.split();
        let output_count = outputs.len();
        let per_sample = self.time_per_sample();

        for sample_idx in 0..samples {
            if self.note_on {
                let wave = self.generate_wave();
                let envelope = self.apply_envelope();
                let out = wave * envelope * self.params.volume.get();

                for buf_idx in 0..output_count {
                    let buff = outputs.get_mut(buf_idx);
                    buff[sample_idx] = out;
                }
            }
            self.time += per_sample;
        }
    }

    fn process_events(&mut self, events: &Events) {
        for event in events.events() {
            match event {
                Event::Midi(ev) => {
                    match ev.data[0] {
                        128 => self.note_off(ev.data[1]),
                        144 => self.note_on(ev.data[1], ev.data[2]),
                        _ => (),
                    }
                }
                _ => (),
            }
        }
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
}

impl RustSynth {
    fn time_per_sample(&self) -> f32 {
        1.0 / self.sample_rate
    }

    fn note_on(&mut self, note: u8, _velocity: u8) {
        self.note = note;
        self.note_on = true;
        self.time = 0.0;
    }

    fn note_off(&mut self, note: u8) {
        if self.note == note {
            self.note_on = false;
        }
    }

    fn generate_wave(&self) -> f32 {
        let freq = self.midi_note_to_freq(self.note);
        (self.time * freq * 2.0 * PI).sin()
    }

    fn midi_note_to_freq(&self, note: u8) -> f32 {
        const A4_FREQ: f32 = 440.0;
        const A4_NOTE: i8 = 69;
        ((note as i8 - A4_NOTE) as f32 / 12.0).exp2() * A4_FREQ
    }

    fn apply_envelope(&self) -> f32 {
        let attack = self.params.attack.get();
        let decay = self.params.decay.get();
        let sustain = self.params.sustain.get();
        let release = self.params.release.get();

        if self.note_on {
            if self.time < attack {
                self.time / attack
            } else if self.time < attack + decay {
                1.0 - (1.0 - sustain) * (self.time - attack) / decay
            } else {
                sustain
            }
        } else {
            if self.time < release {
                sustain * (1.0 - self.time / release)
            } else {
                0.0
            }
        }
    }
}

impl PluginParameters for RustSynthParameters {
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.volume.get(),
            1 => self.attack.get(),
            2 => self.decay.get(),
            3 => self.sustain.get(),
            4 => self.release.get(),
            _ => 0.0,
        }
    }

    fn set_parameter(&self, index: i32, value: f32) {
        match index {
            0 => self.volume.set(value),
            1 => self.attack.set(value),
            2 => self.decay.set(value),
            3 => self.sustain.set(value),
            4 => self.release.set(value),
            _ => (),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Volume".to_string(),
            1 => "Attack".to_string(),
            2 => "Decay".to_string(),
            3 => "Sustain".to_string(),
            4 => "Release".to_string(),
            _ => "".to_string(),
        }
    }

    fn get_parameter_label(&self, index: i32) -> String {
        match index {
            0 => "%".to_string(),
            1 | 2 | 4 => "s".to_string(),
            3 => "%".to_string(),
            _ => "".to_string(),
        }
    }
}

plugin_main!(RustSynth);

