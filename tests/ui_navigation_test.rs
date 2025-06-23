#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::Key;
use kiorg::models::preview_content::PreviewContent;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files, create_test_image, create_test_zip};

#[test]
fn test_g_shortcuts() {
    // Create test files and directories
    let temp_dir = tempdir().unwrap();
    create_test_files(&[
        temp_dir.path().join("dir1"),
        temp_dir.path().join("dir2"),
        temp_dir.path().join("test1.txt"),
        temp_dir.path().join("test2.txt"),
    ]);
    let mut harness = create_harness(&temp_dir);

    let tab = harness.state().tab_manager.current_tab_ref();
    assert_eq!(tab.selected_index, 0);

    // Test G shortcut (go to last entry)
    {
        let modifiers = egui::Modifiers {
            shift: true,
            ..Default::default()
        };
        harness.press_key_modifiers(modifiers, Key::G);
        harness.step();
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.selected_index, tab.entries.len() - 1);
    }

    // a single g press doesn't move selection
    {
        let modifiers = egui::Modifiers {
            shift: false,
            ..Default::default()
        };
        harness.press_key_modifiers(modifiers, Key::G);
        harness.step();
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.selected_index, tab.entries.len() - 1);
    }

    // Test gg shortcut (go to first entry)
    {
        // Second g press should go back to the top
        harness.press_key(Key::G);
        harness.step();
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.selected_index, 0);
    }
}

#[test]
fn test_g_shortcuts_empty_list() {
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    // Clear entries
    {
        let tab = harness.state_mut().tab_manager.current_tab_mut();
        tab.entries.clear();
    }

    // Test G shortcut with empty list
    {
        let modifiers = egui::Modifiers {
            shift: true,
            ..Default::default()
        };
        harness.press_key_modifiers(modifiers, Key::G);
        harness.step();
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.selected_index, 0); // Should stay at 0
    }

    // Test gg shortcut with empty list
    {
        // First g press
        harness.press_key(Key::G);
        // Second g press
        harness.press_key(Key::G);
        harness.step();
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.selected_index, 0); // Should stay at 0
    }
}

#[test]
fn test_parent_directory_selection() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test files and directories
    let test_files = create_test_files(&[
        temp_dir.path().join("dir1"),
        temp_dir.path().join("dir2"),
        temp_dir.path().join("test1.txt"),
        temp_dir.path().join("test2.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Move down to select dir2
    harness.press_key(Key::J);
    harness.step();

    // Navigate into dir2
    harness.press_key(Key::L);
    harness.step();

    // Create a file in dir2
    let dir2_file = test_files[1].join("dir2_file.txt");
    std::fs::File::create(&dir2_file).unwrap();

    // Move down to select dir2_file.txt
    harness.press_key(Key::J);
    harness.step();

    // Navigate to parent directory
    harness.press_key(Key::H);
    harness.step();

    // Verify that dir2 is still selected
    let tab = harness.state().tab_manager.current_tab_ref();
    assert_eq!(
        tab.entries[tab.selected_index].path, test_files[1],
        "dir2 should be selected after navigating to parent directory"
    );
}

#[test]
fn test_parent_directory_with_minus_key() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test files and directories
    let test_files = create_test_files(&[
        temp_dir.path().join("dir1"),
        temp_dir.path().join("dir2"),
        temp_dir.path().join("test1.txt"),
        temp_dir.path().join("test2.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Move down to select dir2
    harness.press_key(Key::J);
    harness.step();

    // Navigate into dir2
    harness.press_key(Key::L);
    harness.step();

    // Create a file in dir2
    let dir2_file = test_files[1].join("dir2_file.txt");
    std::fs::File::create(&dir2_file).unwrap();

    // Move down to select dir2_file.txt
    harness.press_key(Key::J);
    harness.step();

    // Navigate to parent directory using the minus key
    harness.press_key(Key::Minus);
    harness.step();

    // Verify that dir2 is still selected
    let tab = harness.state().tab_manager.current_tab_ref();
    assert_eq!(
        tab.entries[tab.selected_index].path, test_files[1],
        "dir2 should be selected after navigating to parent directory with minus key"
    );
}

#[test]
fn test_prev_path_selection_with_sort() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test directories (order matters for initial selection)
    let test_dirs = create_test_files(&[
        temp_dir.path().join("aaa"), // index 0
        temp_dir.path().join("bbb"), // index 1
        temp_dir.path().join("ccc"), // index 2
    ]);

    // Start the harness in the parent directory
    let mut harness = create_harness(&temp_dir);

    // Initial state: aaa, bbb, ccc (now explicitly sorted Name/Ascending)
    // Select ccc (index 2)
    harness.press_key(Key::J);
    harness.step();
    harness.press_key(Key::J);
    harness.step();
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        2
    );

    // Navigate into bbb
    harness.press_key(Key::L);
    harness.step();
    assert!(harness
        .state()
        .tab_manager
        .current_tab_ref()
        .current_path
        .ends_with("ccc"));

    // Manually set sort order to Descending Name *while inside bbb*
    // (Simulating header click is complex, direct state change is acceptable here)
    {
        let tab_manager = &mut harness.state_mut().tab_manager;
        tab_manager.toggle_sort(kiorg::models::tab::SortColumn::Name); // Sets to None
        tab_manager.toggle_sort(kiorg::models::tab::SortColumn::Name); // Sets Name/Descending
    }
    harness.step(); // Allow state update propagation if needed

    // Navigate back up to the parent directory
    harness.press_key(Key::H);
    harness.step();

    // Now in the parent directory, refresh_entries should have run:
    // 1. Entries read: [aaa, bbb, ccc]
    // 2. Sort applied (Name/Descending): [ccc, bbb, aaa]
    // 3. prev_path (bbb) searched in sorted list
    // 4. selected_index should be 1 (pointing to bbb)

    // Verify the state in the parent directory
    let tab = harness.state().tab_manager.current_tab_ref();
    assert_eq!(
        tab.current_path,
        temp_dir.path(),
        "Current path should be the parent"
    );
    assert_eq!(tab.entries.len(), 3, "Should have 3 entries");

    // Check sorted order
    assert_eq!(tab.entries[0].name, "ccc", "First entry should be ccc");
    assert_eq!(tab.entries[1].name, "bbb", "Second entry should be bbb");
    assert_eq!(tab.entries[2].name, "aaa", "Third entry should be aaa");

    // Check selected index based on prev_path (bbb)
    assert_eq!(tab.selected_index, 0, "Selected index should point to ccc");
    assert_eq!(
        tab.entries[tab.selected_index].path, test_dirs[2],
        "Selected entry should be ccc"
    );
}

#[test]
fn test_mouse_click_selects_and_previews() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test files
    let test_files = create_test_files(&[
        temp_dir.path().join("a.txt"), // index 0
        temp_dir.path().join("b.txt"), // index 1
        temp_dir.path().join("c.txt"), // index 2
    ]);

    // Create some content for b.txt to ensure preview is generated
    std::fs::write(&test_files[1], "Content of b.txt").unwrap();

    // Start the harness
    let mut harness = create_harness(&temp_dir);

    // Initially, index 0 ("a.txt") should be selected
    harness.press_key(Key::J);
    harness.step();
    harness.press_key(Key::K);
    harness.step();
    // step twice for the refresh function to be triggered
    harness.step();

    println!(
        "cached preview path: {:?}",
        harness.state().cached_preview_path
    );
    // Preview cache should be empty or contain preview for a.txt initially
    // (Depending on initial load behavior, let's ensure it doesn't contain b.txt yet)
    assert!(
        harness.state().cached_preview_path != Some(test_files[1].clone()),
        "Preview path should not be b.txt yet"
    );

    // --- Simulate Click on the second entry ("b.txt") ---
    // Calculate the bounding box for the second row (index 1)
    // all the heights and widths are emprically determined from actual UI run
    let header_height = kiorg::ui::style::HEADER_ROW_HEIGHT; // Header row
    let row_height = kiorg::ui::file_list::ROW_HEIGHT; // Entry row
    let banner_height = 27.0;
    let target_y = 2.0f32.mul_add(row_height, banner_height + header_height) + (row_height / 2.0); // Click in the middle of the second entry row
    let target_pos = egui::pos2(200.0, target_y); // Click somewhere within the row horizontally

    // Simulate a primary mouse button click (press and release)
    harness.input_mut().events.push(egui::Event::PointerButton {
        pos: target_pos,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::default(),
    });
    harness.input_mut().events.push(egui::Event::PointerButton {
        pos: target_pos,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::default(),
    });

    harness.step(); // Process the click and update state
    harness.step();

    // --- Assertions ---
    // 1. Check if the selected index is updated to 1 ("b.txt")
    let tab = harness.state().tab_manager.current_tab_ref();
    assert_eq!(
        tab.selected_index, 1,
        "Selected index should be 1 after clicking the second entry"
    );
    assert_eq!(
        tab.entries[tab.selected_index].path, test_files[1],
        "Selected entry should be b.txt"
    );

    // 2. Check if the preview cache now contains the preview for "b.txt"
    // The preview update happens in the right_panel draw phase, which runs during step()
    assert_eq!(
        harness.state().cached_preview_path,
        Some(test_files[1].clone()),
        "Cached preview path should be b.txt after selection"
    );
    // wait for preview to be completed
    for _ in 0..100 {
        harness.step();
        if let Some(PreviewContent::Text(_)) = harness.state().preview_content {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    match &harness.state().preview_content {
        Some(PreviewContent::Text(text)) => {
            assert!(
                text.contains("Content of b.txt"),
                "Preview content should contain 'Content of b.txt'"
            );
        }

        Some(PreviewContent::Image(_)) => {
            panic!("Preview content should be Text variant, not Image")
        }
        Some(PreviewContent::Zip(_)) => {
            panic!("Preview content should be Text variant, not Zip")
        }
        Some(PreviewContent::Pdf(_)) => {
            panic!("Preview content should be Text variant, not PDF")
        }
        Some(PreviewContent::Epub(_)) => {
            panic!("Preview content should be Text variant, not EPUB")
        }
        Some(PreviewContent::Directory(_)) => {
            panic!("Preview content should be Text variant, not Directory")
        }
        Some(other) => {
            panic!("Preview content should be Text variant, got {other:?}");
        }
        None => panic!("Preview content should not be None"),
    }
}

#[test]
fn test_enter_directory() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test directories and files
    create_test_files(&[
        temp_dir.path().join("dir1"),
        temp_dir.path().join("test1.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Select the directory
    {
        let tab = harness.state_mut().tab_manager.current_tab_mut();
        tab.selected_index = 0; // Select dir1
    }
    harness.step();

    // Get the current path before double-click
    let current_path = harness
        .state()
        .tab_manager
        .current_tab_ref()
        .current_path
        .clone();
    let expected_path = current_path.join("dir1");

    // Simulate a double-click by using Enter key (which has the same effect)
    // This is a simplification since egui_kittest doesn't easily support double-click simulation
    harness.press_key(Key::Enter);
    harness.step();

    // Verify that we navigated to the directory
    let new_path = harness
        .state()
        .tab_manager
        .current_tab_ref()
        .current_path
        .clone();
    assert_eq!(
        new_path, expected_path,
        "Should have navigated to the directory"
    );
}

#[test]
fn test_image_preview() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test files including an image
    let image_path = temp_dir.path().join("test.png");
    create_test_image(&image_path);

    // Create a text file for comparison
    let text_path = temp_dir.path().join("test.txt");
    std::fs::write(&text_path, "This is a text file").unwrap();

    // Start the harness
    let mut harness = create_harness(&temp_dir);

    // Select the image file
    harness.press_key(Key::J);
    harness.step();
    harness.press_key(Key::K);
    harness.step();

    for _ in 0..100 {
        harness.step();
        if let Some(PreviewContent::Image(_)) = &harness.state().preview_content {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Check if the preview content is an image
    match &harness.state().preview_content {
        Some(PreviewContent::Image(image_meta)) => {
            // Verify that metadata is present
            assert!(
                !image_meta.metadata.is_empty(),
                "Image metadata should not be empty"
            );
        }
        Some(PreviewContent::Loading(..)) => {
            // Wait for loading to complete
            for _ in 0..100 {
                harness.step();
                if let Some(PreviewContent::Image(_)) = &harness.state().preview_content {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            // Check again after waiting
            match &harness.state().preview_content {
                Some(PreviewContent::Image(image_meta)) => {
                    // Verify that metadata is present
                    assert!(
                        !image_meta.metadata.is_empty(),
                        "Image metadata should not be empty"
                    );
                }
                _ => {
                    panic!("Preview content should be Image variant after loading completes");
                }
            }
        }
        Some(PreviewContent::Pdf(_)) => {
            panic!("Preview content should be Image variant, not PDF");
        }
        Some(PreviewContent::Epub(_)) => {
            panic!("Preview content should be Image variant, not EPUB");
        }
        Some(PreviewContent::Directory(_)) => {
            panic!("Preview content should be Image variant, not Directory");
        }
        Some(other) => {
            panic!("Preview content should be Image variant, got {other:?}");
        }
        None => panic!("Preview content should not be None"),
    }
}

#[test]
fn test_zip_preview() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test files including a zip file
    let zip_path = temp_dir.path().join("test.zip");
    create_test_zip(&zip_path);

    // Create a text file for comparison
    let text_path = temp_dir.path().join("test.txt");
    std::fs::write(&text_path, "This is a text file").unwrap();

    // Start the harness
    let mut harness = create_harness(&temp_dir);

    // select the zip file
    harness.press_key(Key::J);
    harness.step();

    // Check if the preview content is a zip or loading
    let mut is_zip_content = false;

    // Try multiple steps to allow async loading to complete
    for _ in 0..100 {
        match &harness.state().preview_content {
            Some(PreviewContent::Zip(entries)) => {
                // Verify zip entries
                assert!(!entries.is_empty(), "Zip entries should not be empty");

                // Check for expected files
                let file1 = entries.iter().find(|e| e.name == "file1.txt");
                let file2 = entries.iter().find(|e| e.name == "file2.txt");
                let subdir = entries.iter().find(|e| e.name == "subdir/" && e.is_dir);

                assert!(file1.is_some(), "file1.txt should be in the zip entries");
                assert!(file2.is_some(), "file2.txt should be in the zip entries");
                assert!(subdir.is_some(), "subdir/ should be in the zip entries");

                is_zip_content = true;
                break;
            }
            Some(_) => {
                // Still loading, try another step
                std::thread::sleep(std::time::Duration::from_millis(10));
                harness.step();
            }
            None => panic!("Preview content should not be None"),
        }
    }

    assert!(
        is_zip_content,
        "Preview content should eventually be Zip variant"
    );
}

#[test]
fn test_open_directory_vs_open_directory_or_file() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test files and directories
    let test_files = create_test_files(&[
        temp_dir.path().join("dir1"),
        temp_dir.path().join("test1.txt"),
    ]);

    // Write some content to the text file
    std::fs::write(&test_files[1], "Test file content").unwrap();

    // Start the harness
    let mut harness = create_harness(&temp_dir);

    // Test 1: OpenDirectory should not open a file
    {
        // Select the text file (index 1)
        {
            let tab = harness.state_mut().tab_manager.current_tab_mut();
            tab.selected_index = 1; // Select test1.txt
        }
        harness.step();

        // Get the current path before attempting to open
        let current_path = harness
            .state()
            .tab_manager
            .current_tab_ref()
            .current_path
            .clone();

        // Press 'l' key which is mapped to OpenDirectory
        harness.press_key(Key::L);
        harness.step();

        // Verify that the current path hasn't changed (file wasn't opened)
        let new_path = harness
            .state()
            .tab_manager
            .current_tab_ref()
            .current_path
            .clone();
        assert_eq!(
            new_path, current_path,
            "OpenDirectory should not change the current path when selecting a file"
        );

        // Verify that the file is not in the files_being_opened map
        assert!(
            harness.state().files_being_opened.is_empty(),
            "No files should be in the files_being_opened map"
        );
    }

    // Test 2: OpenDirectoryOrFile should open a file
    {
        // Select the text file (index 1)
        {
            let tab = harness.state_mut().tab_manager.current_tab_mut();
            tab.selected_index = 1; // Select test1.txt
        }
        harness.step();

        // Press Enter key which is mapped to OpenDirectoryOrFile
        harness.press_key(Key::Enter);
        harness.step();

        // Check that the specific file is being opened
        let file_path = &test_files[1];
        assert!(
            harness.state().files_being_opened.contains_key(file_path),
            "The specific file should be in the files_being_opened map"
        );
    }

    // Test 3: OpenDirectoryOrFile should also open a directory
    {
        // Select the directory (index 0)
        {
            let tab = harness.state_mut().tab_manager.current_tab_mut();
            tab.selected_index = 0; // Select dir1
        }
        harness.step();

        // Get the current path before attempting to open
        let current_path = harness
            .state()
            .tab_manager
            .current_tab_ref()
            .current_path
            .clone();
        let expected_path = current_path.join("dir1");

        // Press Enter key which is mapped to OpenDirectoryOrFile
        harness.press_key(Key::Enter);
        harness.step();

        // Verify that the current path has changed to the directory
        let new_path = harness
            .state()
            .tab_manager
            .current_tab_ref()
            .current_path
            .clone();
        assert_eq!(
            new_path, expected_path,
            "OpenDirectoryOrFile should change the current path when selecting a directory"
        );
    }
}

#[test]
fn test_page_navigation() {
    // Create a directory with many files to test page navigation
    let temp_dir = tempdir().unwrap();
    let mut test_files = Vec::new();

    // Create 25 test files to ensure we have enough for page navigation
    for i in 0..25 {
        test_files.push(temp_dir.path().join(format!("file_{i:02}.txt")));
    }
    create_test_files(&test_files);

    let mut harness = create_harness(&temp_dir);

    // Initially should be at first entry
    let tab = harness.state().tab_manager.current_tab_ref();
    let initial_selected = tab.selected_index;
    assert_eq!(initial_selected, 0);

    // Debug: Check if scroll_range is available
    println!("scroll_range: {:?}", harness.state().scroll_range);

    // Test Page Down navigation
    harness.press_key(Key::PageDown);
    harness.step();

    let tab = harness.state().tab_manager.current_tab_ref();
    let after_page_down = tab.selected_index;

    println!("Movement: {initial_selected} -> {after_page_down} (page down)");

    // Should have moved forward by more than 1 (even with default fallback of 10)
    assert!(
        after_page_down > initial_selected,
        "Page down should move forward, moved from {initial_selected} to {after_page_down}"
    );

    // Test Page Up navigation
    harness.press_key(Key::PageUp);
    harness.step();

    let tab = harness.state().tab_manager.current_tab_ref();
    let after_page_up = tab.selected_index;

    println!("Movement: {after_page_down} -> {after_page_up} (page up)");

    // Should have moved back toward the beginning
    assert!(
        after_page_up < after_page_down,
        "Page up should move back, from {after_page_down} to {after_page_up}"
    );

    // Test Ctrl+D (alternative page down shortcut)
    let modifiers = egui::Modifiers {
        ctrl: true,
        ..Default::default()
    };
    harness.press_key_modifiers(modifiers, Key::D);
    harness.step();

    let tab = harness.state().tab_manager.current_tab_ref();
    let after_ctrl_d = tab.selected_index;

    println!("Movement: {after_page_up} -> {after_ctrl_d} (ctrl+d)");

    // Should behave like page down
    assert!(
        after_ctrl_d > after_page_up,
        "Ctrl+D should work like page down, from {after_page_up} to {after_ctrl_d}"
    );

    // Test Ctrl+U (alternative page up shortcut)
    let modifiers = egui::Modifiers {
        ctrl: true,
        ..Default::default()
    };
    harness.press_key_modifiers(modifiers, Key::U);
    harness.step();

    let tab = harness.state().tab_manager.current_tab_ref();
    let after_ctrl_u = tab.selected_index;

    println!("Movement: {after_ctrl_d} -> {after_ctrl_u} (ctrl+u)");

    // Should behave like page up
    assert!(
        after_ctrl_u < after_ctrl_d,
        "Ctrl+U should work like page up, from {after_ctrl_d} to {after_ctrl_u}"
    );
}
