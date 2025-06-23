## Tech Context

### Technologies used
*   Rust
*   egui
*   TOML

### Development setup
*   Rust toolchain
*   Cargo package manager
*   To regenerate screenshots: `UPDATE_SNAPSHOTS=1 cargo test --features=snapshot`

### Technical constraints
*   egui is a relatively new UI framework, so there may be limitations or bugs.
*   Cross-platform compatibility may require platform-specific code.
*   Configuration files need to maintain backward compatibility as new preferences are added.

### Dependencies

See `Cargo.toml`.