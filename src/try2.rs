use serde::{Deserialize, Serialize};
use serde_json::json;
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
            "use_declaration" => Ok(Kind::Import),
            "struct_item" => Ok(Kind::Struct),
            "enum_item" => Ok(Kind::Enum),
            "attribute_item" => Ok(Kind::Derive),
            "function_item" => Ok(Kind::Function),
            "impl_item" => Ok(Kind::Impl),
            "method_item" => Ok(Kind::Method),
            "field_declaration" => Ok(Kind::Field),
            "let_declaration" => Ok(Kind::Variable),
            "type_item" => Ok(Kind::Type),
            "trait_item" => Ok(Kind::Trait),
            "if_expression" => Ok(Kind::If),
            "else_clause" => Ok(Kind::Else),
            "loop_expression" => Ok(Kind::Loop),
            "tuple_expression" => Ok(Kind::Tuple),
            "array_expression" => Ok(Kind::Array),
            "call_expression" => Ok(Kind::FunctionCall),
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
        self.build_ast(self.tree.root_node(), &mut ast_root);
        // Convert AST to JSON
        let json_ast = json!(ast_root);
        serde_json::to_string_pretty(&json_ast).unwrap()
    }

    fn build_ast(&self, node: Node, parent: &mut Thing) {
        let node_kind = node.kind().to_string();
        if let Ok(kind) = Kind::from_str(&node_kind) {
            let mut element = Thing::new(kind, self.node_text(node));

            // Extract name if available
            if let Some(name_node) = node.child_by_field_name("name") {
                element.name = Some(self.node_text(name_node));
            }

            // Recursively process child nodes
            for child in node.children(&mut node.walk()) {
                self.build_ast(child, &mut element);
            }

            // Add relations based on parent-child relationships
            element.relations.push(format!("{:?} -> {:?}", parent.kind, element.kind));

            parent.children.push(element);
        }
    }

    fn node_text(&self, node: Node) -> String {
        self.code[node.byte_range()].to_string()
    }
}

fn main() {
    let code =
        std::fs::read_to_string("src/try2.rs").expect("Failed to read the Rust source file.");

    let service = ASTConversionService::new(code);

    let ast_json = service.generate_ast_with_relations();
    println!("{}", ast_json);
}
