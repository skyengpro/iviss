import os
import re

def fix_body(body):
    lines = body.split('\n')
    new_lines = []
    for line in lines:
        stripped = line.strip()
        # Find property definitions: name: type or name?: type
        # Exclude methods, comments, already readonly, and nested objects
        if (stripped and 
            ':' in stripped and 
            not stripped.startswith('readonly') and 
            not stripped.startswith('//') and 
            not stripped.startswith('/*') and
            '{' not in stripped):
            
            # Split by first colon to get property name
            parts = stripped.split(':', 1)
            prop_name_part = parts[0].strip()
            
            # Check if it's a field and not a method
            if '(' not in prop_name_part:
                # It's a field!
                indent = line[:line.find(stripped)]
                new_line = indent + 'readonly ' + stripped
                new_lines.append(new_line)
            else:
                new_lines.append(line)
        else:
            new_lines.append(line)
    return '\n'.join(new_lines)

def fix_content(content):
    # 1. Named Interfaces
    content = re.sub(r'(interface\s+\w+\s*(?:extends\s+[^{]+)?{)([^}]+)(})', 
                     lambda m: m.group(1) + fix_body(m.group(2)) + m.group(3), content, flags=re.DOTALL)
    # 2. Named Types
    content = re.sub(r'(type\s+\w+\s*=\s*{)([^}]+)(})',
                     lambda m: m.group(1) + fix_body(m.group(2)) + m.group(3), content, flags=re.DOTALL)
    # 3. Inline Props in signatures
    content = re.sub(r'(:\s*{)([^}]+)(}\s*\))',
                     lambda m: m.group(1) + fix_body(m.group(2)) + m.group(3), content, flags=re.DOTALL)
    return content

def main():
    root_dir = 'src'
    for root, dirs, files in os.walk(root_dir):
        for file in files:
            if file.endswith('.tsx') or file.endswith('.ts'):
                path = os.path.join(root, file)
                if 'node_modules' in path: continue
                
                with open(path, 'r') as f:
                    content = f.read()
                
                new_content = fix_content(content)
                
                if new_content != content:
                    with open(path, 'w') as f:
                        f.write(new_content)
                    print(f"Fixed {path}")

if __name__ == "__main__":
    main()
