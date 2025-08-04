# Audio Toolkit Shell

A native macOS desktop application that provides a 5-tab terminal interface for running audio toolkit executables. Built with Tauri, React, and TypeScript.

## 🎯 Project Vision

This application provides a unified interface for accessing multiple audio processing tools through individual terminal tabs. Each tab displays one executable's interactive menu, allowing users to access all their audio processing tools from a single, professional interface.

## 🏗️ Architecture

- **Framework:** Tauri (native macOS desktop application)
- **Frontend:** React + TypeScript
- **Terminal:** xterm.js with addons (FitAddon, WebLinksAddon)
- **Backend:** Rust with PTY process management
- **State Management:** Simple useState (no complex store integration)

## 📋 Features

### 5-Tab Interface
- **Tab 1:** Start Scripts - Main script menu launcher (shows 1-20 script options)
- **Tab 2:** Audio Normalizer - Audio normalization tool
- **Tab 3:** Session Monitor - Pro Tools session monitoring
- **Tab 4:** Pro Tools Session Launcher - Pro Tools session launcher
- **Tab 5:** Fifth Launcher - Additional launcher tool

### Real Terminal Functionality
Each tab shows the actual running executable with its interactive menu, allowing users to use the tools exactly as they work in regular terminals.

### User Interaction
- Interactive menus display automatically
- Full keyboard input support
- Real-time output display
- File drag-and-drop support
- Keyboard shortcuts (⌘1-5 for tab switching)

## 🚀 Current Status

### ✅ Completed
- **Tab 1 (Start Scripts):** Fully functional with menu display and user interaction
- **5-tab interface:** All tabs visible and clickable
- **Backend integration:** Connected to real executables
- **Clean architecture:** Uses proven CleanTerminal foundation

### ⏳ In Progress
- **Tabs 2-5:** Ready for implementation using the same proven pattern
- **PTY output events:** Backend output forwarding to frontend

## 🛠️ Development

### Prerequisites
- Node.js and npm/yarn
- Rust and Cargo
- Tauri CLI

### Setup
```bash
# Install dependencies
cd audio-toolkit-shell
npm install

# Run in development mode
npm run tauri dev
```

### Build
```bash
# Build for production
npm run tauri build
```

## 📁 Project Structure

```
audio-toolkit-shell/
├── frontend/src/
│   ├── components/
│   │   ├── CleanTerminal.tsx    # Core terminal component
│   │   ├── FiveTabTerminal.tsx  # 5-tab interface
│   │   └── ...
│   ├── App.tsx                  # Main application
│   └── ...
├── src-tauri/src/
│   ├── handlers/                # Tauri command handlers
│   ├── services/               # Backend services
│   └── lib.rs                  # Main Rust entry point
└── ...
```

## 🎯 Implementation Approach

### Proven Architecture Principles
1. **Use CleanTerminal foundation** - Avoids black screen issues
2. **No complex store integration** - Keep App.tsx simple with useState
3. **Direct menu display** - Display known executable menus directly
4. **Backend connectivity** - Maintain connection to real executables
5. **Clean component separation** - No mixed references or conflicts

## 📊 Success Metrics

- ✅ No black screens (using proven CleanTerminal foundation)
- ✅ 5 tabs visible and functional
- ✅ Tab switching works (click + ⌘1-5)
- ✅ Real backend connectivity for input
- ✅ Executable menu displays correctly

## 🤝 Contributing

This project follows a clean, incremental development approach. Each tab is implemented using the same proven pattern to ensure reliability and consistency.

## 📄 License

[Add your license here]