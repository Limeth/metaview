#![feature(type_alias_enum_variants)]

use std::rc::Rc;
use std::cell::RefCell;
use std::time::{Duration, Instant};
use std::num::NonZeroU32;
use std::sync::Arc;
use vulkano as vk;
use openxr as xr;
use winit::{ElementState, MouseButton, Event, DeviceEvent, WindowEvent, KeyboardInput, VirtualKeyCode, EventsLoop, WindowBuilder, Window};
use winit::dpi::PhysicalSize;
use ammolite::{Ammolite, WorldSpaceModel, UninitializedWindowMedium};
use ammolite::math::*;
use ammolite::camera::{Camera, PitchYawCamera3};
use lazy_static::lazy_static;
use crate::medium::MediumData;

pub mod medium;

lazy_static! {
    static ref PACKAGE_VERSION: (u16, u16, u16) = (
        env!("CARGO_PKG_VERSION_MAJOR").parse()
            .expect("Invalid crate major version, must be u16."),
        env!("CARGO_PKG_VERSION_MINOR").parse()
            .expect("Invalid crate minor version, must be u16."),
        env!("CARGO_PKG_VERSION_PATCH").parse()
            .expect("Invalid crate patch version, must be u16."),
    );
    static ref PACKAGE_NAME: &'static str = env!("CARGO_PKG_NAME");
}

fn construct_model_matrix(scale: f32, translation: &Vec3, rotation: &Vec3) -> Mat4 {
    Mat4::translation(translation)
        * Mat4::rotation_roll(rotation[2])
        * Mat4::rotation_yaw(rotation[1])
        * Mat4::rotation_pitch(rotation[0])
        * Mat4::scale(scale)
}

fn main() {
    // Check arguments
    let model_path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("No model path provided.");
        std::process::exit(1);
    });

    // Build Ammolite
    let events_loop = EventsLoop::new();
    let primary_monitor = events_loop.get_primary_monitor();
    let events_loop = Rc::new(RefCell::new(events_loop));
    let uwm = UninitializedWindowMedium {
        events_loop: events_loop.clone(),
        window_builder: WindowBuilder::new()
            .with_title("metaview")
            .with_dimensions(
                PhysicalSize::new(1280.0, 720.0)
                .to_logical(primary_monitor.get_hidpi_factor())
            ),
        data: MediumData::new_window(events_loop),
    };
    let mut ammolite = Ammolite::<MediumData>::builder(&PACKAGE_NAME, *PACKAGE_VERSION)
        .initialize_openxr()
        .initialize_vulkan()
        /*
         * TODO:
         * `initialize_vulkan` creates the windows already, consider either moving the window
         * creation to this method or to register the windows within `initialize_vulkan`.
         */
        .add_medium_window(uwm)
        .finish_adding_mediums_window()
        .finish_adding_mediums_stereo_hmd()
        .build();

    // Load resources
    let model = ammolite.load_model(model_path);

    // Event loop
    let init_instant = Instant::now();
    let mut previous_frame_instant = init_instant.clone();

    loop {
        let now = Instant::now();
        let elapsed = now.duration_since(init_instant);
        let delta_time = now.duration_since(previous_frame_instant);
        previous_frame_instant = now;
        let secs_elapsed = ((elapsed.as_secs() as f64) + (elapsed.as_nanos() as f64) / (1_000_000_000f64)) as f32;
        if ammolite.handle_events(&delta_time) {
            break;
        }

        let model_matrices = [
            construct_model_matrix(1.0,
                                   &[1.0, 0.0, 2.0].into(),
                                   &[secs_elapsed.sin() * 0.0 * 1.0, secs_elapsed.cos() * 0.0 * 3.0 / 2.0, 0.0].into()),
            construct_model_matrix(1.0,
                                   &[1.0, 1.0, 2.0].into(),
                                   &[secs_elapsed.sin() * 0.0 * 1.0, secs_elapsed.cos() * 0.0 * 3.0 / 2.0, 0.0].into()),
        ];

        let world_space_models = [
            WorldSpaceModel { model: &model, matrix: model_matrices[0].clone() },
            // WorldSpaceModel { model: &model, matrix: model_matrices[1].clone() },
        ];

        ammolite.render(&elapsed, || &world_space_models[..]);
    }
}
