#![feature(or_patterns)]
#![warn(unused_results)]
#![deny(
    missing_copy_implementations,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications
)]

//! DBML is a very simple language with a single document structure, with no imports at all.
//!
//! With this in mind, each request parses the whole document, since there is no need for the
//! additional complexity of storing old document trees.

use std::{collections::HashMap, sync::Mutex};

use log::info;
use once_cell::sync::Lazy;
use tower_lsp::lsp_types::Position as LspPosition;
use tree_sitter::{Language, Node, Parser, Query, QueryCapture, QueryCursor};

use crate::{
    navigation::node_parent_identifier,
    wrappers::{Point, Range},
};
use navigation::search_valid_node;

pub mod file;
pub mod navigation;
pub mod providers;
pub mod wrappers;

extern "C" {
    fn tree_sitter_dbml() -> Language;
}

pub static LANGUAGE: Lazy<Language> = Lazy::new(|| unsafe { tree_sitter_dbml() });

pub static PARSER: Lazy<Mutex<Parser>> = Lazy::new(|| {
    let mut parser = Parser::new();
    parser.set_language(*LANGUAGE).unwrap();
    Mutex::new(parser)
});

#[derive(Debug, Default)]
pub struct IdentifiersMap {
    tables_with_fields: HashMap<String, Vec<FieldInfo>>,
    enums_without_discriminants: Vec<String>,
}

impl IdentifiersMap {
    fn tables(&self) -> Option<Vec<String>> {
        Some(
            self.tables_with_fields
                .keys()
                .map(|keys| keys.to_string())
                .collect::<Vec<String>>(),
        )
    }

    fn fields_of_table(&self, table_name: &str) -> Option<Vec<FieldInfo>> {
        Some(self.tables_with_fields.get_key_value(table_name)?.1.clone())
    }
}

#[derive(Debug, Default, Eq, PartialEq, Clone)]
struct FieldInfo {
    text_name: String,
    /// Field's type
    r#type: String,
    /// Position range which fields are defined
    range: Range,
}

pub fn populate_identifiers<'a>(source: &'a [u8], root_node: Node<'a>) -> IdentifiersMap {
    let mut identifiers_map = IdentifiersMap {
        tables_with_fields: Default::default(),
        enums_without_discriminants: vec![],
    };

    populate_table_identifiers(source, root_node, &mut identifiers_map);
    populate_enum_identifiers(source, root_node, &mut identifiers_map);

    identifiers_map
}
fn populate_enum_identifiers(
    source: &[u8],
    root_node: Node,
    out_identifiers_map: &mut IdentifiersMap,
) {
    let enum_query =
        Query::new(*LANGUAGE, r#"(enum_definition name: (identifier) @name )"#).unwrap();

    let mut query_enum_cursor = QueryCursor::new();
    let enum_nodes = query_for_nodes(&mut query_enum_cursor, source, root_node, &enum_query);

    let enum_identifiers = enum_nodes
        .iter()
        .map(|node| node.utf8_text(source).unwrap().to_string())
        .collect();

    out_identifiers_map.enums_without_discriminants = enum_identifiers;
}

fn populate_table_identifiers(
    source: &[u8],
    root_node: Node,
    out_identifiers_map: &mut IdentifiersMap,
) {
    let fields_query = Query::new(
        *LANGUAGE,
        r#"(table_definition name: (identifier)
                        (field_declaration_list 
                        	(field_declaration name: (identifier) @field_name)))"#,
    )
    .unwrap();

    let mut query_field_cursor = QueryCursor::new();
    let field_nodes = query_for_nodes(&mut query_field_cursor, source, root_node, &fields_query);

    for node in field_nodes {
        let table_name = node_parent_identifier(source, &node, "table_definition", "name").unwrap();
        let table_alias = node_parent_identifier(source, &node, "table_definition", "alias");
        let field_range = node.range();
        let field_name = node.utf8_text(source).unwrap();
        let field_type = node.next_sibling().unwrap().utf8_text(source).unwrap();
        let info = FieldInfo {
            text_name: field_name.to_string(),
            r#type: field_type.to_string(),
            range: field_range.into(),
        };

        if let Some(alias) = table_alias {
            out_identifiers_map
                .tables_with_fields
                .entry(alias)
                .and_modify(|e| e.push(info.clone()))
                .or_insert_with(|| vec![info.clone()]);
        }

        out_identifiers_map
            .tables_with_fields
            .entry(table_name)
            .and_modify(|e| e.push(info.clone()))
            .or_insert_with(|| vec![info]);
    }
}

/// Executes query and return captured nodes.
fn query_for_nodes<'a>(
    cursor: &'a mut QueryCursor,
    source: &'a [u8],
    root_node: Node<'a>,
    fields_query: &'a Query,
) -> Vec<Node<'a>> {
    let field_query_captures = cursor.captures(&fields_query, root_node, move |c| {
        c.utf8_text(source.as_ref()).unwrap().to_string()
    });

    field_query_captures
        .into_iter()
        .map(|query_match| query_match.0.captures)
        .flatten()
        .into_iter()
        .map(|query_capture: &QueryCapture| query_capture.node)
        .collect::<Vec<_>>()
}

fn find_location_on_ast(
    source: &[u8],
    root_node: Node,
    edit_position: LspPosition,
) -> CursorLocation {
    let current_pos = Point::from(edit_position);

    let current_node = root_node
        .named_descendant_for_point_range(*current_pos, *current_pos)
        .unwrap();
    let current_node_kind = current_node.kind();
    let parent_kind = current_node.parent().map(|c| c.kind());

    println!("root node: {:?}", root_node);
    println!("current_node: {:?}", current_pos);
    println!(
        "current_node: {:?}, {:?}",
        current_node.utf8_text(source),
        current_node
    );
    println!("prev sibling: {:?}", current_node.prev_sibling());
    println!(
        "prev sibling: {:?}",
        current_node.prev_sibling().map(|c| c.utf8_text(source))
    );
    println!(
        "parent name: {:?}, kind: {:?}",
        current_node.parent().map(|c| c.utf8_text(source)),
        current_node.parent().map(|c| c.kind())
    );

    println!("is error?, {:?}", root_node.is_error());
    println!("is missing?, {:?}", root_node.is_missing());

    if let "field_attribute_list" = current_node_kind {
        let new_node = search_valid_node(current_pos, root_node, "field_declaration_list");
        info!("{:?}", new_node);
        return CursorLocation::FieldAttributeList;
    }

    match parent_kind {
        Some("field_declaration_list" | "table_definition") => return CursorLocation::Field,
        _ => {}
    }

    // beyond this we have top level
    if current_node.kind() != "project_file" && current_node.kind() != "field_declaration_list" {
        return CursorLocation::Unknown;
    }

    println!("---- Searching for valid node.. ----");

    let current_node = search_valid_node(current_pos, root_node, "project_file");
    let current_node_kind = current_node.map(|c| c.kind());
    let parent_kind = current_node.map(|c| c.parent()).flatten().map(|c| c.kind());

    println!("current_node: {:?}", current_pos);
    println!(
        "current_node: {:?}, {:?}",
        current_node.map(|c| c.utf8_text(source)),
        current_node
    );
    println!("prev sibling: {:?}", current_node.map(|c| c.prev_sibling()));
    println!(
        "prev sibling: {:?}",
        current_node.map(|c| c.prev_sibling().map(|c| c.utf8_text(source)))
    );
    println!(
        "parent name: {:?}, kind: {:?}",
        current_node.map(|c| c.parent().map(|c| c.utf8_text(source))),
        current_node.map(|c| c.parent().map(|c| c.kind()))
    );

    if let Some(node) = current_node {
        let node_text = node.utf8_text(source).unwrap();

        if node_text == ":" || parent_kind == Some("cardinality_op") {
            return CursorLocation::TableField_Table;
        }
        if node_text == "." {
            let table = node.prev_sibling().unwrap().utf8_text(source).unwrap();
            println!("table_name from tablefield {:?}", table);
            return CursorLocation::TableField_Field(table.to_string());
        }
    }

    CursorLocation::Unknown
}

#[allow(non_camel_case_types)]
#[derive(Eq, PartialEq, Clone)]
enum CursorLocation {
    Unknown,
    Table,
    /// We are at the field declaration
    Field,
    FieldAttribute,
    /// We are at a specific field, on its attribute list
    FieldAttributeList,
    Enum,
    /// We are inside a relationship
    TableField_Table,
    TableField_Field(String),
}
