use cpal::ChannelCount;

use dasp::{sample::ToSample, Sample};

pub(crate) fn convert_to_mono(input_data: &[i16], channels: ChannelCount) -> Vec<i16> {
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

pub(crate) fn resample<T: std::fmt::Debug + Sample + ToSample<i16>>(
    data: &[T],
    channels: ChannelCount,
) -> Vec<i16> {
    let data: Vec<i16> = data.iter().map(|v| v.to_sample()).collect();
    convert_to_mono(&data, channels)
}
