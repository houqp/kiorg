Kiorg is a ultra fast, light weight cross platform file management app with a Vim inspired key binding.

It is built using Rust with the egui framework.

## Keyboard shortcuts to relevant features
* `j` to move down to the next entry
* `k` to move up to the previous entry
* `h` to navigate to the parent directory
* `l` or Enter to navigate into a selected folder
* `o/Enter` to open the selected file with external app
* `gg` to go to the first entry
* `G` to go to the last entry
* `D` to delete the selected file or folder with a confirmation prompt
* `r` to rename a file or directory
* space to select an entry
* `y` to copy an entry
* `x` to cut an entry
* `p` to paste an entry
* `/` to enter search filter mode
  - Type keywords to filter the list in real-time
  - Press Enter to confirm the filter and interact with the results
  - Press Esc to cancel or clear the filter
* `q` to exit the application with a confirmation popup that confirms the exit through enter
  - All popups in the app can be closed by pressing `q`, including the exit confirmation popup
* `b` to add/remove the current entry to bookmark
* `B` to toggle the bookmark popup
  - The bookmark menu should be centered to the screen, when it's active, it consumes all the input.
    * User should be able to navigate within the bookmark menu using keyboards.
    * `q` and `Esc` to exit the bookmark popup
    * `d` to delete a bookmark
  - Selecting a bookmark will jump directly to the bookmarked directory
  - Only allow bookmarking directory, not files
  - Bookmarks will be saved to `.config/kiorg/bookmarks.txt`
* `t` to create a new tab in the file browser
  - users can use number key `1`, `2`, `3`, etc to switch between tabs
  - tab numbers are displayed in right side of the top nav banner, with the current tab highlighted
* `?` to toggle help window that displays all the shortcuts in a popup window
* `a` to add file/directory

## Visual design

* Clean layout with compact spacing and alignment
* The application displays files and folders in the current directory with the following information:
  * File/folder names
  * Modified dates
  * File sizes in human readable format (for files only)
* Path truncation for long paths with "..." in the middle
* Bookmarked entries in the left and middle panel should be highlighted with bookmark icons
* The application uses icons to distinguish between files (📄) and folders (📁)
* When file is being opened, flash the relevant file entry

## Other features

* Supprot configurable color schemes through toml config files. Provides a builtin default them that looks like the editor color scheme Sonokai.