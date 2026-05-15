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

## Raycast Integration

MARVIN ships with a headless CLI binary (`marvin-cli`) and a Raycast script command for quick access.

### Build

```bash
cargo build --release --bin marvin-cli
```

### Setup

1. Open Raycast Settings (`Cmd+,`)
2. Go to **Extensions** > click **+** > **Add Script Directory**
3. Select the `raycast-scripts/` folder inside this repository
4. The command **"Paste with MARVIN"** will appear in your Raycast command list

### Assign a Keyboard Shortcut

1. Open Raycast and search for "Paste with MARVIN"
2. Press `Cmd+K` to open the action menu, then select **Set Hotkey**
3. Record your preferred shortcut (e.g. `Ctrl+Opt+V`)

### Configuration

Create `~/.config/marvin/config.toml` to adjust typing speed:

```toml
delay_ms = 40
```

Each invocation reads the file, so changes take effect immediately. Default is 15ms if no config file exists.

### How It Works

1. Copy text to clipboard (supports multi-line)
2. Click into the target VDI window
3. Trigger the hotkey
4. MARVIN simulates keyboard input at the current cursor position
