import os
import ast
from pathlib import Path

def generate_python_map(directory):
    repo_map = []
    repo_map.append("=========================================")
    repo_map.append("🗺️ AIDER REPO MAP (Python AST Extract)")
    repo_map.append("=========================================\n")
    
    for root, _, files in os.walk(directory):
        if "node_modules" in root or ".git" in root or "venv" in root:
            continue
            
        for file in files:
            if file.endswith('.py'):
                filepath = Path(root) / file
                rel_path = filepath.relative_to(directory)
                
                try:
                    with open(filepath, 'r', encoding='utf-8') as f:
                        tree = ast.parse(f.read(), filename=str(filepath))
                        
                    file_map = [f"📄 {rel_path}:"]
                    has_content = False
                    
                    for node in ast.iter_child_nodes(tree):
                        if isinstance(node, ast.ClassDef):
                            file_map.append(f"  class {node.name}:")
                            for class_node in ast.iter_child_nodes(node):
                                if isinstance(class_node, ast.FunctionDef):
                                    file_map.append(f"    def {class_node.name}(...):")
                            has_content = True
                        elif isinstance(node, ast.FunctionDef):
                            file_map.append(f"  def {node.name}(...):")
                            has_content = True
                            
                    if has_content:
                        repo_map.extend(file_map)
                        repo_map.append("")
                except Exception as e:
                    repo_map.append(f"⚠️ Could not parse {rel_path}: {str(e)}")
                    
    return "\n".join(repo_map)

if __name__ == "__main__":
    current_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    target_dir = os.path.abspath(os.path.join(current_dir, ".."))
    
    map_content = generate_python_map(target_dir)
    output_path = os.path.join(target_dir, "V2", "01_quy_trinh_agent_san_pham", "20260424_13_repo_map_vks_ecms_auto.txt")
    
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write(map_content)
    
    print(f"Repo map generated successfully at: {output_path}")
