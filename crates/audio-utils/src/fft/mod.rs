use rustfft::{num_complex::Complex, FftPlanner};

use crate::window::hann_window;

pub fn fft_apply(buf: &[f32]) -> Vec<Complex<f32>> {
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(buf.len());

    let mut buffer: Vec<_> = buf.iter().map(|f| Complex { re: *f, im: 0.0 }).collect();

    fft.process(&mut buffer);
    buffer
}

pub fn to_real(complex: &Complex<f32>) -> f32 {
    f32::sqrt(complex.re.powi(2) + complex.im.powi(2))
}

pub fn relevant_samples(sample_length: usize) -> usize {
    sample_length / 2 + 1
}

pub fn frequency_resolution(sampling_rate: u32, samples_len: u32) -> f32 {
    sampling_rate as f32 / samples_len as f32
}

pub fn spectrum(buf: &[f32], sample_rate: u16) -> Vec<(f32, f32)> {
    let sample_rate = sample_rate as f32;
    let buf = hann_window(buf);
    let fft_data = fft_apply(&buf);
    let frequency_resolution = frequency_resolution(sample_rate as u32, buf.len() as u32);
    let frequencies: Vec<_> = fft_data
        .iter()
        .take(relevant_samples(buf.len()))
        .enumerate()
        .map(|(i, f)| (i as f32 * frequency_resolution, to_real(f)))
        .collect();
    frequencies
}
