use std::{
    collections::VecDeque,
    sync::{
        mpsc::{self, Receiver},
        Arc,
    },
    thread,
    time::{self, Duration},
};

use audioviz::audio_capture::{capture::Capture, config::Config as CaptureConfig};
use audioviz::{
    audio_capture::capture::CaptureReceiver,
    spectrum::{
        stream::{Stream, StreamController},
        Frequency,
    },
};
use serde::{Deserialize, Serialize};
use serial2::SerialPort;
use tauri::{Emitter, Window};

use crate::ToFrontend;

const INIT_COLOR: u8 = b'g';

const PORT: &str = "/dev/pts/2";

fn write_port_color(port: Arc<SerialPort>, color: u8) -> Result<(), ()> {
    port.write(&[color]).unwrap();
    match port.flush() {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}

fn map(x: isize, in_min: isize, in_max: isize, out_min: isize, out_max: isize) -> usize {
    let divisor = (in_max - in_min).max(1);
    ((x - in_min) * (out_max - out_min) / (divisor) + out_min).max(0) as usize
}

pub enum ConverterType {
    Stream(Stream),
    Capture(Capture),
}

pub struct Converter {
    conv_type: ConverterType,
    raw_buf: Vec<f32>,
    show_vec: Vec<f32>,
    pub raw_receiver: Option<CaptureReceiver>,
    pub stream_controller: Option<StreamController>,
    pub config: Config,
    pub resolution: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Visualisation {
    Spectrum,
    Scope,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub audio: audioviz::spectrum::config::StreamConfig,
    pub mirror_x_achsis: bool,
    pub fps: u64,
    pub width: u8,
    pub spacing: u8,
    pub mirror: bool,
    pub visualisation: Visualisation,
}
impl Default for Config {
    fn default() -> Self {
        Config {
            audio: audioviz::spectrum::config::StreamConfig {
                gravity: Some(100.0),
                ..Default::default()
            },
            mirror_x_achsis: true,
            // fps: 60,
            fps: 1,
            width: 1,
            spacing: 0,
            mirror: true,
            visualisation: Visualisation::Spectrum,
        }
    }
}

impl Converter {
    pub fn from_capture(capture: Capture, config: Config) -> Self {
        let raw_receiver = capture.get_receiver().unwrap();
        Self {
            conv_type: ConverterType::Capture(capture),
            raw_buf: Vec::new(),
            show_vec: Vec::new(),
            raw_receiver: Some(raw_receiver),
            stream_controller: None,
            config,
            resolution: 0,
        }
    }

    pub fn from_stream(stream: Stream, config: Config) -> Self {
        let stream_controller = stream.get_controller();
        Self {
            conv_type: ConverterType::Stream(stream),
            raw_buf: Vec::new(),
            show_vec: Vec::new(),
            raw_receiver: None,
            stream_controller: Some(stream_controller),
            config,
            resolution: 0,
        }
    }

    pub fn get_data(&mut self) -> Option<Vec<f32>> {
        if let Some(raw) = &self.raw_receiver {
            let mut data: Vec<f32> = match raw.receive_data() {
                Ok(d) => {
                    let mut b: Vec<f32> = Vec::new();

                    let bufs = d.chunks(1);
                    for buf in bufs {
                        let mut max: f32 = 0.0;
                        for value in buf {
                            let value = value * 30.0 * self.config.audio.processor.volume;
                            if value > max {
                                max = value
                            }
                        }
                        b.push(max)
                    }
                    b
                }
                Err(_) => Vec::new(),
            };
            self.raw_buf.append(&mut data);
            if self.raw_buf.len() >= self.resolution {
                self.show_vec = self.raw_buf[0..self.resolution].to_vec();
                self.raw_buf.drain(..);
            }
            return Some(self.show_vec.clone());
        }
        if let Some(stream) = &self.stream_controller {
            let freqs = stream.get_frequencies();

            let data: Vec<f32> = freqs.into_iter().map(|x| x.volume).collect();

            return Some(data);
        }
        None
    }

    pub fn freqs(&mut self) -> Vec<Frequency> {
        if let Some(stream) = &self.stream_controller {
            let freqs = stream.get_frequencies();
            return freqs;
        }

        panic!("broken");
    }
}

pub fn run(mut converter: Converter, receiver: Receiver<String>, window: Window) {
    loop {
        thread::sleep(Duration::from_millis(1000));
        let min_time_between_updates = Duration::from_millis(70);

        let CODES_MAX = 1;

        let mut raw_port = match SerialPort::open(PORT, 115200) {
            Ok(p) => p,
            Err(err) => {
                eprintln!("Open port: {err:?}");
                continue;
            }
        };
        raw_port
            .set_read_timeout(Duration::from_millis(10))
            .unwrap();
        raw_port
            .set_write_timeout(Duration::from_millis(10))
            .unwrap();

        eprintln!("Port opened.");

        let port = Arc::new(raw_port);

        match write_port_color(port.clone(), INIT_COLOR) {
            Ok(_) => {}
            Err(_) => {
                eprintln!("Not syncing (OUTER)...");
                continue;
            }
        }

        let mut last_index = 0;

        let mut last_update = time::Instant::now();

        let rolling_average_frames = 100;
        let long_historic_frames = rolling_average_frames * 100;

        let mut long_historic = VecDeque::with_capacity(long_historic_frames);

        let mut historic = VecDeque::with_capacity(rolling_average_frames);

        let mut sleeping = true;

        let mut volume_samples: VecDeque<usize> = VecDeque::with_capacity(rolling_average_frames);

        let mut time_of_last_volume_publish = time::Instant::now();

        let mut loop_begin_time = time::Instant::now();

        let mut time_of_last_speed_publish = time::Instant::now();

        'inner: loop {
            let now = time::Instant::now();
            let loop_speed = now - loop_begin_time;
            println!("speed = {loop_speed:?}");
            loop_begin_time = now;

            if now - time_of_last_speed_publish > Duration::from_millis(1000) {
                window.emit("msg", ToFrontend::Speed(loop_speed.as_micros() as usize)).unwrap();
                time_of_last_speed_publish = now
            }

            if let Ok(data) = receiver.try_recv() {
                match port.write(data.as_bytes()) {
                    Ok(_) => {}
                    Err(_) => {
                        break 'inner;
                    }
                }
                match port.flush() {
                    Ok(_) => {}
                    Err(_) => {
                        break 'inner;
                    }
                }
            }
            let values = converter.freqs();

            if time::Instant::now() - time_of_last_volume_publish > Duration::from_millis(100) {
                let mut volume_mean = ((volume_samples.iter().sum::<usize>() as f32)
                    / (volume_samples.len() as f32)
                    * 10.0) as usize;
                if volume_mean > 100 {
                    volume_mean = 100;
                }
                println!("vol_mean = {volume_mean}");
                // We multiply by 10 since the source volume is between 0 and 10.
                window.emit("msg", ToFrontend::Volume(volume_mean)).unwrap();
                time_of_last_volume_publish = time::Instant::now();
            }

            volume_samples.push_back(
                values
                    .iter()
                    .max_by_key(|f| (f.volume * 10.0) as usize)
                    .unwrap()
                    .volume as usize,
            );
            if volume_samples.len() > rolling_average_frames / 2 {
                volume_samples.pop_front();
            }

            let curr: Vec<usize> = values
                .chunks(2)
                // TODO: only look at the base line?
                // .filter(|f| f.iter().all(|e| e.freq < 200.0))
                .map(|f| f.iter().map(|e| e.volume as usize).max().unwrap())
                .collect();

            let curr = curr.iter().max().unwrap_or(&0);

            let curr_unfiltered: usize = values.iter().map(|f| f.volume as usize).sum();

            long_historic.push_back(curr_unfiltered);
            if long_historic.len() >= long_historic_frames {
                long_historic.pop_front();
            }

            historic.push_back(*curr);

            if historic.len() >= rolling_average_frames {
                historic.pop_front();
            }

            let sum = historic.iter().sum::<usize>();
            let avg = sum / rolling_average_frames;
            let max = historic.iter().max().unwrap_or(&usize::MAX);
            let min = historic.iter().min().unwrap_or(&usize::MIN);

            let long_sum = long_historic.iter().sum::<usize>();

            if long_sum == 0 {
                if !sleeping {
                    match write_port_color(port.clone(), b'f') {
                        Ok(_) => {}
                        Err(_) => {
                            eprintln!("Not syncing...");
                            break 'inner;
                        }
                    }
                    eprintln!("long historic is 0: sleeping");
                }

                eprintln!("sleeping");
                thread::sleep(Duration::from_millis(500));
                sleeping = true;
            } else if sleeping {
                match write_port_color(port.clone(), b'n') {
                    Ok(_) => {}
                    Err(_) => {
                        eprintln!("Not syncing...");
                        break 'inner;
                    }
                }
                eprintln!("long = {long_sum}");
                sleeping = false
            }

            let index_mapped = map(*curr as isize, *min as isize, *max as isize, 0, CODES_MAX);

            if last_index == index_mapped {
                continue;
            }

            let now = time::Instant::now();
            if (now - last_update) < min_time_between_updates {
                continue;
            }

            last_index = index_mapped;

            last_update = now;

            eprintln!(
            "index = {index_mapped:02} | curr = {curr:03} | min = {min:03} | avg = {avg:03} | max = {max:03}",
        );

            let output_char = (b'A' + index_mapped as u8) as u32;

            // let index_percent = map(*curr as isize, *min as isize, *max as isize, 0, CODES_MAX);
            window.emit("msg", ToFrontend::Beat(index_mapped)).unwrap();

            match port.write(&[output_char as u8]) {
                Ok(_) => {}
                Err(_) => {
                    eprintln!("Not syncing...");
                    break 'inner;
                }
            }
            port.flush().unwrap();
        }
    }
}

pub fn foo(window: Window) {
    let (sender, receiver) = mpsc::channel();

    let config = Config::default();

    let audio_capture_config = CaptureConfig::default();

    let capture = Capture::init(audio_capture_config).unwrap();
    let converter: Converter = match config.visualisation {
        Visualisation::Spectrum => {
            let stream = Stream::init_with_capture(&capture, config.audio.clone());

            Converter::from_stream(stream, config.clone())
        }
        Visualisation::Scope => Converter::from_capture(capture, config.clone()),
    };

    run(converter, receiver, window);
}
