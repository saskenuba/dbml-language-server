use crate::wrappers::Point;
use log::debug;
use tree_sitter::Node;

/// If current node found is the topmost one, naively searches for another node by subtracting
/// columns until 0.
pub(crate) fn search_valid_node(current_point: Point, root_node: Node) -> Option<Node> {
    while let Some(descendant_node) =
        root_node.descendant_for_point_range(*current_point, *current_point)
    {
        current_point.column.checked_sub(1)?;
        debug!("searching on: {:?}", current_point);

        if descendant_node.kind() != "project_file" {
            return Some(descendant_node);
        }
    }
    None
}

/// Goes up recursively until a kind that matches is found, and then retrieve its child by
/// field_name.
pub(crate) fn node_parent_identifier(
    source: &[u8],
    node: &Node,
    kind: &str,
    field_name: &str,
) -> Option<String> {
    let mynode = node;
    let mut mynode_parent = mynode.parent();

    while let Some(parent) = mynode_parent {
        let parent_kind = parent.kind();

        if parent_kind == kind {
            println!("{:?}", parent.utf8_text(source));
            println!(
                "{:?}",
                parent
                    .child_by_field_name("alias")
                    .map(|c| c.next_sibling().map(|c| c.utf8_text(source)))
            );

            return Some(
                parent
                    .child_by_field_name(field_name)?
                    .utf8_text(source)
                    .unwrap()
                    .to_string(),
            );
        } else if parent_kind == "project_file" {
            return None;
        }

        mynode_parent = parent.parent();
    }
    None
}
