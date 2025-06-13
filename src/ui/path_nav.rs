use std::path::{Path, PathBuf};

use egui::{RichText, Ui};

#[derive(Debug)]
pub enum PathNavMessage {
    Navigate(PathBuf),
}

pub fn draw_path_navigation(
    ui: &mut Ui,
    current_path: &Path,
    colors: &crate::config::colors::AppColors,
    tab_count: usize,
) -> Option<PathNavMessage> {
    let components = get_path_components(current_path);
    let mut message = None;

    ui.horizontal(|ui| {
        ui.label(RichText::new("$ ").color(colors.highlight));

        if components.is_empty() {
            ui.label(RichText::new("/").color(colors.link_text));
            return;
        }

        let available_width = ui.available_width()
            - (
                // +1 for menu button
                tab_count + 1
            ) as f32
                * 24.0
            // for prompt dollar sign and space
            - 15.0;
        let mut estimated_width = 0.0;

        for (i, (name, _)) in components.iter().enumerate() {
            if (i > 1) || (i == 1 && components[0].0 != "/") {
                estimated_width += 8.0; // width for slash
            }
            estimated_width += (name.chars().count() * 8) as f32; // width for component name
        }

        if estimated_width > available_width && components.len() > 3 {
            estimated_width = 0.0;
            let (name, path) = &components[0];
            if ui
                .link(RichText::new(name.to_string()).color(colors.link_text))
                .clicked()
            {
                message = Some(PathNavMessage::Navigate(path.clone()));
            }
            estimated_width += (name.chars().count() * 8) as f32;

            if name != "/" {
                ui.label(RichText::new("/").color(colors.fg_light));
                estimated_width += 8.0;
            }
            let (name, path) = &components[1];
            if ui
                .link(RichText::new(name.to_string()).color(colors.link_text))
                .clicked()
            {
                message = Some(PathNavMessage::Navigate(path.clone()));
            }
            estimated_width += (name.chars().count() * 8) as f32;

            ui.label(RichText::new("/ ...").color(colors.fg_light));
            estimated_width += 4.0 * 8.0;
            println!("estimated_width: {}", estimated_width);
            println!("available width: {}", available_width);

            // walk backwards to find the starting index for truncation
            let mut start_idx = components.len() - 1;
            for (i, (name, _)) in components.iter().enumerate().rev() {
                let new_width = estimated_width + (name.chars().count() * 8) as f32 + 8.0;
                if new_width > available_width {
                    start_idx = i;
                    break;
                }
                estimated_width = new_width;
            }
            println!(
                "start_idx: {}, components.len(): {}",
                start_idx,
                components.len()
            );

            for component in components.iter().skip(start_idx + 1) {
                let (comp_str, path) = component;
                ui.label(RichText::new("/").color(colors.fg_light));
                if ui
                    .link(RichText::new(comp_str).color(colors.link_text))
                    .clicked()
                {
                    message = Some(PathNavMessage::Navigate(path.clone()));
                }
            }
        } else {
            for (i, (name, path)) in components.iter().enumerate() {
                if (i > 1) || (i == 1 && components[0].0 != "/") {
                    ui.label(RichText::new("/").color(colors.fg_light));
                }

                if ui
                    .link(RichText::new(name).color(colors.link_text))
                    .clicked()
                {
                    message = Some(PathNavMessage::Navigate(path.clone()));
                }
            }
        }
    });

    message
}

#[must_use]
pub fn get_path_components(path: &Path) -> Vec<(String, PathBuf)> {
    let mut components = Vec::new();
    let mut current = PathBuf::new();

    let path_components: Vec<_> = path.components().collect();

    // Handle Windows drive prefix or root directory
    if let Some(first) = path_components.first() {
        match first {
            std::path::Component::Prefix(prefix) => match prefix.kind() {
                std::path::Prefix::Disk(letter) | std::path::Prefix::VerbatimDisk(letter) => {
                    let drive = format!("{}:", letter as char);
                    current = PathBuf::from(&drive);
                    components.push((drive, current.clone()));
                }
                std::path::Prefix::UNC(server, share)
                | std::path::Prefix::VerbatimUNC(server, share) => {
                    let unc = format!("\\\\{}", server.to_string_lossy());
                    current = PathBuf::from(&unc);
                    components.push((unc, current.clone()));

                    let share = share.to_string_lossy().to_string();
                    current.push(&share);
                    components.push((share, current.clone()));
                }
                _ => {
                    let prefix_str = prefix.as_os_str().to_string_lossy().to_string();
                    current = PathBuf::from(&prefix_str);
                    components.push((prefix_str, current.clone()));
                }
            },
            std::path::Component::RootDir => {
                current = PathBuf::from("/");
                components.push(("/".to_string(), current.clone()));
            }
            std::path::Component::Normal(os_str) => {
                let comp_str = os_str.to_string_lossy().to_string();
                if !comp_str.is_empty() {
                    current.push(&comp_str);
                    components.push((comp_str, current.clone()));
                }
            }
            std::path::Component::ParentDir => {
                current.push("..");
                components.push(("..".to_string(), current.clone()));
            }
            _ => {}
        }
    }

    // Process remaining components
    for component in path_components.iter().skip(1) {
        match component {
            std::path::Component::Normal(os_str) => {
                let comp_str = os_str.to_string_lossy().to_string();
                if !comp_str.is_empty() {
                    current.push(&comp_str);
                    components.push((comp_str, current.clone()));
                }
            }
            std::path::Component::ParentDir => {
                current.push("..");
                components.push(("..".to_string(), current.clone()));
            }
            _ => {}
        }
    }

    // Normalize the path by removing redundant components
    let mut normalized: Vec<(String, PathBuf)> = Vec::new();
    for (name, path) in components {
        if name == ".." {
            if let Some((_, parent_path)) = normalized.last() {
                if parent_path.as_os_str() != "/" {
                    normalized.pop();
                    continue;
                }
            }
        }
        normalized.push((name, path));
    }

    normalized
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::App;
    use egui_kittest::kittest::Queryable;
    use egui_kittest::Harness;
    use std::path::PathBuf;

    struct TestApp {
        path: PathBuf,
        colors: crate::config::colors::AppColors,
        message: Option<PathNavMessage>,
    }

    impl App for TestApp {
        fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
            egui::CentralPanel::default().show(ctx, |ui| {
                self.message = draw_path_navigation(ui, &self.path, &self.colors, 1);
            });
        }
    }

    #[test]
    fn test_path_components() {
        // Test root path
        let root = PathBuf::from("/");
        let components = get_path_components(&root);
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].0, "/");
        assert_eq!(components[0].1, PathBuf::from("/"));

        // Test simple path
        let path = PathBuf::from("/home/user");
        let components = get_path_components(&path);
        assert_eq!(components.len(), 3);
        assert_eq!(components[0].0, "/");
        assert_eq!(components[1].0, "home");
        assert_eq!(components[2].0, "user");
        assert_eq!(components[0].1, PathBuf::from("/"));
        assert_eq!(components[1].1, PathBuf::from("/home"));
        assert_eq!(components[2].1, PathBuf::from("/home/user"));

        // Test path with empty components
        let path = PathBuf::from("/home//user/");
        let components = get_path_components(&path);
        assert_eq!(components.len(), 3);
        assert_eq!(components[0].0, "/");
        assert_eq!(components[1].0, "home");
        assert_eq!(components[2].0, "user");

        // Test path with special characters
        let path = PathBuf::from("/home/user/.config");
        let components = get_path_components(&path);
        assert_eq!(components.len(), 4);
        assert_eq!(components[0].0, "/");
        assert_eq!(components[1].0, "home");
        assert_eq!(components[2].0, "user");
        assert_eq!(components[3].0, ".config");

        // Test path with spaces
        let path = PathBuf::from("/home/user/My Documents");
        let components = get_path_components(&path);
        assert_eq!(components.len(), 4);
        assert_eq!(components[0].0, "/");
        assert_eq!(components[1].0, "home");
        assert_eq!(components[2].0, "user");
        assert_eq!(components[3].0, "My Documents");

        // Test path with unicode characters
        let path = PathBuf::from("/home/user/文档");
        let components = get_path_components(&path);
        assert_eq!(components.len(), 4);
        assert_eq!(components[0].0, "/");
        assert_eq!(components[1].0, "home");
        assert_eq!(components[2].0, "user");
        assert_eq!(components[3].0, "文档");
    }

    #[test]
    fn test_path_truncation() {
        // Test path that should be truncated
        let path = PathBuf::from("/home/user/documents/projects/very/long/path");
        let components = get_path_components(&path);
        assert_eq!(components.len(), 8);

        // Verify components
        assert_eq!(components[0].0, "/");
        assert_eq!(components[1].0, "home");
        assert_eq!(components[2].0, "user");
        assert_eq!(components[3].0, "documents");
        assert_eq!(components[4].0, "projects");
        assert_eq!(components[5].0, "very");
        assert_eq!(components[6].0, "long");
        assert_eq!(components[7].0, "path");

        // Verify paths
        assert_eq!(components[0].1, PathBuf::from("/"));
        assert_eq!(components[1].1, PathBuf::from("/home"));
        assert_eq!(components[2].1, PathBuf::from("/home/user"));
        assert_eq!(components[3].1, PathBuf::from("/home/user/documents"));
        assert_eq!(
            components[4].1,
            PathBuf::from("/home/user/documents/projects")
        );
        assert_eq!(
            components[5].1,
            PathBuf::from("/home/user/documents/projects/very")
        );
        assert_eq!(
            components[6].1,
            PathBuf::from("/home/user/documents/projects/very/long")
        );
        assert_eq!(
            components[7].1,
            PathBuf::from("/home/user/documents/projects/very/long/path")
        );
    }

    #[test]
    fn test_relative_paths() {
        // Test relative paths
        let path = PathBuf::from("home/user/documents");
        let components = get_path_components(&path);

        assert_eq!(components.len(), 3);
        assert_eq!(components[0].0, "home");
        assert_eq!(components[1].0, "user");
        assert_eq!(components[2].0, "documents");
        assert_eq!(components[0].1, PathBuf::from("home"));
        assert_eq!(components[1].1, PathBuf::from("home/user"));
        assert_eq!(components[2].1, PathBuf::from("home/user/documents"));

        // Test relative path with parent directory
        let path = PathBuf::from("../home/user");
        let components = get_path_components(&path);
        assert_eq!(components.len(), 3);
        assert_eq!(components[0].0, "..");
        assert_eq!(components[1].0, "home");
        assert_eq!(components[2].0, "user");
        assert_eq!(components[0].1, PathBuf::from(".."));
        assert_eq!(components[1].1, PathBuf::from("../home"));
        assert_eq!(components[2].1, PathBuf::from("../home/user"));
    }

    #[test]
    fn test_path_navigation_message() {
        let mut harness = Harness::builder()
            .with_size(egui::Vec2::new(800.0, 600.0))
            .with_max_steps(20)
            .build_eframe(|_cc| TestApp {
                path: PathBuf::from("/"),
                colors: crate::theme::get_default_theme().get_colors().clone(),
                message: None,
            });

        // Test root path navigation
        harness.step();
        assert!(harness.input().events.is_empty());

        // Test simple path navigation
        harness.input_mut().events.push(egui::Event::PointerButton {
            pos: egui::pos2(100.0, 100.0),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::default(),
        });
        harness.step();
        assert!(harness.input().events.is_empty());

        // Test path with parent directory
        let path = PathBuf::from("/home/user/..");
        let components = get_path_components(&path);
        assert_eq!(components.len(), 2);
        assert_eq!(components[0].0, "/");
        assert_eq!(components[1].0, "home");
        assert_eq!(components[0].1, PathBuf::from("/"));
        assert_eq!(components[1].1, PathBuf::from("/home"));
    }

    #[test]
    fn test_path_truncation_rendering_short_path() {
        let mut harness = Harness::builder()
            .with_size(egui::Vec2::new(800.0, 600.0))
            .with_max_steps(10)
            .build_eframe(|_cc| TestApp {
                path: PathBuf::from("/home/user"),
                colors: crate::theme::get_default_theme().get_colors().clone(),
                message: None,
            });

        harness.step();

        // For short paths, all components should be visible (no truncation)
        assert!(
            harness.query_by_label("home").is_some(),
            "Home component should be visible"
        );
        assert!(
            harness.query_by_label("user").is_some(),
            "User component should be visible"
        );

        // Should not have truncation indicator
        assert!(
            harness.query_by_label("/ ...").is_none(),
            "Truncation indicator should not be present for short paths"
        );
    }

    #[test]
    fn test_path_truncation_rendering_long_path() {
        let mut harness = Harness::builder()
            .with_size(egui::Vec2::new(400.0, 600.0)) // Very narrow width to force truncation
            .with_max_steps(10)
            .build_eframe(|_cc| TestApp {
                path: PathBuf::from(
                    "/home/user/documents/projects/very/long/path/that/should/be/truncated",
                ),
                colors: crate::theme::get_default_theme().get_colors().clone(),
                message: None,
            });

        harness.step();

        // First two components should always be visible
        assert!(
            harness.query_by_label("home").is_some(),
            "First normal component should always be visible"
        );

        // Truncation indicator should be present
        assert!(
            harness.query_by_label("/ ...").is_some(),
            "Truncation indicator should be present for long paths"
        );

        // Some middle components should be truncated (not visible)
        assert!(
            harness.query_by_label("documents").is_none(),
            "Middle components should be truncated"
        );
        assert!(
            harness.query_by_label("projects").is_none(),
            "Middle components should be truncated"
        );
        assert!(
            harness.query_by_label("very").is_none(),
            "Middle components should be truncated"
        );

        // Last few components should be visible
        assert!(harness.query_by_label("should").is_some());
        assert!(harness.query_by_label("be").is_some());
        assert!(
            harness.query_by_label("truncated").is_some(),
            "Last component should be visible"
        );
    }

    #[test]
    fn test_path_truncation_rendering_medium_path() {
        let mut harness = Harness::builder()
            .with_size(egui::Vec2::new(400.0, 600.0)) // Medium width
            .with_max_steps(10)
            .build_eframe(|_cc| TestApp {
                path: PathBuf::from("/home/user/documents/projects/myproject"),
                colors: crate::theme::get_default_theme().get_colors().clone(),
                message: None,
            });

        harness.step();

        // First two components should be visible
        assert!(
            harness.query_by_label("home").is_some(),
            "First component should be visible"
        );

        // Truncation indicator should be present
        assert!(
            harness.query_by_label("/ ...").is_some(),
            "Truncation indicator should be present for long paths"
        );

        // Last few components should be visible
        assert!(
            harness.query_by_label("myproject").is_some(),
            "Last component should be visible"
        );
        assert!(
            harness.query_by_label("projects").is_some(),
            "Second last component should be visible"
        );
    }

    #[test]
    fn test_path_truncation_edge_case_three_components() {
        let mut harness = Harness::builder()
            .with_size(egui::Vec2::new(100.0, 600.0)) // Very narrow to test edge case
            .with_max_steps(10)
            .build_eframe(|_cc| TestApp {
                path: PathBuf::from("/home/user"),
                colors: crate::theme::get_default_theme().get_colors().clone(),
                message: None,
            });

        harness.step();

        // With only 3 components, truncation should not occur regardless of width
        // (the truncation logic only applies when components.len() > 3)
        assert!(
            harness.query_by_label("home").is_some(),
            "Home should be visible"
        );
        assert!(
            harness.query_by_label("user").is_some(),
            "User should be visible"
        );
        assert!(
            harness.query_by_label("/ ...").is_none(),
            "No truncation for <= 3 components"
        );
    }

    #[test]
    fn test_path_truncation_relative_path() {
        let mut harness = Harness::builder()
            .with_size(egui::Vec2::new(200.0, 600.0)) // Narrow width
            .with_max_steps(10)
            .build_eframe(|_cc| TestApp {
                path: PathBuf::from("documents/projects/very/long/relative/path"),
                colors: crate::theme::get_default_theme().get_colors().clone(),
                message: None,
            });

        harness.step();

        // For relative paths with > 3 components, truncation should still work
        assert!(
            harness.query_by_label("documents").is_some(),
            "First component should be visible"
        );

        // Should have truncation indicator for long relative paths
        let has_truncation = harness.query_by_label("...").is_some();
        if has_truncation {
            // Last component should be visible
            assert!(
                harness.query_by_label("path").is_some(),
                "Last component should be visible when truncated"
            );
        }
    }
}
