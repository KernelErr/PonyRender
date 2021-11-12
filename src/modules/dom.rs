use std::collections::{HashMap, HashSet};

pub type AttrMap = HashMap<String, String>;

#[derive(Debug, PartialEq)]
pub enum NodeType {
    Text(String),
    Element(ElementData),
    Comment(String),
}

#[derive(Debug, PartialEq)]
pub struct Node {
    pub children: Vec<Node>,
    pub node_type: NodeType,
}

#[derive(Debug, PartialEq)]
pub struct ElementData {
    pub tag_name: String,
    pub attributes: AttrMap,
}

impl ElementData {
    pub fn id(&self) -> Option<&String> {
        self.attributes.get("id")
    }

    pub fn classes(&self) -> HashSet<&str> {
        match self.attributes.get("class") {
            Some(classlist) => classlist.split(' ').collect(),
            None => HashSet::new(),
        }
    }
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
        assert_eq!(
            node.node_type,
            NodeType::Element(ElementData {
                tag_name: "div".to_string(),
                attributes: AttrMap::new(),
            })
        );
    }

    #[test]
    fn test_node_comment() {
        let node = Node::comment("foo".to_string());
        assert_eq!(node.node_type, NodeType::Comment("foo".to_string()));
    }
}
