use crate::model::Node;

pub fn with_class(node: Node, class: &str) -> Node {
    const CSS_CLASS_ATTR: &str = "class";

    let class = match node.attrs().get(CSS_CLASS_ATTR) {
        Some(existing) => format!("{} {}", existing, class),
        None => class.to_string(),
    };
    node.with_attr(CSS_CLASS_ATTR, &class)
}

pub fn with_id(node: Node, id: &str) -> Node {
    const CSS_ID_ATTR: &str = "id";

    node.with_attr(CSS_ID_ATTR, id)
}

pub fn floating_menu(node: Node) -> Node {
    const CLASS: &str = "floating-menu";

    with_class(node, CLASS)
}

pub fn title(node: Node) -> Node {
    const CLASS: &str = "title";

    with_class(node, CLASS)
}
