// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{thread, time::Duration};

// use beat_detector::recording;
use blaulicht_lib::{
    audio,
    utils::{self, init_logger},
};
use cpal::traits::DeviceTrait;

fn main() {
    init_logger();

    // let devices = utils::get_input_devices_flat();
    // if devices.is_empty() {
    //     panic!("No devices");
    // }
    //
    // let host = devices[0].0.name().to_string();
    // let device_name = devices[0].1.name().unwrap();
    //
    // println!("Selected default audio device: {host} | {device_name}");
    //
    // let device = Some(blaulicht_lib::device_from_names(host, device_name).unwrap());
    //
    // println!("start beat detector");
    // let handle = recording::start_detector_thread(
    //     move |_info| {
    //         // info.
    //         // println!("found beat!");
    //         // let mut rgb_buffer_locked = rgb_buffer.lock().unwrap();
    //         // for xrgb_pxl in rgb_buffer_locked.iter_mut() {
    //         //     *xrgb_pxl = 0x00ffffffff;
    //         // }
    //         println!("Beat!");
    //         // window.emit("msg", ToFrontend::Beat).unwrap();
    //     },
    //     Some(device.clone().unwrap()),
    // )
    // .unwrap();

    // thread::spawn(|| {
    //     audio::foo();
    // });

    blaulicht_lib::run()
}
