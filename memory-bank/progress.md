## Progress

### What works
*   Basic file navigation (j, k, h, l, Enter, gg, G)
*   File opening (o)
*   File deletion (D)
*   File renaming (r)
*   File selection (space)
*   File copying (y)
*   File cutting (x)
*   File pasting (p)
*   Application exiting (q)
*   Bookmark management (b, B)
*   Tab creation and switching (t, 1, 2, 3, etc.)
*   Help window (?)
*   Configurable color schemes
*   Column sort order persistence between sessions (fully implemented and tested)
*   Search filter with robust visual highlighting and support for large directories, Unicode, and long filenames
    - Real-time filtering as you type
    - Orange highlighting of matching text
    - Search state persists after Enter
    - Clear filter with Esc
*   Add file/directory (a)
*   Right click context menu with operations:
    - Add new file/directory
    - Rename selected file/directory
    - Delete selected file/directory
    - Copy selected file/directory
    - Cut selected file/directory
    - Paste copied/cut file/directory
    - Context-aware enabling/disabling of options

### What's left to build
*   Persist app state to disk
*   Display linked files with icons
*   PDF preview
*   cache directory list
*   Shortcut to toggle sort
*   Fuzzy directory jump
*   Case-sensitive/insensitive search toggle
*   Regular expression search support

### Current status
The application is in a functional state with core file management features implemented. Recent improvements include enhanced search functionality with visual feedback through text highlighting and a comprehensive right-click context menu that provides intuitive access to file operations.

### Known issues
*   PDF preview is not yet implemented.

### Evolution of project decisions
*   The project initially started as a simple file manager but has evolved to include more advanced features such as bookmark management and tab creation.
*   Configuration management has been expanded to store more user preferences beyond just color schemes.
