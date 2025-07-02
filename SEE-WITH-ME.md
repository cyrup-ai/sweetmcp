# SEE-WITH-ME Workflow 👁️

This document explains how to set up a visual feedback workflow where both you and Claude can see the same webpage during development.

## Why This Matters

When working on frontend development, Claude can only see HTML/CSS code by default, not the visual rendering. This workflow enables Claude to take screenshots and see exactly what you see in your browser.

## Prerequisites

- Node.js installed (for live-server and capture-website-cli)
- A project with HTML/CSS files
- macOS, Linux, or Windows with a terminal

## Setup Instructions

### 1. Install Live-Server for Auto-Reload

Live-server provides automatic browser refresh when files change:

```bash
# Install globally (one-time)
npm install -g live-server

# Or use npx (no install needed)
npx live-server
```

### 2. Install Screenshot Tool

This tool allows Claude to capture webpages as PNG images:

```bash
# Install globally
npm install -g capture-website-cli
```

### 3. Start the Development Server

Navigate to your project directory and start live-server:

```bash
# If your HTML is in the current directory
npx live-server . --port=8000

# If your HTML is in a subdirectory (e.g., docs/)
cd docs && npx live-server . --port=8000

# For background/persistent server, use tmux
tmux new-session -d -s my-server 'npx live-server . --port=8000'
```

### 4. Enable Directory Access (if needed)

If Claude needs to save screenshots to /tmp or other directories:

```bash
# In your terminal (varies by Claude interface)
/add-dir /tmp
```

## Usage Workflow

### For You (Human):
1. Open http://localhost:8000 in your browser
2. Make requests for changes
3. See updates automatically refresh in your browser

### For Claude:
1. Make HTML/CSS changes
2. Save the file
3. Take a screenshot:
   ```bash
   capture-website http://localhost:8000 --output=/path/to/screenshot.png --width=1200 --height=800 --overwrite
   ```
4. Read the screenshot using the Read tool
5. Both parties can now see the same visual result

## Example Commands for Claude

```bash
# Take a screenshot (adjust path as needed)
capture-website http://localhost:8000 --output=/Volumes/samsung_t9/projects/y/sweetmcp/screenshot.png --width=1200 --height=800 --overwrite

# Then read it
# Claude uses: Read tool with the screenshot path
```

## Troubleshooting

### Port Already in Use
```bash
# Kill process on port 8000
lsof -ti:8000 | xargs kill -9
```

### Screenshot Fails
- Ensure the server is running
- Check if the URL is accessible
- Verify capture-website-cli is installed
- Make sure the output path is in an allowed directory

### Live-Server Not Auto-Reloading
- Check if you saved the file
- Ensure live-server is running in the correct directory
- Try clearing browser cache

## Benefits

- **Real-time feedback**: No manual browser refresh needed
- **Visual verification**: Claude can see rendered output, not just code
- **Efficient workflow**: Make change → Save → Both see result instantly
- **Better communication**: "Can you make the shadow stronger?" → Claude can verify the result

## Alternative Setups

### VS Code Live Server Extension
If using VS Code, install the "Live Server" extension and right-click HTML file → "Open with Live Server"

### Python Simple Server (no auto-reload)
```bash
python3 -m http.server 8000
```

### Browser Extensions
Install "Auto Refresh Plus" or similar for any local server

## Quick Start Summary

```bash
# One-time setup
npm install -g live-server capture-website-cli

# Each session
cd your-project
npx live-server . --port=8000

# Claude can then capture and view
capture-website http://localhost:8000 --output=./screenshot.png --width=1200 --height=800 --overwrite
```

Now both you and Claude can "see" the same thing during development! 🎨✨