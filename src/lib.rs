use nih_plug::prelude::*;
use std::sync::Arc;

pub mod wave;
pub use wave::{saw_wave, saw_with_gain, sine_wave, sine_with_gain};
pub mod envelope;
pub use envelope::{apply_gain_curve, envelope_follower};

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

struct SubrouRs {
    params: Arc<SubrouRsParams>,
    sample_rate: f32,
}

#[derive(Params)]
struct SubrouRsParams {
    /// Post gain applied after the generated saw wave.
    #[id = "post_gain"]
    pub post_gain: FloatParam,

    /// Fundamental pitch for the generated saw wave.
    #[id = "pitch"]
    pub pitch: FloatParam,

    /// Output channel, `0` for all channels or 1-based channel index.
    #[id = "out_channel"]
    pub out_channel: IntParam,
}

impl Default for SubrouRs {
    fn default() -> Self {
        Self {
            params: Arc::new(SubrouRsParams::default()),
            sample_rate: 44100.0,
        }
    }
}

impl Default for SubrouRsParams {
    fn default() -> Self {
        Self {
            post_gain: FloatParam::new(
                "Post Gain",
                1.0,
                FloatRange::Skewed {
                    min: 0.0,
                    max: util::db_to_gain(6.0),
                    factor: FloatRange::gain_skew_factor(util::MINUS_INFINITY_DB, 6.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(10.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            pitch: FloatParam::new(
                "Pitch",
                440.0,
                FloatRange::Linear { min: 10.0, max: 2000.0 },
            ),
            out_channel: IntParam::new(
                "Output Channel",
                0,
                IntRange::Linear { min: 0, max: 10 },
            ),
        }
    }
}

impl Plugin for SubrouRs {
    const NAME: &'static str = "Subrou Rs";
    const VENDOR: &'static str = "Daishi Suzuki";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "zukky.rikugame@gmail.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    }];


    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        self.sample_rate = buffer_config.sample_rate as f32;
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let num_samples = buffer.samples();
        if num_samples == 0 {
            return ProcessStatus::Normal;
        }

        let slices = buffer.as_slice();
        let num_channels = slices.len().max(1);

        // Sum input to mono
        let mut mono = vec![0.0f32; num_samples];
        for channel in slices.iter() {
            for (i, &sample) in channel.iter().enumerate() {
                mono[i] += sample;
            }
        }
        for sample in &mut mono {
            *sample /= num_channels as f32;
        }

        // Envelope from mono input
        let curve = envelope_follower(&mono, 10.0, 10.0, self.sample_rate);

        // Generate saw wave with envelope gain
        let freq = self.params.pitch.smoothed.next();
        let post = self.params.post_gain.smoothed.next();
        let mut saw = Vec::with_capacity(num_samples);
        for (i, gain) in curve.iter().enumerate() {
            let phase = 2.0 * std::f32::consts::PI * freq * (i as f32) / self.sample_rate;
            saw.push(saw_wave(phase, 3) * *gain * post);
        }

        let out_ch = self.params.out_channel.value();
        if out_ch == 0 {
            for channel in slices.iter_mut() {
                for (i, sample) in channel.iter_mut().enumerate() {
                    *sample += saw[i];
                }
            }
        } else {
            let idx = (out_ch - 1) as usize;
            if idx < slices.len() {
                for (i, sample) in slices[idx].iter_mut().enumerate() {
                    *sample = saw[i];
                }
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for SubrouRs {
    const CLAP_ID: &'static str = "com.zukky.subrou-rs";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("SubBaseMaker");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for SubrouRs {
    const VST3_CLASS_ID: [u8; 16] = *b"Subrou!!!!!!!!!!";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(SubrouRs);
nih_export_vst3!(SubrouRs);

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyContext;

    impl ProcessContext<SubrouRs> for DummyContext {
        fn plugin_api(&self) -> PluginApi { PluginApi::Vst3 }
        fn execute_background(&self, _task: ()) {}
        fn execute_gui(&self, _task: ()) {}
        fn transport(&self) -> &Transport { unreachable!("transport unused") }
        fn next_event(&mut self) -> Option<PluginNoteEvent<SubrouRs>> { None }
        fn send_event(&mut self, _event: PluginNoteEvent<SubrouRs>) {}
        fn set_latency_samples(&self, _samples: u32) {}
        fn set_current_voice_capacity(&self, _capacity: u32) {}
    }

    #[test]
    fn test_process_silence() {
        let mut plugin = SubrouRs::default();
        plugin.params.post_gain.smoothed.reset(plugin.params.post_gain.value());
        plugin.params.pitch.smoothed.reset(plugin.params.pitch.value());
        let mut left = vec![0.0_f32; 64];
        let mut right = vec![0.0_f32; 64];
        let mut buffer = Buffer::default();
        unsafe { buffer.set_slices(64, |out| *out = vec![&mut left, &mut right]) };
        let mut aux = AuxiliaryBuffers { inputs: &mut [], outputs: &mut [] };
        let mut ctx = DummyContext;
        plugin.process(&mut buffer, &mut aux, &mut ctx);
        for ch in buffer.as_slice() {
            assert!(ch.iter().all(|&s| s == 0.0));
        }
    }

    #[test]
    fn test_process_generates_audio() {
        let mut plugin = SubrouRs::default();
        plugin.params.post_gain.smoothed.reset(plugin.params.post_gain.value());
        plugin.params.pitch.smoothed.reset(plugin.params.pitch.value());
        let mut left = vec![1.0_f32; 64];
        let mut right = vec![1.0_f32; 64];
        let mut buffer = Buffer::default();
        unsafe { buffer.set_slices(64, |out| *out = vec![&mut left, &mut right]) };
        let mut aux = AuxiliaryBuffers { inputs: &mut [], outputs: &mut [] };
        let mut ctx = DummyContext;
        plugin.process(&mut buffer, &mut aux, &mut ctx);
        let slices = buffer.as_slice();
        assert!(slices[0].iter().any(|&s| s != 1.0));
        assert_eq!(slices[0], slices[1]);
    }
}
