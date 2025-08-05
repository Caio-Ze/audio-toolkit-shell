# Design Document

## Overview

The Audio Toolkit Shell refactoring project aims to clean up the codebase by identifying and removing legacy Python scripts and old implementations that are no longer needed. Based on the README analysis, the project has successfully migrated from a Tauri + React architecture to a native Rust implementation using eframe/egui. The refactoring will focus on removing remnants of the old architecture while preserving the current working Rust implementation.

## Architecture

### Current Architecture (To Preserve)
The current working implementation follows this architecture:
- **Native Rust Application**: Built with eframe and egui for GUI
- **Terminal Management**: Uses portable-pty for real terminal functionality
- **Configuration System**: TOML-based configuration for tabs and settings
- **Multi-tab Interface**: Each tab can run different executables

### Legacy Components (To Remove)
Based on the project analysis and README documentation, the following legacy components have been identified and need to be removed:

**Confirmed Legacy Files:**
- `src-tauri/fix_terminal_ui.py` - Python script for UI fixes (no longer needed)
- `src-tauri/fix_ui_complete.py` - Python script for completion UI fixes (no longer needed)  
- `src-tauri/fix_ui_rendering.py` - Python script for rendering fixes (no longer needed)
- `dev-mocks/mock_launcher.sh` - Development mock script (conflicts with "no mock scripts" requirement)
- `test_wrapper.sh` (both root and src-tauri) - Potential legacy test scripts

**Files Requiring Analysis:**
- Tauri configuration files in `capabilities/` and `gen/` directories (may be unused after migration)
- Various documentation files (CHANGELOG.md, CONFIGURATION.md, TECHNICAL.md) - need content review
- `.DS_Store` files - should be removed and added to .gitignore

## Components and Interfaces

### Core Components to Analyze
1. **Main Rust Application** (`src-tauri/src/main.rs`)
   - Verify this contains the complete eframe/egui implementation
   - Ensure all terminal functionality is properly integrated
   - Confirm no dependencies on Python scripts

2. **Configuration System** (`src-tauri/config.toml`)
   - Validate all referenced paths and executables exist and are functional
   - Ensure tab configurations are properly structured
   - Remove any legacy configuration entries from previous architecture

3. **Build System** (`src-tauri/Cargo.toml`)
   - Verify dependencies align with current eframe/egui architecture
   - Remove any unused dependencies from Tauri + React migration
   - Ensure no Python or Node.js dependencies remain

4. **Tauri Infrastructure** (`capabilities/`, `gen/`, `icons/`)
   - Determine if Tauri configuration directories are still needed
   - Assess if icons directory contains current application icons
   - Remove unused Tauri-specific files if migration is complete

5. **Documentation Files**
   - Review CHANGELOG.md, CONFIGURATION.md, TECHNICAL.md for accuracy
   - Update documentation to reflect current architecture
   - Remove outdated information about previous implementations

### Legacy Detection Interface
The refactoring system will implement:
- **File Analysis Module**: Scans directory structure for legacy files and categorizes them
- **Python Script Detector**: Identifies and validates removal of Python fix scripts
- **Dependency Checker**: Validates current Cargo.toml dependencies against actual usage
- **Configuration Validator**: Ensures config.toml references valid paths and executables
- **Tauri Artifact Analyzer**: Determines which Tauri-specific files are still needed
- **Documentation Updater**: Updates documentation to reflect clean state and current architecture
- **Git Integration**: Ensures each cleanup step is properly committed with descriptive messages

## Data Models

### File Classification Model
```rust
enum FileStatus {
    Current,        // Part of current working implementation
    Legacy,         // Old implementation to be removed
    Unknown,        // Requires manual review
    Configuration,  // Config file to be validated
    Documentation,  // Documentation file to be reviewed/updated
    SystemFile,     // System files like .DS_Store to be removed
}

struct FileAnalysis {
    path: PathBuf,
    status: FileStatus,
    purpose: String,
    dependencies: Vec<String>,
    last_modified: SystemTime,
    removal_safety: SafetyLevel,
}

enum SafetyLevel {
    Safe,           // Can be removed without risk
    RequiresTest,   // Removal requires build/functionality test
    Manual,         // Requires manual review before removal
}
```

### Cleanup Plan Model
```rust
struct CleanupPlan {
    files_to_remove: Vec<PathBuf>,
    files_to_update: Vec<(PathBuf, String)>,
    validation_steps: Vec<String>,
    backup_required: bool,
    commit_strategy: CommitStrategy,
}

enum CommitStrategy {
    PerFile,        // Commit each file removal separately
    PerCategory,    // Commit by file type (Python scripts, docs, etc.)
    SingleCommit,   // All changes in one commit
}
```

## Error Handling

### File Operation Errors
- **Permission Errors**: Handle cases where files cannot be deleted due to permissions
- **Git Operation Errors**: Handle cases where git commits or pushes fail
- **Dependency Errors**: Detect when removing a file would break current functionality
- **Backup Failures**: Ensure backup operations complete successfully before deletion

### Validation Errors
- **Configuration Validation**: Handle invalid paths or missing executables in config.toml
- **Build Verification**: Ensure the project still builds after cleanup operations
- **Functionality Testing**: Verify core terminal features work after refactoring
- **Python Script Dependencies**: Detect if any Python scripts are still referenced

### Recovery Mechanisms
- **Git-based Recovery**: Use git history for rollback capability
- **Incremental Commits**: Each cleanup step committed separately for granular rollback
- **Build Verification**: Test build success after each major cleanup step
- **Functionality Baseline**: Establish working baseline before starting cleanup

## Testing Strategy

### Pre-Cleanup Validation
1. **Functionality Baseline**: Verify current eframe/egui implementation works correctly
2. **Build Verification**: Ensure `cargo build` succeeds without errors
3. **Configuration Testing**: Validate all config.toml entries reference valid executables
4. **Git Status Check**: Ensure working directory is clean before starting

### Cleanup Validation
1. **Python Script Analysis**: Verify Python scripts are not referenced in Rust code
2. **File Dependency Analysis**: Ensure no current files depend on legacy components
3. **Build Testing**: Run `cargo build` after each cleanup step
4. **Configuration Integrity**: Verify config.toml remains valid after changes

### Post-Cleanup Verification
1. **Complete Build Test**: Full `cargo build --release` from clean state
2. **Application Launch Test**: Verify the application starts and displays correctly
3. **Terminal Functionality**: Test that configured tabs launch and work properly
4. **Configuration Validation**: Ensure all configured executables still function
5. **Documentation Accuracy**: Verify documentation reflects current state

### Test Strategy Implementation
- **Manual Testing**: Step-by-step verification of core functionality
- **Build Verification**: Automated cargo build testing after each step
- **Git Integration**: Each step committed separately for easy rollback
- **Configuration Testing**: Validate config.toml syntax and referenced paths

## Version Control Strategy

### Git Workflow
The refactoring process will follow a disciplined git workflow to ensure traceability and easy rollback:

1. **Pre-Cleanup Baseline**: Ensure working directory is clean and current state is committed
2. **Incremental Commits**: Each file or logical group of files removed in separate commits
3. **Descriptive Messages**: Clear commit messages describing what was removed and why
4. **Push Strategy**: Push changes to GitHub after each commit for backup and collaboration
5. **Rollback Preparation**: Each commit is atomic and can be reverted independently

### Commit Message Format
```
refactor: remove [file/component] - [reason]

Examples:
- refactor: remove fix_terminal_ui.py - legacy Python UI fix script no longer needed
- refactor: remove dev-mocks directory - conflicts with no-mock-scripts requirement  
- refactor: clean up .DS_Store files and update .gitignore
- refactor: remove unused Tauri capabilities - migration to native Rust complete
```

### Branch Strategy
- Work directly on main branch with incremental commits
- Each commit represents a safe, tested state
- GitHub push after each commit ensures remote backup

### Change Tracking and User Communication
- **Change Summary**: Maintain a running list of all modifications made
- **Impact Assessment**: Document the effect of each change on functionality
- **User Reporting**: Provide clear summary of what was removed and why
- **Rollback Instructions**: Document how to revert specific changes if needed
- **Verification Steps**: Show user how to verify the application still works correctly

## Implementation Phases

### Phase 1: Analysis and Discovery
- Scan project structure and classify all files by status
- Identify Python fix scripts for removal (fix_terminal_ui.py, fix_ui_complete.py, fix_ui_rendering.py)
- Analyze Tauri-specific directories (capabilities/, gen/) for current relevance
- Review documentation files for accuracy and current relevance
- Generate prioritized cleanup plan with safety assessments

### Phase 2: Safe Legacy Removal
- Remove Python fix scripts (confirmed legacy)
- Remove dev-mocks directory (conflicts with "no mock scripts" requirement)
- Clean up system files (.DS_Store) and update .gitignore
- Remove unused test wrapper scripts
- Commit each removal separately with descriptive messages

### Phase 3: Configuration and Dependency Cleanup
- Analyze and clean unused dependencies in Cargo.toml
- Validate and optimize config.toml entries
- Remove unused Tauri configuration files if migration is complete
- Update .gitignore to prevent future system file commits

### Phase 4: Documentation and Final Validation
- Update documentation to reflect current architecture
- Remove outdated information about previous implementations
- Verify project builds and runs correctly after all changes
- Test all configured terminal functionality
- Push all changes to GitHub with proper commit history
- Provide user with summary of all changes made and their impact