use std::path::PathBuf;
use std::time::Duration;
use egui::{Context, InputState, Event, Key};
use crate::app::Kiorg;
use crate::models::dir_entry::DirEntry;

fn setup_test_app() -> (Kiorg, Context) {
    let ctx = Context::default();
    let initial_dir = PathBuf::from("test_dir");
    let app = Kiorg::new(&eframe::CreationContext::_new_kittest(ctx.clone()), initial_dir);
    (app, ctx)
}

fn simulate_key_press(ctx: &Context, key: Key, shift: bool) {
    let mut input = InputState::default();
    input.modifiers.shift = shift;
    input.events.push(Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers: input.modifiers,
    });
    
    // Replace the entire input state
    ctx.input_mut(|i| {
        i.events = input.events;
        i.modifiers = input.modifiers;
    });
}

#[test]
fn test_g_shortcuts() {
    let (mut app, ctx) = setup_test_app();
    
    // Add some test entries
    {
        let tab = app.tab_manager.current_tab();
        tab.entries = vec![
            DirEntry {
                name: "file1".to_string(),
                path: PathBuf::from("test_dir/file1"),
                is_dir: false,
                modified: std::time::SystemTime::now(),
                size: 100,
            },
            DirEntry {
                name: "file2".to_string(),
                path: PathBuf::from("test_dir/file2"),
                is_dir: false,
                modified: std::time::SystemTime::now(),
                size: 200,
            },
            DirEntry {
                name: "file3".to_string(),
                path: PathBuf::from("test_dir/file3"),
                is_dir: false,
                modified: std::time::SystemTime::now(),
                size: 300,
            },
        ];
    }

    // Test G shortcut (go to last entry)
    {
        simulate_key_press(&ctx, Key::G, true);
        app.handle_key_press(&ctx);
        let tab = app.tab_manager.current_tab();
        assert_eq!(tab.selected_index, 2); // Should be at the last entry
    }

    // Test gg shortcut (go to first entry)
    {
        // First g press
        simulate_key_press(&ctx, Key::G, false);
        app.handle_key_press(&ctx);

        // Wait a bit less than 500ms
        std::thread::sleep(Duration::from_millis(400));

        // Second g press
        simulate_key_press(&ctx, Key::G, false);
        app.handle_key_press(&ctx);

        let tab = app.tab_manager.current_tab();
        assert_eq!(tab.selected_index, 0); // Should be at the first entry
    }

    // Test that single g press doesn't move selection
    {
        simulate_key_press(&ctx, Key::G, false);
        app.handle_key_press(&ctx);

        // Wait more than 500ms
        std::thread::sleep(Duration::from_millis(600));

        let tab = app.tab_manager.current_tab();
        assert_eq!(tab.selected_index, 0);
    }
}

#[test]
fn test_g_shortcuts_empty_list() {
    let (mut app, ctx) = setup_test_app();
    
    // Clear entries
    {
        let tab = app.tab_manager.current_tab();
        tab.entries.clear();
    }

    // Test G shortcut with empty list
    {
        simulate_key_press(&ctx, Key::G, true);
        app.handle_key_press(&ctx);
        let tab = app.tab_manager.current_tab();
        assert_eq!(tab.selected_index, 0); // Should stay at 0
    }

    // Test gg shortcut with empty list
    {
        // First g press
        simulate_key_press(&ctx, Key::G, false);
        app.handle_key_press(&ctx);

        // Second g press
        simulate_key_press(&ctx, Key::G, false);
        app.handle_key_press(&ctx);

        let tab = app.tab_manager.current_tab();
        assert_eq!(tab.selected_index, 0); // Should stay at 0
    }
} 