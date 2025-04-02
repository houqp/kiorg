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
) -> Option<PathNavMessage> {
    let components = get_path_components(current_path);
    let mut message = None;

    ui.horizontal(|ui| {
        ui.label(RichText::new("$ ").color(colors.gray));

        if components.is_empty() {
            ui.label(RichText::new("/").color(colors.yellow));
        } else {
            let mut path_str = String::new();
            for (i, (name, _)) in components.iter().enumerate() {
                if i > 0 {
                    path_str.push('/');
                }
                path_str.push_str(name);
            }

            let available_width = ui.available_width();
            let estimated_width = path_str.len() as f32 * 7.0;

            if estimated_width > available_width && components.len() > 4 {
                if ui.link(RichText::new(&components[0].0).color(colors.yellow)).clicked() {
                    message = Some(PathNavMessage::Navigate(components[0].1.clone()));
                }

                ui.label(RichText::new("/").color(colors.gray));
                ui.label(RichText::new("...").color(colors.gray));

                let start_idx = components.len() - 2;
                for component in components.iter().skip(start_idx) {
                    let (comp_str, path) = component;
                    ui.label(RichText::new("/").color(colors.gray));
                    if ui.link(RichText::new(comp_str).color(colors.yellow)).clicked() {
                        message = Some(PathNavMessage::Navigate(path.clone()));
                    }
                }
            } else {
                for (i, (name, path)) in components.iter().enumerate() {
                    if (i > 1) || (i == 1 && components[0].0 != "/") {
                        ui.label(RichText::new("/").color(colors.gray));
                    }

                    if ui.link(RichText::new(name).color(colors.yellow)).clicked() {
                        message = Some(PathNavMessage::Navigate(path.clone()));
                    }
                }
            }
        }
    });

    message
}

pub fn get_path_components(path: &Path) -> Vec<(String, PathBuf)> {
    let mut components = Vec::new();
    let mut current = PathBuf::new();

    let path_components: Vec<_> = path.components().collect();

    // Handle Windows drive prefix or root directory
    if let Some(first) = path_components.first() {
        match first {
            std::path::Component::Prefix(prefix) => {
                match prefix.kind() {
                    std::path::Prefix::Disk(letter) | std::path::Prefix::VerbatimDisk(letter) => {
                        let drive = format!("{}:", letter as char);
                        current = PathBuf::from(&drive);
                        components.push((drive, current.clone()));
                    }
                    std::path::Prefix::UNC(server, share) | std::path::Prefix::VerbatimUNC(server, share) => {
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
                }
            }
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
    use std::path::PathBuf;
    use egui_kittest::Harness;
    use eframe::App;

    struct TestApp {
        path: PathBuf,
        colors: crate::config::colors::AppColors,
        message: Option<PathNavMessage>,
    }

    impl App for TestApp {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            egui::CentralPanel::default().show(ctx, |ui| {
                self.message = draw_path_navigation(ui, &self.path, &self.colors);
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
        assert_eq!(components[4].1, PathBuf::from("/home/user/documents/projects"));
        assert_eq!(components[5].1, PathBuf::from("/home/user/documents/projects/very"));
        assert_eq!(components[6].1, PathBuf::from("/home/user/documents/projects/very/long"));
        assert_eq!(components[7].1, PathBuf::from("/home/user/documents/projects/very/long/path"));
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
                colors: crate::config::colors::AppColors::default(),
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
} 