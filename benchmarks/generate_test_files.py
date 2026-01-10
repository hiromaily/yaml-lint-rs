#!/usr/bin/env python3
"""Generate YAML test files of various sizes for benchmarking."""

import os

def generate_yaml_content(num_items: int) -> str:
    """Generate valid YAML content with the specified number of items."""
    lines = ["---", "# Generated YAML file for benchmarking", ""]
    
    for i in range(num_items):
        lines.append(f"item_{i}:")
        lines.append(f"  name: \"Item number {i}\"")
        lines.append(f"  value: {i * 100}")
        lines.append(f"  enabled: {'true' if i % 2 == 0 else 'false'}")
        lines.append(f"  tags:")
        lines.append(f"    - tag_{i}_a")
        lines.append(f"    - tag_{i}_b")
        lines.append(f"  metadata:")
        lines.append(f"    created: 2024-01-{(i % 28) + 1:02d}")
        lines.append(f"    version: {i}.0.0")
        lines.append("")
    
    return "\n".join(lines)


def main():
    sizes = [
        ("small", 10),      # ~100 lines
        ("medium", 100),    # ~1,000 lines
        ("large", 1000),    # ~10,000 lines
        ("xlarge", 5000),   # ~50,000 lines
    ]
    
    os.makedirs("files", exist_ok=True)
    
    for name, num_items in sizes:
        content = generate_yaml_content(num_items)
        filename = f"files/{name}.yaml"
        with open(filename, "w") as f:
            f.write(content)
        
        line_count = content.count("\n")
        print(f"Generated {filename}: {num_items} items, ~{line_count} lines")


if __name__ == "__main__":
    main()
