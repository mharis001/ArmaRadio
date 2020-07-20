use std::collections::HashMap;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, SystemTime};

use alto::{Alto, DeviceObject};
use arma_rs::{rv, rv_handler};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rust_embed::RustEmbed;

mod source;
use crate::source::SoundSource;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref AL: Alto = {
        // OpenAL needs to live next to Arma
        let openal = std::path::Path::new("OpenAL32.dll");
        if !openal.exists() {
            // let mut resp = reqwest::blocking::get("https://github.com/Dynulo/ArmaRadio/releases/download/0.0/OpenAL32.dll").expect("request failed");
            // let mut out = std::fs::File::create(&openal).expect("failed to create file");
            // std::io::copy(&mut resp, &mut out).expect("failed to copy content");
            let dll = Assets::get("OpenAL32.dll").unwrap();
            let mut out = std::fs::File::create(&openal).expect("failed to create file");
            std::io::copy(&mut std::io::Cursor::new(dll), &mut out).expect("failed to copy content");
        }
        Alto::load_default().unwrap()
    };
    static ref SOURCES: Mutex<HashMap<String, SoundSource>> = Mutex::new(HashMap::new());
    static ref CONTEXT: alto::Context = {
        let device = AL.open(None).unwrap();
        println!("{:?}", device.specifier());
        device.new_context(None).unwrap()
    };
}

static mut TIMESTAMP: Option<SystemTime> = None;

#[derive(RustEmbed)]
#[folder = "embed"]
struct Assets;

#[rv]
unsafe fn start() {
    TIMESTAMP = Some(SystemTime::now());
    thread::spawn(|| loop {
        let dur = SystemTime::now()
            .duration_since(TIMESTAMP.unwrap())
            .unwrap();
        if dur > Duration::from_secs(3) {
            let mut sources = SOURCES.lock().unwrap();
            let keys = sources
                .keys()
                .map(|s| s.to_owned())
                .collect::<Vec<String>>();
            for key in keys {
                sources.remove(&key);
            }
        }
        thread::sleep(Duration::from_secs(1));
    });
}

#[rv]
unsafe fn heartbeat() {
    TIMESTAMP = Some(SystemTime::now());
}

#[rv]
fn id() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .collect::<String>()
        .to_lowercase()
}

#[rv]
fn create(source: String, sid: String) -> String {
    SOURCES
        .lock()
        .unwrap()
        .insert(sid.clone(), SoundSource::new(source));
    sid
}

#[rv]
fn destroy(sid: String) -> bool {
    SOURCES.lock().unwrap().remove(&sid).is_some()
}

#[rv]
fn pos(sid: String, x: f32, y: f32, z: f32) {
    if let Some(src) = SOURCES.lock().unwrap().get_mut(&sid) {
        src.set_position([x, y, z]);
    }
}

#[rv]
fn gain(sid: String, gain: f32) {
    if let Some(src) = SOURCES.lock().unwrap().get_mut(&sid) {
        src.set_gain(gain);
    }
}

#[rv]
fn orientation(dx: f32, dy: f32, dz: f32, ux: f32, uy: f32, uz: f32) {
    CONTEXT
        .set_orientation(([dx, dy, dz], [ux, uy, uz]))
        .unwrap();
}

#[rv]
fn list() -> String {
    let sources = SOURCES
        .lock()
        .unwrap()
        .keys()
        .map(|s| s.to_owned())
        .collect::<Vec<String>>();
    format!("[{}]", sources.join(","))
}

#[rv_handler]
fn init() {
    CONTEXT.set_position([0.0, 0.0, 0.0]).unwrap();
    CONTEXT.set_velocity([0.0, 0.0, 0.0]).unwrap();
    CONTEXT
        .set_orientation(([0.0, 0.0, 1.0], [0.0, 1.0, 0.0]))
        .unwrap();
    CONTEXT.set_meters_per_unit(1.0).unwrap();
    CONTEXT.set_distance_model(alto::DistanceModel::Inverse);
    CONTEXT.set_doppler_factor(0.2);
}
