import re

# Read the file
with open('src/main.rs', 'r') as f:
    content = f.read()

# Fix the first UI rendering section (around line 943-963)
pattern1 = r'''// Render colored text segments
                    ui\.horizontal_wrapped\(\|ui\| \{
                        for row in &tab\.terminal_emulator\.buffer \{
                            ui\.horizontal\(\|ui\| \{
                                for cell in row \{
                                    let mut rich_text = egui::RichText::new\(cell\.character\.to_string\(\)\)
                                \.font\(egui::FontId::monospace\(12\.0\)\)
                                \.color\(cell\.color\);
                            
                            if cell\.bold \{
                                rich_text = rich_text\.strong\(\);
                            \}
                            
                                                ui\.label\(rich_text\);

                            
                                            \}

                            
                                        \}\);
                    \}\);'''

replacement1 = '''// Render terminal emulator buffer
                    ui.vertical(|ui| {
                        for row in &tab.terminal_emulator.buffer {
                            ui.horizontal(|ui| {
                                for cell in row {
                                    let mut rich_text = egui::RichText::new(cell.character.to_string())
                                        .font(egui::FontId::monospace(12.0))
                                        .color(cell.color);
                                    
                                    if cell.bold {
                                        rich_text = rich_text.strong();
                                    }
                                    
                                    ui.label(rich_text);
                                }
                            });
                        }
                    });'''

content = re.sub(pattern1, replacement1, content, flags=re.MULTILINE | re.DOTALL)

# Also fix any remaining similar patterns
content = re.sub(
    r'ui\.horizontal_wrapped\(\|ui\| \{\s*for row in &tab\.terminal_emulator\.buffer',
    'ui.vertical(|ui| {\n                        for row in &tab.terminal_emulator.buffer',
    content
)

# Write back to file
with open('src/main.rs', 'w') as f:
    f.write(content)

print("Fixed UI rendering structure completely")
