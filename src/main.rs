use tree_sitter::{Node, Parser, Tree};
use tree_sitter_rust;

use serde_json::{json, Value};
use std::env;
use std::fs;

#[derive(Debug, serde::Serialize)]
struct Thing {
    name: String,
    attributes: Vec<Value>,
    children: Vec<Thing>,
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

    fn extract_imports(&self, node: Node) -> Vec<Thing> {
        let mut imports = Vec::new();
        for child in node.children(&mut node.walk()) {
            if child.kind() == "use_declaration" {
                imports.push(Thing {
                    name: self.node_text(child),
                    attributes: vec![],
                    children: vec![],
                });
            }
        }
        imports
    }

    fn extract_functions(&self, node: Node) -> Vec<Thing> {
    let mut functions = Vec::new();
    for child in node.children(&mut node.walk()) {
        if child.kind() == "function_item" {
            let function_name_node = child.child_by_field_name("name").unwrap();
            let function_name = self.node_text(function_name_node);
            let parameters = self.extract_parameters(child);
            let body = self.node_text(child);
            let called_methods = self.extract_called_methods(child);
            let local_variables = self.extract_method_variables(child);

            // Create a JSON object for attributes
            let attributes_json = json!({
                "parameters": parameters,
                "body": body,
                "called_methods": called_methods,
                "local_variables": local_variables
            });

            // Collect attributes into a Vec<serde_json::Value>
            let attributes = vec![
                attributes_json["parameters"].clone(),
                attributes_json["body"].clone(),
                attributes_json["called_methods"].clone(),
                attributes_json["local_variables"].clone(),
            ];

            functions.push(Thing {
                name: function_name,
                attributes, // Use the constructed attributes vector
                children: vec![],
            });
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

    fn extract_structs(&self, node: Node) -> Vec<Thing> {
        let mut structs = Vec::new();
        for child in node.children(&mut node.walk()) {
            if child.kind() == "struct_item" {
                if let Some(struct_name_node) = child.child_by_field_name("name") {
                    let struct_name = self.node_text(struct_name_node);
                    let fields = self.extract_fields(child);
                    structs.push(Thing {
                        name: struct_name,
                        attributes: vec![],
                        children: fields,
                    });
                }
            }
        }
        structs
    }

    fn extract_fields(&self, struct_node: Node) -> Vec<Thing> {
    let mut fields = Vec::new();

    // Check for the presence of the "body" child node
    if let Some(body_node) = struct_node.child_by_field_name("body") {
        for field in body_node.named_children(&mut body_node.walk()) {
            // Attempt to get the field name and handle the case where it may not exist
            let field_name_node = field.child_by_field_name("name");
            let field_name = match field_name_node {
                Some(name_node) => self.node_text(name_node),
                None => {
                    eprintln!("Warning: 'name' field not found in field node.");
                    continue; // Skip this field if name is missing
                }
            };

            // Attempt to get the field type, if available
            let field_type = field.child_by_field_name("type").map(|n| self.node_text(n));

            // Extract attributes
            let attributes = self.extract_metadata(field);

            // Construct the Thing object and push it to the fields vector
            fields.push(Thing {
                name: field_name,
                attributes: vec![json!({
                    "type": field_type,
                    "attributes": attributes
                })],
                children: vec![],
            });
        }
    }
    fields
}


    fn extract_enums(&self, node: Node) -> Vec<Thing> {
        let mut enums = Vec::new();
        for child in node.children(&mut node.walk()) {
            if child.kind() == "enum_item" {
                let enum_name_node = child.child_by_field_name("name").unwrap();
                let enum_name = self.node_text(enum_name_node);
                let variants = self.extract_variants(child);
                enums.push(Thing {
                    name: enum_name,
                    attributes: vec![],
                    children: variants,
                });
            }
        }
        enums
    }

    fn extract_variants(&self, enum_node: Node) -> Vec<Thing> {
        let mut variants = Vec::new();
        if let Some(body_node) = enum_node.child_by_field_name("body") {
            for variant in body_node.named_children(&mut body_node.walk()) {
                let variant_name = self.node_text(variant.child_by_field_name("name").unwrap());
                variants.push(Thing {
                    name: variant_name,
                    attributes: vec![],
                    children: vec![],
                });
            }
        }
        variants
    }

    fn extract_relations(&self, node: Node) -> Vec<Thing> {
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
                    relations.push(Thing {
                        name: type_name,
                        attributes: vec![json!({
                            "type": "impl",
                            "trait": trait_name,
                            "generics": generic_params,
                        })],
                        children: vec![],
                    });
                }
            } else if child.kind() == "attribute_item" {
                if let Some(attribute_text) = self.extract_metadata(child).get(0) {
                    if attribute_text["attribute"]
                        .as_str()
                        .unwrap_or("")
                        .contains("derive")
                    {
                        relations.push(Thing {
                            name: "derive".to_string(),
                            attributes: vec![attribute_text.clone()],
                            children: vec![],
                        });
                    }
                }
            }
        }
        relations
    }

    fn extract_constants(&self, node: Node) -> Vec<Thing> {
        let mut constants = Vec::new();
        for child in node.children(&mut node.walk()) {
            if child.kind() == "const_item" {
                let constant_name = self.node_text(child.child_by_field_name("name").unwrap());
                let constant_value =
                    self.node_text(child.child_by_field_name("value").unwrap_or(child));
                constants.push(Thing {
                    name: constant_name,
                    attributes: vec![json!({
                        "value": constant_value
                    })],
                    children: vec![],
                });
            }
        }
        constants
    }

    fn extract_modules_and_impls(&self, node: Node) -> Vec<Thing> {
        let mut modules = Vec::new();
        for child in node.children(&mut node.walk()) {
            if child.kind() == "mod_item" {
                let module_name = self.node_text(child.child_by_field_name("name").unwrap());
                modules.push(Thing {
                    name: module_name,
                    attributes: vec![],
                    children: vec![],
                });
            }
        }
        modules
    }

    fn extract_metadata(&self, node: Node) -> Vec<Value> {
    let mut metadata = Vec::new();
    for child in node.children(&mut node.walk()) {
        if child.kind() == "attribute_item" {
            // Use match to handle the result of child.child_by_field_name
            match child.child_by_field_name("attribute") {
                Some(attribute_name_node) => {
                    let attribute_name = self.node_text(attribute_name_node);
                    metadata.push(json!({
                        "attribute": attribute_name,
                    }));
                }
                None => {
                    eprintln!("Warning: 'attribute' field not found in child node.");
                    // Optionally, you can skip this child or add a default value
                }
            }
        }
    }
    metadata
}


    fn extract_nested(&self, node: Node) -> Vec<Thing> {
        let mut nested_items = Vec::new();
        for child in node.children(&mut node.walk()) {
            if child.kind() == "function_item" {
                let function_name_node = child.child_by_field_name("name").unwrap();
                let function_name = self.node_text(function_name_node);
                nested_items.push(Thing {
                    name: function_name,
                    attributes: vec![],
                    children: vec![],
                });
            } else if child.kind() == "struct_item" {
                let struct_name_node = child.child_by_field_name("name").unwrap();
                let struct_name = self.node_text(struct_name_node);
                nested_items.push(Thing {
                    name: struct_name,
                    attributes: vec![],
                    children: vec![],
                });
            } else if child.kind() == "enum_item" {
                let enum_name_node = child.child_by_field_name("name").unwrap();
                let enum_name = self.node_text(enum_name_node);
                nested_items.push(Thing {
                    name: enum_name,
                    attributes: vec![],
                    children: vec![],
                });
            }
        }
        nested_items
    }

    fn extract_globals(&self, node: Node) -> Vec<Thing> {
        let mut globals = Vec::new();
        for child in node.children(&mut node.walk()) {
            if child.kind() == "let_declaration" {
                let variable_name = self.node_text(child.child_by_field_name("name").unwrap());
                let value_node = child.child_by_field_name("value");
                let value_type = value_node.map(|n| self.node_text(n));
                globals.push(Thing {
                    name: variable_name,
                    attributes: vec![json!({
                        "type": value_type
                    })],
                    children: vec![],
                });
            }
        }
        globals
    }

    fn extract_schema(&self, node: Node) -> Vec<Thing> {
        let mut schemas = Vec::new();
        for child in node.children(&mut node.walk()) {
            if child.kind() == "struct_item" {
                let struct_name = child.child_by_field_name("name").map(|n| self.node_text(n));
                let fields = self.extract_fields(child);
                schemas.push(Thing {
                    name: struct_name.unwrap_or_else(|| "unknown".to_string()),
                    attributes: vec![],
                    children: fields,
                });
            }
        }
        schemas
    }

    fn node_text(&self, node: Node) -> String {
    let start = node.start_byte();
    let end = node.end_byte();
    self.code[start..end].to_string()
}
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Please provide a Rust file path as an argument.");
        return;
    }

    let path = &args[1];
    let code = fs::read_to_string(path).expect("Unable to read file");
    let service = ASTConversionService::new(code);
    let json_output = service.generate_json();
    println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
}
