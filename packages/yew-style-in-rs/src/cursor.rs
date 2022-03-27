use std::iter::Peekable;
use std::str::Chars;

// Parser for `@keyframes anim_name {}` in code to replace anim_names.
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

    pub fn take_until(&mut self, delimiter: char) -> Result<String, String> {
        let mut ret = String::new();
        loop {
            if let Some(&c) = self.content.peek() {
                if delimiter == c {
                    return Ok(ret);
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
}
