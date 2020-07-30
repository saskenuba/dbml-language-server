use std::collections::HashMap;

use log::info;
use tower_lsp::lsp_types::{Position as LspPosition, TextEdit, WorkspaceEdit};
use tree_sitter::{Node, Query, QueryCursor};
use url::Url;

use crate::{
    file::parse_file,
    wrappers::{Point, Range},
    LANGUAGE,
};

const RENAME_TABLE_RULE: &[&str; 2] = &[
    "((table_definition name: (identifier) @table_name) (#eq? @table_name \"{}\"))",
    "((table_field table: (identifier) @table_name )(#eq? @table_name \"{}\"))",
];

const RENAME_FIELD_RULE: &[&str; 2] = &[
    // Table with field declaration
    "((table_definition name: (identifier) @table_name (field_declaration_list (field_declaration \
     name: (identifier) @field_name ) ))
    (#eq? @table_name \"{}\" ) (#eq? @field_name \"{}\" ))",
    // Relationships
    "((table_field field: (identifier) @field_name) (#eq? @field_name \"{}\"))",
];

fn replace_rule_with_pattern(rule: &[&str], pattern: &str) -> Vec<String> {
    rule.iter().map(|c| c.replace("{}", pattern)).collect()
}

pub fn rename(
    source: String,
    edit_position: LspPosition,
    new_name: String,
    file_location: Url,
) -> Option<WorkspaceEdit> {
    let tree = parse_file(&source, None).unwrap();
    let mut edits_per_document = HashMap::new();
    let mut current_doc_changes: Vec<TextEdit> = Vec::new();

    let root_node = tree.root_node();

    // Retrieve current node
    let point_conversion = Point::from(edit_position);
    let current_node = root_node
        .descendant_for_point_range(*point_conversion, *point_conversion)
        .unwrap();

    // we can only rename if it is an identifier or an enum in a field
    if current_node.kind() != "identifier" {
        return None;
    }

    let current_node_value = current_node.utf8_text(source.as_ref()).unwrap();
    let current_node_kind = current_node.kind();
    let wat = current_node.prev_sibling();
    let papi = current_node.parent().unwrap();

    // TODO: Set rule by finding from where we are renaming

    // println!("rules");
    // let oi = table_rename_rules(&source, root_node, current_node_value, &|c: Node| {
    //     (c.utf8_text(&source).unwrap().to_string(), c.range())
    // });
    // println!("{:?}", oi);
    // oi.iter().for_each(|(_name, range)| {
    //     let edit = TextEdit {
    //         range: Range::from(*range).0,
    //         new_text: new_name.clone(),
    //     };
    //     current_doc_changes.push(edit);
    // });

    // let oi = field_rename_rules(&source, current_node_value, root_node);
    // oi.iter().for_each(|range| {
    //     let edit = TextEdit {
    //         range: range.0,
    //         new_text: new_name.clone(),
    //     };
    //     current_doc_changes.push(edit);
    // });

    // let mut cursor = root_node.walk();
    // 'outer: loop {
    // println!("Kind: {:?}", cursor.node().kind());
    // println!("Value: {:?}", cursor.node().utf8_text(&*source));
    // println!("field_name: {:?}", cursor.node().to_sexp());
    //
    // if current_node_value == cursor.node().utf8_text(&*source).unwrap() {
    // let oi = TextEdit {
    // range: Range::from(cursor.node().range()).0,
    // new_text: new_name.clone(),
    // };
    // current_doc_changes.push(oi)
    // }
    //
    // if !cursor.goto_first_child() && !cursor.goto_next_sibling() {
    // 'inner: loop {
    // if !cursor.goto_parent() {
    // break 'outer;
    // }
    // if cursor.goto_next_sibling() {
    // break 'inner;
    // }
    // }
    // }
    // }

    edits_per_document.insert(file_location, current_doc_changes);
    Some(WorkspaceEdit {
        changes: Some(edits_per_document),
        document_changes: None,
    })
}

/// Should resolve current node based on some rules
enum RenameRules {
    // Parent table: self field, indexes
    // On every relationship
    // Not table name
    Field,
    // Everytime
    // Not field names
    Table,
    // Only tables, field types
    Enum,
}

fn table_rename_rules<T, B: AsRef<[u8]>>(
    source: B,
    rule: &[&str],
    root_node: Node,
    identifier_to_replace: &str,
    func: &dyn Fn(Node) -> T,
) -> Vec<T> {
    let mut query_list: Vec<T> = vec![];

    let rule_with_pattern = replace_rule_with_pattern(rule, identifier_to_replace);
    for rule in rule_with_pattern.iter() {
        let query = Query::new(*LANGUAGE, rule).unwrap();

        let mut cursor = QueryCursor::new();
        let captures =
            cursor.captures(&query, root_node, |c| c.utf8_text(source.as_ref()).unwrap());

        for matched in captures {
            for capture in matched.0.captures.iter() {
                query_list.push(func(capture.node));
            }
        }
    }
    query_list
}

fn field_rename_rules<B: AsRef<[u8]>>(source: B, to_replace: &str, root_node: Node) -> Vec<Range> {
    let mut query_list: Vec<_> = vec![];

    let rule_with_pattern = replace_rule_with_pattern(RENAME_FIELD_RULE, to_replace);
    println!("rules: {:#?}", rule_with_pattern);
    for rule in rule_with_pattern.iter() {
        let query = Query::new(*LANGUAGE, rule).unwrap();

        let mut cursor = QueryCursor::new();
        let captures =
            cursor.captures(&query, root_node, |c| c.utf8_text(source.as_ref()).unwrap());

        for matched in captures {
            for capture in matched.0.captures.iter() {
                let node = capture.node;
                if node.parent().unwrap().kind() == "table_definition" {
                    continue;
                }
                info!("{}", node.utf8_text(source.as_ref()).unwrap());
                info!("{}", node.kind());
                query_list.push(Range::from(node.range()));
            }
        }
    }
    query_list
}
