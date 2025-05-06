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

#[derive(serde::Serialize)] // Add Serialize for sending data
struct FormData {
    name: String,
    age: String,
}

#[derive(Default)]
struct MyApp {
    name: String,
    age: String,
}

impl MyApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
} // <-- Add missing closing brace for impl MyApp

// Function to send request to backend
#[cfg(target_arch = "wasm32")]
fn trigger_server_log(name: &str, age: &str) { // Accept name and age
    let form_data = FormData {
        name: name.to_string(),
        age: age.to_string(),
    };
    wasm_bindgen_futures::spawn_local(async move { // move form_data into the async block
        let client = reqwest::Client::new();
        // Dynamically get hostname and construct URL for port 3000
        let location = web_sys::window().unwrap().location();
        let hostname = location.hostname().unwrap_or_else(|_| "localhost".to_string()); // Fallback to localhost if hostname fails
        let url = format!("http://{}:3000/api/log", hostname);
        log::info!("Sending request to: {}", url); // Log the constructed URL
        match client.post(&url) // Use the constructed URL
            .json(&form_data) // Send data as JSON
            .send().await {
            Ok(response) => {
                if response.status().is_success() {
                    log::info!("Successfully sent data to server.");
                } else {
                    log::error!("Failed to send data: {}", response.status());
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
        // Use a movable window instead of CentralPanel
        egui::Window::new("Input Form")
            .default_open(true) // Keep window open by default
            .resizable(true)
            .show(ctx, |ui| {
                ui.vertical(|ui| { // Use vertical layout within the window
                    // Name field
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.name);
                    });

                    // Age field
                    ui.horizontal(|ui| {
                        ui.label("Age:");
                        ui.text_edit_singleline(&mut self.age);
                    });

                    // Add some spacing
                    ui.add_space(10.0);

                    // Button
                    if ui.button("Click Me").clicked() {
                        // Call the function to send request to backend with current name and age
                        #[cfg(target_arch = "wasm32")]
                        trigger_server_log(&self.name, &self.age);

                        // Provide feedback in the browser console as well (optional)
                        log::info!("Button clicked, attempting to send data: Name='{}', Age='{}'", self.name, self.age);
                    }
                });
            });
    }
}
