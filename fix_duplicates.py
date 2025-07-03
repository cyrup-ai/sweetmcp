import os
import re

# Find all plugin files with duplicate return_error functions
plugin_files = []
for root, dirs, files in os.walk("sweetmcp-plugins"):
    for file in files:
        if file in ["plugin.rs", "pdk.rs"]:
            filepath = os.path.join(root, file)
            plugin_files.append(filepath)

for filepath in plugin_files:
    if not os.path.exists(filepath):
        continue
        
    with open(filepath, 'r') as f:
        content = f.read()
    
    # Find the internal module
    internal_match = re.search(r'pub\(crate\) mod internal \{(.*?)\n\}', content, re.DOTALL)
    if not internal_match:
        continue
    
    internal_content = internal_match.group(1)
    
    # Find all return_error functions
    return_error_pattern = r'(    pub\(crate\) fn return_error\(e: extism_pdk::Error\) -> i32 \{[^}]+\})'
    return_error_matches = list(re.finditer(return_error_pattern, internal_content, re.DOTALL))
    
    if len(return_error_matches) > 1:
        print(f"Found {len(return_error_matches)} return_error functions in {filepath}")
        
        # Keep only the first one
        first_func = return_error_matches[0].group(0)
        
        # Remove all return_error functions
        cleaned_internal = re.sub(return_error_pattern, '', internal_content, flags=re.DOTALL)
        
        # Add back just the first one
        cleaned_internal = first_func + cleaned_internal
        
        # Replace the internal module content
        new_content = content.replace(internal_match.group(0), 
                                     f'pub(crate) mod internal {{{cleaned_internal}\n}}')
        
        with open(filepath, 'w') as f:
            f.write(new_content)
        print(f"Fixed {filepath}")
