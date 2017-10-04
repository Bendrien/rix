//#![no_std]
extern crate core;
extern crate sample;
extern crate arraydeque;
#[macro_use]
extern crate vst2;

mod mixer;

use vst2::buffer::AudioBuffer;
use vst2::plugin::{Info, Plugin};

#[derive(Default)]
struct Rix {
    src: usize,
    trgt: usize,
    gain: f32,
    pan: f32,
    mute: bool,
    matrix: [mixer::Link; mixer::INPUTS * mixer::OUTPUTS],
}

impl Plugin for Rix {
    fn process(&mut self, buffer: AudioBuffer<f32>) {
        // Split out the input and output buffers into two vectors
        let (inputs, outputs) = buffer.split();

        // TODO: wrap into mixer::Buffer
        //mixer::process_matrix(&self.matrix[..inputs.len() *outputs.len()], inputs[..], &mut outputs);

        // For each buffer, transform the samples
        for (input_buffer, output_buffer) in inputs.iter().zip(outputs) {
            for (input_sample, output_sample) in input_buffer.iter().zip(output_buffer) {
                *output_sample = *input_sample;
            }
        }
    }


    fn get_info(&self) -> Info {
        Info {
            name: "rix".to_string(),
            vendor: "thumbsuckr".to_string(),
            unique_id: 51226, // Used by hosts to differentiate between plugins.
            inputs: mixer::INPUTS as i32,
            outputs: mixer::OUTPUTS as i32,
            parameters: 5,
            ..Default::default()
        }
    }

    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.src as f32 / mixer::INPUTS as f32,
            1 => self.trgt as f32 / mixer::OUTPUTS as f32,
            2 => self.gain,
            3 => self.pan,
            4 => if self.mute { 1. } else { 0. },
            _ => 0.0,
        }
    }

    fn set_parameter(&mut self, index: i32, value: f32) {
        match index {
            // We don't want to divide by zero, so we'll clamp the value
            0 => self.src = (value * mixer::INPUTS as f32) as usize,
            1 => self.trgt = (value * mixer::OUTPUTS as f32) as usize,
            2 => self.gain = value,
            3 => self.pan = value,
            4 => self.mute = if value > 0.5 { false } else { true },
            _ => (),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Source".to_string(),
            1 => "Target".to_string(),
            2 => "Gain".to_string(),
            3 => "Pan".to_string(),
            4 => "Mute".to_string(),
            _ => "".to_string(),
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("{}", self.src),
            1 => format!("{}", self.trgt),
            2 => format!("{}", self.gain),
            3 => format!("{}", self.pan),
            4 => format!("{}", self.mute),
            _ => "".to_string(),
        }
    }
}

plugin_main!(Rix); // Important!