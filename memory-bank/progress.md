## Progress

### What works

* Basic file navigation (j, k, h, l, Enter, gg, G)
* File opening (o/Enter)
* File deletion (D)
* File renaming (r)
* File selection (space)
* File copying (y)
* File cutting (x)
* File pasting (p)
* Application exiting (q)
* Bookmark management (b, B)
* Tab creation and switching (t, 1, 2, 3, etc.)
* Help window (?)
* Configurable color schemes
* Column sort order persistence between sessions (fully implemented and tested)
* Search filter with robust visual highlighting and support for large directories, Unicode, and long filenames
  * Real-time filtering as you type
  * Orange highlighting of matching text
  * Search state persists after Enter
  * Clear filter with Esc
* Add file/directory (a)
* Right click context menu with operations:
  * Add new file/directory
  * Rename selected file/directory
  * Delete selected file/directory
  * Copy selected file/directory
  * Cut selected file/directory
  * Paste copied/cut file/directory
  * Context-aware enabling/disabling of options
* Tab selection preservation when switching between tabs at runtime
* SVG preview using the resvg crate
* Image previews using egui's Image widget with direct URI source paths
* Zip file preview showing contained files and folders
* PDF preview with metadata display and rendered first page
* EPUB preview with metadata display and cover image
* Configurable keyboard shortcuts through TOML config files
* 'g' namespace key similar to Vim for special shortcut combinations
* Soft/hard link files display with dedicated icons
* Directory history navigation with Ctrl+O (back) and Ctrl+I (forward) within each tab

### What's left to build

* cache directory list
* Shortcut to toggle sort
* Fuzzy directory jump (integrate with fzf)
* Case-sensitive/insensitive search toggle
* Regular expression search support
* In rename mode, start the cursor after the extension
* Renaming a file doesn't clear the image rendering cache, so it still displays the old image
* Open file with alternative commands leveraging open::commands and open::with_command
* render PDF preview using pdfium_render or pathfinder_rasterize
  * see <https://github.com/servo/pathfinder/issues/157>
* Add debounce to preview to reduce io and compute