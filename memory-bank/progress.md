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
*   Column sort order persistence between sessions
*   Search filter with visual highlighting (`/`, Enter, Esc)
    - Real-time filtering as you type
    - Orange highlighting of matching text
    - Search state persists after Enter
    - Clear filter with Esc
*   Add file/directory (a)

### What's left to build
*   PDF preview
*   cache directory list
*   Shortcut to toggle sort
*   Fuzzy directory jump
*   Case-sensitive/insensitive search toggle
*   Regular expression search support
*   Fix global static variable in bookmark module
*   Right click menu for various operations

### Current status
The application is in a functional state with core file management features implemented. Recent improvements to the search functionality make it more user-friendly with visual feedback through text highlighting.

### Known issues
*   PDF preview is not yet implemented.

### Evolution of project decisions
*   The project initially started as a simple file manager but has evolved to include more advanced features such as bookmark management and tab creation.
*   Configuration management has been expanded to store more user preferences beyond just color schemes.
