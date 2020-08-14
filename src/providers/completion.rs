use tower_lsp::lsp_types::{CompletionContext, Position as LspPosition};
use tree_sitter::Tree;

use crate::{find_location_on_ast, CursorLocation, IdentifiersMap};

const FIELD_ATTRIBUTES: &[&str] = &["not null", "null", "pk", "unique", "increment", "ref:"];
const KEYWORDS: &[&str] = &["table", "enum", "ref:"];
const PRIMITIVE_TYPES: &[&str] = &["int", "float", "text", "varchar"];

pub fn complete_at_point(
    source: String,
    tree: Tree,
    identifiers: &IdentifiersMap,
    edit_position: LspPosition,
    context: CompletionContext,
) -> Option<Vec<String>> {
    let root_node = tree.root_node();
    let completion_character = context.trigger_character.as_deref();

    let valid_position = find_location_on_ast(source.as_bytes(), root_node, edit_position);

    if CursorLocation::TableField_Table == valid_position {
        let tables_identifiers = identifiers.tables()?;
        return Some(tables_identifiers);
    }

    if let CursorLocation::TableField_Field(table_name) = valid_position {
        let fields = identifiers.fields_of_table(&table_name)?;
        return Some(
            fields
                .iter()
                .map(|field| field.text_name.to_owned())
                .collect(),
        );
    }

    // dentro de tabelas

    // caso estiver dentro de um field_declaration_list
    // é por conta que está dentro de uma tabela
    // fornecer atributos e enums se for o caso
    if CursorLocation::Field == valid_position {
        return Some(
            [
                identifiers.enums_without_discriminants.as_slice(),
                PRIMITIVE_TYPES
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .as_slice(),
            ]
            .concat(),
        );
    }

    // se o current node for field_attribute_list
    // ou receber o caractere '['
    // fornecer lista de atributos
    // TODO: concat with available enums
    if completion_character == Some("[") || valid_position == CursorLocation::FieldAttributeList {
        return Some(FIELD_ATTRIBUTES.iter().map(|c| c.to_string()).collect());
    }
    None
}
