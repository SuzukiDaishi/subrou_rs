pub fn envelope_follower(samples: &[f32], attack_ms: f32, release_ms: f32, sample_rate: f32) -> Vec<f32> {
    let attack_coeff = if attack_ms <= 0.0 {
        1.0
    } else {
        let attack_samples = attack_ms * 0.001 * sample_rate;
        1.0 - (-2.2_f32 / attack_samples).exp()
    };
    let release_coeff = if release_ms <= 0.0 {
        1.0
    } else {
        let release_samples = release_ms * 0.001 * sample_rate;
        1.0 - (-2.2_f32 / release_samples).exp()
    };

    let mut env = 0.0_f32;
    let mut curve = Vec::with_capacity(samples.len());
    for &s in samples {
        let target = s.abs();
        if target > env {
            env += attack_coeff * (target - env);
        } else {
            env += release_coeff * (target - env);
        }
        curve.push(env);
    }
    curve
}

pub fn apply_gain_curve(samples: &mut [f32], curve: &[f32]) {
    assert_eq!(samples.len(), curve.len());
    for (s, &g) in samples.iter_mut().zip(curve.iter()) {
        *s *= g;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_follower_constant() {
        let samples = vec![1.0_f32; 100];
        let curve = envelope_follower(&samples, 10.0, 10.0, 100.0);
        assert_eq!(curve.len(), samples.len());
        assert!(*curve.last().unwrap() > 0.9);
    }

    #[test]
    fn test_envelope_follower_step_attack() {
        // Step from silence to full scale. The envelope should rise gradually
        // according to the attack time.
        let mut samples = vec![0.0_f32; 50];
        samples.extend(vec![1.0_f32; 50]);
        let curve = envelope_follower(&samples, 10.0, 10.0, 1000.0);
        // Check initial silence region stays near zero
        assert!(curve[25] < 0.1);
        // The envelope should rise but not reach one immediately
        assert!(curve[55] > 0.2 && curve[55] < 1.0);
    }

    #[test]
    fn test_envelope_follower_release() {
        // Start loud and drop to silence, verifying release behaviour
        let mut samples = vec![1.0_f32; 50];
        samples.extend(vec![0.0_f32; 50]);
        let curve = envelope_follower(&samples, 1.0, 20.0, 1000.0);
        // Attack region should be close to one
        assert!(curve[10] > 0.8);
        // After release period the value should have decreased
        assert!(curve[90] < 0.3);
    }

    #[test]
    fn test_apply_gain_curve() {
        let mut samples = vec![1.0_f32; 4];
        let curve = vec![0.0_f32, 0.5, 0.5, 1.0];
        apply_gain_curve(&mut samples, &curve);
        assert_eq!(samples, vec![0.0, 0.5, 0.5, 1.0]);
    }
}
