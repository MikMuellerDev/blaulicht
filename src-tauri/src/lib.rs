pub mod audio;
mod inputs;
pub mod utils;

use std::{
    sync::{
        mpsc::{self, Receiver, Sender, SyncSender, TryRecvError},
        Mutex,
    },
    thread,
    time::Duration,
};

use async_std::task;
use beat_detector::recording;
use cpal::{
    traits::{DeviceTrait, StreamTrait},
    Device, HostId,
};
use serde::Serialize;
use tauri::{AppHandle, Builder, Emitter, Manager, State, Window};
use utils::init_logger;

use crate::inputs::start;

struct AppData {
    welcome_message: &'static str,
    from_frontend: Mutex<Sender<FromFrontend>>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
// #[tauri::command]
// fn greet(name: &str, state: State<'_, Mutex<AppData>>) -> String {
//     format!(
//         "{}, {}! You've been greeted from Rust!",
//         name,
//         state.lock().unwrap().welcome_message
//     )
// }
//
//
#[derive(Serialize)]
struct FrontendDev {
    host: String,
    device: String,
}

#[tauri::command]
fn list_devices() -> Vec<FrontendDev> {
    let devices = utils::get_input_devices_flat();

    devices
        .iter()
        .map(|(host, dev)| FrontendDev {
            host: host.name().to_string(),
            device: dev.name().unwrap().to_string(),
        })
        .collect()
}

pub fn device_from_names(host_id: String, dev_id: String) -> Option<Device> {
    let devices = utils::get_input_devices_flat();

    let selected_device: Option<(HostId, Device)> =
        devices.into_iter().find(|(this_host, this_dev)| {
            this_host.name() == host_id && this_dev.name().unwrap() == dev_id
        });

    let Some((_, device)) = selected_device else {
        return None;
    };

    Some(device)
}

#[tauri::command]
fn select_device(state: State<'_, AppData>, host: String, device: String) -> Result<(), ()> {
    let sender = state.from_frontend.lock().unwrap();

    let device = device_from_names(host, device).unwrap();

    sender
        .send(FromFrontend::SelectInputDevice(device.clone()))
        .unwrap();

    println!("Selected device: {}", device.name().unwrap());

    Ok(())
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Heartbeat {
    seq: usize,
}

#[tauri::command]
fn socket(state: State<'_, AppData>, window: Window) {
    let sender = &state.from_frontend;
    let sender = sender.lock().unwrap();
    match sender.send(FromFrontend::NewWindow(window)) {
        Ok(_) => {}
        Err(err) => {
            println!("Failed to send new window: {err}");
        }
    };
}

#[derive(Serialize, Clone, Copy)]
enum ToFrontend {
    Volume(usize),
    Beat(usize),
    Speed(usize),
    Heartbeat,
}

#[derive(Clone)]
enum FromFrontend {
    NewWindow(Window),
    SelectInputDevice(Device),
}

async fn audio_thread(from_frontend: Receiver<FromFrontend>) {
    let begin_msg = from_frontend.recv().unwrap();
    println!("[audio] Frontend connected!");

    let FromFrontend::NewWindow(window) = begin_msg else {
        panic!("Illegal behaviour");
    };

    // let mut count = 0;
    // let mut increment = true;
    // let step = 25;
    let heartbeat_delay = Duration::from_millis(1000);

    let mut device: Option<Device> = None;
    let mut device_changed = false;

    loop {
        thread::sleep(heartbeat_delay);
        window.emit("msg", ToFrontend::Heartbeat).unwrap();

        match from_frontend.try_recv() {
            Ok(FromFrontend::NewWindow(_)) => todo!(),
            Ok(FromFrontend::SelectInputDevice(dev)) => {
                device = Some(dev.clone());
                device_changed = true;
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                unreachable!("broken")
            }
        };

        if device.is_none() {
            let devices = utils::get_input_devices_flat();
            if devices.is_empty() {
                panic!("No devices");
            }

            let host = devices[0].0.name().to_string();
            let device_name = devices[0].1.name().unwrap();

            println!("Selected default audio device: {host} | {device_name}");

            device = Some(device_from_names(host, device_name).unwrap());

            device_changed = true;
        } else if device_changed {
            let window = window.clone();
            thread::spawn(|| audio::foo(window));

            device_changed = false;
            println!(
                "Started audio detector thread: {}...",
                device.clone().unwrap().name().unwrap()
            );
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (from_frontend_sender, from_frontend_receiver) = mpsc::channel();

    // thread::spawn(|| {
    //     audio::foo();
    // });

    //
    // tauri::async_runtime::spawn(async {
    //     let devices = utils::get_input_devices_flat();
    //     if devices.is_empty() {
    //         panic!("No devices");
    //     }
    //
    //     let host = devices[0].0.name().to_string();
    //     let device_name = devices[0].1.name().unwrap();
    //
    //     println!("Selected default audio device: {host} | {device_name}");
    //
    //     let device = Some(device_from_names(host, device_name).unwrap());
    //
    //     println!("start beat detector");
    //     let handle = recording::start_detector_thread(
    //         move |_info| {
    //             // info.
    //             // println!("found beat!");
    //             // let mut rgb_buffer_locked = rgb_buffer.lock().unwrap();
    //             // for xrgb_pxl in rgb_buffer_locked.iter_mut() {
    //             //     *xrgb_pxl = 0x00ffffffff;
    //             // }
    //             println!("Beat!");
    //             // window.emit("msg", ToFrontend::Beat).unwrap();
    //         },
    //         Some(device.clone().unwrap()),
    //     )
    //     .unwrap();
    // });
    //
    // task::block_on(async {
    //     task::sleep(std::time::Duration::from_secs(100)).await;
    // });
    //
    // return;

    task::spawn(async { audio_thread(from_frontend_receiver).await });

    Builder::default()
        .plugin(tauri_plugin_shell::init())
        // .plugin(tauri_plugin_websocket::init())
        .setup(|app| {
            app.manage(AppData {
                welcome_message: "Welcome to Tauri!",
                from_frontend: Mutex::new(from_frontend_sender),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            socket,
            list_devices,
            select_device
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
