use std::f32::consts::PI;

pub fn hann_window(samples: &[f32]) -> Vec<f32> {
    let len = samples.len() as f32;
    samples
        .iter()
        .enumerate()
        .map(|(n, sample)| {
            let two_pi_i = 2.0 * PI * n as f32;
            let multiplier = 0.5 * (1.0 - f32::cos(two_pi_i / len));
            multiplier * sample
        })
        .collect()
}
