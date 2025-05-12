# Cline WASM Project

This project demonstrates a simple Rust application compiled to WebAssembly (Wasm) and run in a web browser.

## Prerequisites

Before you begin, ensure you have the following installed:

## Configuration

This project uses a `.env` file to store sensitive information such as database connection details. After checking out the repository, you will need to copy the `.env.example` file to `.env` and enter your database connection details.

1.  **Rust Toolchain & Wasm Target:**
    *   Install Rust using `rustup` from [https://rustup.rs/](https://rustup.rs/). This includes `cargo`.
    *   Add the Wasm compilation target required for building the frontend:
        ```bash
        rustup target add wasm32-unknown-unknown
        ```

## Manual Build & Run (Alternative)

This method uses `wasm-pack` to build the frontend assets, which can then be served by any static file server. Note that this method **does not** automatically run the backend server or handle API proxying like the recommended `trunk` development method.

1.  **Install Additional Prerequisites:**
    *   Install `wasm-pack`:
        ```bash
        cargo install wasm-pack
        ```
    *   Ensure you have a local web server (e.g., Python's built-in server or `miniserve`):
        ```bash
        # Example using miniserve
        cargo install miniserve
        ```
2.  **Clone the Repository:**
    ```bash
    # Replace with the actual repository URL
    git clone <repository-url>
    cd cline-wasm
    ```
3.  **Build the Wasm Package:** Navigate to the project's root directory and run:
    ```bash
    wasm-pack build --target web
    ```
    This command compiles the Rust code into Wasm and generates necessary JavaScript bindings in a `pkg/` directory.

4.  **Run the Backend Server (Manually):** If you need the backend functionality, you still need to run it separately as described in the Development section (which will be added next):
    ```bash
    cargo run --bin server --features axum,tokio,tower-http,chrono
    ```

5.  **Start a Local Web Server:** From the project's root directory, start a web server to serve the `index.html` file and the `pkg/` directory.
    *   **Using Python 3:**
        ```bash
        python -m http.server 8080
        ```
    *   **Using `miniserve`:**
        ```bash
        miniserve . --index index.html -p 8080
        ```
    *   **Important:** When running this way, the frontend code making the request to `http://localhost:8080/api/log` will **fail** unless you manually adjust the URL in `src/main.rs` to point directly to the backend (`http://localhost:3000/api/log`) and handle potential CORS issues, as `trunk`'s proxy is not involved.

6.  **Open in Browser:** Open your web browser and navigate to `http://localhost:8080`.

## Development (Recommended)

This method uses `trunk` for a streamlined development experience with auto-reloading and proxying to the backend. It assumes you have completed the main **Prerequisites** section (Rust toolchain and Wasm target installed).

1.  **Install Development Tools:** Install `trunk` and `wasm-bindgen-cli`:
    ```bash
    cargo install --locked trunk
    cargo install wasm-bindgen-cli
    ```
2.  **Clone the Repository:**
    ```bash
    # Replace with the actual repository URL
    git clone <repository-url>
    cd cline-wasm
    ```
3.  **Run the Backend Server:** Open a terminal in the project's root directory and run the backend server, explicitly enabling the required features:
    ```bash
    cargo run --bin server --features axum,tokio,tower-http,chrono
    ```
    This will compile and start the backend server, which listens on `http://localhost:3000`. Keep this terminal running.

4.  **Run the Frontend Dev Server:** Open a *second* terminal in the project's root directory and run `trunk`:
    ```bash
    # For local development only:
    # trunk serve --open

    # To allow access from other devices on your network (recommended for testing on different hosts):
    trunk serve --address 0.0.0.0 --port 8080 --open
    ```
    This command will:
    *   Build the Wasm frontend.
    *   Start the `trunk` development server (listening on the specified address and port, e.g., `http://0.0.0.0:8080`).
    *   The frontend application now connects directly to the backend server (running on port 3000). The previous proxy configuration in `Trunk.toml` has been removed as the frontend dynamically determines the backend URL.
    *   Automatically rebuild and reload the page when frontend code changes.
    *   The `--open` flag (optional) will attempt to open the application in your default web browser.

5.  **Test the Interaction:**
    *   Once the page loads in the browser (e.g., at `http://<your-local-ip>:8080/`), enter some data and click the "Click Me" button.
    *   Check the terminal where you ran `cargo run --bin server`. You should see a message similar to "Received data: Name = [your_name], Age = [your_age]" printed each time you click the button.
    *   You will also see log messages in the browser's developer console.
