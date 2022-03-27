use std::iter::Peekable;
use std::str::Chars;

// Simple parser
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

    pub fn next(&mut self) -> Option<char> {
        self.content.next()
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

    pub fn take_until(&mut self, delimiters: &[char]) -> Result<(String, char), String> {
        let mut ret = String::new();
        loop {
            if let Some(&c) = self.content.peek() {
                if delimiters.contains(&c) {
                    return Ok((ret, c));
                } else {
                    ret.push(c);
                    self.content.next();
                }
            } else {
                return Err(ret);
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

    pub fn take_brace(&mut self) -> Option<String> {
        self.take('{')?;
        let mut content = vec![];
        while let Ok((value, delimiter)) = self.take_until(&['{', '}']) {
            content.push(value);
            if delimiter == '{' {
                let value = self.take_brace()?;
                let value = String::new() + "{" + &value + "}";
                content.push(value);
            } else if delimiter == '}' {
                self.take('}')?;
                break;
            }
        }
        Some(content.join(""))
    }
}
