pub fn saw_wave(phase: f32, terms: u32) -> f32 {
    let mut sum = 0.0_f32;
    for n in 1..=terms {
        let n_f = n as f32;
        // (-1)^(n+1) coefficient
        let sign = if n % 2 == 0 { -1.0_f32 } else { 1.0_f32 };
        sum += sign * (phase * n_f).sin() / n_f;
    }
    (2.0 / std::f32::consts::PI) * sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saw_wave_zero_phase() {
        let v = saw_wave(0.0, 10);
        assert!(v.abs() < 1e-6);
    }

    #[test]
    fn test_saw_wave_more_terms() {
        // With a single term this approximates a sine, increasing terms
        // should approach the ideal saw shape.
        let low = saw_wave(std::f32::consts::FRAC_PI_2, 1);
        let high = saw_wave(std::f32::consts::FRAC_PI_2, 50);
        let diff_low = (low - 0.5).abs();
        let diff_high = (high - 0.5).abs();
        assert!(diff_high < diff_low);
        assert!(diff_high < 0.1);
    }

    #[test]
    fn test_saw_wave_pi_over_two() {
        let v = saw_wave(std::f32::consts::FRAC_PI_2, 200);
        assert!((v - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_saw_wave_negative_pi_over_two() {
        let v = saw_wave(-std::f32::consts::FRAC_PI_2, 200);
        assert!((v + 0.5).abs() < 0.01);
    }
}

pub fn sine_wave(freq: f32, sample_rate: f32, sample_index: usize) -> f32 {
    let phase = 2.0 * std::f32::consts::PI * freq * (sample_index as f32) / sample_rate;
    phase.sin()
}

pub fn sine_with_gain(freq: f32, sample_rate: f32, curve: &[f32]) -> Vec<f32> {
    curve
        .iter()
        .enumerate()
        .map(|(i, &g)| sine_wave(freq, sample_rate, i) * g)
        .collect()
}

pub fn saw_with_gain(freq: f32, sample_rate: f32, terms: u32, curve: &[f32]) -> Vec<f32> {
    curve
        .iter()
        .enumerate()
        .map(|(i, &g)| {
            let phase = 2.0 * std::f32::consts::PI * freq * (i as f32) / sample_rate;
            saw_wave(phase, terms) * g
        })
        .collect()
}

#[cfg(test)]
mod modulated_tests {
    use super::*;

    #[test]
    fn test_sine_with_gain_length() {
        let curve = vec![0.0, 0.5, 1.0];
        let out = sine_with_gain(1.0, 3.0, &curve);
        assert_eq!(out.len(), curve.len());
    }

    #[test]
    fn test_sine_wave_values() {
        // A simple 1Hz sine at 4Hz sample rate should hit 1.0 at the second sample
        let val = sine_wave(1.0, 4.0, 1);
        assert!((val - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_saw_with_gain_length() {
        let curve = vec![1.0; 5];
        let out = saw_with_gain(100.0, 44100.0, 3, &curve);
        assert_eq!(out.len(), curve.len());
    }

    #[test]
    fn test_saw_with_gain_values() {
        // Constant gain should produce a non-zero waveform matching `saw_wave`
        let curve = vec![1.0_f32; 3];
        let out = saw_with_gain(10.0, 10.0, 3, &curve);
        let expected = (0..3)
            .map(|i| {
                let phase = 2.0 * std::f32::consts::PI * 10.0 * (i as f32) / 10.0;
                saw_wave(phase, 3)
            })
            .collect::<Vec<_>>();
        assert_eq!(out, expected);
    }
}
