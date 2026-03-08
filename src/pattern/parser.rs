use crate::config::parse_key;
use crate::pattern::ast::{ActionSpec, Edge, Node, NodeKind, RepeatKind};
use crate::signal::SignalKind;

#[derive(Debug)]
pub struct ParseError(pub String);

pub fn parse_pattern(input: &str) -> Result<Node, ParseError> {
    let mut p = Parser::new(input);
    let expr = p.parse_expr()?;
    p.skip_ws();
    if !p.eof() {
        return Err(ParseError(format!(
            "unexpected trailing input at position {}",
            p.pos
        )));
    }
    Ok(expr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use evdev::KeyCode as Key;

    #[test]
    fn parse_simple_press_release() {
        let ast = parse_pattern("Super_L g g! => emit(nav.stop)").unwrap();
        match ast.kind {
            NodeKind::Seq(seq) => {
                assert_eq!(seq.len(), 3);
                match &seq[0].kind {
                    NodeKind::Event(Edge::Press(k)) => assert_eq!(*k, Key::KEY_LEFTMETA),
                    _ => panic!("expected press"),
                }
                match &seq[2].actions[0] {
                    ActionSpec::Emit(name, SignalKind::Fire) => assert_eq!(name, "nav.stop"),
                    _ => panic!("expected emit"),
                }
            }
            _ => panic!("expected seq"),
        }
    }

    #[test]
    fn parse_end_on() {
        let ast = parse_pattern("a end_on(b!|c!) => emit(stop)").unwrap();
        match ast.kind {
            NodeKind::Seq(seq) => {
                assert_eq!(seq.len(), 2);
                match &seq[1].kind {
                    NodeKind::Alt(alts) => {
                        assert_eq!(alts.len(), 2);
                        for alt in alts {
                            assert!(matches!(alt.actions[0], ActionSpec::End));
                            match &alt.kind {
                                NodeKind::Event(Edge::Release(_)) => {}
                                _ => panic!("expected release event"),
                            }
                        }
                    }
                    _ => panic!("expected alt"),
                }
            }
            _ => panic!("expected seq"),
        }
    }
}

struct Parser<'a> {
    src: &'a str,
    chars: Vec<char>,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        Parser {
            src,
            chars: src.chars().collect(),
            pos: 0,
        }
    }

    fn eof(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).cloned()
    }

    fn bump(&mut self) -> Option<char> {
        if self.eof() {
            None
        } else {
            let ch = self.chars[self.pos];
            self.pos += 1;
            Some(ch)
        }
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(ch) if ch.is_whitespace()) {
            self.pos += 1;
        }
    }

    fn parse_expr(&mut self) -> Result<Node, ParseError> {
        let mut alts = vec![self.parse_seq()?];
        loop {
            self.skip_ws();
            if self.match_char('|') {
                alts.push(self.parse_seq()?);
            } else {
                break;
            }
        }
        if alts.len() == 1 {
            Ok(alts.remove(0))
        } else {
            Ok(Node::new(NodeKind::Alt(alts)))
        }
    }

    fn parse_seq(&mut self) -> Result<Node, ParseError> {
        let mut nodes = vec![];
        loop {
            self.skip_ws();
            if self.eof() || self.peek() == Some(')') || self.peek() == Some('|') {
                break;
            }
            let node = self.parse_rep()?;
            nodes.push(node);
        }
        if nodes.is_empty() {
            Ok(Node::new(NodeKind::Epsilon))
        } else if nodes.len() == 1 {
            Ok(nodes.remove(0))
        } else {
            Ok(Node::new(NodeKind::Seq(nodes)))
        }
    }

    fn parse_rep(&mut self) -> Result<Node, ParseError> {
        let mut node = self.parse_primary()?;
        self.skip_ws();
        let mut applied_repeat = false;
        if let Some(q) = self.peek() {
            let kind = match q {
                '*' => Some(RepeatKind::ZeroOrMore),
                '+' => Some(RepeatKind::OneOrMore),
                '?' => Some(RepeatKind::Optional),
                _ => None,
            };
            if let Some(k) = kind {
                self.bump();
                node = Node::new(NodeKind::Repeat {
                    node: Box::new(node),
                    kind: k,
                });
                applied_repeat = true;
            }
        }
        self.skip_ws();
        if self.match_arrow() {
            let actions = self.parse_actions()?;
            if applied_repeat {
                if let NodeKind::Repeat { node: inner, .. } = &mut node.kind {
                    inner.actions.extend(actions);
                }
            } else {
                node.actions = actions;
            }
        }
        Ok(node)
    }

    fn parse_primary(&mut self) -> Result<Node, ParseError> {
        self.skip_ws();
        if self.peek_ident("end_on") {
            return self.parse_end_on();
        }
        if self.match_char('(') {
            let expr = self.parse_expr()?;
            self.skip_ws();
            if !self.match_char(')') {
                return Err(ParseError("expected ')'".into()));
            }
            return Ok(expr);
        }
        if self.match_epsilon() {
            return Ok(Node::new(NodeKind::Epsilon));
        }
        self.parse_key_token()
    }

    fn parse_key_token(&mut self) -> Result<Node, ParseError> {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' || ch == '.' {
                self.bump();
            } else {
                break;
            }
        }
        if self.pos == start {
            return Err(ParseError(format!("expected key at position {}", self.pos)));
        }
        let raw: String = self.chars[start..self.pos].iter().collect();
        let is_release = if self.peek() == Some('!') {
            self.bump();
            true
        } else {
            false
        };
        // parse key name
        let key = parse_key(&raw).map_err(|e: Box<dyn std::error::Error>| ParseError(e.to_string()))?;
        let edge = if is_release {
            Edge::Release(key)
        } else {
            Edge::Press(key)
        };
        Ok(Node::new(NodeKind::Event(edge)))
    }

    fn parse_actions(&mut self) -> Result<Vec<ActionSpec>, ParseError> {
        self.skip_ws();
        if self.match_char('[') {
            let mut acts = vec![];
            loop {
                self.skip_ws();
                acts.push(self.parse_action()?);
                self.skip_ws();
                if self.match_char(']') {
                    break;
                }
                if !self.match_char(',') {
                    return Err(ParseError("expected ',' or ']' in actions".into()));
                }
            }
            Ok(acts)
        } else {
            Ok(vec![self.parse_action()?])
        }
    }

    fn parse_action(&mut self) -> Result<ActionSpec, ParseError> {
        self.skip_ws();
        let ident = self.parse_ident()?;
        self.skip_ws();
        if ident == "noop" {
            return Ok(ActionSpec::Noop);
        }
        if ident == "end" {
            return Ok(ActionSpec::End);
        }
        if !self.match_char('(') {
            return Err(ParseError("expected '(' after action".into()));
        }
        let param = self.parse_param()?;
        if !self.match_char(')') {
            return Err(ParseError("expected ')' after action param".into()));
        }
        let kind = match ident.as_str() {
            "emit" => SignalKind::Fire,
            "emit_start" => SignalKind::StartRepeat,
            "emit_stop" => SignalKind::StopRepeat,
            _ => return Err(ParseError(format!("unknown action '{}'", ident))),
        };
        Ok(ActionSpec::Emit(param, kind))
    }

    fn parse_ident(&mut self) -> Result<String, ParseError> {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                self.bump();
            } else {
                break;
            }
        }
        if self.pos == start {
            return Err(ParseError(format!("expected identifier at {}", self.pos)));
        }
        Ok(self.chars[start..self.pos].iter().collect())
    }

    fn parse_param(&mut self) -> Result<String, ParseError> {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch == ')' {
                break;
            }
            self.bump();
        }
        Ok(self.chars[start..self.pos].iter().collect::<String>().trim().to_string())
    }

    fn match_char(&mut self, ch: char) -> bool {
        if self.peek() == Some(ch) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn match_arrow(&mut self) -> bool {
        self.skip_ws();
        let save = self.pos;
        if self.match_char('=') && self.match_char('>') {
            true
        } else {
            self.pos = save;
            false
        }
    }

    fn match_epsilon(&mut self) -> bool {
        let save = self.pos;
        if self.match_char('ε') {
            return true;
        }
        if self.match_char('e') {
            // allow "eps"?
            self.pos = save;
            return false;
        }
        self.pos = save;
        false
    }

    fn peek_ident(&self, word: &str) -> bool {
        self.chars
            .iter()
            .skip(self.pos)
            .take(word.len())
            .collect::<String>()
            == word
    }

    fn parse_end_on(&mut self) -> Result<Node, ParseError> {
        self.parse_ident()?; // end_on
        self.skip_ws();
        if !self.match_char('(') {
            return Err(ParseError("expected '(' after end_on".into()));
        }
        let mut edges = vec![];
        loop {
            self.skip_ws();
            edges.push(self.parse_edge_only()?);
            self.skip_ws();
            if self.match_char(')') {
                break;
            }
            if !self.match_char('|') {
                return Err(ParseError("expected '|' or ')' in end_on".into()));
            }
        }
        self.skip_ws();
        let mut actions = vec![ActionSpec::End];
        if self.match_arrow() {
            let mut extra = self.parse_actions()?;
            actions.append(&mut extra);
        }
        let nodes: Vec<Node> = edges
            .into_iter()
            .map(|e| {
                let mut n = Node::new(NodeKind::Event(e));
                n.actions = actions.clone();
                n
            })
            .collect();
        Ok(Node::new(NodeKind::Alt(nodes)))
    }

    fn parse_edge_only(&mut self) -> Result<Edge, ParseError> {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' || ch == '.' {
                self.bump();
            } else {
                break;
            }
        }
        if self.pos == start {
            return Err(ParseError(format!("expected key at position {}", self.pos)));
        }
        let raw: String = self.chars[start..self.pos].iter().collect();
        let is_release = if self.peek() == Some('!') {
            self.bump();
            true
        } else {
            false
        };
        let key = parse_key(&raw).map_err(|e: Box<dyn std::error::Error>| ParseError(e.to_string()))?;
        Ok(if is_release { Edge::Release(key) } else { Edge::Press(key) })
    }
}
