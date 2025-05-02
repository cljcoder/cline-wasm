#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Hello World App",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(MyApp::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}


#[derive(Default)]
struct MyApp {}

impl MyApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}


// Function to send request to backend
#[cfg(target_arch = "wasm32")]
fn trigger_server_log() {
    wasm_bindgen_futures::spawn_local(async {
        let client = reqwest::Client::new();
        // URL of the backend server endpoint
        // Use the full URL including the host where trunk serves the frontend.
        // The trunk proxy will intercept this request based on the path.
        let url = "http://localhost:8080/api/log"; // Use absolute URL
        match client.post(url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    log::info!("Successfully triggered server log.");
                } else {
                    log::error!("Failed to trigger server log: {}", response.status());
                }
            }
            Err(err) => {
                log::error!("Error sending request to server: {}", err);
            }
        }
    });
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                // Add a button instead of the heading
                if ui.button("Click Me").clicked() {
                    // Call the function to send request to backend
                    #[cfg(target_arch = "wasm32")]
                    trigger_server_log();

                    // Provide feedback in the browser console as well (optional)
                    log::info!("Button clicked, attempting to trigger server log...");
                }
            });
        });
    }
}
