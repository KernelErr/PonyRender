use super::css;
use super::dom;
use std::cmp::Reverse;

pub fn parse_html(source: String) -> dom::Node {
    let mut nodes = Parser {
        pos: 0,
        input: source,
    }
    .parse_nodes();

    if nodes.len() == 1 {
        nodes.swap_remove(0)
    } else {
        dom::Node::elem("html".to_string(), dom::AttrMap::new(), nodes)
    }
}

pub fn parse_css(source: String) -> css::Stylesheet {
    let mut parser = Parser {
        pos: 0,
        input: source,
    };
    css::Stylesheet {
        rules: parser.parse_rules(),
    }
}

struct Parser {
    pos: usize,
    input: String,
}

impl Parser {
    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn consume_char(&mut self) -> char {
        let mut iter = self.input[self.pos..].char_indices();
        let (_, cur_char) = iter.next().unwrap();
        let (next_pos, _) = iter.next().unwrap_or((1, ' '));
        self.pos += next_pos;
        cur_char
    }

    fn consume_while<F>(&mut self, test: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char());
        }
        result
    }

    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

    // HTML Part Below

    fn parse_tag_name(&mut self) -> String {
        self.consume_while(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-'))
    }

    fn parse_node(&mut self) -> dom::Node {
        match self.next_char() {
            '<' => {
                if self.starts_with("<!--") {
                    self.parse_comment()
                } else {
                    self.parse_element()
                }
            }
            _ => self.parse_text(),
        }
    }

    fn parse_text(&mut self) -> dom::Node {
        let text = self.consume_while(|c| c != '<');
        dom::Node::text(text)
    }

    fn parse_comment(&mut self) -> dom::Node {
        // FIX ME: When format is wrong
        self.consume_char();
        self.consume_char();
        self.consume_char();
        self.consume_char();
        let mut text = self.consume_while(|c| c != '-');
        while !self.eof() && !self.starts_with("-->") {
            text.push(self.consume_char());
        }
        self.consume_char();
        self.consume_char();
        self.consume_char();
        dom::Node::comment(text)
    }

    fn parse_element(&mut self) -> dom::Node {
        // FIX ME: When format is wrong
        self.consume_char();
        let tag_name = self.parse_tag_name();
        let attrs = self.parse_attributes();
        // FIX ME: When format is wrong
        self.consume_char();

        let children = self.parse_nodes();

        // FIX ME: When format is wrong
        self.consume_char();
        self.consume_char();
        self.parse_tag_name();
        self.consume_char();
        dom::Node::elem(tag_name, attrs, children)
    }

    fn parse_attr(&mut self) -> (String, String) {
        let name = self.parse_tag_name();
        // FIX ME: When format is wrong
        self.consume_char();
        let value = self.parse_attr_value();
        (name, value)
    }

    fn parse_attr_value(&mut self) -> String {
        let open_quote = self.consume_char();
        // FIX ME: When format is wrong
        let value = self.consume_while(|c| c != open_quote);
        self.consume_char();
        value
    }

    fn parse_attributes(&mut self) -> dom::AttrMap {
        let mut attributes = dom::AttrMap::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '>' {
                break;
            }
            let (name, value) = self.parse_attr();
            attributes.insert(name, value);
        }
        attributes
    }

    fn parse_nodes(&mut self) -> Vec<dom::Node> {
        let mut nodes = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() || self.starts_with("</") {
                break;
            }
            nodes.push(self.parse_node());
        }
        nodes
    }

    // HTML Ends

    // CSS Part Below

    fn parse_rules(&mut self) -> Vec<css::Rule> {
        let mut rules = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() {
                break;
            }
            rules.push(self.parse_rule());
        }
        rules
    }

    fn parse_simple_selector(&mut self) -> css::SimpleSelector {
        let mut selector = css::SimpleSelector {
            tag_name: None,
            id: None,
            class: Vec::new(),
        };
        while !self.eof() {
            match self.next_char() {
                '#' => {
                    self.consume_char();
                    selector.id = Some(self.parse_identifier());
                }
                '.' => {
                    self.consume_char();
                    selector.class.push(self.parse_identifier());
                }
                '*' => {
                    self.consume_char();
                }
                c if valid_identifier_char(c) => {
                    selector.tag_name = Some(self.parse_identifier());
                }
                _ => break,
            }
        }
        selector
    }

    fn parse_rule(&mut self) -> css::Rule {
        css::Rule {
            selectors: self.parse_selectors(),
            declarations: self.parse_declarations(),
        }
    }

    fn parse_selectors(&mut self) -> Vec<css::Selector> {
        let mut selectors: Vec<css::Selector> = Vec::new();
        loop {
            selectors.push(css::Selector::Simple(self.parse_simple_selector()));
            self.consume_whitespace();
            match self.next_char() {
                ',' => {
                    self.consume_char();
                    self.consume_whitespace();
                }
                '{' => break,
                _c => {
                    // FIX ME: When format is wrong
                }
            }
        }
        selectors.sort_by_key(|b| Reverse(b.specificity()));
        selectors
    }

    fn parse_declarations(&mut self) -> Vec<css::Declaration> {
        assert_eq!(self.consume_char(), '{');
        let mut declarations = Vec::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '}' {
                self.consume_char();
                break;
            }
            declarations.push(self.parse_declaration());
        }
        declarations
    }

    fn parse_declaration(&mut self) -> css::Declaration {
        let property_name = self.parse_identifier();
        self.consume_whitespace();
        assert_eq!(self.consume_char(), ':');
        self.consume_whitespace();
        let value = self.parse_value();
        self.consume_whitespace();
        assert_eq!(self.consume_char(), ';');

        css::Declaration {
            name: property_name,
            value,
        }
    }

    fn parse_identifier(&mut self) -> String {
        self.consume_while(valid_identifier_char)
    }

    fn parse_value(&mut self) -> css::Value {
        match self.next_char() {
            '0'..='9' => self.parse_length(),
            '#' => self.parse_color(),
            _ => css::Value::Keyword(self.parse_identifier()),
        }
    }

    fn parse_length(&mut self) -> css::Value {
        css::Value::Length(self.parse_float(), self.parse_unit())
    }

    fn parse_float(&mut self) -> f32 {
        let s = self.consume_while(|c| matches!(c, '0'..='9' | '.'));
        s.parse().unwrap()
    }

    fn parse_unit(&mut self) -> css::Unit {
        match &*self.parse_identifier().to_ascii_lowercase() {
            "px" => css::Unit::Px,
            _ => panic!("unrecognized unit"),
        }
    }

    fn parse_color(&mut self) -> css::Value {
        assert_eq!(self.consume_char(), '#');
        css::Value::Color(css::Color {
            red: self.parse_hex_pair(),
            green: self.parse_hex_pair(),
            blue: self.parse_hex_pair(),
            alpha: 255,
        })
    }

    fn parse_hex_pair(&mut self) -> u8 {
        let s = &self.input[self.pos..self.pos + 2];
        self.pos += 2;
        u8::from_str_radix(s, 16).unwrap()
    }

    // CSS Ends
}

fn valid_identifier_char(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text() {
        let mut parser = Parser {
            pos: 0,
            input: "Hello, world!".to_string(),
        };
        let node = parser.parse_node();
        assert_eq!(node, dom::Node::text("Hello, world!".to_string()));
    }

    #[test]
    fn test_parse_comment() {
        let mut parser = Parser {
            pos: 0,
            input: "<!-- This is a comment -->".to_string(),
        };
        let node = parser.parse_node();
        assert_eq!(node, dom::Node::comment(" This is a comment ".to_string()));
    }

    #[test]
    fn test_parse_element() {
        let mut parser = Parser {
            pos: 0,
            input: "<p>Hello, <b>world</b></p>".to_string(),
        };
        let node = parser.parse_node();
        assert_eq!(
            node,
            dom::Node::elem(
                "p".to_string(),
                dom::AttrMap::new(),
                vec![
                    dom::Node::text("Hello, ".to_string()),
                    dom::Node::elem(
                        "b".to_string(),
                        dom::AttrMap::new(),
                        vec![dom::Node::text("world".to_string())],
                    ),
                ]
            )
        );
    }

    #[test]
    fn test_parse_element_attributes() {
        let mut parser = Parser {
            pos: 0,
            input: "<p id=\"test\">Hello, <b id=\"test\">world</b></p>".to_string(),
        };
        let node = parser.parse_node();
        let mut attributes = dom::AttrMap::new();
        attributes.insert("id".to_string(), "test".to_string());
        assert_eq!(
            node,
            dom::Node::elem(
                "p".to_string(),
                attributes.clone(),
                vec![
                    dom::Node::text("Hello, ".to_string()),
                    dom::Node::elem(
                        "b".to_string(),
                        attributes,
                        vec![dom::Node::text("world".to_string())],
                    ),
                ]
            )
        );
    }

    #[test]
    fn test_parse_complex_element() {
        let mut parser = Parser {
            pos: 0,
            input: "<p id=\"test\">Hello, <!-- This is a comment --><b id=\"test\">world</b></p>"
                .to_string(),
        };
        let node = parser.parse_node();
        let mut attributes = dom::AttrMap::new();
        attributes.insert("id".to_string(), "test".to_string());
        assert_eq!(
            node,
            dom::Node::elem(
                "p".to_string(),
                attributes.clone(),
                vec![
                    dom::Node::text("Hello, ".to_string()),
                    dom::Node::comment(" This is a comment ".to_string()),
                    dom::Node::elem(
                        "b".to_string(),
                        attributes,
                        vec![dom::Node::text("world".to_string())],
                    ),
                ]
            )
        );
    }

    #[test]
    fn test_parse_css() {
        use super::css::SimpleSelector;
        let mut parser = Parser {
            pos: 0,
            input: "p { color: #ff0000; }".to_string(),
        };
        let rule = parser.parse_rules();
        assert_eq!(
            rule,
            vec![css::Rule {
                selectors: vec![css::Selector::Simple(SimpleSelector {
                    tag_name: Some("p".to_string()),
                    id: None,
                    class: vec![],
                })],
                declarations: vec![css::Declaration {
                    name: "color".to_string(),
                    value: css::Value::Color(css::Color {
                        red: 255,
                        green: 0,
                        blue: 0,
                        alpha: 255,
                    })
                }]
            }]
        );
    }
}
