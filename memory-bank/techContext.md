## Tech Context

### Technologies used
*   Rust
*   egui
*   TOML

### Development setup
*   Rust toolchain
*   Cargo package manager

### Technical constraints
*   egui is a relatively new UI framework, so there may be limitations or bugs.
*   Cross-platform compatibility may require platform-specific code.
*   Configuration files need to maintain backward compatibility as new preferences are added.

### Dependencies
*   eframe = "0.31.1"
*   egui = "0.31.1"
*   chrono = "0.4.40"
*   humansize = "2.1.3"
*   toml = "0.8.8"
*   serde = { version = "1.0.193", features = ["derive"] }
*   dirs-next = "2.0.0"
*   open = "4.0"
*   pdfium-render = "0"
*   image = "0.24.8"
*   clap = { version = "4.5.1", features = ["derive"] }
*   egui_kittest = { version = "0.31.1", features = ["eframe"] } (dev dependency)
*   tempfile = "3.8" (dev dependency)

### Tool usage patterns
*   `cargo build` to build the application.
*   `cargo run` to run the application.
*   `cargo test` to run the tests.
*   `cargo clippy` to check for code style issues.
*   Always run `cargo clippy` after making changes to ensure code quality and adherence to Rust best practices.
