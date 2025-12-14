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

    let app = Kiorg::new_with_config_dir(
        &cc,
        Some(temp_dir.path().to_path_buf()),
        Some(test_config_dir),
    )
    .expect("Failed to create Kiorg app");

    (app, temp_dir, config_temp_dir)
}

/// Benchmark the center panel draw method with different scenarios
fn bench_center_panel_draw(c: &mut Criterion) {
    let mut group = c.benchmark_group("center_panel_draw");
    group.sample_size(20); // Reduce from default 100 to 10 samples

    group.bench_function("baseline", |b| {
        b.iter_batched(
            || create_test_app(200),
            |(mut app, _temp_dir, _config_temp_dir)| {
                let ctx = egui::Context::default();
                let input = egui::RawInput::default();

                let _ = ctx.run(input, |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let available_rect = ui.available_rect_before_wrap();
                        let width = available_rect.width();
                        let height = available_rect.height();

                        // Call the actual center_panel::draw function
                        kiorg::ui::center_panel::draw(&mut app, ui, width, height);
                    });
                });
            },
            BatchSize::LargeInput,
        );
    });

    group.finish();
}

/// Benchmark with search functionality
fn bench_center_panel_with_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("center_panel_search");
    group.sample_size(20);

    group.bench_function("with_search", |b| {
        b.iter_batched(
            || {
                let (mut app, temp_dir, config_temp_dir) = create_test_app(200);
                // Set a search query to filter results
                app.search_bar.query = Some("file_".to_string()); // More general search to match more files
                // Clear selection to avoid index mismatch issues
                app.tab_manager.current_tab_mut().selected_index = 0;
                (app, temp_dir, config_temp_dir)
            },
            |(mut app, _temp_dir, _config_temp_dir)| {
                let ctx = egui::Context::default();
                let input = egui::RawInput::default();

                let _ = ctx.run(input, |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let available_rect = ui.available_rect_before_wrap();
                        let width = available_rect.width();
                        let height = available_rect.height();

                        // Call the actual center_panel::draw function with search active
                        kiorg::ui::center_panel::draw(&mut app, ui, width, height);
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
    targets = bench_center_panel_draw, bench_center_panel_with_search
);

criterion_main!(benches);
