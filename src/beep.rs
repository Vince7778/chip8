use std::{error::Error, f32::consts::PI, thread, sync::mpsc::{self, Sender}};

use cpal::{traits::{HostTrait, DeviceTrait, StreamTrait}, SampleFormat, Sample, Device, StreamConfig, Stream, BuildStreamError};

pub enum BeepInstructions {
    Play,
    Pause,
    Stop
}

pub struct Beep {
    sender: Sender<BeepInstructions>
}

impl Beep {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or("No audio output device available")?;

        let supported_config = device.default_output_config()?;

        let sample_format = supported_config.sample_format();

        let (send, recv) = mpsc::channel::<BeepInstructions>();

        thread::spawn(move || {
            let stream = match sample_format {
                SampleFormat::F32 => Self::create_stream::<f32>(&device, &supported_config.into()),
                SampleFormat::I16 => Self::create_stream::<i16>(&device, &supported_config.into()),
                SampleFormat::U16 => Self::create_stream::<u16>(&device, &supported_config.into()),
            }.unwrap();

            loop {
                match recv.recv() {
                    Ok(BeepInstructions::Play) => {
                        stream.play().unwrap();
                    },
                    Ok(BeepInstructions::Pause) => {
                        stream.pause().unwrap();
                    }
                    Ok(BeepInstructions::Stop) => {
                        stream.pause().unwrap();
                        return;
                    }
                    Err(e) => eprintln!("Error in beep playback: {}", e),
                };
            }
        });

        Ok(Self {
            sender: send
        })
    }

    pub fn play(&self) -> Result<(), mpsc::SendError<BeepInstructions>> {
        self.sender.send(BeepInstructions::Play)
    }

    pub fn pause(&self) -> Result<(), mpsc::SendError<BeepInstructions>> {
        self.sender.send(BeepInstructions::Pause)
    }

    pub fn stop(&self) -> Result<(), mpsc::SendError<BeepInstructions>> {
        self.sender.send(BeepInstructions::Stop)
    }

    fn create_stream<T: Sample>(device: &Device, config: &StreamConfig) -> Result<Stream, BuildStreamError> {
        let sample_rate = config.sample_rate.0 as f32;
        let channels = config.channels as usize;

        let mut cur_time = 0f32;
        let mut get_next = move || {
            cur_time = (cur_time + 1f32) % sample_rate;
            (cur_time * 880f32 * PI / sample_rate).sin()
        };

        device.build_output_stream(
            config,
            move |data: &mut [T], _| {
                for ch in data.chunks_mut(channels) {
                    let v = Sample::from::<f32>(&get_next());
                    for sample in ch.iter_mut() {
                        *sample = v;
                    }
                }
            }, 
            |err| { eprintln!("Error! {}", err)}
        )
    }
}

impl Drop for Beep {
    fn drop(&mut self) {
        self.stop().unwrap();
    }
}
