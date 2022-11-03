use dasp::{sample::ToSample, Sample};

pub fn convert_to_mono(input_data: &[i16], channels: u16) -> Vec<i16> {
    if channels != 1 {
        let mut result = Vec::with_capacity(input_data.len() / 2);
        result.extend(
            input_data
                .chunks_exact(2)
                .map(|chunk| chunk[0] / 2 + chunk[1] / 2),
        );
        result
    } else {
        input_data.to_owned()
    }
}

pub fn resample<T: std::fmt::Debug + Sample + ToSample<i16>>(
    data: &[T],
    channels: u16,
) -> Vec<i16> {
    let data: Vec<i16> = data.iter().map(|v| v.to_sample()).collect();
    convert_to_mono(&data, channels)
}

pub fn split_channels<T: Copy>(buf: &[T], channels: u16) -> Vec<Vec<T>> {
    let mut buffer: Vec<Vec<T>> = vec![vec![]; channels.into()];

    for chunked_data in buf.chunks(channels.into()) {
        for (i, v) in chunked_data.iter().enumerate() {
            buffer[i].push(*v);
        }
    }

    buffer
}
