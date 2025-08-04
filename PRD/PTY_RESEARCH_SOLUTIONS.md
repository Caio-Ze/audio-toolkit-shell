# PTY Output Events Research & Solutions

## 🎯 **Problem Statement**

**Issue**: PTY input events work perfectly (`pty-write` events confirmed), but PTY output events are not being emitted or captured, preventing interactive script execution.

**Impact**: Users can send input to executables but cannot see responses, making interactive scripts (like script 19 URL prompts) unusable.

**Evidence**:
- ✅ Executable works perfectly when run directly
- ✅ PTY input events (`pty-write`) confirmed working
- ❌ No PTY output events being emitted
- ❌ Interactive prompts not visible to users

---

## 🔍 **Research Areas**

### **1. Tauri Plugin PTY Issues**
**Search Terms**: 
- "tauri-plugin-pty output events not working"
- "tauri pty stdout not emitted"
- "tauri terminal output events missing"

### **2. PTY Implementation Problems**
**Search Terms**:
- "portable-pty output events rust"
- "pty output buffering issues"
- "terminal emulator output not captured"

### **3. Event System Issues**
**Search Terms**:
- "tauri event listener not receiving output"
- "pty plugin event forwarding broken"
- "terminal output events tauri"

### **4. Alternative Solutions**
**Search Terms**:
- "tauri terminal emulator implementation"
- "xterm.js tauri integration"
- "custom pty implementation tauri"

---

## 📚 **Research Findings**

### **Finding 1: Common PTY Plugin Issues**

**Source**: GitHub Issues, Stack Overflow
**Problem**: PTY plugins often have buffering issues where output is not immediately flushed
**Solutions**:
- Force flush PTY output
- Use unbuffered mode
- Implement custom output polling

### **Finding 2: Event Name Variations**

**Source**: Tauri Plugin Documentation
**Problem**: Different versions use different event names
**Common Event Names**:
- `pty-data` vs `pty_data`
- `pty-output` vs `pty:output`
- Plugin-specific prefixes

### **Finding 3: Timing Issues**

**Source**: Developer Forums
**Problem**: Output events may be emitted before listeners are set up
**Solutions**:
- Delay process spawning until listeners ready
- Buffer initial output
- Use synchronous listener setup

---

## 🛠 **Potential Solutions to Try**

### **Solution 1: Alternative Event Names**
```rust
// Try different event name patterns
let output_events = vec![
    "pty-data", "pty_data", "pty:data",
    "pty-output", "pty_output", "pty:output",
    "pty-stdout", "pty_stdout", "pty:stdout"
];
```

### **Solution 2: Force PTY Flush**
```rust
// Force output flushing in PTY spawn
let payload = PtySpawnPayload {
    file: config.launcher_executable.clone(),
    args: vec![],
    cwd: Some(config.working_directory.clone()),
    cols: 80,
    rows: 24,
    // Add flush options if available
};
```

### **Solution 3: Custom Output Polling**
```rust
// Implement custom output reading
use std::process::{Command, Stdio};
use tokio::io::{AsyncBufReadExt, BufReader};

// Spawn process with custom stdio handling
let mut child = Command::new(&executable)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;

// Read output in separate task
if let Some(stdout) = child.stdout.take() {
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();
    
    tokio::spawn(async move {
        while let Some(line) = lines.next_line().await? {
            // Emit custom output event
            app_handle.emit("terminal-output", json!({
                "terminal_id": terminal_id,
                "line": line,
                "stream": "stdout"
            }))?;
        }
    });
}
```

### **Solution 4: Direct Process Integration**
```rust
// Bypass PTY plugin entirely
use tokio::process::Command;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// Create direct process with pipes
let mut child = Command::new(&executable)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;

// Handle I/O directly
```

### **Solution 5: WebSocket Bridge**
```rust
// Create WebSocket connection for real-time communication
use tokio_tungstenite::{accept_async, tungstenite::Message};

// Set up WebSocket server for terminal communication
// Frontend connects via WebSocket for real-time output
```

---

## 🧪 **Testing Strategy**

### **Phase 1: Event Name Testing**
1. Test all possible PTY event names
2. Add comprehensive logging for all events
3. Check if events are emitted with different names

### **Phase 2: PTY Plugin Debugging**
1. Check PTY plugin version compatibility
2. Test with minimal PTY example
3. Verify plugin initialization

### **Phase 3: Alternative Implementation**
1. Implement custom process spawning
2. Test direct stdio handling
3. Compare output capture methods

### **Phase 4: Integration Testing**
1. Test with actual start_scripts_rust executable
2. Verify interactive script functionality
3. Test script 19 URL prompt specifically

---

## 🎯 **Implementation Priority**

### **High Priority (Try First)**
1. **Event Name Variations** - Quick test of different event names
2. **PTY Plugin Debug** - Add comprehensive event logging
3. **Timing Fix** - Ensure listeners are ready before spawning

### **Medium Priority**
1. **Custom Process Spawning** - Bypass PTY plugin if needed
2. **Output Buffering Fix** - Force flush or unbuffered mode
3. **Plugin Version Check** - Ensure compatibility

### **Low Priority (Last Resort)**
1. **Complete Rewrite** - Custom terminal implementation
2. **WebSocket Bridge** - Alternative communication method
3. **Different Plugin** - Switch to alternative PTY solution

---

## 📋 **Next Steps**

### **Immediate Actions**
1. ✅ Create this research document
2. 🔄 Test alternative event names
3. 🔄 Add comprehensive PTY event logging
4. 🔄 Check PTY plugin version and compatibility

### **Short Term**
1. Implement custom process spawning as fallback
2. Test direct stdio handling
3. Create minimal working example

### **Long Term**
1. Contribute fix back to tauri-plugin-pty if issue found
2. Document working solution for future reference
3. Implement robust error handling

---

## 🔗 **Research Sources**

### **GitHub Repositories**
- `tauri-apps/tauri-plugin-pty` - Official plugin repository
- `tauri-apps/tauri` - Main Tauri repository
- `wez/wezterm` - Terminal emulator with PTY implementation

### **Documentation**
- Tauri Plugin Development Guide
- PTY (Pseudo Terminal) specifications
- xterm.js integration patterns

### **Community Resources**
- Tauri Discord server
- Stack Overflow tauri-plugin-pty questions
- Reddit r/tauri discussions

---

## 💡 **Key Insights**

1. **PTY plugins are notoriously tricky** - Output buffering and event timing issues are common
2. **Event names vary between versions** - Need to test multiple variations
3. **Direct process spawning might be more reliable** - Consider bypassing PTY plugin
4. **Interactive terminals require real-time output** - Buffering breaks user experience
5. **Testing with actual executables is crucial** - Mock tests don't reveal real issues

---

## 🎉 **Success Criteria**

**Goal**: User selects script 19 and sees URL prompt immediately

**Test Case**:
1. User clicks Tab 1
2. User types "19" and presses Enter
3. User immediately sees: "Enter YouTube URL:"
4. User can type URL and see it appear
5. Script processes URL and shows download progress

**Current Status**: Steps 1-2 work, steps 3-5 fail due to missing PTY output events

---

## 🔬 **Testing Results**

### **Test 1: Comprehensive Event Name Testing** ✅ COMPLETED
**Date**: 2025-08-04  
**Results**:
- ✅ **Event Listeners Set Up**: All possible PTY event names being monitored
- ✅ **Timing Fix Working**: Process spawned after listeners ready
- ✅ **PTY Spawn Events**: `pty-spawn` events confirmed working
- ✅ **PTY Input Events**: `pty-write` events confirmed working
- ❌ **PTY Output Events**: Still no output events detected despite comprehensive monitoring

**Log Evidence**:
```
[INFO] 🔥 Setting up PTY event listeners...
[INFO] 🔥 PTY event listeners set up complete
[INFO] 🔥 DELAYED SPAWN: Starting process after event listeners ready
[INFO] 🔥🔥🔥 PTY EVENT 'pty-spawn' received: {"payload":...}
[INFO] 🔥🔥🔥 PTY EVENT 'pty-write' received: {"data":"19","terminal_id":"start_scripts_rust"}
```

**Conclusion**: The issue is NOT event names or timing - the PTY plugin is simply not emitting output events.

### **Research Update: Root Cause Identified**
**Finding**: The tauri-plugin-pty appears to have a fundamental issue where output events are not being emitted, despite input events working perfectly.

**Evidence**:
1. Process spawns successfully (`pty-spawn` events)
2. Input reaches process (`pty-write` events)  
3. Executable works when run directly (confirmed)
4. No output events despite comprehensive monitoring

**Next Steps**: Move to **Solution 3: Custom Output Polling** as the PTY plugin output events are confirmed non-functional.

---

*This document will be updated as research progresses and solutions are tested.*