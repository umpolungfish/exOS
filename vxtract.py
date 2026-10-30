import json
from collections import defaultdict
import sys
from pathlib import Path

def extract_expressions(json_path):
    """
    Extract all 'expression' fields from the manuscript JSON.
    Works for both 'rohonc' and 'voynich' structures.
    """
    json_path = Path(json_path)
    
    if not json_path.exists():
        print(f"Error: File not found: {json_path}")
        return

    with open(json_path, 'r', encoding='utf-8') as f:
        data = json.load(f)

    # Determine which top-level key contains the content
    if "rohonc" in data:
        section = data["rohonc"]
        section_name = "rohonc"
    elif "voynich" in data:
        section = data["voynich"]
        section_name = "voynich"
    else:
        section = data
        section_name = "root"

    print(f"✅ Loaded {section_name} section with {len(section)} entries.\n")

    # Collect expressions and group identical ones
    grouped = defaultdict(list)

    for key, entry in sorted(section.items()):
        if isinstance(entry, dict) and "expression" in entry:
            expr = entry["expression"]
            grouped[expr].append(key)

    # Print results
    print(f"Found {len(grouped)} distinct expression variants across {len(section)} entries.\n")
    print("=" * 100)

    for variant_num, (expression, keys) in enumerate(grouped.items(), 1):
        print(f"\n📌 Variant {variant_num} — {len(keys)} entries")
        print(f"Keys: {', '.join(sorted(keys))}")
        print("\nExpression:")
        print(expression)
        print("\n" + "-" * 100)

    # Optional: Save to a clean JSON file
    output_file = json_path.parent / f"extracted_expressions_{section_name}.json"
    result_dict = {key: entry["expression"] for key, entry in section.items() 
                   if isinstance(entry, dict) and "expression" in entry}
    
    with open(output_file, 'w', encoding='utf-8') as f:
        json.dump(result_dict, f, indent=2, ensure_ascii=False)
    
    print(f"\n💾 All expressions saved to: {output_file}")


# ====================== RUN ======================
if __name__ == "__main__":
    # Default path (works in the current environment)
    default_path = "./rhcmanuscript_zfct.json"
    
    if len(sys.argv) > 1:
        path = sys.argv[1]
    else:
        path = default_path

    extract_expressions(path)
