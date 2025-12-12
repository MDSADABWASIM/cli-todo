# cli-todo

Simple Interactive Terminal Todo App in Rust. This is meant to be an experimental playground for testing ideas on Immediate TUIs.

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

## Controls

|Keys|Description|
|---|---|
|<kbd>k</kbd>, <kbd>j</kbd>|Move cursor up and down|
|<kbd>Shift+K</kbd>, <kbd>Shift+J</kbd>|Drag the current item up and down|
|<kbd>g</kbd>, <kbd>G</kbd> | Jump to the start, end of the current item list|
|<kbd>r</kbd>|Rename the current item|
|<kbd>i</kbd>|Insert a new item|
|<kbd>d</kbd>|Delete the current list item|
|<kbd>q</kbd>|Quit|
|<kbd>TAB</kbd>|Switch between the TODO and DONE panels|
|<kbd>Enter</kbd>|Perform an action on the highlighted UI element|

**Made with** :heart: **and Rust**
