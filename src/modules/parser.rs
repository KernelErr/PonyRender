use super::dom;

pub fn parse(source: String) -> dom::Node {
    let mut nodes = Parser {
        pos: 0,
        input: source,
    }.parse_nodes();

    if nodes.len() == 1 {
        nodes.swap_remove(0)
    } else {
        dom::Node::elem("html".to_string(), dom::AttrMap::new(), nodes)
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

    fn parse_tag_name(&mut self) -> String {
        self.consume_while(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' => true,
            _ => false,
        })
    }

    fn parse_node(&mut self) -> dom::Node {
        match self.next_char() {
            '<' => {
                if self.starts_with("<!--") {
                    self.parse_comment()
                } else {
                    self.parse_element()
                }
            },
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
        let mut text = self.consume_while(|c| c!= '-');
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
        assert_eq!(
            node,
            dom::Node::text("Hello, world!".to_string())
        );
    }

    #[test]
    fn test_parse_comment() {
        let mut parser = Parser {
            pos: 0,
            input: "<!-- This is a comment -->".to_string(),
        };
        let node = parser.parse_node();
        assert_eq!(
            node,
            dom::Node::comment(" This is a comment ".to_string())
        );
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
    fn test_parse_complex_element() {
        let mut parser = Parser {
            pos: 0,
            input: "<p>Hello, <!-- This is a comment --><b>world</b></p>".to_string(),
        };
        let node = parser.parse_node();
        assert_eq!(
            node,
            dom::Node::elem(
                "p".to_string(),
                dom::AttrMap::new(),
                vec![
                    dom::Node::text("Hello, ".to_string()),
                    dom::Node::comment(" This is a comment ".to_string()),
                    dom::Node::elem(
                        "b".to_string(),
                        dom::AttrMap::new(),
                        vec![dom::Node::text("world".to_string())],
                    ),
                ]
            )
        );
    }
}