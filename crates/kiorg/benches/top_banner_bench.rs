#![allow(clippy::let_underscore_untyped)]
#![allow(dead_code)]

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use eframe::egui;
use kiorg::Kiorg;
use std::fs::File;
use std::path::PathBuf;
use tempfile::tempdir;

/// Create some test files for benchmarking
fn create_test_files(base_path: &std::path::Path, count: usize) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Create some directories
    for i in 0..count / 4 {
        let dir_path = base_path.join(format!("dir_{i:03}"));
        std::fs::create_dir(&dir_path).unwrap();
        paths.push(dir_path);
    }

    // Create some files
    for i in 0..count {
        let file_path = base_path.join(format!("file_{i:03}.txt"));
        File::create(&file_path).unwrap();
        paths.push(file_path);
    }

    paths
}

/// Create a Kiorg app instance for benchmarking
fn create_test_app(file_count: usize) -> (Kiorg, tempfile::TempDir, tempfile::TempDir) {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let config_temp_dir = tempdir().expect("Failed to create config temp directory");
    let test_config_dir = config_temp_dir.path().to_path_buf();

    // Create test files
    create_test_files(temp_dir.path(), file_count);

    // Create egui context and creation context
    let ctx = egui::Context::default();
    let cc = eframe::CreationContext::_new_kittest(ctx);

    let app = Kiorg::new(
        &cc,
        Some(temp_dir.path().to_path_buf()),
        Some(test_config_dir),
    )
    .expect("Failed to create Kiorg app");

    (app, temp_dir, config_temp_dir)
}

/// Create app with deep directory structure to test path navigation performance
fn create_app_with_deep_path(depth: usize) -> (Kiorg, tempfile::TempDir, tempfile::TempDir) {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let config_temp_dir = tempdir().expect("Failed to create config temp directory");
    let test_config_dir = config_temp_dir.path().to_path_buf();

    // Create a deep directory structure
    let mut current_path = temp_dir.path().to_path_buf();
    for i in 0..depth {
        current_path = current_path.join(format!("level_{i:03}"));
        std::fs::create_dir(&current_path).unwrap();
    }

    // Create some files in the deep directory
    for i in 0..10 {
        let file_path = current_path.join(format!("file_{i:03}.txt"));
        File::create(&file_path).unwrap();
    }

    // Create egui context and creation context
    let ctx = egui::Context::default();
    let cc = eframe::CreationContext::_new_kittest(ctx);

    let mut app = Kiorg::new(
        &cc,
        Some(temp_dir.path().to_path_buf()),
        Some(test_config_dir),
    )
    .expect("Failed to create Kiorg app");

    // Navigate to the deep path
    app.navigate_to_dir(current_path);

    (app, temp_dir, config_temp_dir)
}

/// Benchmark with deep path to stress path navigation component
fn bench_top_banner_deep_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("top_banner_deep_path");
    group.sample_size(20);

    group.bench_function("deep_path_10_levels", |b| {
        b.iter_batched(
            || create_app_with_deep_path(10),
            |(mut app, _temp_dir, _config_temp_dir)| {
                let ctx = egui::Context::default();
                let input = egui::RawInput::default();

                let _ = ctx.run(input, |ctx| {
                    egui::TopBottomPanel::top("top_banner").show(ctx, |ui| {
                        // Call the actual top_banner::draw function with deep path
                        kiorg::ui::top_banner::draw(&mut app, ui);
                    });
                });
            },
            BatchSize::LargeInput,
        );
    });

    group.bench_function("deep_path_20_levels", |b| {
        b.iter_batched(
            || create_app_with_deep_path(20),
            |(mut app, _temp_dir, _config_temp_dir)| {
                let ctx = egui::Context::default();
                let input = egui::RawInput::default();

                let _ = ctx.run(input, |ctx| {
                    egui::TopBottomPanel::top("top_banner").show(ctx, |ui| {
                        // Call the actual top_banner::draw function with very deep path
                        kiorg::ui::top_banner::draw(&mut app, ui);
                    });
                });
            },
            BatchSize::LargeInput,
        );
    });

    group.bench_function("deep_path_50_levels", |b| {
        b.iter_batched(
            || create_test_app(50),
            |(mut app, _temp_dir, _config_temp_dir)| {
                let ctx = egui::Context::default();
                let input = egui::RawInput::default();

                let _ = ctx.run(input, |ctx| {
                    egui::TopBottomPanel::top("top_banner").show(ctx, |ui| {
                        // Call the actual top_banner::draw function
                        kiorg::ui::top_banner::draw(&mut app, ui);
                    });
                });
            },
            BatchSize::LargeInput,
        );
    });

    group.finish();
}

/// Benchmark with multiple tabs to test tab rendering performance
fn bench_top_banner_multiple_tabs(c: &mut Criterion) {
    let mut group = c.benchmark_group("top_banner_multiple_tabs");
    group.sample_size(20);

    group.bench_function("multiple_tabs", |b| {
        b.iter_batched(
            || {
                let (mut app, temp_dir, config_temp_dir) = create_test_app(50);

                // Create multiple tabs by navigating to different directories
                let base_path = temp_dir.path();
                for i in 1..5 {
                    let tab_dir = base_path.join(format!("dir_{i:03}"));
                    if tab_dir.exists() {
                        app.tab_manager.add_tab(tab_dir);
                    }
                }

                (app, temp_dir, config_temp_dir)
            },
            |(mut app, _temp_dir, _config_temp_dir)| {
                let ctx = egui::Context::default();
                let input = egui::RawInput::default();

                let _ = ctx.run(input, |ctx| {
                    egui::TopBottomPanel::top("top_banner").show(ctx, |ui| {
                        // Call the actual top_banner::draw function with multiple tabs
                        kiorg::ui::top_banner::draw(&mut app, ui);
                    });
                });
            },
            BatchSize::LargeInput,
        );
    });

    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_top_banner_deep_path, bench_top_banner_multiple_tabs
);

criterion_main!(benches);
