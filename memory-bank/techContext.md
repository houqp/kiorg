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
*   font-kit = "0"
*   chrono = "0.4.40"
*   humansize = "2.1.3"
*   toml = "0.8.8"
*   serde = { version = "1.0.193", features = ["derive"] }
*   serde_json = "1.0"
*   dirs = "6"
*   open = "4.0"
*   clap = { version = "4.5.1", features = ["derive"] }
*   egui_term = { git = "https://github.com/houqp/egui_term.git", rev = "b8b4a1b79e10e07bd64d8104449e8904b466b3cd", optional = true }
*   notify = "8"
*   egui_extras = { version = "0", features = ["all_loaders"] }
*   zip = "4"
*   egui-notify = "0"
*   epub = "2"
*   file_type = "0"
*   tracing = "0.1"
*   tracing-subscriber = { version = "0.3", features = ["env-filter"] }
*   image = { version = "0" }
*   kamadak-exif = "0"
*   pdf_render = { git = "https://github.com/pdf-rs/pdf_render.git", rev = "9a31988b091495c65a54236fd6cdce8f1fa2afd0", features = ["embed"] }
*   pdf = { git = "https://github.com/pdf-rs/pdf" }
*   pathfinder_geometry = { git = "https://github.com/servo/pathfinder" }
*   pathfinder_export = { git = "https://github.com/servo/pathfinder" }
*   criterion = "0.5" (dev dependency for benchmarking)
*   egui_kittest = { version = "0.31.1", features = ["eframe"] } (dev dependency)
*   tempfile = "3.8" (dev dependency)

### Tool usage patterns
*   `cargo build` to build the application.
*   `cargo run` to run the application.
*   `cargo test` to run the tests.
*   `cargo clippy` to check for code style issues.
*   `cargo bench` to run performance benchmarks.
*   Always run `cargo clippy` after making changes to ensure code quality and adherence to Rust best practices.
*   Use benchmarks to measure performance improvements and identify bottlenecks.
*   Features can be toggled with cargo: `cargo build --features terminal` or `cargo build --features debug`.
