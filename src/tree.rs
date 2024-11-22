const COMMENT_POSITION: usize = 40;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    bytes: Vec<u8>,
    more_bytes: Vec<u8>,
    comment: Option<String>,
    children: Vec<Node>,
    embedded: bool,
}

fn _write_indent(indent: usize, output: &mut String) -> usize {
    let mut size = 0usize;
    for _ in 0..indent {
        output.push_str("   ");
        size += 3;
    }
    size
}

impl Node {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            more_bytes: vec![],
            comment: None,
            children: vec![],
            embedded: false,
        }
    }

    pub fn with_more_bytes(mut self, more_bytes: Vec<u8>) -> Self {
        self.more_bytes = more_bytes;
        self
    }

    pub fn with_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }

    pub fn with_child(mut self, child: Node) -> Self {
        self.children.push(child);
        self
    }

    pub fn with_children(mut self, children: Vec<Node>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn mark_embedded(mut self) -> Self {
        self.embedded = true;
        self
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn calculate_comment_position(&self, indent: usize) -> usize {
        let mut comment_position = indent * 3
            + self.bytes.len() * 2
            + (if self.more_bytes.is_empty() {
                0
            } else {
                1 + self.more_bytes.len() * 2
            })
            + 2;
        for child in &self.children {
            let child_comment_position = child.calculate_comment_position(indent + 1);
            if child_comment_position > comment_position {
                comment_position = child_comment_position;
            }
        }
        comment_position
    }

    fn _write(&self, indent: usize, comment_position: usize, output: &mut String) {
        if self.embedded {
            _write_indent(indent, output);
            output.push_str("-- embedded --\n");
        }

        let mut position = _write_indent(indent, output);
        self.bytes.iter().for_each(|b| {
            output.push_str(&format!("{:02x}", b));
            position += 2;
        });

        if !self.more_bytes.is_empty() {
            output.push(' ');
            position += 1;
            self.more_bytes.iter().for_each(|b| {
                output.push_str(&format!("{:02x}", b));
                position += 2;
            });
        }

        if let Some(comment) = &self.comment {
            if position < comment_position - 2 {
                for _ in 0..(comment_position - position) {
                    output.push(' ');
                }
            } else {
                output.push_str("  ");
            }
            output.push_str("-- ");
            output.push_str(comment);
        }
        output.push('\n');
        for child in &self.children {
            child._write(indent + 1, comment_position, output);
        }

        if self.embedded {
            _write_indent(indent, output);
            output.push_str("--------------\n");
        }
    }

    pub fn write(&self, output: &mut String) {
        let mut comment_position = self.calculate_comment_position(0);
        if comment_position > COMMENT_POSITION {
            comment_position = COMMENT_POSITION;
        }

        self._write(0, comment_position, output);
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::Node;

    #[test]
    fn write_1() {
        let tree = Node::new(vec![0x01, 0xff])
            .with_comment(String::from("comment 1"))
            .with_children(vec![
                Node::new(vec![0x02])
                    .with_more_bytes(vec![0xff, 0xff])
                    .with_comment(String::from("comment 1-1"))
                    .with_children(vec![
                        Node::new(vec![0x03]).with_comment(String::from("comment 1-1-1"))
                    ]),
                Node::new(vec![0x04])
                    .with_comment(String::from("comment 1-2"))
                    .mark_embedded()
                    .with_children(vec![
                        Node::new(vec![0x05]).with_comment(String::from("comment 1-2-1"))
                    ]),
                Node::new(vec![0x06])
                    .with_comment(String::from("comment 1-3"))
                    .with_children(vec![
                        Node::new(vec![0x07]).with_comment(String::from("comment 1-3-1"))
                    ]),
            ]);

        let expected = vec![
            "01ff        -- comment 1",
            "   02 ffff  -- comment 1-1",
            "      03    -- comment 1-1-1",
            "   -- embedded --",
            "   04       -- comment 1-2",
            "      05    -- comment 1-2-1",
            "   --------------",
            "   06       -- comment 1-3",
            "      07    -- comment 1-3-1",
            "",
        ]
        .join("\n");

        let mut actual = String::new();
        tree.write(&mut actual);
        assert_eq!(actual, expected);
    }

    #[test]
    fn write_2() {
        let tree = Node::new(vec![0x01, 0xff])
            .with_comment(String::from("comment 1"))
            .with_children(vec![
                Node::new(vec![0xff].repeat(50)).with_comment(String::from("comment 1-1")),
                Node::new(vec![0x02]).with_comment(String::from("comment 1-2")),
            ]);

        let expected = vec![
            "01ff                                    -- comment 1",
            &["   ", &"ff".repeat(50), "  -- comment 1-1"].concat(),
            "   02                                   -- comment 1-2",
            "",
        ]
        .join("\n");

        let mut actual = String::new();
        tree.write(&mut actual);
        assert_eq!(actual, expected);
    }
}
