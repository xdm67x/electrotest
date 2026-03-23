# Test Guide - Electrotest with CDP

## Launch Electron App with Debugging

```bash
cd examples/electron-app

# With pnpm
pnpm start:debug

# Or with npm
npm run start:debug

# Or directly
pnpm electron . --remote-debugging-port=9222
```

## Launch Electrotest

In another terminal:

```bash
cargo run
```

## Test Workflow

1. **Select the Electron process** in the picker (arrows + Enter)

2. **In the console, type CDP commands:**

   ```
   > connect
   # Displays: "Connected to CDP on port 9222"
   # Displays: "Page title: <page title>"

   > evaluate document.title
   # Displays: "Result: <title>"

   > evaluate window.location.href
   # Displays: "Result: file://..."

   > screenshot test.png
   # Displays: "Screenshot saved to test.png"

   > navigate https://example.com
   # Displays: "Navigated to https://example.com"

   > cdp-status
   # Displays: "CDP: Connected on port Some(9222)"

   > disconnect
   # Displays: "Disconnected from CDP"
   ```

3. **Return to picker:** Press `Tab`

4. **Quit:** Press `Esc` or `Ctrl+C`

## Available Commands

| Command | Description |
|---------|-------------|
| `connect` | Connect to CDP (auto-detects port) |
| `disconnect` | Disconnect from CDP |
| `evaluate <js>` | Execute JavaScript in Electron |
| `screenshot [path]` | Capture screenshot (default: screenshot.png) |
| `navigate <url>` | Navigate to URL |
| `cdp-status` | Show CDP connection status |
| `help` | Show help |
| `status` | Show Electron process status |
| `pid` | Show attached process PID |
| `clear` | Clear logs |

## Troubleshooting

- **"No CDP port detected"**: Launch Electron with `--remote-debugging-port=9222`
- **"Failed to connect"**: Check that the port is open (`lsof -i :9222`)
- **Process not visible**: Refresh with `r` in the picker
