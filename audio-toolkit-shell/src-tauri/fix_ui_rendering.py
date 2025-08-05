import re

# Read the file
with open('src/main.rs', 'r') as f:
    content = f.read()

# Pattern to find the broken UI rendering blocks
pattern = r'(\s+)for row in &tab\.terminal_emulator\.buffer \{\s*let mut rich_text = egui::RichText::new\(&cell\.character\.to_string\(\)\)'

# Replacement with proper nested structure
replacement = r'''\1for row in &tab.terminal_emulator.buffer {
\1    ui.horizontal(|ui| {
\1        for cell in row {
\1            let mut rich_text = egui::RichText::new(cell.character.to_string())'''

# Apply the replacement
content = re.sub(pattern, replacement, content, flags=re.MULTILINE | re.DOTALL)

# Also need to fix the closing braces
content = re.sub(r'(\s+)ui\.label\(rich_text\);\s*\}', r'\1            ui.label(rich_text);\n\1        }\n\1    });', content)

# Write back to file
with open('src/main.rs', 'w') as f:
    f.write(content)

print("Fixed UI rendering structure")
