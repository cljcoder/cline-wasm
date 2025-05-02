# Cline WASM Project

This project demonstrates a simple Rust application compiled to WebAssembly (Wasm) and run in a web browser.

## Prerequisites

Before you begin, ensure you have the following installed:

1.  **Rust Toolchain:** Install Rust using `rustup` from [https://rustup.rs/](https://rustup.rs/). This includes `cargo`.
2.  **wasm-pack:** Install `wasm-pack` for building Rust Wasm projects:
    ```bash
    cargo install wasm-pack
    ```
3.  **Local Web Server:** You'll need a simple HTTP server to serve the files locally. If you have Python installed, you can use its built-in server. Alternatively, you can install one like `miniserve`:
    ```bash
    cargo install miniserve
    ```

## Compilation

1.  **Clone the repository:**
    ```bash
    # Replace with the actual repository URL
    git clone <repository-url>
    cd cline-wasm
    ```
2.  **Build the Wasm package:** Navigate to the project's root directory (where `Cargo.toml` is located) and run:
    ```bash
    wasm-pack build --target web
    ```
    This command compiles the Rust code into Wasm and generates necessary JavaScript bindings in a `pkg/` directory.

## Running the Application

1.  **Start a local web server:** From the project's root directory, start a web server to serve the `index.html` file and the `pkg/` directory.

    *   **Using Python 3:**
        ```bash
        python -m http.server
        ```
    *   **Using `miniserve`:**
        ```bash
        miniserve . --index index.html
        ```
    *   **Other servers:** Use your preferred local web server, ensuring it serves the root directory.

2.  **Open in Browser:** Open your web browser and navigate to the address provided by the server (usually `http://localhost:8000` or `http://127.0.0.1:8080`). You should see the application running.

## Development with Trunk and Backend Server

This project now includes a simple backend server for handling API requests from the frontend. For development, you'll need to run both the backend server and the `trunk` development server.

1.  **Install Prerequisites:**
    *   Ensure you have the Rust toolchain installed ([https://rustup.rs/](https://rustup.rs/)).
    *   Install `trunk` and `wasm-bindgen-cli`:
        ```bash
        cargo install --locked trunk
        cargo install wasm-bindgen-cli
        ```

2.  **Run the Backend Server:** Open a terminal in the project's root directory and run the backend server, explicitly enabling the required features:
    ```bash
    cargo run --bin server --features axum,tokio,tower-http
    ```
    This will compile and start the backend server, which listens on `http://localhost:3000`. Keep this terminal running.

3.  **Run the Frontend Dev Server:** Open a *second* terminal in the project's root directory and run `trunk`:
    ```bash
    trunk serve --open
    ```
    This command will:
    *   Build the Wasm frontend.
    *   Start the `trunk` development server (usually on `http://localhost:8080`).
    *   Proxy requests starting with `/api/` to the backend server running on port 3000 (as configured in `Trunk.toml`).
    *   Automatically rebuild and reload the page when frontend code changes.
    *   The `--open` flag will automatically open the application in your default web browser.

4.  **Test the Interaction:**
    *   Once the page loads in the browser, click the "Click Me" button.
    *   Check the terminal where you ran `cargo run --bin server`. You should see the message "I am server" printed each time you click the button.
    *   You will also see log messages in the browser's developer console.
