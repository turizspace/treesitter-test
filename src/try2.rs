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
            .set_language(&tree_sitter_rust::LANGUAGE.into())
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
        let kind_prnt = Kind::Root;
        let space = " ".repeat((level * 2) as usize);
        if level <= lvl_prnt {
            println!("{} {}: {}", space, level, node.kind());
            let body = self.node_text(node);
            // let namey = astring(node.child_by_field_name("name"));
            println!("          ·{}·", &body[0..cmp::min(18, body.len())]);
        }
        if let Some(_) = node.child_by_field_name("name") {
            //
        };
        let node_kind = node.kind().to_string();
        let is_parents_name = node_kind == "type_identifier";
        if is_parents_name {
            parent.name = Some(self.node_text(node).to_string());
        }
        if let Ok(kind) = Kind::from_str(&node_kind) {
            let mut element = Thing::new(kind, self.node_text(node));
            // Simplified relation handling (e.g., function to method, struct to field)
            if node.child_count() > 0 {
                for child in node.children(&mut node.walk()) {
                    self.build_ast(level + 1, child, &mut element);
                    element
                        .relations
                        .push(format!("{:?} -> {:?}", parent.kind, element.kind));
                }
            }
            parent.children.push(element);
        } else {
            if level == lvl_prnt {
                // println!("               ...")
            }
        }
    }
    fn node_text(&self, node: Node) -> String {
        self.code[node.byte_range()].to_string()
    }
}

fn astring(a: Option<String>) -> String {
    a.unwrap_or("".to_string())
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
