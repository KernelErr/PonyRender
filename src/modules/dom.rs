use std::collections::HashMap;

pub type AttrMap = HashMap<String, String>;

#[derive(Debug, PartialEq)]
enum NodeType {
    Text(String),
    Element(ElementData),
    Comment(String),
}

#[derive(Debug, PartialEq)]
pub struct Node {
    children: Vec<Node>,
    node_type: NodeType,
}

#[derive(Debug, PartialEq)]
struct ElementData {
    tag_name: String,
    attributes: AttrMap,
}

impl Node {
    pub fn text(data: String) -> Node {
        Node {
            children: vec![],
            node_type: NodeType::Text(data),
        }
    }

    pub fn elem(name: String, attrs: AttrMap, children: Vec<Node>) -> Node {
        Node {
            children,
            node_type: NodeType::Element(ElementData {
                tag_name: name,
                attributes: attrs,
            }),
        }
    }

    pub fn comment(data: String) -> Node {
        Node {
            children: vec![],
            node_type: NodeType::Comment(data),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_text() {
        let node = Node::text("foo".to_string());
        assert_eq!(node.node_type, NodeType::Text("foo".to_string()));
    }

    #[test]
    fn test_node_elem() {
        let node = Node::elem("div".to_string(), AttrMap::new(), vec![]);
        assert_eq!(node.node_type, NodeType::Element(ElementData {
            tag_name: "div".to_string(),
            attributes: AttrMap::new(),
        }));
    }

    #[test]
    fn test_node_comment() {
        let node = Node::comment("foo".to_string());
        assert_eq!(node.node_type, NodeType::Comment("foo".to_string()));
    }
}