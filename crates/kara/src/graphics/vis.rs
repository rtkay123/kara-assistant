use rustfft::{num_complex::Complex, FftPlanner};

pub(crate) fn fft(buf: &[i16]) -> Vec<Complex<f32>> {
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(buf.len());

    let mut buffer: Vec<_> = buf
        .iter()
        .map(|chunk| Complex {
            re: *chunk as f32,
            im: 0.0,
        })
        .collect();
    fft.process(&mut buffer);
    buffer
}
