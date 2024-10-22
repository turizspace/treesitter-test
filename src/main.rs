use tree_sitter::{Language, Node, Parser, Tree};
use tree_sitter_rust;

use serde_json::{json, Value};
use std::env;
use std::fs;

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
    fn gen_json(&self) -> Value {
        let root_node = self.tree.root_node();
        let mut j = json!({});
        for child in root_node.children(&mut root_node.walk()) {}
        j
    }
    fn generate_json(&self) -> Value {
        let root_node = self.tree.root_node();
        json!({
            "imports": self.extract_imports(root_node),
            "functions": self.extract_functions(root_node),
            "structs": self.extract_structs(root_node),
            "enums": self.extract_enums(root_node),
            "relations": self.extract_relations(root_node),
            "constants": self.extract_constants(root_node),
            "modules_and_impls": self.extract_modules_and_impls(root_node),
            "metadata": self.extract_metadata(root_node),
            "nested_items": self.extract_nested(root_node),
            "globals": self.extract_globals(root_node),
            "schemas": self.extract_schema(root_node),
        })
    }
    fn extract_imports(&self, node: Node) -> Vec<Value> {
        let mut imports = Vec::new();
        for child in node.children(&mut node.walk()) {
            if child.kind() == "use_declaration" {
                imports.push(json!({
                    "name": self.node_text(child)
                }));
            }
        }
        imports
    }
    fn extract_functions(&self, node: Node) -> Vec<Value> {
        let mut functions = Vec::new();
        for child in node.children(&mut node.walk()) {
            if child.kind() == "function_item" {
                let function_name_node = child.child_by_field_name("name").unwrap();
                let function_name = self.node_text(function_name_node);
                let parameters = self.extract_parameters(child);
                let body = self.node_text(child);
                let called_methods = self.extract_called_methods(child);
                let local_variables = self.extract_method_variables(child);
                functions.push(json!({
                    "name": function_name,
                    "parameters": parameters,
                    "body": body,
                    "called_methods": called_methods,
                    "local_variables": local_variables
                }));
            }
        }
        functions
    }
    fn extract_parameters(&self, function_node: Node) -> Vec<Value> {
        let mut parameters = Vec::new();
        if let Some(parameters_node) = function_node.child_by_field_name("parameters") {
            for param in parameters_node.named_children(&mut parameters_node.walk()) {
                let param_name = self.node_text(param);
                let param_type = param.child_by_field_name("type").map(|n| self.node_text(n));
                let is_mutable = param.kind() == "mut";
                let is_reference = param_name.starts_with('&');
                let default_value = param
                    .child_by_field_name("default_value")
                    .map(|n| self.node_text(n));
                parameters.push(json!({
                    "name": param_name,
                    "type": param_type,
                    "is_mutable": is_mutable,
                    "is_reference": is_reference,
                    "default_value": default_value,
                }));
            }
        }
        parameters
    }
    fn extract_called_methods(&self, function_node: Node) -> Vec<Value> {
        let mut called_methods = Vec::new();
        for descendant in function_node.children(&mut function_node.walk()) {
            if descendant.kind() == "call_expression" {
                if let Some(method_name_node) = descendant.child_by_field_name("function") {
                    let method_name = self.node_text(method_name_node);
                    called_methods.push(json!({
                        "name": method_name
                    }));
                }
            }
        }
        called_methods
    }
    fn extract_method_variables(&self, function_node: Node) -> Vec<Value> {
        let mut variables = Vec::new();
        for descendant in function_node.children(&mut function_node.walk()) {
            if descendant.kind() == "let_declaration" {
                let variable_name = self.node_text(descendant.child_by_field_name("name").unwrap());
                let value_node = descendant.child_by_field_name("value");
                let value_type = value_node.map(|n| self.node_text(n));
                variables.push(json!({
                    "name": variable_name,
                    "type": value_type
                }));
            }
        }
        variables
    }
    fn extract_structs(&self, node: Node) -> Vec<Value> {
        let mut structs = Vec::new();
        for child in node.children(&mut node.walk()) {
            if child.kind() == "struct_item" {
                if let Some(struct_name_node) = child.child_by_field_name("name") {
                    let struct_name = self.node_text(struct_name_node);
                    let fields = self.extract_fields(child);
                    structs.push(json!({
                        "name": struct_name,
                        "fields": fields
                    }));
                }
            }
        }
        structs
    }
    fn extract_fields(&self, struct_node: Node) -> Vec<Value> {
        let mut fields = Vec::new();
        if let Some(body_node) = struct_node.child_by_field_name("body") {
            for field in body_node.named_children(&mut body_node.walk()) {
                let field_name = self.node_text(field.child_by_field_name("name").unwrap());
                let field_type = field.child_by_field_name("type").map(|n| self.node_text(n));
                let attributes = self.extract_metadata(field);
                fields.push(json!({
                    "name": field_name,
                    "type": field_type,
                    "attributes": attributes
                }));
            }
        }
        fields
    }
    fn extract_enums(&self, node: Node) -> Vec<Value> {
        let mut enums = Vec::new();
        for child in node.children(&mut node.walk()) {
            if child.kind() == "enum_item" {
                let enum_name_node = child.child_by_field_name("name").unwrap();
                let enum_name = self.node_text(enum_name_node);
                let variants = self.extract_variants(child);
                enums.push(json!({
                    "name": enum_name,
                    "variants": variants
                }));
            }
        }
        enums
    }
    fn extract_variants(&self, enum_node: Node) -> Vec<Value> {
        let mut variants = Vec::new();
        if let Some(body_node) = enum_node.child_by_field_name("body") {
            for variant in body_node.named_children(&mut body_node.walk()) {
                let variant_name = self.node_text(variant.child_by_field_name("name").unwrap());
                variants.push(json!({
                    "name": variant_name
                }));
            }
        }
        variants
    }
    fn extract_relations(&self, node: Node) -> Vec<Value> {
        let mut relations = Vec::new();
        for child in node.children(&mut node.walk()) {
            if child.kind() == "impl_item" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let type_name = self.node_text(name_node);
                    let trait_node = child.child_by_field_name("trait");
                    let trait_name = trait_node.map(|n| self.node_text(n));
                    let generic_params = child
                        .child_by_field_name("generic_parameters")
                        .map(|n| self.node_text(n));
                    relations.push(json!({
                        "type": "impl",
                        "for": type_name,
                        "trait": trait_name,
                        "generics": generic_params,
                    }));
                }
            } else if child.kind() == "attribute_item" {
                if let Some(attribute_text) = self.extract_metadata(child).get(0) {
                    if attribute_text["attribute"]
                        .as_str()
                        .unwrap_or("")
                        .contains("derive")
                    {
                        relations.push(json!({
                            "type": "derive",
                            "details": attribute_text
                        }));
                    }
                }
            }
        }
        relations
    }
    fn extract_constants(&self, node: Node) -> Vec<Value> {
        let mut constants = Vec::new();
        for child in node.children(&mut node.walk()) {
            if child.kind() == "const_item" {
                let constant_name = self.node_text(child.child_by_field_name("name").unwrap());
                let constant_value =
                    self.node_text(child.child_by_field_name("value").unwrap_or(child));
                constants.push(json!({
                    "name": constant_name,
                    "value": constant_value
                }));
            }
        }
        constants
    }
    fn extract_modules_and_impls(&self, node: Node) -> Vec<Value> {
        let mut modules = Vec::new();
        for child in node.children(&mut node.walk()) {
            if child.kind() == "mod_item" || child.kind() == "impl_item" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = self.node_text(name_node);
                    modules.push(json!({
                        "type": child.kind(),
                        "name": name,
                        "children": self.extract_nested(child),
                    }));
                }
            }
        }
        modules
    }
    fn extract_metadata(&self, node: Node) -> Vec<Value> {
        let mut metadata = Vec::new();
        for child in node.children(&mut node.walk()) {
            if child.kind() == "attribute_item" {
                let attribute_text = self.node_text(child);
                metadata.push(json!({
                    "attribute": attribute_text
                }));
            }
        }
        metadata
    }
    fn extract_nested(&self, node: Node) -> Vec<Value> {
        let mut nested_items = Vec::new();
        for child in node.children(&mut node.walk()) {
            println!("===> child.kind() = {}", child.kind());
            // match child.kind() {
            // "mod_item" | "impl_item" | "function_item" | "struct_item" | "fn" => {
            if let Some(name_node) = child.child_by_field_name("name") {
                nested_items.push(json!({
                    "type": child.kind(),
                    "name": self.node_text(name_node),
                    "children": self.extract_nested(child),
                }));
            } else {
                let new_nested_items = self.extract_nested(child);
                // nested_items.push(json!({
                //     "type": child.kind(),
                //     "children": self.extract_nested(child),
                // }));
            }
            // }
            // _ => {}
            // }
        }
        nested_items
    }
    fn extract_globals(&self, node: Node) -> Vec<Value> {
        let mut globals = Vec::new();
        for child in node.children(&mut node.walk()) {
            if child.kind() == "static_item" {
                let global_name = self.node_text(child.child_by_field_name("name").unwrap());
                let global_value =
                    self.node_text(child.child_by_field_name("value").unwrap_or(child));
                globals.push(json!({
                    "name": global_name,
                    "value": global_value,
                }));
            }
        }
        globals
    }
    fn extract_schema(&self, node: Node) -> Vec<Value> {
        let mut schemas = Vec::new();
        for child in node.children(&mut node.walk()) {
            if child.kind() == "struct_item" {
                let struct_name = self.node_text(child.child_by_field_name("name").unwrap());
                let attributes = self.extract_metadata(child);
                let fields = self.extract_fields(child);
                // Extract relationships based on field attributes or annotations
                let relationships = fields
                    .iter()
                    .filter_map(|field| {
                        if let Some(attribute) = field.get("attributes") {
                            if attribute.as_str().unwrap_or("").contains("foreign_key") {
                                Some(json!({
                                    "field": field.get("name"),
                                    "relationship": "foreign_key"
                                }))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                schemas.push(json!({
                    "struct": struct_name,
                    "attributes": attributes,
                    "fields": fields,
                    "relationships": relationships
                }));
            }
        }
        schemas
    }
    fn node_text(&self, node: Node) -> String {
        self.code[node.byte_range()].to_string()
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <rust_source_file>", args[0]);
        std::process::exit(1);
    }
    let file_path = &args[1];
    let code = fs::read_to_string(file_path).expect("Failed to read the Rust source file.");

    let service = ASTConversionService::new(code);
    let json_output = service.generate_json();

    // Pretty-print the JSON output
    println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
}
