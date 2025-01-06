# TUI-Journal Custom Themes:

It's possible to override the styles used in the app or parts of them. Custom themes will be read from the file `theme.toml` in the configuration directory within the `tui-journal` directory.

## Table of Contents

- [Getting Started](#getting-started)
- [Themes structures](#themes-structures)
  - [Themes Groups](#themes-groups)
  - [Themes Types](#themes-types)
    - [Color Type](#color-type)
    - [Style Type](#style-type)
- [Example](#example)

## Getting started:

To get started, users can use the provided CLI subcommands under `tjournal theme`. These include specifying the path for the themes file, printing the default themes to be used as a base, or writing the themes directly into the themes file.

```
Provides commands regarding changing themes and styles of the app

Usage: tjournal theme <COMMAND>

Commands:
  print-path      Prints the path to the user themes file [aliases: path]
  print-default   Dumps the styles with the default values to be used as a reference and base for user custom themes [aliases: default]
  write-defaults  Creates user custom themes file if doesn't exist then writes the default styles to it [aliases: write]
  help            Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## Themes structures:

### Themes Groups:

Themes are divided into the following groups:
- **general**: Styles for general controls in the app, including journal pop-ups, filter and sorting pop-ups, and more.
- **journals_list**: Styles for the main list of journals. These styles are differentiated from the general ones since they are more important and contain more information than general list items.
- **editor**: Styles for the built-in editor.
- **msgbox**: Colors for message-box prompts (Questions, Errors, Warnings, etc.).

### Themes Types:

The main types used to define themes are `Color` and `Style`.

#### Color Type

Represents the color value of an item or field. It can be defined as a terminal color, such as `Black` or `Reset`, or as an RGB value in hexadecimal format `#RRGGBB`.

#### Style Type

Represents a complete style definition with the following fields:
- **fg**: Foreground color.
- **bg**: Background color.
- **modifiers**: Modifiers change the way a piece of text is displayed. They are bitflags, so they can be easily combined.
  The available modifiers are: `BOLD | DIM | ITALIC | UNDERLINED | SLOW_BLINK | RAPID_BLINK | REVERSED | HIDDEN | CROSSED_OUT`.
- **underline_color**: The color of the underline parts if the `UNDERLINED` modifier is active.

Here is an example of a style with all elements defined:

```toml
# Example of a style with all possible elements
[example_style]
fg = "#0AFA96" # Foreground Color. Colors can be in hex-decimal format "#RRGGBB"
bg = "Black" # Background Color. Also it can be one of terminal colors.
# Modifiers with all available flags. Flags can be combined as in example.
modifiers = "BOLD | DIM | ITALIC | UNDERLINED | SLOW_BLINK | RAPID_BLINK | REVERSED | HIDDEN | CROSSED_OUT"
underline_color = "Magenta" # Color for underline element if activated  
```
It's worth mentioning that not all fields must be defined. Missing parts will be filled with their default values.

## Example:

Here is a small example of overriding some of the themes. For a full list of all available style fields, please use the CLI subcommands to print the default themes.

```toml
[general]
list_item_selected = { fg = "LightYellow", modifiers = "BOLD" }
input_block_active = { fg = "Blue" }

[general.input_block_invalid]
fg = "Red"
modifiers = "BOLD | SLOW_BLINK"

[general.input_corsur_active]
fg = "Black"
bg = "LightYellow"
modifiers = "RAPID_BLINK"

[general.list_highlight_active]
fg = "Black"
bg = "LightGreen"
modifiers = "UNDERLINED"
underline_color = "Magenta"

[journals_list.block_inactive]
fg = "Grey"
modifiers = ""

[journals_list.highlight_active]
fg = "Red"
bg = "LightGreen"
modifiers = "BOLD | ITALIC"

[journals_list.highlight_inactive]
fg = "Grey"
bg = "LightBlue"
modifiers = "BOLD"

[journals_list.title_active]
fg = "Reset"
modifiers = "BOLD | UNDERLINED"

[editor.block_insert]
fg = "LightGreen"
modifiers = "BOLD"

[editor.cursor_insert]
fg = "Green"
bg = "LightGreen"
modifiers = "RAPID_BLINK"

[msgbox]
error = "#105577"
question = "Magenta"
```
