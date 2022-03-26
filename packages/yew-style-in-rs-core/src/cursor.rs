//! Parser for CSS

use std::iter::Peekable;
use std::str::Chars;

use crate::ast::*;

pub enum ParseError<T> {
    Fatal,
    Ignorable(T, String),
}

#[derive(Clone, Debug)]
pub struct Cursor<'a> {
    content: Peekable<Chars<'a>>,
}
impl<'a> Cursor<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            content: content.chars().peekable(),
        }
    }

    pub fn is_empty(&mut self) -> bool {
        self.content.peek().is_none()
    }

    pub fn peek(&mut self, ch: char) -> bool {
        self.content.peek() == Some(&ch)
    }

    pub fn skip_white_space(&mut self) {
        while self
            .content
            .peek()
            .map(|&c| c.is_whitespace() || c == '\r' || c == '\n')
            .unwrap_or_default()
        {
            self.content.next();
        }
    }

    pub fn take(&mut self, ch: char) -> Option<char> {
        if let Some(c) = self.content.next() {
            if c == ch {
                Some(ch)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn take_until(&mut self, delimiters: &[char]) -> Option<(String, char)> {
        let mut ret = String::new();
        loop {
            if let Some(&c) = self.content.peek() {
                if delimiters.contains(&c) {
                    return Some((ret, c));
                } else {
                    ret.push(c);
                    self.content.next();
                }
            } else {
                return None;
            }
        }
    }

    pub fn take_until_whitespace(&mut self) -> Option<String> {
        let mut ret = String::new();
        loop {
            if let Some(&c) = self.content.peek() {
                if c.is_whitespace() {
                    return Some(ret);
                } else {
                    ret.push(c);
                    self.content.next();
                }
            } else {
                return None;
            }
        }
    }

    pub fn take_paren(&mut self) -> Option<String> {
        self.take('(')?;
        let mut content = vec![];
        while let Some((value, delimiter)) = self.take_until(&['(', ')']) {
            content.push(value);
            if delimiter == '(' {
                let value = self.take_paren()?;
                let value = String::new() + "(" + &value + ")";
                content.push(value);
            } else if delimiter == ')' {
                self.take(')')?;
                break;
            }
        }
        Some(String::new() + "(" + &content.join("") + ")")
    }

    pub fn parse_selectors(&mut self) -> Option<Selectors> {
        let mut selectors: Vec<String> = vec![String::new()];
        while let Some((value, delimiter)) = self.take_until(&['(', ',', '{', ';']) {
            if delimiter == '(' {
                let s = selectors.iter_mut().last().unwrap();
                *s += &(value + &self.take_paren()?);
            } else if delimiter == ',' {
                let s = selectors.iter_mut().last().unwrap();
                *s += value.trim_end();
                self.take(',')?;
                self.skip_white_space();
                selectors.push(String::new());
            } else if delimiter == '{' {
                let s = selectors.iter_mut().last().unwrap();
                *s += value.trim_end();
                break;
            } else if delimiter == ';' {
                return None;
            }
        }
        if selectors.iter().last().unwrap().is_empty() {
            selectors.pop();
        }
        Some(Selectors(selectors))
    }

    pub fn parse_at_rule(&mut self) -> Result<AtRule, ParseError<AtRule>> {
        self.skip_white_space();
        self.take('@').ok_or(ParseError::Fatal)?;
        let rule_name = self.take_until_whitespace().ok_or(ParseError::Fatal)?;
        self.skip_white_space();
        if let Some((value, delimiter)) = self.take_until(&['{', ';']) {
            let rule_value = value;
            let rule_value = rule_value.trim_end().to_string();

            if delimiter == '{' {
                self.take('{').ok_or(ParseError::Fatal)?;

                match self.parse_declaration_list() {
                    Ok(declarations) => {
                        self.skip_white_space();
                        self.take('}').ok_or(ParseError::Fatal)?;
                        Ok(AtRule {
                            rule_name,
                            rule_value,
                            block: Some(declarations),
                        })
                    }
                    Err(ParseError::Fatal) => return Err(ParseError::Fatal),
                    Err(ParseError::Ignorable(declarations, msg)) => {
                        self.skip_white_space();
                        self.take('}').ok_or(ParseError::Fatal)?;
                        Err(ParseError::Ignorable(
                            AtRule {
                                rule_name,
                                rule_value,
                                block: Some(declarations),
                            },
                            msg,
                        ))
                    }
                }
            } else {
                self.take(';').ok_or(ParseError::Fatal)?;
                Ok(AtRule {
                    rule_name,
                    rule_value,
                    block: None,
                })
            }
        } else {
            Err(ParseError::Fatal)
        }
    }

    pub fn parse_qualified_rule(&mut self) -> Result<QualifiedRule, ParseError<QualifiedRule>> {
        self.skip_white_space();
        let selectors = self.parse_selectors().ok_or(ParseError::Fatal)?;
        self.take('{').ok_or(ParseError::Fatal)?;
        match self.parse_declaration_list() {
            Ok(declarations) => {
                self.skip_white_space();
                self.take('}').ok_or(ParseError::Fatal)?;
                Ok(QualifiedRule {
                    selectors,
                    block: declarations,
                })
            }
            Err(ParseError::Fatal) => return Err(ParseError::Fatal),
            Err(ParseError::Ignorable(declarations, msg)) => {
                self.skip_white_space();
                self.take('}').ok_or(ParseError::Fatal)?;
                Err(ParseError::Ignorable(
                    QualifiedRule {
                        selectors,
                        block: declarations,
                    },
                    msg,
                ))
            }
        }
    }

    pub fn parse_property(&mut self) -> Option<Property> {
        self.skip_white_space();
        let (value, _) = self.take_until(&[':', '{', '}', ';'])?;
        let property = {
            self.take(':')?;
            value.trim_end().to_string()
        };
        self.skip_white_space();
        let (value, _) = self.take_until(&[':', '{', '}', ';'])?;
        let value = {
            if self.peek(';') {
                self.take(';');
                value.trim_end().to_string()
            } else if self.peek('}') {
                value.trim_end().to_string()
            } else {
                return None;
            }
        };
        Some(Property { property, value })
    }

    // If Declarations encounters an error in the middle,
    // the part before the error can be used by ignoring the error.
    pub fn parse_declaration_list(
        &mut self,
    ) -> Result<Vec<Declaration>, ParseError<Vec<Declaration>>> {
        let mut declarations = vec![];
        let mut is_err = false;
        let mut is_nesting = false;
        let mut err_msg = String::new();
        loop {
            self.skip_white_space();
            if self.is_empty() || self.peek('}') {
                if is_err {
                    return Err(ParseError::Ignorable(declarations, err_msg));
                } else {
                    return Ok(declarations);
                }
            } else if self.peek('@') {
                match self.parse_at_rule() {
                    Ok(at_rule) => declarations.push(Declaration::AtRule(at_rule)),
                    Err(ParseError::Fatal) => {
                        return Err(ParseError::Ignorable(
                            declarations,
                            "[CSS parse error] at-rule parse error".into(),
                        ))
                    }
                    Err(ParseError::Ignorable(at_rule, msg)) => {
                        is_err = true;
                        err_msg = msg;
                        declarations.push(Declaration::AtRule(at_rule));
                    }
                }
                is_nesting = true;
            } else if self.peek('&') {
                match self.parse_qualified_rule() {
                    Ok(rule) => declarations.push(Declaration::QualifiedRule(rule)),
                    Err(ParseError::Fatal) => {
                        return Err(ParseError::Ignorable(
                            declarations,
                            "[CSS parse error] qualified rule parse error".into(),
                        ))
                    }
                    Err(ParseError::Ignorable(rule, msg)) => {
                        is_err = true;
                        err_msg = msg;
                        declarations.push(Declaration::QualifiedRule(rule));
                    }
                }
                is_nesting = true;
            } else {
                // property after nesting is ignored
                if is_nesting {
                    return Err(ParseError::Ignorable(
                        declarations,
                        "[CSS parse error] property after nesting is invalid and ignored".into(),
                    ));
                } else {
                    if let Some(property) = self.parse_property() {
                        declarations.push(Declaration::Property(property));
                    } else {
                        return Err(ParseError::Ignorable(
                            declarations,
                            "[CSS parse error] property declaration is expected".into(),
                        ));
                    }
                }
            }
        }
    }
}
