use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tree_sitter::*;

// Simplified struct to hold AST elements for demonstration
#[derive(Default, Serialize, Deserialize)]
struct ASTElement {
    name: String,
    kind: String,
    children: Vec<ASTElement>,
    relations: Vec<String>, // Simplified relation representation
}

impl ASTElement {
    fn new(name: String, kind: String) -> Self {
        ASTElement {
            name,
            kind,
            children: Vec::new(),
            relations: Vec::new(),
        }
    }
}

struct ASTConversionService {
    code: String,
    tree: Tree,
}

impl ASTConversionService {
    fn new(code: String) -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .expect("Error loading Rust grammar");
        let tree = parser.parse(&code, None).expect("Failed to parse code");
        ASTConversionService { code, tree }
    }
    fn generate_ast_with_relations(&self) -> String {
        let mut ast_root = ASTElement::new("Root".to_string(), "Root".to_string());
        self.build_ast(self.tree.root_node(), &mut ast_root);

        // Convert AST to JSON
        let json_ast = json!(ast_root);
        serde_json::to_string_pretty(&json_ast).unwrap()
    }
    fn build_ast(&self, node: Node, parent: &mut ASTElement) {
        if let Some(node_name) = node.child_by_field_name("name") {
            let body = self.node_text(node);
            println!("{}: {} => {}", node.kind(), node_name, body);
        };
        let mut element =
            ASTElement::new(self.node_text(node).to_string(), node.kind().to_string());
        // Simplified relation handling (e.g., function to method, struct to field)
        if node.child_count() > 0 {
            for child in node.children(&mut node.walk()) {
                self.build_ast(child, &mut element);
                element
                    .relations
                    .push(format!("{} -> {}", parent.name, element.name));
            }
        }
        parent.children.push(element);
    }
    fn node_text(&self, node: Node) -> String {
        self.code[node.byte_range()].to_string()
    }
}

// Example usage (assuming `create_ast_from_code_file` is defined as before)
fn main() {
    // let language = "python"; // Example language
    // let file_path = "path/to/example.py"; // Example file path

    let code =
        std::fs::read_to_string("src/try2.rs").expect("Failed to read the Rust source file.");

    let service = ASTConversionService::new(code);

    let ast_json = service.generate_ast_with_relations();
    // println!("{}", ast_json);
}
