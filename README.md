# Freedesktop Desktop Entry Parser

A Rust library for parsing and manipulating Linux `.desktop` files according to the [freedesktop.org Desktop Entry Specification](https://specifications.freedesktop.org/desktop-entry-spec/latest/).

## Features

- Full support for Desktop Entry Specification v1.5
- Locale-aware string handling
- Icon resolution using freedesktop icon theme
- Support for desktop actions
- Strong type safety with Rust's type system

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
freedesktop-file-parser = "0.1.0"
```

### Basic Example

```rust
use freedesktop_entry::parse;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = r#"[Desktop Entry]
Type=Application
Name=Firefox
Exec=firefox %u
Icon=firefox
Categories=Network;WebBrowser;
    "#;

    let desktop_file = parse(content)?;
    
    // Access basic properties
    println!("Name: {}", desktop_file.entry.name.default);
    
    // Check entry type
    if let EntryType::Application(app) = &desktop_file.entry.entry_type {
        println!("Exec: {}", app.exec.as_ref().unwrap());
    }

    Ok(())
}
```

### Handling Localized Strings

```rust
use freedesktop_entry::LocaleString;

// Access different translations
if let Some(de_name) = desktop_file.entry.name.variants.get("de") {
    println!("German name: {}", de_name);
}
```

### Working with Actions

```rust
// Access application actions
for (action_name, action) in &desktop_file.actions {
    println!("Action: {}", action.name.default);
    if let Some(exec) = &action.exec {
        println!("Action command: {}", exec);
    }
}
```

## Supported Fields

The library supports all standard fields from the Desktop Entry Specification, including:

- Basic fields (Type, Name, GenericName, NoDisplay, Comment, Icon)
- Application fields (Exec, TryExec, Path, Terminal, MimeType, Categories)
- Startup fields (StartupNotify, StartupWMClass)
- Display fields (Hidden, OnlyShowIn, NotShowIn)
- Actions
- DBus activation

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [freedesktop.org](https://www.freedesktop.org/) for the Desktop Entry Specification
- [freedesktop-icons](https://crates.io/crates/freedesktop-icons) for icon resolution support

## Status

This project is under active development. While it implements the full specification, please report any bugs or missing features.
