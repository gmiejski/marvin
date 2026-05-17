# MARVIN

**Most Annoying Remote VDI Input Nuisance**

A keyboard simulation tool for pasting scripts into clipboard-restricted VDI environments.

https://github.com/user-attachments/assets/45d96d1b-014c-45af-9d96-91dbad59d5aa

## GUI Usage

1. Build: `cargo build --release`
2. Run: `./target/release/marvin`
3. Grant Accessibility permission when prompted (System Settings > Privacy & Security > Accessibility)
4. Paste your script into the editor, set delay and countdown
5. Click into the target VDI window
6. **Shift+Click** to start typing, **Esc** to abort

## CLI Usage

A headless binary (`marvin-cli`) for scripting and integration with tools like Raycast.

### Build

```bash
cargo build --release --bin marvin-cli
```

### Grant Accessibility Permission

`marvin-cli` uses the same input simulation as the GUI and requires Accessibility access. On first run, macOS will prompt you — grant it under System Settings > Privacy & Security > Accessibility.

### Standalone

```bash
# Type a string
marvin-cli "hello world"

# Type from clipboard
pbpaste | marvin-cli -

# Supports \n and \t escape sequences in arguments
marvin-cli "line1\nline2"
```

### Configuration

Create `~/.config/marvin/config.toml` to adjust typing speed:

```toml
delay_ms = 40
```

Each invocation reads the file, so changes take effect immediately. Default is 15ms if no config file exists.

### Raycast Integration

Requires [Raycast](https://raycast.com/) installed.

1. Open Raycast Settings (`Cmd+,`)
2. Go to **Extensions** > click **+** > **Add Script Directory**
3. Select the `raycast-scripts/` folder inside this repository
4. The command **"Paste with MARVIN"** will appear in your Raycast command list
5. To assign a hotkey: search for "Paste with MARVIN", press `Cmd+K`, select **Set Hotkey** (e.g. `Ctrl+Opt+V`)

**Workflow:** Copy text to clipboard → click into VDI window → trigger hotkey → MARVIN types it out.
