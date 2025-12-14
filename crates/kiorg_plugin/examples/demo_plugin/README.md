# Demo Kiorg Plugin

This is a demo plugin demonstrating the simplified Kiorg plugin system. It provides a basic preview command that always returns "hello world".

## Building

```bash
cargo build --release
```

The plugin binary will be created at `target/release/kiorg_plugin_demo`.

## Usage

The plugin is designed to be executed by the Kiorg plugin system, but can be tested manually:

```bash
./target/release/kiorg_plugin_demo
```
