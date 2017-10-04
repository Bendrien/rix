extern crate cpal;
extern crate futures;
extern crate core;

use futures::stream::Stream;
use futures::task;
use futures::task::Executor;
use futures::task::Run;

use std::sync::Arc;
use std::thread;
use std::time::Duration;

struct MyExecutor;

impl Executor for MyExecutor {
    fn execute(&self, r: Run) {
        r.run();
    }
}

fn main() {
    let endpoints: Vec<_> = cpal::get_endpoints_list().collect();
    let formats: Vec<_> = endpoints.iter().map(|ep| ep.get_supported_formats_list()
        .unwrap().next().expect("Failed to get endpoint format")).collect();



    let endpoint = cpal::get_default_endpoint().expect("Failed to get default endpoint");




    let format = endpoint.get_supported_formats_list().unwrap().next().expect("Failed to get endpoint format");
    println!("{:?}", format);

    let event_loop = cpal::EventLoop::new();
    let executor = Arc::new(MyExecutor);

    let bla: Vec<_> = endpoints.iter().zip(formats).map(|(e, f)|
        cpal::Voice::new(&endpoint, &format, &event_loop).expect("Failed to create a voice")).collect();
    let (mut voice, stream) = cpal::Voice::new(&endpoint, &format, &event_loop).expect("Failed to create a voice");

    // Produce a sinusoid of maximum amplitude.
    let samples_rate = format.samples_rate.0 as f32;
    let blub = 2.0 * core::f32::consts::PI / samples_rate;
    let mut data_source = (0u64..).map(move |i| {
        let phi = i as f32 * blub;
        let m = (phi * 0.3).sin();
        println!("{:?}", m);
        (phi, m)
    })
        .map(move |(phi, m)| (phi + m * 0.2) * 60.0)     // 440 Hz
        .map(move |phi| phi.sin());

    voice.play();
    task::spawn(stream.for_each(move |buffer| -> Result<_, ()> {
        match buffer {
            cpal::UnknownTypeBuffer::U16(mut buffer) => {
                for (sample, value) in buffer.chunks_mut(format.channels.len()).zip(&mut data_source) {
                    let value = ((value * 0.5 + 0.5) * std::u16::MAX as f32) as u16;
                    for out in sample.iter_mut() { *out = value; }
                }
            },

            cpal::UnknownTypeBuffer::I16(mut buffer) => {
                for (sample, value) in buffer.chunks_mut(format.channels.len()).zip(&mut data_source) {
                    let value = (value * std::i16::MAX as f32) as i16;
                    for out in sample.iter_mut() { *out = value; }
                }
            },

            cpal::UnknownTypeBuffer::F32(mut buffer) => {
                for (sample, value) in buffer.chunks_mut(format.channels.len()).zip(&mut data_source) {
                    for out in sample.iter_mut() { *out = value; }
                }
            },
        };

        Ok(())
    })).execute(executor);

//    thread::spawn(move || {
//        loop {
//            thread::sleep(Duration::from_millis(500));
//            voice.pause();
//            thread::sleep(Duration::from_millis(500));
//            voice.play();
//        }
//    });

    event_loop.run();
}