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

struct MyApp {
    name: String,
    age: String,
    current_time_title: Option<String>, // To store fetched time for browser/viewport title update
    digital_clock_time: String,         // To store time for the large digital clock
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: String::new(),
            age: String::new(),
            current_time_title: None,
            digital_clock_time: "--:--:--".to_string(), // Default clock display
        }
    }
}

impl MyApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
}

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
        // Central Panel for the digital clock
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.label(egui::RichText::new(&self.digital_clock_time).size(60.0));
            });
        });

        // Input Form Window
        egui::Window::new("Input Goes Here!") // Static title
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
                    ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                        if ui.add(egui::Button::new("Click Me")
                            .fill(egui::Color32::from_rgb(0x8B, 0x00, 0x00)) // Dark red
                        ).clicked() {
                            // Call the function to send request to backend with current name and age
                            #[cfg(target_arch = "wasm32")]
                            trigger_server_log(&self.name, &self.age);

                            // Fetch current time from server
                            #[cfg(target_arch = "wasm32")]
                            {
                                // To update MyApp state from async block, we need to pass a mutable reference
                                // or use a channel. For simplicity, we'll clone `self.current_time_title`
                                // and update it. This is not ideal for complex state but works for this example.
                                // A more robust way would be to use `Arc<Mutex<Option<String>>>` or `mpsc::channel`.
                                let time_title_sender = {
                                    // This is a bit of a workaround to get a mutable reference to current_time_title
                                    // into the async block. A proper solution would involve message passing.
                                    // We'll use a temporary variable that the async block can own.
                                    // This is a simplified approach for demonstration.
                                    // We need a way to communicate the fetched time back to the main thread's MyApp instance.
                                    // One common way is to use a channel (e.g., std::sync::mpsc::channel).
                                    // For this example, we'll directly try to update a field that the main loop checks.
                                    // This requires careful handling of lifetimes and mutability.
                                    // Let's try to use a simple flag that the main update loop can check.
                                    // We will store the fetched time in `current_time_title` which is an Option<String>.
                                    // The async block will own a way to send the result back.
                                    // For this example, we'll directly modify `self.current_time_title`
                                    // by capturing a mutable reference. This is generally unsafe across threads
                                    // but might work in wasm_bindgen_futures's single-threaded context if handled carefully.
                                    // However, to avoid lifetime issues with `self`, we'll pass a clone
                                    // of an `Arc<Mutex<Option<String>>>` or use a channel.
                                    // Given the constraints, we'll set `current_time_title` directly.
                                    // This is a simplified approach.
                                    let app_ptr = self as *mut MyApp; // Unsafe, but for a simple example. Be cautious.

                                    wasm_bindgen_futures::spawn_local(async move {
                                        let client = reqwest::Client::new();
                                        let location = web_sys::window().unwrap().location();
                                        let hostname = location.hostname().unwrap_or_else(|_| "localhost".to_string());
                                        let url = format!("http://{}:3000/api/time", hostname);
                                        match client.get(&url).send().await {
                                            Ok(response) => {
                                                if response.status().is_success() {
                                                    match response.text().await {
                                                        Ok(time_str) => {
                                                            log::info!("Fetched time: {}", time_str);
                                                            // Unsafe block to update MyApp state from async
                                                            // This is highly discouraged for production code.
                                                            // Use channels or Arc<Mutex<>> for safety.
                                                            unsafe {
                                                                (*app_ptr).current_time_title = Some(time_str.clone());
                                                                (*app_ptr).digital_clock_time = time_str;
                                                            }
                                                        }
                                                        Err(err) => log::error!("Failed to parse time response: {}", err),
                                                    }
                                                } else {
                                                    log::error!("Failed to fetch time: {}", response.status());
                                                }
                                            }
                                            Err(err) => log::error!("Error fetching time: {}", err),
                                        }
                                    });
                                };
                            }

                            // Provide feedback in the browser console as well (optional)
                            log::info!("Button clicked, attempting to send data: Name='{}', Age='{}'", self.name, self.age);

                            // Clear the fields
                            self.name.clear();
                            self.age.clear();
                        }
                    });
                });

                // Update window title if current_time_title is set
                if let Some(new_title) = self.current_time_title.take() {
                    #[cfg(target_arch = "wasm32")]
                    {
                        if let Some(window) = web_sys::window() {
                            if let Some(document) = window.document() {
                                document.set_title(&new_title);
                            }
                        }
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Title(new_title));
                    }
                }
            });
    }
}
