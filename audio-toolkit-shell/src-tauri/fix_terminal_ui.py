import re

# Read the file
with open('src/main.rs', 'r') as f:
    content = f.read()

# Replace the UI rendering structure to use terminal emulator buffer
# Pattern 1: Replace the loop variable and add nested structure
content = re.sub(
    r'for row in &tab\.terminal_emulator\.buffer \{\s*let mut rich_text = egui::RichText::new\(&colored_text\.text\)',
    'for row in &tab.terminal_emulator.buffer {\n                            ui.horizontal(|ui| {\n                                for cell in row {\n                                    let mut rich_text = egui::RichText::new(cell.character.to_string())',
    content,
    flags=re.MULTILINE
)

# Pattern 2: Replace the color and bold references
content = re.sub(r'\.color\(colored_text\.color\)', '.color(cell.color)', content)
content = re.sub(r'if colored_text\.bold', 'if cell.bold', content)

# Pattern 3: Add proper closing braces for nested structure
content = re.sub(
    r'(\s+)ui\.label\(rich_text\);\s*\}',
    r'\1                    ui.label(rich_text);\n\1                }\n\1            });',
    content
)

# Write back to file
with open('src/main.rs', 'w') as f:
    f.write(content)

print("Fixed terminal UI rendering structure")
