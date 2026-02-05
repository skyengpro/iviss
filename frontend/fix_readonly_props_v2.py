import os
import re

def fix_content(content):
    # 1. Fix Named Interfaces and Types ending with Props
    # Example: interface MyProps { foo: string; }
    patterns = [
        re.compile(r'((?:interface|type)\s+\w+Props\s*(?:=)?\s*{)([^}]+)(})', re.MULTILINE | re.DOTALL),
        # 2. Fix Inline Props in function signatures
        # Example: function MyComp({ foo }: { foo: string })
        re.compile(r'(\w+\s*:\s*{)([^}]+)(}\s*\))', re.MULTILINE | re.DOTALL)
    ]
    
    current_content = content
    for pattern in patterns:
        def replacer(match):
            header = match.group(1)
            body = match.group(2)
            footer = match.group(3)
            
            lines = body.split('\n')
            new_lines = []
            for line in lines:
                stripped = line.strip()
                # Match property: type but exclude already readonly, comments, and nested objects
                # Also exclude methods like 'render(): void' or 'onReset(): void' if they don't have ':'
                if stripped and ':' in stripped and not stripped.startswith('readonly') and not stripped.startswith('//') and not stripped.startswith('/*'):
                    # Check if it's a property (has :) and not a method (doesn't start with function or have => in the name part)
                    prop_name = stripped.split(':')[0].strip()
                    if '(' not in prop_name:
                        indent = line[:line.find(stripped)]
                        new_line = indent + 'readonly ' + stripped
                        new_lines.append(new_line)
                    else:
                        new_lines.append(line)
                else:
                    new_lines.append(line)
            
            return header + '\n'.join(new_lines) + footer
        
        current_content = pattern.sub(replacer, current_content)
    
    return current_content

def main():
    root_dir = 'src'
    for root, dirs, files in os.walk(root_dir):
        for file in files:
            if file.endswith('.tsx') or file.endswith('.ts'):
                path = os.path.join(root, file)
                with open(path, 'r') as f:
                    content = f.read()
                
                new_content = fix_content(content)
                
                if new_content != content:
                    with open(path, 'w') as f:
                        f.write(new_content)
                    print(f"Fixed {path}")

if __name__ == "__main__":
    main()
