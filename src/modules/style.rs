use super::css::{Rule, Selector, SimpleSelector, Specificity, Stylesheet, Value};
use super::dom::{ElementData, Node, NodeType};
use super::layout::Display;
use std::collections::HashMap;

type PropertyMap = HashMap<String, Value>;
type MatchedRule<'a> = (Specificity, &'a Rule);

#[derive(Debug, PartialEq)]
pub struct StyledNode<'a> {
    node: &'a Node,
    specified_values: PropertyMap,
    pub children: Vec<StyledNode<'a>>,
}

impl<'a> StyledNode<'a> {
    pub fn value(&self, name: &str) -> Option<Value> {
        self.specified_values.get(name).cloned()
    }

    pub fn display(&self) -> Display {
        match self.value("display") {
            Some(Value::Keyword(s)) => match &*s {
                "block" => Display::Block,
                "none" => Display::None,
                _ => Display::Inline,
            },
            _ => Display::Inline,
        }
    }

    pub fn lookup(&self, name: &str, fallback_name: &str, default: &Value) -> Value {
        self.value(name)
            .unwrap_or_else(|| self.value(fallback_name).unwrap_or_else(|| default.clone()))
    }
}

fn matches(elem: &ElementData, selector: &Selector) -> bool {
    match *selector {
        Selector::Simple(ref simple_selector) => matches_simple_selector(elem, simple_selector),
    }
}

fn matches_simple_selector(elem: &ElementData, selector: &SimpleSelector) -> bool {
    // Check type selector
    if selector.tag_name.iter().any(|name| elem.tag_name != *name) {
        return false;
    }

    // Check ID selector
    if selector.id.iter().any(|id| elem.id() != Some(id)) {
        return false;
    }

    // Check class selectors
    let elem_classes = elem.classes();
    if selector
        .class
        .iter()
        .any(|class| !elem_classes.contains(&**class))
    {
        return false;
    }

    // We didn't find any non-matching selector components.
    true
}

fn match_rule<'a>(elem: &ElementData, rule: &'a Rule) -> Option<MatchedRule<'a>> {
    rule.selectors
        .iter()
        .find(|selector| matches(elem, *selector))
        .map(|selector| (selector.specificity(), rule))
}

fn matching_rules<'a>(elem: &ElementData, stylesheet: &'a Stylesheet) -> Vec<MatchedRule<'a>> {
    stylesheet
        .rules
        .iter()
        .filter_map(|rule| match_rule(elem, rule))
        .collect()
}

fn specified_values(elem: &ElementData, stylesheet: &Stylesheet) -> PropertyMap {
    let mut values = HashMap::new();
    let mut rules = matching_rules(elem, stylesheet);

    rules.sort_by(|&(a, _), &(b, _)| a.cmp(&b));
    for (_, rule) in rules {
        for declaration in &rule.declarations {
            values.insert(declaration.name.clone(), declaration.value.clone());
        }
    }
    values
}

pub fn style_tree<'a>(root: &'a Node, stylesheet: &'a Stylesheet) -> StyledNode<'a> {
    StyledNode {
        node: root,
        specified_values: match root.node_type {
            NodeType::Element(ref elem) => specified_values(elem, stylesheet),
            NodeType::Text(_) => HashMap::new(),
            NodeType::Comment(_) => HashMap::new(),
        },
        children: root
            .children
            .iter()
            .map(|child| style_tree(child, stylesheet))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::super::css::Rule;
    use super::super::css::{Selector, SimpleSelector};
    use super::super::dom::{AttrMap, ElementData};
    use super::*;

    #[test]
    fn test_match_rule() {
        let elem = ElementData {
            tag_name: "div".to_string(),
            attributes: AttrMap::new(),
        };

        let rule = Rule {
            selectors: vec![Selector::Simple(SimpleSelector {
                tag_name: Some("div".to_string()),
                id: None,
                class: vec![],
            })],
            declarations: vec![],
        };

        assert!(match_rule(&elem, &rule).is_some());
    }

    #[test]
    fn test_match_rule_no_match() {
        let elem = ElementData {
            tag_name: "div".to_string(),
            attributes: AttrMap::new(),
        };

        let rule = Rule {
            selectors: vec![Selector::Simple(SimpleSelector {
                tag_name: Some("span".to_string()),
                id: None,
                class: vec![],
            })],
            declarations: vec![],
        };

        assert!(match_rule(&elem, &rule).is_none());
    }

    #[test]
    fn test_match_rule_id_match() {
        let mut attributes = AttrMap::new();
        attributes.insert("id".to_string(), "foo".to_string());
        let elem = ElementData {
            tag_name: "div".to_string(),
            attributes: attributes,
        };

        let rule = Rule {
            selectors: vec![Selector::Simple(SimpleSelector {
                tag_name: Some("div".to_string()),
                id: Some("foo".to_string()),
                class: vec![],
            })],
            declarations: vec![],
        };

        assert!(match_rule(&elem, &rule).is_some());
    }
}
