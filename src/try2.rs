use serde::{Deserialize, Serialize};
use serde_json::json;
use std::cmp;
use std::str::FromStr;
use tree_sitter::*;

// Simplified struct to hold AST elements for demonstration
#[derive(Default, Serialize, Deserialize, Debug)]
struct Thing {
    kind: Kind,
    name: Option<String>,
    text: String,
    children: Vec<Thing>,
    relations: Vec<String>, // Simplified relation representation
}

impl Thing {
    fn new(kind: Kind, text: String) -> Self {
        Thing {
            kind,
            name: None,
            text,
            children: Vec::new(),
            relations: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum Kind {
    Root,
    Comment,
    Import,
    Struct,
    Enum,
    Derive,
    Function,
    Method,
    Field,
    Variable,
    Type,
    Trait,
    Impl,
    If,
    Else,
    Loop,
    Tuple,
    Array,
    FunctionCall,
}

impl FromStr for Kind {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "source_file" => Ok(Kind::Root),
            "line_comment" => Ok(Kind::Comment),
            "import" => Ok(Kind::Import),
            "struct_item" => Ok(Kind::Struct),
            "enum_item" => Ok(Kind::Enum),
            "attribute_item" => Ok(Kind::Derive),
            "function_item" => Ok(Kind::Function),
            "impl_item" => Ok(Kind::Impl),
            _ => Err(()),
        }
    }
}
impl Default for Kind {
    fn default() -> Self {
        Kind::Comment
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
            .set_language(&tree_sitter_rust::LANGUAGE.into()) // Use Rust grammar for Tree-sitter
            .expect("Error loading Rust grammar");
        let tree = parser.parse(&code, None).expect("Failed to parse code");
        ASTConversionService { code, tree }
    }

    fn generate_ast_with_relations(&self) -> String {
        let mut ast_root = Thing::new(Kind::Root, "Root".to_string());
        self.build_ast(0, self.tree.root_node(), &mut ast_root);
        // Convert AST to JSON
        let json_ast = json!(ast_root);
        serde_json::to_string_pretty(&json_ast).unwrap()
    }

    fn build_ast(&self, level: u64, node: Node, parent: &mut Thing) {
        let lvl_prnt = 4;
        let space = " ".repeat((level * 2) as usize);

        if level <= lvl_prnt {
            println!("{} {}: {}", space, level, node.kind());
            let body = self.node_text(node);
            println!("          ·{}·", &body[0..cmp::min(18, body.len())]);
        }

        let node_kind = node.kind().to_string();

        // Match against `Kind` enum
        if let Ok(kind) = Kind::from_str(&node_kind) {
            let mut element = Thing::new(kind, self.node_text(node));

            // Simplified relation handling (e.g., function to method, struct to field)
            if node.child_count() > 0 {
                for child in node.children(&mut node.walk()) {
                    self.build_ast(level + 1, child, &mut element);
                    element.relations.push(format!("{:?} -> {:?}", parent.kind, element.kind));
                }
            }

            // Add the node only if it is part of the `Kind` enum
            parent.children.push(element);
        } else {
            // Skip nodes that don't match with the `Kind` enum (e.g., detailed expressions, literals)
            if level == lvl_prnt {
                // Optionally, print skipped nodes for debugging
                // println!("               ...")
            }
        }
    }

    // Capture function or method nodes
    fn build_function_or_method(&self, level: u64, node: Node, parent: &mut Thing) {
        let node_kind = node.kind().to_string();
        if node_kind == "function_item" || node_kind == "method_item" {
            let method_text = self.node_text(node);
            let mut method_element = Thing::new(Kind::Function, method_text);

            // Add parameters, body, etc., by recursing into the children
            for child in node.children(&mut node.walk()) {
                self.build_ast(level + 1, child, &mut method_element);
            }

            parent.children.push(method_element);
        }
    }

    fn node_text(&self, node: Node) -> String {
        self.code[node.byte_range()].to_string()
    }
}

fn astring(a: Option<String>) -> String {
    a.unwrap_or("".to_string())
}

fn main() {
    // Read the Rust file content to be parsed
    let code = std::fs::read_to_string("src/try2.rs").expect("Failed to read the Rust source file.");

    let service = ASTConversionService::new(code);

    let ast_json = service.generate_ast_with_relations();
    // Output the generated AST in JSON format
    println!("{}", ast_json);
}
