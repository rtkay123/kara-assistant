use std::{ops::Range, time::Duration};

use audio_utils::fft::{convert_buffer, merge_buffers};

#[derive(Debug, Clone)]
pub enum Event {
    RequestData(crossbeam_channel::Sender<Vec<f32>>),
    SendData(Vec<f32>),
    RequestRefresh,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub buffering: u8,
    pub smoothing_size: u8,
    pub smoothing_amount: u8,
    pub resolution: u16,
    pub refresh_rate: u8,
    pub frequency_scale_range: Range<u16>,
    pub frequency_scale_amount: u8,
    pub density_reduction: u8,
    pub max_frequency: u16,
    pub volume: f32,
}
impl Default for Config {
    fn default() -> Self {
        Config {
            buffering: 5,
            smoothing_size: 10,
            smoothing_amount: 5,
            resolution: 3000,
            refresh_rate: 60,
            frequency_scale_range: Range {
                start: 50,
                end: 1000,
            },
            frequency_scale_amount: 1,
            density_reduction: 5,
            max_frequency: 22050,
            volume: 1.0,
        }
    }
}

pub fn visualise(
    refresh_rate: u16,
    config: &Config,
    event_receiver: crossbeam_channel::Receiver<Event>,
    event_sender: crossbeam_channel::Sender<Event>,
) {
    let config = config.clone();
    tokio::task::spawn_blocking(move || {
        let mut buffer: Vec<f32> = Vec::new();
        let mut calculated_buffer: Vec<f32> = Vec::new();
        let mut smoothing_buffer: Vec<Vec<f32>> = Vec::new();
        let mut smoothed_buffer: Vec<f32> = Vec::new();

        loop {
            match event_receiver.recv().unwrap() {
                Event::SendData(mut b) => {
                    buffer.append(&mut b);
                    let resolution = config.resolution.into();
                    while buffer.len() > resolution {
                        let c_b = convert_buffer(
                            &buffer[0..resolution],
                            config.max_frequency,
                            config.volume,
                            &config.frequency_scale_range,
                            config.smoothing_amount,
                            config.smoothing_size,
                            config.frequency_scale_amount,
                        );

                        calculated_buffer = if !calculated_buffer.is_empty() {
                            merge_buffers(&[calculated_buffer, c_b])
                        } else {
                            c_b
                        };
                        // remove already calculated parts
                        buffer.drain(0..resolution);
                    }
                }
                Event::RequestData(sender) => {
                    sender
                        .send(smoothed_buffer.clone())
                        .expect("audio thread lost connection to bridge");
                }
                Event::RequestRefresh => {
                    if !calculated_buffer.is_empty() {
                        smoothing_buffer.push(calculated_buffer.clone());
                    }
                    smoothed_buffer = if !smoothing_buffer.is_empty() {
                        merge_buffers(&smoothing_buffer)
                    } else {
                        Vec::new()
                    };
                    while smoothing_buffer.len() > config.buffering.into() {
                        smoothing_buffer.remove(0);
                    }
                }
            }
        }
    });

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(1000 / refresh_rate as u64)).await;
            event_sender.send(Event::RequestRefresh).unwrap();
        }
    });
}
