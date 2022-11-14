use std::{
    ops::Range,
    sync::{Arc, Mutex},
};

use audio_utils::fft::{convert_buffer, merge_buffers};

use crate::config::{Configuration, Visualiser};

#[derive(Debug, Clone)]
pub enum Event {
    RequestData(crossbeam_channel::Sender<Vec<f32>>),
    SendData(Vec<f32>),
    RequestRefresh,
}

pub fn visualise(
    config: Arc<Mutex<Configuration>>,
    event_receiver: crossbeam_channel::Receiver<Event>,
) {
    let frequency_scale_range = Range {
        start: 50,
        end: 1000,
    };
    tokio::task::spawn_blocking(move || {
        let mut buffer: Vec<f32> = Vec::new();
        let mut calculated_buffer: Vec<f32> = Vec::new();
        let mut smoothing_buffer: Vec<Vec<f32>> = Vec::new();
        let mut smoothed_buffer: Vec<f32> = Vec::new();

        loop {
            match event_receiver.recv().unwrap() {
                Event::SendData(mut b) => {
                    let config = config.lock().unwrap();
                    let config = match &config.audio {
                        Some(audio) => audio.visualiser.clone(),
                        None => Visualiser::default(),
                    };
                    buffer.append(&mut b);
                    let resolution = config.resolution.into();
                    while buffer.len() > resolution {
                        let c_b = convert_buffer(
                            &buffer[0..resolution],
                            22050,
                            config.loudness,
                            &frequency_scale_range,
                            config.smoothing_amount as u8,
                            config.smoothing_size as u8,
                            1,
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
                    let config = config.lock().unwrap();
                    let config = match &config.audio {
                        Some(audio) => audio.visualiser.clone(),
                        None => Visualiser::default(),
                    };
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
}
