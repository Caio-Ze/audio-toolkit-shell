# Requirements Document

## Introduction

Create a simple 5-tab interface where each tab shows one of the existing, fully-functional executable terminals. The executables already work perfectly with user interaction - we just need to display them in tabs.

## Requirements

### Requirement 1

**User Story:** As a user, I want to see 5 tabs at the top of the application, so that I can access each of my 5 audio processing executables.

#### Acceptance Criteria

1. WHEN the application opens THEN the system SHALL display 5 tabs labeled: "Start Scripts", "Audio Normalizer", "Session Monitor", "Pro Tools Session Launcher", "Fifth Launcher"
2. WHEN I click on any tab THEN the system SHALL switch to show that terminal
3. WHEN I use keyboard shortcuts ⌘1-5 THEN the system SHALL switch to the corresponding tab

### Requirement 2

**User Story:** As a user, I want each tab to show the actual running executable with its interactive menu, so that I can use the tools exactly as they work in regular terminals.

#### Acceptance Criteria

1. WHEN I click on "Start Scripts" tab THEN the system SHALL show the start_scripts_rust executable with its 1-20 script menu
2. WHEN I click on "Audio Normalizer" tab THEN the system SHALL show the audio-normalizer-interactive executable with its interface
3. WHEN I click on "Session Monitor" tab THEN the system SHALL show the session-monitor executable with its interface
4. WHEN I click on "Pro Tools Session Launcher" tab THEN the system SHALL show the ptsl-launcher executable with its interface
5. WHEN I click on "Fifth Launcher" tab THEN the system SHALL show the fifth executable with its interface

### Requirement 3

**User Story:** As a user, I want to interact with each executable exactly as I would in a normal terminal, so that I can use all the existing functionality without any changes.

#### Acceptance Criteria

1. WHEN an executable shows its menu THEN I SHALL see all the options displayed automatically
2. WHEN I type input in any terminal THEN the executable SHALL receive that input and respond normally
3. WHEN an executable produces output THEN I SHALL see that output displayed in real-time
4. WHEN I drag files into any terminal THEN the executable SHALL receive the file paths as it normally would