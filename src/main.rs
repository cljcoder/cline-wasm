#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
#[cfg(target_arch = "wasm32")]
use {
    gloo_net::http::Request as GlooRequest,
    gloo_timers::callback::Interval,
    std::sync::{Arc, Mutex},
};

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
    #[cfg(target_arch = "wasm32")]
    current_time_title: Arc<Mutex<Option<String>>>,
    #[cfg(target_arch = "wasm32")]
    digital_clock_time: Arc<Mutex<String>>,
    #[cfg(not(target_arch = "wasm32"))]
    current_time_title: Option<String>, // For native
    #[cfg(not(target_arch = "wasm32"))]
    digital_clock_time: String, // For native

    #[cfg(target_arch = "wasm32")]
    _time_fetch_interval: Option<Interval>, // Keep interval alive
    #[cfg(target_arch = "wasm32")]
    error_message: Arc<Mutex<Option<String>>>,
}

#[cfg(target_arch = "wasm32")]
async fn fetch_time_task(
    time_display: Arc<Mutex<String>>,
    title_display: Arc<Mutex<Option<String>>>,
    ctx: egui::Context,
) {
    let location = web_sys::window().unwrap().location();
    let hostname = location.hostname().unwrap_or_else(|_| "localhost".to_string());
    let url = format!("http://{}:3000/api/time", hostname);

    match GlooRequest::get(&url).send().await {
        Ok(response) => {
            if response.ok() { // status in 200-299 range
                match response.text().await {
                    Ok(time_str) => {
                        log::info!("Fetched time: {}", time_str);
                        *time_display.lock().unwrap() = time_str.clone();
                        *title_display.lock().unwrap() = Some(time_str);
                        ctx.request_repaint(); // Request a repaint to show new time
                    }
                    Err(err) => log::error!("Failed to parse time response: {}", err),
                }
            } else {
                log::error!("Failed to fetch time: status {}", response.status());
            }
        }
        Err(err) => log::error!("Error fetching time: {}", err),
    }
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: String::new(),
            age: String::new(),
            #[cfg(target_arch = "wasm32")]
            current_time_title: Arc::new(Mutex::new(None)),
            #[cfg(target_arch = "wasm32")]
            digital_clock_time: Arc::new(Mutex::new("--:--:--".to_string())),
            #[cfg(not(target_arch = "wasm32"))]
            current_time_title: None,
            #[cfg(not(target_arch = "wasm32"))]
            digital_clock_time: "--:--:--".to_string(),
            #[cfg(target_arch = "wasm32")]
            _time_fetch_interval: None,
            #[cfg(target_arch = "wasm32")]
            error_message: Arc::new(Mutex::new(None)),
        }
    }
}

impl MyApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();

        #[cfg(target_arch = "wasm32")]
        {
            let time_display_clone = app.digital_clock_time.clone();
            let title_display_clone = app.current_time_title.clone();
            let ctx_clone = cc.egui_ctx.clone();

            // Initial fetch
            wasm_bindgen_futures::spawn_local(fetch_time_task(
                time_display_clone.clone(),
                title_display_clone.clone(),
                ctx_clone.clone(),
            ));

            // Setup interval
            let interval = Interval::new(30_000, move || {
                wasm_bindgen_futures::spawn_local(fetch_time_task(
                    time_display_clone.clone(),
                    title_display_clone.clone(),
                    ctx_clone.clone(),
                ));
            });
            app._time_fetch_interval = Some(interval);
        }
        app
    }
}

// Function to send request to backend
#[cfg(target_arch = "wasm32")]
fn trigger_server_log(name: &str, age: &str, error_message: Arc<Mutex<Option<String>>>, ctx: egui::Context) {
    let form_data = FormData {
        name: name.to_string(),
        age: age.to_string(),
    };
    wasm_bindgen_futures::spawn_local(async move {
        let location = web_sys::window().unwrap().location();
        let hostname = location.hostname().unwrap_or_else(|_| "localhost".to_string());
        let url = format!("http://{}:3000/api/log", hostname);
        log::info!("Sending request to: {}", url);

        let request_builder_result = GlooRequest::post(&url).json(&form_data);
        match request_builder_result {
            Ok(request_builder) => {
                match request_builder.send().await {
                    Ok(response) => {
                        if response.ok() {
                            log::info!("Successfully sent data to server.");
                            *error_message.lock().unwrap() = None;
                            ctx.request_repaint();
                        } else {
                            log::error!("Failed to send data: {}", response.status());
                            match response.text().await {
                                Ok(error_text) => {
                                    log::error!("Error body: {}", error_text);
                                    // Parse the JSON error response
                                    if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
                                        if let Some(error_message_text) = error_json.get("error").and_then(|v| v.as_str()) {
                                            // Store the error message in the state variable
                                            *error_message.lock().unwrap() = Some(error_message_text.to_string());
                                            ctx.request_repaint();
                                            // Clear the error message after 3 seconds
                                            gloo_timers::callback::Timeout::new(3000, move || {
                                                *error_message.lock().unwrap() = None;
                                                ctx.request_repaint();
                                            }).forget();
                                        }
                                    }
                                },
                                Err(err) => {
                                    log::error!("Failed to get error text: {}", err);
                                }
                            }
                        }
                    }
                    Err(err) => {
                        log::error!("Error sending request to server: {}", err);
                    }
                }
            }
            Err(err) => {
                log::error!("Error serializing form data to JSON: {}", err);
            }
        }
    });
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Central Panel for the digital clock
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                #[cfg(target_arch = "wasm32")]
                let time_str = self.digital_clock_time.lock().unwrap().clone();
                #[cfg(not(target_arch = "wasm32"))]
                let time_str = self.digital_clock_time.clone();
                ui.label(egui::RichText::new(&time_str).size(60.0));
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

                    // Display error message
                    #[cfg(target_arch = "wasm32")]
                    if let Some(error_message_text) = self.error_message.lock().unwrap().as_ref() {
                        ui.colored_label(egui::Color32::RED, error_message_text);
                    }

                    // Button
                    ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                        if ui.add(egui::Button::new("Click Me")
                            .fill(egui::Color32::from_rgb(0x8B, 0x00, 0x00)) // Dark red
                        ).clicked() {
                            #[cfg(target_arch = "wasm32")]
                            trigger_server_log(&self.name, &self.age, self.error_message.clone(), ctx.clone());

                            #[cfg(target_arch = "wasm32")]
                            {
                                // Manually trigger a time fetch on button click
                                wasm_bindgen_futures::spawn_local(fetch_time_task(
                                    self.digital_clock_time.clone(),
                                    self.current_time_title.clone(),
                                    ctx.clone(),
                                ));
                            }
                            
                            log::info!("Button clicked, attempting to send data: Name='{}', Age='{}'", self.name, self.age);
                            self.name.clear();
                            self.age.clear();
                        }
                    });
                });

                #[cfg(target_arch = "wasm32")]
                {
                    if let Some(new_title) = self.current_time_title.lock().unwrap().take() {
                        if let Some(window) = web_sys::window() {
                            if let Some(document) = window.document() {
                                document.set_title(&new_title);
                            }
                        }
                    }
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Some(new_title) = self.current_time_title.take() {
                         ctx.send_viewport_cmd(egui::ViewportCommand::Title(new_title));
                    }
                }
            });
    }
}
