# Design Document

## Overview

Simple 5-tab interface where each tab shows one CleanTerminal component connected to one running executable. Uses proven CleanTerminal foundation to avoid black screen issues.

## Architecture

```
App.tsx (Clean - NO store hooks)
├── Tab Bar (5 clickable tabs)
└── Content Area
    └── CleanTerminal (one at a time, based on active tab)
        ├── Tab 1: start_scripts_rust ✅ IMPLEMENTED
        ├── Tab 2: audio_normalizer ⏳ TODO
        ├── Tab 3: session_monitor ⏳ TODO
        ├── Tab 4: ptsl_launcher ⏳ TODO
        └── Tab 5: fifth_launcher ⏳ TODO
```

## Components

### App.tsx
- Shows 5 tabs at the top
- Shows one CleanTerminal component based on which tab is active
- NO store hooks (prevents black screen issues)
- Simple state management with useState

### CleanTerminal (enhanced)
- Based on proven CleanTerminal foundation
- Enhanced with direct menu display for each executable
- Connects to actual executable processes for input
- Handles user input/output
- **AVOIDS complex store integration that causes black screens**

## Data Flow

1. App shows 5 tabs with simple useState
2. Backend spawns actual executables
3. User clicks tab → App shows CleanTerminal with that terminal's ID
4. CleanTerminal displays the executable's menu directly → User sees the menu immediately
5. User interacts → CleanTerminal sends input to backend executable

## Implementation Notes

- **Use CleanTerminal foundation** - Proven to work without black screens
- **Direct menu display** - Display known executable menus directly in frontend
- **Backend connectivity** - Maintain connection to real executables for input
- **Keep it simple** - Minimal complexity, maximum reliability

## Success Criteria

- ✅ **Tab 1 COMPLETE** - start_scripts_rust menu displays and works perfectly
- ⏳ **Tabs 2-5** - Ready for implementation using same proven pattern