# cli-todo

A blazingly fast Rust TUI todo app with Vim keybindings for terminal purists. An experimental playground for Immediate Mode rendering—where every keystroke triggers a full UI rebuild, no state diffing required.

## Installation

### From crates.io

If you have Rust installed, you can install `todo` directly from crates.io:

```console
$ cargo install cli-todo
```

### From source

1.  Clone the repository.
2.  Run the installation script:

```console
$ ./install.sh
```

Or manually with cargo:

```console
$ cargo install --path .
```

## Usage

Once installed, you can run the application using the `todo` command:

```console
$ todo
```

To see the list of controls, you can use the `--help` flag:

```console
$ todo --help
```

## Controls (Vim-style keymaps)

### Navigation
|Keys|Description|
|---|---|
|<kbd>k</kbd>, <kbd>j</kbd>|Move cursor up and down|
|<kbd>h</kbd>, <kbd>l</kbd>|Switch between TODO (left) and DONE (right) panels|
|<kbd>g</kbd>, <kbd>G</kbd>|Jump to the start/end of the current item list|
|<kbd>TAB</kbd>|Switch between the TODO and DONE panels|

### Item Manipulation
|Keys|Description|
|---|---|
|<kbd>K</kbd>, <kbd>J</kbd>|Drag the current item up and down|
|<kbd>i</kbd>|Insert a new item at cursor position|
|<kbd>o</kbd>|Insert a new item below cursor (vim style)|
|<kbd>O</kbd>|Insert a new item above cursor (vim style)|
|<kbd>r</kbd>|Rename the current item|
|<kbd>c</kbd>|Change item (clear and enter insert mode)|
|<kbd>C</kbd>|Change entire line (clear and enter insert mode)|
|<kbd>d</kbd>, <kbd>x</kbd>|Delete the current list item|
|<kbd>Enter</kbd>|Move item between TODO and DONE|

### Mode Switching
|Keys|Description|
|---|---|
|<kbd>ESC</kbd>|Exit insert/rename mode (back to normal mode)|
|<kbd>Enter</kbd>|Confirm edit and exit insert mode|
|<kbd>q</kbd>|Quit the application|

**Made with** :heart: **and Rust**
