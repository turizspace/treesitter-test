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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
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
            // Add more mappings as needed
            _ => Err(()),
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
            .set_language(tree_sitter_rust::LANGUAGE)
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

        // Handle specific node types and naming
        if let Some(name_node) = node.child_by_field_name("name") {
            parent.name = Some(self.node_text(name_node).to_string());
        }

        let node_kind = node.kind().to_string();
        if let Ok(kind) = Kind::from_str(&node_kind) {
            let mut element = Thing::new(kind, self.node_text(node));

            // Sort children based on their byte positions
            let mut children: Vec<Node> = node.children(&mut node.walk()).collect();
            children.sort_by_key(|child| child.start_byte());

            for child in children {
                self.build_ast(level + 1, child, &mut element);
                element.relations.push(format!("{:?} -> {:?}", parent.kind, element.kind));
            }

            parent.children.push(element);
        } else {
            if level == lvl_prnt {
                // Optional debugging message for nodes that are not recognized by Kind
                println!("{} Unrecognized node kind: {}", space, node.kind());
            }
        }
    }

    fn node_text(&self, node: Node) -> String {
        self.code[node.byte_range()].to_string()
    }
}

fn main() {
    let code = std::fs::read_to_string("src/try1.rs").expect("Failed to read the Rust source file.");
    let service = ASTConversionService::new(code);

    let ast_json = service.generate_ast_with_relations();
    println!("{}", ast_json);
}
