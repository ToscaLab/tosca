mod info;
mod parameters;
mod screenshot;
mod stream;

use std::net::Ipv4Addr;
use std::sync::Arc;

use ascot_library::hazards::Hazard;
use ascot_library::input::Input;
use ascot_library::route::Route;

use ascot_axum::actions::error::ErrorResponse;
use ascot_axum::actions::ok::ok_stateful;
use ascot_axum::actions::serial::{serial_stateful, serial_stateless};
use ascot_axum::actions::stream::stream_stateful;
use ascot_axum::device::Device;
use ascot_axum::error::Error;
use ascot_axum::server::Server;
use ascot_axum::service::ServiceConfig;

use async_lock::Mutex;

use clap::Parser;

use nokhwa::{
    native_api_backend, query,
    utils::{CameraFormat, CameraIndex, FrameFormat, RequestedFormatType, Resolution},
    NokhwaError,
};

use tracing::{error, info};
use tracing_subscriber::filter::LevelFilter;

use crate::info::{show_available_cameras, show_camera_info};
use crate::parameters::{
    change_camera, format_absolute_framerate, format_absolute_resolution, format_closest,
    format_exact, format_highest_framerate, format_highest_resolution, format_random,
};
use crate::screenshot::{
    screenshot_absolute_framerate, screenshot_absolute_resolution, screenshot_closest,
    screenshot_exact, screenshot_highest_framerate, screenshot_highest_resolution,
    screenshot_random,
};
use crate::stream::show_camera_stream;

fn startup_error(error: &str) -> Error {
    Error::external(format!("{error} at server startup"))
}

fn startup_with_error(description: &str, error: impl std::error::Error) -> Error {
    Error::external(format!(
        r"
            {description} at server startup
            Info: {error}
        "
    ))
}

fn camera_error(description: &'static str, error: impl std::error::Error) -> ErrorResponse {
    ErrorResponse::internal_with_error(description, error)
}

fn thread_error<T: std::fmt::Display>(msg: &str, e: T) {
    error!("{msg}");
    error!("{e}");
}

#[derive(Clone)]
struct CameraConfig {
    index: CameraIndex,
    format_type: RequestedFormatType,
}

#[derive(Clone)]
struct InternalState {
    camera: Arc<Mutex<CameraConfig>>,
}

impl InternalState {
    fn new(camera: CameraConfig) -> Self {
        Self {
            camera: Arc::new(Mutex::new(camera)),
        }
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Server address.
    ///
    /// Only an `Ipv4` address is accepted.
    #[arg(short, long, default_value_t = Ipv4Addr::UNSPECIFIED)]
    address: Ipv4Addr,

    /// Server host name.
    #[arg(short = 'n', long)]
    hostname: String,

    /// Server port.
    #[arg(short, long, default_value_t = 3000)]
    port: u16,

    /// Service domain.
    #[arg(short, long, default_value = "device")]
    domain: String,

    /// Service type.
    #[arg(short = 't', long = "type", default_value = "General Device")]
    service_type: String,
}

fn camera_format(
    x: u32,
    y: u32,
    fps: u32,
    fourcc: impl AsRef<str>,
) -> Result<CameraFormat, NokhwaError> {
    let fourcc = fourcc.as_ref().parse::<FrameFormat>()?;
    let resolution = Resolution::new(x, y);
    Ok(CameraFormat::new(resolution, fourcc, fps))
}

fn change_format(device: Device<InternalState>) -> Device<InternalState> {
    // Route to change camera index.
    let change_camera_route = Route::get("/change-camera")
        .description("Change camera.")
        .with_input(Input::characters_sequence("index", "0"));

    // Route to change format type to random.
    let change_format_random_route =
        Route::get("/random").description("Change stream format type to random.");

    // Route to change format type to highest resolution.
    let change_format_absolute_highest_resolution_route =
        Route::get("/absolute-highest-resolution")
            .description("Change stream format type to absolute highest resolution.");

    // Route to change format type to highest frame rate.
    let change_format_absolute_highest_framerate_route = Route::get("/absolute-highest-framerate")
        .description("Change stream format to absolute highest framerate.");

    // Route to change format type to highest resolution.
    let change_format_highest_resolution_route = Route::post("/highest-resolution")
        .description("Change stream format to highest resolution.")
        .with_inputs([Input::u32("x", 1920), Input::u32("y", 1080)]);

    // Route to change format type to highest framerate.
    let change_format_highest_framerate_route = Route::post("/highest-framerate")
        .description("Change stream format to highest framerate.")
        .with_input(Input::u32("fps", 30));

    // Route to change format type to exact type.
    let change_format_exact_route = Route::post("/exact")
        .description("Change stream format to exact type.")
        .with_inputs([
            Input::u32("x", 1920),
            Input::u32("y", 1080),
            Input::u32("fps", 30),
            Input::characters_sequence("fourcc", "YUYV"),
        ]);

    // Route to change format type to closest type.
    let change_format_closest_route = Route::post("/closest")
        .description("Change stream to closest type.")
        .with_inputs([
            Input::u32("x", 1920),
            Input::u32("y", 1080),
            Input::u32("fps", 30),
            Input::characters_sequence("fourcc", "YUYV"),
        ]);

    device
        .add_action(serial_stateful(change_camera_route, change_camera))
        .add_action(ok_stateful(change_format_random_route, format_random))
        .add_action(ok_stateful(
            change_format_absolute_highest_resolution_route,
            format_absolute_resolution,
        ))
        .add_action(ok_stateful(
            change_format_absolute_highest_framerate_route,
            format_absolute_framerate,
        ))
        .add_action(ok_stateful(
            change_format_highest_resolution_route,
            format_highest_resolution,
        ))
        .add_action(ok_stateful(
            change_format_highest_framerate_route,
            format_highest_framerate,
        ))
        .add_action(ok_stateful(change_format_exact_route, format_exact))
        .add_action(ok_stateful(change_format_closest_route, format_closest))
}

fn screenshot(device: Device<InternalState>) -> Device<InternalState> {
    // Route to take a screenshot with a random format.
    let screenshot_random_route = Route::get("/screenshot-random")
        .description("Screenshot with a random camera format.")
        .with_slice_hazards(&[
            Hazard::ElectricEnergyConsumption,
            Hazard::TakeDeviceScreenshots,
            Hazard::TakePictures,
        ]);

    // Route to view screenshot with absolute resolution.
    let screenshot_absolute_resolution_route = Route::get("/screenshot-absolute-resolution")
        .description("Screenshot from a camera with absolute resolution.")
        .with_slice_hazards(&[
            Hazard::ElectricEnergyConsumption,
            Hazard::TakeDeviceScreenshots,
            Hazard::TakePictures,
        ]);

    // Route to view screenshot with absolute framerate.
    let screenshot_absolute_framerate_route = Route::get("/screenshot-absolute-framerate")
        .description("Screenshot from a camera with absolute framerate.")
        .with_slice_hazards(&[
            Hazard::ElectricEnergyConsumption,
            Hazard::TakeDeviceScreenshots,
            Hazard::TakePictures,
        ]);

    // Route to view screenshot with highest resolution.
    let screenshot_highest_resolution_route = Route::post("/screenshot-highest-resolution")
        .description("Screenshot from a camera with highest resolution.")
        .with_slice_hazards(&[
            Hazard::ElectricEnergyConsumption,
            Hazard::TakeDeviceScreenshots,
            Hazard::TakePictures,
        ])
        .with_inputs([Input::u32("x", 1920), Input::u32("y", 1080)]);

    // Route to view screenshot with highest framerate.
    let screenshot_highest_framerate_route = Route::post("/screenshot-highest-framerate")
        .description("Screenshot from a camera with highest framerate.")
        .with_slice_hazards(&[
            Hazard::ElectricEnergyConsumption,
            Hazard::TakeDeviceScreenshots,
            Hazard::TakePictures,
        ])
        .with_input(Input::u32("fps", 30));

    // Route to view screenshot with exact approach.
    let screenshot_exact_route = Route::post("/screenshot-exact")
        .description("Screenshot from a camera with exact type.")
        .with_slice_hazards(&[
            Hazard::ElectricEnergyConsumption,
            Hazard::TakeDeviceScreenshots,
            Hazard::TakePictures,
        ])
        .with_inputs([
            Input::u32("x", 1920),
            Input::u32("y", 1080),
            Input::u32("fps", 30),
            Input::characters_sequence("fourcc", "YUYV"),
        ]);

    // Route to view screenshot with closest type.
    let screenshot_closest_route = Route::post("/screenshot-closest")
        .description("Screenshot from a camera with closest type.")
        .with_slice_hazards(&[
            Hazard::ElectricEnergyConsumption,
            Hazard::TakeDeviceScreenshots,
            Hazard::TakePictures,
        ])
        .with_inputs([
            Input::u32("x", 1920),
            Input::u32("y", 1080),
            Input::u32("fps", 30),
            Input::characters_sequence("fourcc", "YUYV"),
        ]);

    device
        .add_action(stream_stateful(screenshot_random_route, screenshot_random))
        .add_action(stream_stateful(
            screenshot_absolute_resolution_route,
            screenshot_absolute_resolution,
        ))
        .add_action(stream_stateful(
            screenshot_absolute_framerate_route,
            screenshot_absolute_framerate,
        ))
        .add_action(stream_stateful(
            screenshot_highest_resolution_route,
            screenshot_highest_resolution,
        ))
        .add_action(stream_stateful(
            screenshot_highest_framerate_route,
            screenshot_highest_framerate,
        ))
        .add_action(stream_stateful(screenshot_exact_route, screenshot_exact))
        .add_action(stream_stateful(
            screenshot_closest_route,
            screenshot_closest,
        ))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing subscriber.
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .init();

    // Command line parser.
    let cli = Cli::parse();

    // This initialization is necessary only on MacOS, but we are also going
    // to use this call to verify if everything went well.
    nokhwa::nokhwa_initialize(|granted| {
        if granted {
            info!("Nokhwa initialized correctly.");
        } else {
            info!("Nokhwa not initialized correctly. Exiting the process.");
            std::process::exit(1);
        }
    });

    // Retrieve native API camera backend
    let camera_backend = native_api_backend().ok_or(startup_error("No camera backend found"))?;

    // Retrieve all cameras present on a system
    let cameras = query(camera_backend)
        .map_err(|e| startup_with_error("The backend cannot find any camera", e))?;

    // Retrieve first camera present in the system
    let first_camera = cameras
        .first()
        .ok_or(startup_error("No cameras found in the system"))?;

    // Camera configuration.
    let camera = CameraConfig {
        index: first_camera.index().clone(),
        format_type: RequestedFormatType::None,
    };

    // Route to view camera stream.
    let camera_stream_route = Route::get("/stream")
        .description("View camera stream.")
        .with_slice_hazards(&[
            Hazard::ElectricEnergyConsumption,
            Hazard::VideoDisplay,
            Hazard::VideoRecordAndStore,
        ]);

    // Route to view all available cameras.
    let view_cameras_route = Route::get("/view-all").description("View all system cameras.");

    // Route to view camera info.
    let camera_info_route = Route::get("/view-info").description("View current camera data.");

    // A camera device which is going to be run on the server.
    let device = Device::with_state(InternalState::new(camera))
        .main_route("/camera")
        .add_action(stream_stateful(camera_stream_route, show_camera_stream))
        .add_action(serial_stateless(view_cameras_route, show_available_cameras))
        .add_action(serial_stateful(camera_info_route, show_camera_info));

    let device = change_format(device);
    let device = screenshot(device);

    // Run a discovery service and the device on the server.
    // FIXME: Remove Box when ascot-library uses heap collections and not
    // stacked ones.
    Box::pin(
        Server::new(device)
            .address(cli.address)
            .port(cli.port)
            .service(
                ServiceConfig::mdns_sd("camera")
                    .hostname(&cli.hostname)
                    .domain_name(&cli.domain)
                    .service_type(&cli.service_type),
            )
            .run(),
    )
    .await
}
